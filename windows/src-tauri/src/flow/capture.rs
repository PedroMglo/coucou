//! One dictation's immutable ASR/text options. No TTS, inference-provider or
//! desktop grant is inferred from this captured data.
use std::sync::{Arc,Weak,atomic::{AtomicBool,Ordering}};
use tauri::AppHandle;
use crate::{speech::{audio,Transcript},voice::RecognitionScope};
use super::{Flow,store::{Store,Style},logic::FlowKey,text::{self,Replacement,Snippet,Prepared}};

#[derive(Clone,PartialEq)]
pub(super) struct Options {
    pub key:FlowKey,pub mic:String,pub enabled:bool,pub keep_days:i64,
    pub cleanup:bool,pub style:Style,pub words:Vec<String>,
    replacements:Vec<Replacement>,snippets:Vec<Snippet>,
}
impl Options {
    fn capture(store:&Store,front:Option<&str>)->Result<Self,String>{
        let count=store.words.words.len().checked_add(store.words.replacements.len()).and_then(|n|n.checked_add(store.snippets.len())).ok_or("As opções do ditado excedem o limite.")?;
        if count>4096{return Err("O dicionário e os snippets excedem o limite de 4096 entradas por ditado.".into());}
        let mut bytes=0usize;
        for value in store.words.words.iter().map(String::as_str)
            .chain(store.words.replacements.iter().flat_map(|r|[r.id.as_str(),r.from.as_str(),r.to.as_str()]))
            .chain(store.snippets.iter().flat_map(|s|[s.id.as_str(),s.trigger.as_str(),s.text.as_str()])){
            bytes=bytes.checked_add(value.len()).ok_or("As opções do ditado excedem o limite.")?;
            if bytes>8*1024*1024{return Err("O dicionário e os snippets excedem 8 MiB por ditado.".into());}
        }
        Ok(Self{key:store.prefs.key,mic:store.prefs.mic.clone(),enabled:store.prefs.enabled,
            keep_days:store.prefs.keep_days,cleanup:store.prefs.cleanup,style:store.style_for(front),words:store.words.words.clone(),
            replacements:store.words.replacements.clone(),snippets:store.snippets.clone()})
    }
    fn matches(&self,store:&Store,front:Option<&str>)->bool{
        self.key==store.prefs.key&&self.mic==store.prefs.mic&&self.enabled==store.prefs.enabled&&self.keep_days==store.prefs.keep_days
            &&self.cleanup==store.prefs.cleanup&&self.style==store.style_for(front)&&self.words==store.words.words
            &&self.replacements==store.words.replacements&&self.snippets==store.snippets
    }
    fn prepare(&self,raw:&str)->Prepared{text::prepare(raw,&self.words,&self.replacements,&self.snippets)}
}
pub(super) struct Input {
    recognition:RecognitionScope,owner:Weak<Flow>,front:Option<String>,_work:Work,
    pub options:Options,pub source:audio::Source,pub stop:Arc<AtomicBool>,
}
impl Input {
    pub fn capture(app:&AppHandle,flow:&Arc<Flow>,front:Option<&str>)->Result<Self,String>{
        let stop=Arc::new(AtomicBool::new(false));let work=Work::claim(flow,stop.clone())?;
        let recognition=RecognitionScope::capture(app)?;
        let options={let store=flow.store.lock().map_err(|_|"As preferências do ditado estão indisponíveis.")?;Options::capture(&store,front)?};
        if !options.enabled||options.mic.len()>1024||options.mic.chars().any(char::is_control){return Err("A captura de ditado não está ativa ou o microfone é inválido.".into());}
        let source=audio::Source::Mic((!options.mic.is_empty()).then(||options.mic.clone()));
        let input=Self{recognition,owner:Arc::downgrade(flow),front:front.map(str::to_owned),options,source,stop,_work:work};input.check()?;Ok(input)
    }
    fn owner_current(&self)->Result<(),String>{
        if self.stop.load(Ordering::Acquire){return Err("O ditado foi cancelado.".into());}
        let flow=self.owner.upgrade().ok_or("O ditado já não tem dono.")?;
        let began=std::time::Instant::now();
        let store=loop{
            if self.stop.load(Ordering::Acquire){return Err("O ditado foi cancelado.".into());}
            match flow.store.try_lock(){
                Ok(store)=>break store,
                Err(std::sync::TryLockError::WouldBlock)if began.elapsed()<std::time::Duration::from_millis(50)=>std::thread::sleep(std::time::Duration::from_millis(1)),
                Err(_)=>return Err("As preferências do ditado estão ocupadas ou indisponíveis.".into()),
            }
        };
        if self.stop.load(Ordering::Acquire)||!self.options.matches(&store,self.front.as_deref()){return Err("As opções do ditado mudaram ou o pedido foi cancelado; não foi redirecionado.".into());}Ok(())
    }
    pub fn check(&self)->Result<(),String>{self.owner_current()?;self.recognition.check()}
    pub fn record(&self,text:&str,app:Option<&str>,seconds:f64)->Result<Option<super::store::Entry>,String>{
        self.check()?;
        let flow=self.owner.upgrade().ok_or("O ditado já não tem dono.")?;
        let mut store=loop{
            if self.stop.load(Ordering::Acquire){return Err("O ditado foi cancelado; não foi publicado novo histórico.".into());}
            match flow.store.try_lock(){Ok(store)=>break store,Err(std::sync::TryLockError::WouldBlock)=>std::thread::sleep(std::time::Duration::from_millis(2)),Err(_)=>return Err("O histórico está indisponível.".into())}
        };
        if self.stop.load(Ordering::Acquire)||!self.options.matches(&store,self.front.as_deref()){return Err("O contexto mudou antes de guardar o histórico.".into());}
        self.recognition.check()?;
        if self.stop.load(Ordering::Acquire){return Err("O ditado foi cancelado.".into());}
        Ok(store.record(text,app,seconds,super::store::today(),super::store::now_ms()))
    }
    pub fn ready(&self)->bool{self.recognition.ready()}
    pub fn transcribe(&self,samples:&[f32])->Option<Transcript>{self.recognition.transcribe(samples,Some(&self.stop),||self.owner_current().is_ok())}
    pub fn prepare(&self,raw:&str)->Prepared{self.options.prepare(raw)}
}
impl Drop for Input{fn drop(&mut self){self.stop.store(true,Ordering::Release);}}
pub(super) fn cancel_processing(flow:&Flow){if let Ok(slot)=flow.dictation_cancel.lock(){if let Some(token)=&*slot{token.store(true,Ordering::Release);}}}
/// Exclusion belongs to the existing Flow worker. Cancel revokes immediately;
/// the owner retains this slot until its thread returns, never at Cancel ACK.
pub(super) struct Work{owner:Weak<Flow>,token:Arc<AtomicBool>}
impl Work{
    pub fn current(&self)->bool{
        if self.token.load(Ordering::Acquire){return false;}
        let Some(flow)=self.owner.upgrade()else{return false;};
        let Ok(slot)=flow.dictation_cancel.try_lock()else{return false;};
        !self.token.load(Ordering::Acquire)&&slot.as_ref().is_some_and(|token|Arc::ptr_eq(token,&self.token))
    }
    pub fn claim(flow:&Arc<Flow>,token:Arc<AtomicBool>)->Result<Self,String>{
        let mut slot=flow.dictation_cancel.lock().map_err(|_|"O ditado está indisponível.")?;
        if slot.is_some(){return Err("O ditado anterior ainda está a terminar.".into());}
        *slot=Some(token.clone());Ok(Self{owner:Arc::downgrade(flow),token})
    }
}
impl Drop for Work{fn drop(&mut self){if let Some(flow)=self.owner.upgrade(){if let Ok(mut slot)=flow.dictation_cancel.lock(){if slot.as_ref().is_some_and(|t|Arc::ptr_eq(t,&self.token)){*slot=None;}}}}}

#[cfg(test)]#[path="capture_tests.rs"]mod tests;

/// Fresh manual source, captured before native waits. Clearing/replacing the
/// last output invalidates this occurrence instead of retargeting its data.
pub(super)struct LastText{pub text:String,work:Work,revision:u64,epoch:String}
impl LastText{
    pub fn capture(flow:&Arc<Flow>)->Result<Option<Self>,String>{
        let work=Work::claim(flow,Arc::new(AtomicBool::new(false)))?;
        if !work.current(){return Err("A cópia já não está ativa.".into());}
        let (text,revision,epoch)={
            let store=flow.store.try_lock().map_err(|_|"O texto anterior está ocupado.")?;
            if !store.has_last_text(){return Ok(None);}
            (store.last_text_ref().unwrap().to_owned(),store.last_output_revision(),store.last_output_epoch().to_owned())
        };
        let captured=Self{text,work,revision,epoch};
        if !captured.current(){return Err("O texto anterior mudou durante a preparação.".into());}Ok(Some(captured))
    }
    pub fn current(&self)->bool{
        if !self.work.current(){return false;}
        let Some(flow)=self.work.owner.upgrade()else{return false;};
        let began=std::time::Instant::now();
        let store=loop{
            if !self.work.current(){return false;}
            match flow.store.try_lock(){
                Ok(store)=>break store,
                Err(std::sync::TryLockError::WouldBlock)if began.elapsed()<std::time::Duration::from_millis(50)=>std::thread::sleep(std::time::Duration::from_millis(1)),
                Err(_)=>return false,
            }
        };
        store.last_output_epoch()==self.epoch&&store.last_output_revision()==self.revision&&store.has_last_text()&&self.work.current()
    }
}
