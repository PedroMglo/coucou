//! Native response/notification ordering through actual private stdio pipes.
use super::*;
use std::path::PathBuf;
const A:&str="019a734b-0000-7000-8000-000000000001";
const B:&str="019a734b-0000-7000-8000-000000000002";
const SERVER:&str=r#"import json,pathlib,sys,time
root=pathlib.Path('.')
mode=(root/'policy-mode').read_text()
A='019a734b-0000-7000-8000-000000000001'
B='019a734b-0000-7000-8000-000000000002'
def send(v): print(json.dumps(v),flush=True)
def policy(profile='native-safe'):
    return {'approvalPolicy':'never','approvalsReviewer':'user','sandboxPolicy':{'type':'readOnly','networkAccess':False},'cwd':'/fixture/repo','activePermissionProfile':{'id':profile,'extends':None},'model':'PRIVATE MODEL','developerInstructions':'PRIVATE INSTRUCTIONS'}
def update(thread,settings): send({'method':'thread/settings/updated','params':{'threadId':thread,'threadSettings':settings}})
def completed(text):
    send({'method':'item/completed','params':{'threadId':A,'turnId':'turn-1','item':{'id':'final','type':'agentMessage','text':text}}})
    send({'method':'turn/completed','params':{'threadId':A,'turn':{'id':'turn-1','status':'completed'}}})
for line in sys.stdin:
    p=json.loads(line)
    with (root/'policy-requests.jsonl').open('a') as f: f.write(json.dumps(p)+'\n')
    m=p.get('method')
    if m=='initialized': continue
    if m is None:
        if p.get('id')==77: completed('withdrawn' if 'error' in p else ('denied' if p.get('result',{}).get('decision')=='decline' else 'wrongly accepted'))
        continue
    result={}
    if m=='account/read': result={'account':{'type':'chatgpt'}}
    if m=='config/read': result={'config':{}}
    if m in ['thread/start','thread/resume']:
        result=policy();result['sandbox']=result.pop('sandboxPolicy');result['thread']={'id':B if mode=='resume-wrong' else A}
    if m=='turn/start': result={'turn':{'id':'turn-1'}}
    if 'id' in p: send({'id':p['id'],'result':result})
    if m in ['thread/start','thread/resume'] and mode=='ordered': update(A,policy('new-native-profile'))
    if m=='turn/start':
        send({'id':77,'method':'item/commandExecution/requestApproval','params':{'threadId':A,'turnId':'turn-1','itemId':'item-1','startedAtMs':1000,'command':'PRIVATE COMMAND','availableDecisions':['accept','decline']}})
        if mode in ['same','change','foreign','invalid']:
            until=time.monotonic()+4
            while not (root/'policy-callback-entered').exists():
                if time.monotonic()>until: sys.exit(4)
                time.sleep(.002)
            settings=policy()
            if mode=='change': settings['activePermissionProfile']['id']='changed-profile'
            if mode=='invalid': settings['sandboxPolicy']={'type':'futureSandbox'}
            if mode=='same': settings['model']='DIFFERENT PRIVATE MODEL';settings['developerInstructions']='DIFFERENT PRIVATE INSTRUCTIONS'
            update(B if mode=='foreign' else A,settings)
"#;
struct Fixture{cwd:PathBuf,exe:PathBuf}
impl Fixture{
    fn new(mode:&str)->Self{let(cwd,exe)=super::tests::mock();std::fs::write(cwd.join("app-server"),SERVER).unwrap();std::fs::write(cwd.join("policy-mode"),mode).unwrap();Self{cwd,exe}}
    fn received(&self)->Vec<Value>{std::fs::read_to_string(self.cwd.join("policy-requests.jsonl")).unwrap().lines().map(|l|serde_json::from_str(l).unwrap()).collect()}
}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.cwd);}}
fn options(resume:Option<&str>)->TurnOptions<'_>{TurnOptions{model:"fixture-free",system:"Synthetic policy fixture",history:&[],query:"Original request",tools:&[],session_id:resume,full_access:false,effort:None,name:None}}
#[test]
fn creation_and_resume_capture_confirmed_native_policy_before_the_request_without_widening_it(){
    for resume in [None,Some(A)]{let f=Fixture::new("normal");let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let mut count=0;let r=c.answer_native(options(resume),|_,_|panic!("no tool"),|_,_,ticket|{let identity=ticket.unwrap().identity();assert_eq!(identity.native_policy.coverage,"approvalSandboxSnapshot");assert!(identity.native_policy.hash.is_some());assert_eq!(identity.native_policy.source,Some("threadResponse"));assert!(!serde_json::to_string(identity).unwrap().contains("PRIVATE"));count+=1;Some(json!({"decision":"decline"}))},|_|{},|_|{}).unwrap();assert_eq!(r.text,"denied");assert_eq!(count,1);drop(c);let request=f.received().into_iter().find(|r|matches!(r["method"].as_str(),Some("thread/start"|"thread/resume"))).unwrap();assert!(request["params"].get("approvalPolicy").is_none());assert!(request["params"].get("sandbox").is_none());}
}
#[test]
fn native_policy_change_or_invalid_update_withdraws_the_old_review_and_replies_unavailable_instead_of_hanging(){
    for mode in ["change","invalid"]{let f=Fixture::new(mode);let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let marker=f.cwd.join("policy-callback-entered");let mut count=0;let r=c.answer_native(options(None),|_,_|panic!("no tool"),|_,_,ticket|{let ticket=ticket.unwrap();assert!(ticket.current());count+=1;std::fs::write(&marker,"ready").unwrap();let deadline=Instant::now()+Duration::from_secs(3);while ticket.current(){assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(2));}assert!(ticket.policy_invalidated());Some(json!({"decision":"accept"}))},|_|{},|_|{}).unwrap();assert_eq!(r.text,"withdrawn");assert_eq!(count,1);drop(c);let sent:Vec<_>=f.received().into_iter().filter(|v|v["id"]==77&&v.get("method").is_none()).collect();assert_eq!(sent.len(),1);assert_eq!(sent[0]["error"]["code"],-32000);assert!(sent[0].get("result").is_none());}
}
#[test]
fn identical_permissions_and_foreign_policy_updates_preserve_the_original_live_review(){
    for mode in ["same","foreign"]{let f=Fixture::new(mode);let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let requests=c.requests.clone();let marker=f.cwd.join("policy-callback-entered");let r=c.answer_native(options(None),|_,_|panic!("no tool"),|_,_,ticket|{let ticket=ticket.unwrap();let original=ticket.identity().native_policy.sequence.unwrap();std::fs::write(&marker,"ready").unwrap();let deadline=Instant::now()+Duration::from_secs(3);loop{let observed=requests.policy_snapshot(if mode=="same"{A}else{B});if if mode=="same"{observed.sequence>Some(original)}else{observed.hash.is_some()}{break;}assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(2));}assert!(ticket.current());Some(json!({"decision":"decline"}))},|_|{},|_|{}).unwrap();assert_eq!(r.text,"denied");}
}
#[test]
fn policy_notification_received_after_handshake_is_not_overwritten_when_the_main_consumer_reads_the_response(){
    let f=Fixture::new("ordered");let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let requests=c.requests.clone();let r=c.answer_native(options(None),|_,_|panic!("no tool"),|_,_,ticket|{let p=&ticket.unwrap().identity().native_policy;assert_eq!(p.source,Some("threadNotification"));assert_eq!(p.sequence,Some(2));assert_eq!(p.hash,requests.policy_snapshot(A).hash);Some(json!({"decision":"decline"}))},|_|{},|_|{}).unwrap();assert_eq!(r.text,"denied");
}
#[test]
fn resume_to_a_different_native_thread_fails_before_model_turn_or_adopting_the_returned_session(){
    let f=Fixture::new("resume-wrong");let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let mut adopted=false;let r=c.answer_native(options(Some(A)),|_,_|panic!("no tool"),|_,_,_|panic!("no approval"),|_|adopted=true,|_|{});assert!(r.err().unwrap().contains("different native thread"));assert!(!adopted);drop(c);assert!(!f.received().iter().any(|v|v["method"]=="turn/start"));
}
