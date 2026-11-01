//! One human decision, bound to its captured origin. Neither a pending entry
//! nor a presentation ID grants native permissions or reconstructs a session.
use super::*;
use crate::session_chat::types::{Target,Origin};
use ring::rand::{SecureRandom,SystemRandom};
const ENDED:&str="Este pedido já terminou.";
const MAX_PENDING:usize=128;
const MAX_REVIEW:usize=256*1024;
#[cfg(target_os="linux")]const MAX_GOAL_REVIEW:usize=2*1024*1024;
#[cfg(target_os="linux")]const MAX_GOAL_PENDING:usize=16;
const MAX_ANSWER:usize=64*1024;

impl RequestPolicy{
    fn live(&self,expires:Instant,now:Instant)->bool{
        now<expires&&match self{
            Self::Native{cancel,source}=>!cancel.load(Ordering::Acquire)&&source.as_ref().is_none_or(|source|source.current()),
            Self::LocalOnce{deadline,cancel}=>now<*deadline&&!cancel.load(Ordering::Acquire),
            #[cfg(target_os="linux")]Self::GoalRecipe{source}=>now<source.deadline()&&source.pending(),
        }
    }
}
pub(super)fn live(request:&PendingRequest)->bool{request.policy.live(request.expires_at,Instant::now())}
enum DecisionTarget{Session(Target),#[cfg(target_os="linux")]Goal(crate::pet_runtime::recipe_decisions::Target)}
impl PartialEq for DecisionTarget{fn eq(&self,other:&Self)->bool{match(self,other){(Self::Session(a),Self::Session(b))=>a==b,#[cfg(target_os="linux")](Self::Goal(a),Self::Goal(b))=>a==b,#[allow(unreachable_patterns)]_=>false}}}
fn target(value:&Value)->Result<DecisionTarget,String>{
    #[cfg(target_os="linux")]if value["origin"]=="goal"{let t:crate::pet_runtime::recipe_decisions::Target=serde_json::from_value(value.clone()).map_err(|_|"O destino do objetivo não é válido.".to_string())?;return if t.valid(){Ok(DecisionTarget::Goal(t))}else{Err("O destino do objetivo não é válido.".into())};}
    let t:Target=serde_json::from_value(value.clone()).map_err(|_|"O destino do pedido não é válido.".to_string())?;
    if crate::pet_config::pet_id(&t.provider)!=Some(t.pet_id.as_str())||t.session_id.is_empty()||t.session_id.len()>512||t.session_id.chars().any(char::is_control){return Err("O destino não pertence ao pet e à sessão apresentados.".into());}
    match t.origin{
        Origin::Mochi=>{if t.conversation_id.as_deref()!=Some(&t.session_id){return Err("A conversa do pedido não corresponde à sessão apresentada.".into());}},
        Origin::External=>t.validate_external(&t.provider,&t.pet_id)?,
    }Ok(DecisionTarget::Session(t))
}
fn request_target(request:&PendingRequest)->&Value{request.request.get("goalTarget").unwrap_or(&request.request["target"])}
fn questions(request:&Value)->Result<Vec<&str>,String>{
    if request["kind"]!="question"{return Ok(vec![]);}
    let rows=request["questions"].as_array().filter(|rows|rows.len()<=32).ok_or("Formulário nativo não suportado.")?;
    let mut ids=vec![];
    for row in rows{let id=row["id"].as_str().filter(|s|!s.is_empty()&&s.len()<=256&&!s.chars().any(char::is_control)).ok_or("Identificador da pergunta inválido.")?;if ids.contains(&id){return Err("Perguntas nativas repetidas.".into());}ids.push(id);}
    Ok(ids)
}
pub(super)fn validate_decision(request:&PendingRequest,answer:&Value,supplied:Option<&Value>)->Result<(),String>{
    if !live(request){return Err(ENDED.into());}
    #[cfg(target_os="linux")]if let RequestPolicy::GoalRecipe{source}=&request.policy{if answer["allow"].as_bool()!=Some(false)&&!source.current(){return Err("A validação nativa está temporariamente indisponível. Repete a decisão neste mesmo pedido.".into());}}
    let supplied=target(supplied.ok_or("A decisão exige o destino exato do pedido." )?)?;
    if supplied!=target(request_target(request))?{return Err("A decisão não pertence à sessão apresentada.".into());}
    if serde_json::to_vec(answer).map_err(|_|"Decisão inválida.")?.len()>MAX_ANSWER{return Err("A resposta excede o limite deste pedido.".into());}
    let object=answer.as_object().ok_or("Decisão inválida.")?;
    let allow=answer["allow"].as_bool().ok_or("Escolhe permitir ou recusar este pedido." )?;
    if allow{crate::settings::effect_allowed()?;}
    if object.keys().any(|key|key!="allow"&&key!="answers"&&key!="nativeChoice"){return Err("A decisão não pode alterar o âmbito do pedido nativo.".into());}
    if let Some(choices)=&request.choices{choices.resolve(answer)?;}else if answer.get("nativeChoice").is_some(){return Err("Este pedido não oferece escolhas nativas.".into());}
    let ids=questions(&request.request)?;
    if let Some(answers)=answer.get("answers"){
        let answers=answers.as_object().ok_or("Respostas inválidas.")?;
        for(id,values)in answers{
            if !ids.contains(&id.as_str()){return Err("A resposta não pertence às perguntas deste pedido.".into());}
            let values=values.as_array().filter(|v|v.len()<=32).ok_or("Respostas inválidas.")?;
            if !allow&&!values.is_empty(){return Err("Uma recusa não pode enviar respostas afirmativas.".into());}
            if values.iter().any(|v|v.as_str().is_none_or(|s|s.len()>8192||s.contains('\0'))){return Err("Resposta inválida ou demasiado longa.".into());}
        }
    }
    if matches!(request.policy,RequestPolicy::LocalOnce{..})&&request.request["kind"]=="question"{return Err("A aprovação local autoriza apenas o comando apresentado.".into());}
    #[cfg(target_os="linux")]if matches!(&request.policy,RequestPolicy::GoalRecipe{..})&&(request.request["kind"]!="permission"||answer.get("nativeChoice").is_some()||answer.get("answers").is_some_and(|a|a.as_object().is_none_or(|a|!a.is_empty()))){return Err("A decisão autoriza apenas a receita exata apresentada.".into());}
    if let Some(form)=&request.native_form{form.resolve(answer)?;}
    Ok(())
}
pub(super)fn register(requests:&Mutex<HashMap<String,PendingRequest>>,id:String,request:PendingRequest)->Result<(),String>{
    if let Some(form)=&request.native_form{
        if request.choices.is_some()||!matches!(&request.policy,RequestPolicy::Native{source:Some(source),..}if form.matches(source))
            ||request.request["nativeForm"]!="mcp"||request.request["questions"]!=form.questions()||request.request["detail"]!=form.detail(){return Err("O formulário não pertence à origem deste pedido.".into());}
    }else if request.request.get("nativeForm").is_some(){return Err("Este pedido não tem um formulário nativo revisto.".into());}
    if let Some(choices)=&request.choices{
        if !matches!(&request.policy,RequestPolicy::Native{source:Some(source),..}if choices.matches(source))||request.request["nativeChoices"]!=choices.public(){return Err("As escolhas não pertencem à origem deste pedido.".into());}
    }
    #[cfg(target_os="linux")]match &request.policy{RequestPolicy::GoalRecipe{source}=>{if !source.matches(&request.request)||request.native_form.is_some()||request.choices.is_some(){return Err("A revisão não pertence à origem nativa do objetivo.".into());}},_=>{if request.request.get("goalTarget").is_some()||request.request["target"]["origin"]=="goal"{return Err("Este pedido não tem uma origem nativa de objetivo.".into());}}}
    target(request_target(&request))?;questions(&request.request)?;if request.request["id"]!=id{return Err("O identificador do pedido não corresponde ao registo.".into());}
    let review_limit=MAX_REVIEW;
    #[cfg(target_os="linux")]let review_limit=if matches!(&request.policy,RequestPolicy::GoalRecipe{..}){MAX_GOAL_REVIEW}else{review_limit};
    if serde_json::to_vec(&request.request).map_err(|_|"Pedido inválido.")?.len()>review_limit{return Err("O pedido excede o limite da revisão.".into());}
    let mut all=requests.lock().map_err(|_|"Os pedidos estão indisponíveis.".to_string())?;
    if !live(&request){return Err(ENDED.into());}
    #[cfg(target_os="linux")]if matches!(&request.policy,RequestPolicy::GoalRecipe{..}){
        if all.values().filter(|r|matches!(&r.policy,RequestPolicy::GoalRecipe{..})).count()>=MAX_GOAL_PENDING{return Err("Já existem demasiadas receitas pendentes.".into());}
        let code=request.request["voiceCode"].as_str().filter(|s|s.len()==6&&s.bytes().all(|c|c.is_ascii_digit())).ok_or("O código de revisão é inválido.")?;
        if all.values().any(|r|r.request.get("goalTarget").is_some()&&r.request["goalTarget"]==request.request["goalTarget"]){return Err("Esta receita já tem uma decisão pendente.".into());}
        if all.values().any(|r|r.request["goalTarget"].is_object()&&r.request["voiceCode"]==code){return Err("O código de revisão já está em uso. Repete o pedido.".into());}
    }
    if all.len()>=MAX_PENDING||all.contains_key(&id){return Err("Já existem demasiados pedidos pendentes.".into());}
    all.insert(id,request);Ok(())
}
pub(super)fn answer_in(chat:&Chat,requests:&Mutex<HashMap<String,PendingRequest>>,id:&str,answer:Value,supplied:Option<Value>)->Result<(),String>{
    let t=target(supplied.as_ref().ok_or("A decisão exige o destino exato do pedido.")?)?;
    let consume=||{
        let mut all=requests.try_lock().map_err(|_|"Os pedidos estão a atualizar. Tenta responder novamente.".to_string())?;
        let request=all.get(id).ok_or(ENDED)?;
        validate_decision(request,&answer,supplied.as_ref())?;
        let native=if let Some(form)=&request.native_form{Some(form.resolve(&answer)?)}else{request.choices.as_ref().map(|c|c.resolve(&answer)).transpose()?};
        let request=all.remove(id).ok_or(ENDED)?;drop(all);
        if !live(&request){return Err(ENDED.into());}
        let mut answer=answer;if let Some(native)=native{answer["nativeResponse"]=native;}
        request.reply.send(answer).map_err(|_|ENDED.to_string())
    };
    match t{
        DecisionTarget::Session(t)if t.origin==Origin::Mochi=>chat.with_live_conversation(&t.provider,&t.session_id,consume)?.ok_or_else(||ENDED.to_string())?,
        _=>consume(),
    }
}
fn id_from_boot(boot:&str,local:bool,counter:&AtomicU64)->Option<String>{
    let mut current=counter.load(Ordering::Acquire);
    loop{let next=current.checked_add(1)?;match counter.compare_exchange_weak(current,next,Ordering::AcqRel,Ordering::Acquire){Ok(_)=>return Some(format!("cli-{}-{boot}-{current}",if local{"local"}else{"native"})),Err(actual)=>current=actual}}
}
pub(super)fn next_id(local:bool)->Option<String>{
    static BOOT:OnceLock<Result<String,()>>=OnceLock::new();
    let boot=BOOT.get_or_init(||{let mut bytes=[0u8;32];SystemRandom::new().fill(&mut bytes).map_err(|_|())?;Ok(bytes.iter().map(|b|format!("{b:02x}")).collect())}).as_ref().ok()?;
    id_from_boot(boot,local,&NEXT)
}
#[cfg(test)]#[path="cli_decisions_tests.rs"]mod tests;
