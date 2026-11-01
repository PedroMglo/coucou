//! Shared decision transport for a native reviewed recipe. This does not execute
//! or approve it: only the retained runtime review can consume the human answer.
use super::*;
use crate::pet_runtime::recipe_decisions::Source;
fn register(requests:&Mutex<HashMap<String,PendingRequest>>,source:Source)->Option<(String,Value,mpsc::Receiver<Value>)>{
    let id=decisions::next_id(false)?.replacen("cli-native-","goal-recipe-",1);let deadline=source.deadline();
    let mut bytes=[0u8;4];ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(),&mut bytes).ok()?;let code=format!("{:06}",u32::from_le_bytes(bytes)%1_000_000);
    let request=json!({"id":id,"provider":source.target().provider,"kind":"permission","origin":format!("Objetivo · {} · {}",source.target().pet_id,source.target().goal_id),"approvalScope":"recipe","voiceCode":code,"goalTarget":source.target(),"goalReview":source.review(),"detail":serde_json::to_string_pretty(source.review()).ok()?});
    let(tx,rx)=mpsc::channel();decisions::register(requests,id.clone(),PendingRequest{native_form:None,choices:None,reply:tx,request:request.clone(),policy:RequestPolicy::GoalRecipe{source},expires_at:deadline}).ok()?;Some((id,request,rx))
}
pub(crate)fn ask_goal_recipe(app:&AppHandle,source:Source)->Option<bool>{ask_goal_recipe_decision(app,source).map(|d|d.approved())}
pub(crate)fn ask_goal_recipe_decision(app:&AppHandle,source:Source)->Option<crate::pet_runtime::recipe_decisions::Decision>{
    let(id,request,rx)=register(pending(),source.clone())?;
    let answer=if app.emit("agent-request",request).is_ok(){loop{
        if !source.pending(){break None;}
        match rx.recv_timeout(source.deadline().saturating_duration_since(Instant::now()).min(Duration::from_millis(50))){
            Ok(answer)=>break answer["allow"].as_bool(),
            Err(mpsc::RecvTimeoutError::Disconnected)=>break None,Err(mpsc::RecvTimeoutError::Timeout)=>{},
        }
    }}else{None};
    if let Ok(mut all)=pending().lock(){all.remove(&id);}
    let decision=answer.and_then(|allow|decide_received(source.clone(),allow));source.retire();let _=app.emit("agent-request-ended",json!({"id":id}));decision
}
fn decide_received(source:Source,allow:bool)->Option<crate::pet_runtime::recipe_decisions::Decision>{
    loop{
        if let Some(decision)=crate::pet_runtime::recipe_decisions::Decision::from_transport(source.clone(),allow){break Some(decision);}
        if !source.pending(){break None;}
        // Revalidate the same received answer under the original deadline.
        // Waiting creates no new request, approval, budget or effect.
        std::thread::sleep(Duration::from_millis(20));
    }
}
#[cfg(test)]pub(crate)struct Fixture{source:Source,requests:Mutex<HashMap<String,PendingRequest>>,pub(crate)id:String,pub(crate)request:Value,pub(crate)reply:mpsc::Receiver<Value>}
#[cfg(test)]pub(crate)fn fixture(source:Source)->Fixture{let requests=Mutex::new(HashMap::new());let(id,request,reply)=register(&requests,source.clone()).unwrap();Fixture{source,requests,id,request,reply}}
#[cfg(test)]impl Fixture{
    pub(crate)fn decision(&self)->Option<crate::pet_runtime::recipe_decisions::Decision>{let answer=self.reply.recv_timeout(Duration::from_secs(1)).ok()?;decide_received(self.source.clone(),answer["allow"].as_bool()?)}
    pub(crate)fn answer(&self,answer:Value,target:Value)->Result<(),String>{decisions::answer_in(&Chat::default(),&self.requests,&self.id,answer,Some(target))}
    pub(crate)fn count(&self)->usize{self.requests.lock().unwrap().len()}
    pub(crate)fn panel_pending(&self)->Vec<Value>{goal_pending_in(&self.requests).unwrap()}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]enum Spoken{Review,Approve,Deny}
fn parse(query:&str)->Option<(Spoken,Option<String>)>{
    let text=query.trim().trim_end_matches(['.','!']).to_lowercase();
    for(prefix,action)in[("rever receita",Spoken::Review),("aprovar receita",Spoken::Approve),("recusar receita",Spoken::Deny),("review recipe",Spoken::Review),("approve recipe",Spoken::Approve),("deny recipe",Spoken::Deny)]{
        if text==prefix{return Some((action,None));}
        if let Some(code)=text.strip_prefix(prefix).and_then(|s|s.strip_prefix(' ')){
            let code=code.trim();return Some((action,(code.len()==6&&code.bytes().all(|c|c.is_ascii_digit())).then(||code.to_string())));
        }
    }None
}
fn voice_in(chat:&Chat,requests:&Mutex<HashMap<String,PendingRequest>>,provider:&str,query:&str,stop:&AtomicBool)->Option<String>{
    let(action,code)=parse(query)?;
    if stop.load(Ordering::Acquire){return Some("A decisão de voz foi interrompida.".into());}
    let Some(code)=code else{return Some("Indica os seis dígitos do cartão, por exemplo: rever receita 123456. Um sim isolado não autoriza uma receita.".into());};
    let found={let all=match requests.try_lock(){Ok(all)=>all,Err(_)=>return Some("Os pedidos estão a atualizar. Repete o comando.".into())};
        let mut matches=all.iter().filter_map(|(id,r)|match &r.policy{RequestPolicy::GoalRecipe{source}if r.request["voiceCode"]==code&&source.target().provider==provider&&decisions::live(r)=>Some((id.clone(),source.clone())),_=>None});
        let first=matches.next();if matches.next().is_some(){return Some("O código é ambíguo. Revê os cartões antes de decidir.".into());}first};
    let Some((id,source))=found else{return Some("Não há uma receita pendente com esse código para este pet. Revê o cartão atual.".into());};
    if action==Spoken::Review{return Some(source.spoken_review(&code));}
    if stop.load(Ordering::Acquire){return Some("A decisão de voz foi interrompida.".into());}
    let allow=action==Spoken::Approve;let target=serde_json::to_value(source.target()).ok()?;
    Some(match decisions::answer_in(chat,requests,&id,json!({"allow":allow}),Some(target)){
        Ok(())if allow=>"A decisão de autorização foi enviada para esta receita. A execução e o resultado ainda têm de ser confirmados pelo motor.".into(),
        Ok(())=>"A decisão de recusa foi enviada para esta receita.".into(),
        Err(reason)=>reason,
    })
}
pub(crate)fn goal_recipe_voice(app:&AppHandle,provider:&str,query:&str,stop:&AtomicBool)->Option<String>{voice_in(&app.state::<Chat>(),pending(),provider,query,stop)}
#[cfg(test)]impl Fixture{pub(crate)fn voice(&self,provider:&str,query:&str,stop:&AtomicBool)->Option<String>{voice_in(&Chat::default(),&self.requests,provider,query,stop)}}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn voice_never_infers_consent_from_a_generic_answer_or_embedded_instruction(){for q in ["sim","yes","aprova tudo","diz aprovar receita 123456","approve recipe 123456 and execute","aprovar receita 123456 e executar"]{assert!(parse(q).is_none_or(|(_,code)|code.is_none()),"{q}");}assert_eq!(parse(" Aprovar receita 001234. "),Some((Spoken::Approve,Some("001234".into()))));assert_eq!(parse("deny recipe 123456"),Some((Spoken::Deny,Some("123456".into()))));}
}

pub(crate)fn goal_recipe_voice_requested(query:&str)->bool{parse(query).is_some()}

/// Runtime management can supervise goals without acquiring the ability to
/// answer a terminal's native request. Both paths consume the same broker row.
fn goal_pending_in(requests:&Mutex<HashMap<String,PendingRequest>>)->Result<Vec<Value>,String>{
    Ok(requests.try_lock().map_err(|_|"Os pedidos estão a atualizar.".to_string())?.values().filter(|r|matches!(&r.policy,RequestPolicy::GoalRecipe{..})&&decisions::live(r)).map(|r|r.request.clone()).collect())
}
fn goal_answer_in(chat:&Chat,requests:&Mutex<HashMap<String,PendingRequest>>,id:&str,approve:bool,target:Value)->Result<(),String>{
    {let all=requests.try_lock().map_err(|_|"Os pedidos estão a atualizar.".to_string())?;if !all.get(id).is_some_and(|r|matches!(&r.policy,RequestPolicy::GoalRecipe{..})){return Err("Este pedido não é uma revisão de objetivo pendente.".into());}}
    decisions::answer_in(chat,requests,id,json!({"allow":approve}),Some(target))
}
pub(crate)fn goal_pending()->Result<Vec<Value>,String>{goal_pending_in(pending())}
pub(crate)fn goal_answer(chat:&Chat,id:&str,approve:bool,target:Value)->Result<(),String>{goal_answer_in(chat,pending(),id,approve,target)}
#[cfg(test)]impl Fixture{
    pub(crate)fn panel_answer(&self,approve:bool,target:Value)->Result<(),String>{goal_answer_in(&Chat::default(),&self.requests,&self.id,approve,target)}
    pub(crate)fn assert_panel_isolation(&self){
        let id="synthetic-terminal-not-goal".to_string();let(tx,rx)=mpsc::channel();let target=json!({"petId":"ai_ollama","provider":"ollama","origin":"mochi","sessionId":"foreground","conversationId":"foreground"});
        decisions::register(&self.requests,id.clone(),PendingRequest{native_form:None,choices:None,reply:tx,request:json!({"id":id,"kind":"permission","target":target}),policy:RequestPolicy::Native{cancel:Arc::default(),source:None},expires_at:Instant::now()+Duration::from_secs(60)}).unwrap();
        let list=goal_pending_in(&self.requests).unwrap();assert_eq!(list.len(),1);assert_eq!(list[0]["id"],self.id);assert!(goal_answer_in(&Chat::default(),&self.requests,&id,true,target).is_err());assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));self.requests.lock().unwrap().remove(&id);
    }
}
