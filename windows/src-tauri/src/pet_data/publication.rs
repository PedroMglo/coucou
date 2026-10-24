//! Initial two-root publication and native admission. No runtime path redirect
//! is enabled here. Markers describe app data, never agent execution consent.
use super::{inventory::Area,private_fs::Node,staging,staging_fs::{self,Dir},startup::Claim,Error,Layout,LegacyReview,Result,Summary};
use serde::{Deserialize,Serialize};
use std::{ffi::OsStr,io::Read,os::unix::ffi::OsStrExt,path::{Path,PathBuf}};
use ring::{digest::{Context,SHA256},rand::{SecureRandom,SystemRandom}};

const HEADER:&str=".profile.json";
const COMPLETE:&str=".complete.json";
const ADMITTED:&str=".admitted.json";
const PAYLOAD:&str="payload";
const MAX_RECORD:usize=4096;
const MAX_STAGED_RECORDS:usize=16;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Header{version:u32,pair:String,area:Area,layout:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Complete{version:u32,pair:String,area:Area,content:String,summary:Summary}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Admission{version:u32,pair:String,layout:String,config:String,data:String}
#[derive(Debug)]pub(crate)struct Failure{pub error:Error,pub pair:Option<String>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate)enum State{Absent,Incomplete,AdmissionRecordPresent}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]enum Step{ConfigCreated,DataCreated,BeforeCopy(usize),Copied,ConfigComplete,DataComplete,Admitted}

fn hex(bytes:&[u8])->String{bytes.iter().map(|b|format!("{b:02x}")).collect()}
fn digest(bytes:&[u8])->String{hex(ring::digest::digest(&SHA256,bytes).as_ref())}
fn valid_hex(s:&str,n:usize)->bool{s.len()==n&&s.bytes().all(|c|c.is_ascii_digit()||(b'a'..=b'f').contains(&c))}
fn layout_key(layout:&Layout)->String{
    let mut hash=Context::new(&SHA256);hash.update(b"coucou-profile-layout-v1\0");
    for path in [&layout.legacy_config,&layout.legacy_data]{let bytes=path.as_os_str().as_bytes();hash.update(&(bytes.len()as u64).to_le_bytes());hash.update(bytes);}
    hex(hash.finish().as_ref())
}
fn encode<T:Serialize>(value:&T)->Result<Vec<u8>>{let bytes=serde_json::to_vec(value).map_err(|_|Error::UnsupportedProfile)?;if bytes.len()>MAX_RECORD{Err(Error::Limit)}else{Ok(bytes)}}
fn read(dir:&Dir,name:&str)->Result<Option<Vec<u8>>>{
    let parent=dir.node()?;let Some(node)=parent.child(OsStr::new(name))?else{return Ok(None)};
    if node.directory||node.len>MAX_RECORD as u64||!staging_fs::private_file(&node.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}
    let mut bytes=vec![];(&node.file).take(MAX_RECORD as u64+1).read_to_end(&mut bytes).map_err(|_|Error::Unavailable)?;
    node.current()?;parent.same_child(OsStr::new(name),&node)?;
    if bytes.len()>MAX_RECORD||bytes.len()as u64!=node.len{return Err(Error::SourceChanged);}Ok(Some(bytes))
}
fn parse<T:for<'a>Deserialize<'a>>(bytes:&[u8])->Result<T>{serde_json::from_slice(bytes).map_err(|_|Error::UnsupportedProfile)}
fn controls(root:&Dir,area:Area)->Result<()>{
    let mut staged=0;
    for name in root.node()?.names(5+MAX_STAGED_RECORDS)?{
        if matches!(name.to_str(),Some(HEADER|COMPLETE|PAYLOAD))
            ||area==Area::Config&&(name==ADMITTED||name==resume::INTENT){continue;}
        let text=name.to_str().ok_or(Error::UnsupportedProfile)?;
        if !text.strip_prefix(".publish-").is_some_and(|nonce|valid_hex(nonce,32)){return Err(Error::UnsupportedProfile);}
        staged+=1;if staged>MAX_STAGED_RECORDS{return Err(Error::Limit);}
        // Unpublished metadata is preserved, bounded and never parsed as an
        // intent, completion/admission receipt, or execution authority. Only
        // the final literal records take part in native recovery decisions.
        let _=read(root,text)?.ok_or(Error::SourceChanged)?;
    }Ok(())
}
fn missing(path:&Path)->Result<bool>{Node::root(path).map(|n|n.is_none())}

/// Read-only exception evidence issued by this native publisher or re-attested
/// under a genuine fresh Claim during data recovery. Not a writable permit or
/// an execution grant; general legacy inspection still refuses existing roots.
#[derive(Clone)]pub(super)struct OwnedRoots{layout:Layout,config:Dir,data:Dir,config_header:Vec<u8>,data_header:Vec<u8>,pair:String}
impl OwnedRoots{
    pub(super)fn current_for(&self,layout:&Layout)->Result<()>{
        if layout!=&self.layout{return Err(Error::UnsafePath);}
        for(root,expected,area)in[(&self.config,&self.config_header,Area::Config),(&self.data,&self.data_header,Area::Data)]{
            root.check()?;controls(root,area)?;
            if read(root,HEADER)?.as_deref()!=Some(expected.as_slice()){return Err(Error::SourceChanged);}root.dir(OsStr::new(PAYLOAD))?.check()?;
        }Ok(())
    }
    fn existing(claim:&Claim,layout:Layout)->Result<Self>{
        claim.for_layout(&layout)?;
        let config=Dir::root(layout.config())?;let data=Dir::root(layout.data())?;
        let config_header=read(&config,HEADER)?.ok_or(Error::UnsupportedProfile)?;let data_header=read(&data,HEADER)?.ok_or(Error::UnsupportedProfile)?;
        let a:Header=parse(&config_header)?;let b:Header=parse(&data_header)?;
        if a.version!=1||b.version!=1||a.area!=Area::Config||b.area!=Area::Data||a.pair!=b.pair||!valid_hex(&a.pair,32)||a.layout!=layout_key(&layout)||b.layout!=a.layout{return Err(Error::UnsupportedProfile);}
        let roots=Self{layout,config,data,config_header,data_header,pair:a.pair};roots.current_for(&roots.layout)?;claim.current()?;Ok(roots)
    }
    fn payload(&self,area:Area)->Result<Dir>{match area{Area::Config=>&self.config,Area::Data=>&self.data}.dir(OsStr::new(PAYLOAD))}
}
fn root(parent:&Path,target:&Path,area:Area,pair:&str,key:&str)->Result<(Dir,Vec<u8>)>{
    let parent=Dir::parent(parent)?;
    let mut nonce=[0u8;16];SystemRandom::new().fill(&mut nonce).map_err(|_|Error::Unavailable)?;
    let staged=format!(".pets-profile-stage-{}",hex(&nonce));
    let root=parent.mkdir(OsStr::new(&staged))?;
    let bytes=encode(&Header{version:1,pair:pair.into(),area,layout:key.into()})?;
    root.publish_new(HEADER,&bytes)?;root.mkdir(OsStr::new(PAYLOAD))?;root.sync()?;
    let root=parent.publish_dir(OsStr::new(&staged),target.file_name().ok_or(Error::UnsafePath)?,&root)?;
    Ok((root,bytes))
}
/// Owns the genuine process claim until completion or refusal. Roots are created
/// inactive before source inventory, so source metadata checks stay strict.
pub(crate)struct Publication{claim:Claim,roots:OwnedRoots,review:LegacyReview}
impl Publication{
    pub(crate)fn begin(claim:Claim,layout:Layout)->std::result::Result<Self,Failure>{Self::begin_using(claim,layout,|_|Ok(()))}
    fn begin_using(claim:Claim,layout:Layout,mut step:impl FnMut(Step)->Result<()>)->std::result::Result<Self,Failure>{
        let mut pair=None;
        let result=(||->Result<Self>{
            claim.for_layout(&layout)?;
            if !missing(layout.config())?||!missing(layout.data())?{return Err(Error::TargetExists);}
            // Bootstrap must create its bounded application parents under native
            // ownership first; never create arbitrary ancestor trees here.
            let _=Dir::parent(&layout.legacy_config)?;let _=Dir::parent(&layout.legacy_data)?;
            // Reject known incompatible/oversized data before leaving any new
            // destination behind. Fresh typed review still follows creation.
            let _=LegacyReview::inspect(layout.clone())?;
            let mut random=[0u8;16];SystemRandom::new().fill(&mut random).map_err(|_|Error::Unavailable)?;let id=hex(&random);pair=Some(id.clone());let key=layout_key(&layout);
            let(config,config_header)=root(&layout.legacy_config,layout.config(),Area::Config,&id,&key)?;step(Step::ConfigCreated)?;claim.current()?;
            let(data,data_header)=root(&layout.legacy_data,layout.data(),Area::Data,&id,&key)?;step(Step::DataCreated)?;claim.current()?;
            let roots=OwnedRoots{layout:layout.clone(),config,data,config_header,data_header,pair:id};roots.current_for(&layout)?;
            let review=LegacyReview::for_publication(layout,roots.clone())?;
            claim.current()?;resume::intent(&roots,&review)?;claim.current()?;Ok(Self{claim,roots,review})
        })();result.map_err(|error|Failure{error,pair})
    }
    pub(crate)fn publish(self)->std::result::Result<Active,Failure>{self.publish_using(|_|Ok(()))}
    fn publish_using(self,mut step:impl FnMut(Step)->Result<()>)->std::result::Result<Active,Failure>{
        let pair=Some(self.roots.pair.clone());
        let result=(||->Result<Active>{
            self.current()?;let config=self.roots.payload(Area::Config)?;let data=self.roots.payload(Area::Data)?;
            staging::copy_publication(&self.review,&config,&data,|index|{
                step(Step::BeforeCopy(index))?;self.claim.for_layout(&self.roots.layout)?;self.roots.current_for(&self.roots.layout)
            })?;
            staging::verify_publication(&self.review,&config,&data)?;self.current()?;step(Step::Copied)?;
            self.current()?;let records=completion(&self.roots,&self.review)?;
            self.roots.config.publish_new(COMPLETE,&records.0)?;step(Step::ConfigComplete)?;self.current()?;
            self.roots.data.publish_new(COMPLETE,&records.1)?;step(Step::DataComplete)?;self.current()?;
            // Verify again at the pair admission boundary. Neither root alone
            // authorizes a reader to construct defaults or persist anything.
            staging::verify_publication(&self.review,&config,&data)?;
            admit(&self.claim,&self.roots,&records.0,&records.1)?;step(Step::Admitted)?;
            Active::from_roots(self.claim,self.roots)
        })();result.map_err(|error|Failure{error,pair})
    }
    fn current(&self)->Result<()>{self.claim.for_layout(&self.roots.layout)?;self.roots.current_for(&self.roots.layout)?;self.review.ensure_current()}
}
fn completion(roots:&OwnedRoots,review:&LegacyReview)->Result<(Vec<u8>,Vec<u8>)>{
    let content=staging::content_fingerprint(review)?;let mut records=vec![];
    for area in[Area::Config,Area::Data]{records.push(encode(&Complete{version:1,pair:roots.pair.clone(),area,content:content.clone(),summary:review.summary()})?);}
    Ok((records.remove(0),records.remove(0)))
}
fn validate_complete(roots:&OwnedRoots,a:&[u8],b:&[u8])->Result<()>{
    let a:Complete=parse(a)?;let b:Complete=parse(b)?;
    if a.version!=1||b.version!=1||a.pair!=roots.pair||b.pair!=a.pair||a.area!=Area::Config||b.area!=Area::Data||a.content!=b.content||!valid_hex(&a.content,64)||a.summary!=b.summary
        ||a.summary.files.checked_add(a.summary.directories).is_none_or(|n|n>super::inventory::MAX_NODES)||a.summary.bytes>super::inventory::MAX_BYTES||a.summary.excluded_roots>512{return Err(Error::UnsupportedProfile);}Ok(())
}
fn admission(roots:&OwnedRoots,a:&[u8],b:&[u8])->Result<Vec<u8>>{
    validate_complete(roots,a,b)?;encode(&Admission{version:1,pair:roots.pair.clone(),layout:layout_key(&roots.layout),config:digest(a),data:digest(b)})
}
fn admit(claim:&Claim,roots:&OwnedRoots,a:&[u8],b:&[u8])->Result<()>{
    claim.for_layout(&roots.layout)?;roots.current_for(&roots.layout)?;
    if read(&roots.config,COMPLETE)?.as_deref()!=Some(a)||read(&roots.data,COMPLETE)?.as_deref()!=Some(b){return Err(Error::SourceChanged);}
    roots.config.sync()?;roots.data.sync()?;claim.current()?;
    roots.config.publish_new(ADMITTED,&admission(roots,a,b)?)?;roots.config.sync()?;claim.current()
}
/// Active means application data admitted under a current native claim. Agent
/// sessions/permissions/consent/budgets must still follow their own brokers.
pub(crate)struct Active{claim:Claim,roots:OwnedRoots}
pub(crate)struct Paths{pub config:PathBuf,pub data:PathBuf}
fn current_data(roots:&OwnedRoots)->Result<()>{
    roots.current_for(&roots.layout)?;let a=read(&roots.config,COMPLETE)?.ok_or(Error::UnsupportedProfile)?;let b=read(&roots.data,COMPLETE)?.ok_or(Error::UnsupportedProfile)?;
    if read(&roots.config,ADMITTED)?.as_deref()!=Some(admission(roots,&a,&b)?.as_slice()){return Err(Error::UnsupportedProfile);}Ok(())
}
impl Active{
    fn from_roots(claim:Claim,roots:OwnedRoots)->Result<Self>{let active=Self{claim,roots};active.current()?;Ok(active)}
    pub(crate)fn open(claim:Claim,layout:Layout)->Result<Self>{let roots=OwnedRoots::existing(&claim,layout)?;Self::from_roots(claim,roots)}
    pub(crate)fn current(&self)->Result<()>{
        self.claim.for_layout(&self.roots.layout)?;current_data(&self.roots)
    }
    pub(crate)fn paths(&self)->Result<Paths>{self.current()?;Ok(Paths{config:self.roots.layout.config().join(PAYLOAD),data:self.roots.layout.data().join(PAYLOAD)})}
    pub(super)fn into_storage(self)->Result<Storage>{self.current()?;Ok(Storage{lease:self.claim.into_storage()?,roots:self.roots,invalid:std::sync::atomic::AtomicBool::new(false)})}
}
pub(super)struct Storage{lease:super::startup::StorageLease,roots:OwnedRoots,invalid:std::sync::atomic::AtomicBool}
impl Storage{
    pub(super)fn current(&self)->Result<()>{
        use std::sync::atomic::Ordering;
        if self.invalid.load(Ordering::Acquire){return Err(Error::OwnershipLost);}
        let result=self.lease.current().and_then(|_|current_data(&self.roots));if result.is_err(){self.invalid.store(true,Ordering::Release);}result
    }
    pub(super)fn paths(&self)->Result<Paths>{self.current()?;Ok(Paths{config:self.roots.layout.config().join(PAYLOAD),data:self.roots.layout.data().join(PAYLOAD)})}
}
/// Finish an interrupted admission only when the entire copied payload still
/// matches a fresh native legacy review. Partial/changed/unknown data is retained
/// and refused; this does not silently overwrite or replay file/model effects.
pub(crate)fn recover_complete(claim:Claim,layout:Layout)->Result<Active>{
    let roots=OwnedRoots::existing(&claim,layout.clone())?;
    if read(&roots.config,ADMITTED)?.is_some(){return Active::from_roots(claim,roots);}
    let review=LegacyReview::for_publication(layout,roots.clone())?;
    staging::verify_publication(&review,&roots.payload(Area::Config)?,&roots.payload(Area::Data)?)?;claim.current()?;
    let(a,b)=completion(&roots,&review)?;let mut expected:[Complete;2]=[parse(&a)?,parse(&b)?];let mut diagnostic=None;
    for(index,root)in[&roots.config,&roots.data].into_iter().enumerate(){
        if let Some(existing)=read(root,COMPLETE)?{
            let record:Complete=parse(&existing)?;
            // Excluded old roots are diagnostic, not copied data. A newly
            // created log must not redefine the original completion receipt.
            if diagnostic.is_some_and(|n|n!=record.summary.excluded_roots){return Err(Error::SourceChanged);}
            diagnostic=Some(record.summary.excluded_roots);let mut normalized=record.clone();normalized.summary.excluded_roots=expected[index].summary.excluded_roots;
            if normalized!=expected[index]||record.summary.excluded_roots>512{return Err(Error::SourceChanged);}
        }
    }
    if let Some(diagnostic)=diagnostic{for record in &mut expected{record.summary.excluded_roots=diagnostic;}}
    let a=encode(&expected[0])?;let b=encode(&expected[1])?;
    for(root,expected)in[(&roots.config,&a),(&roots.data,&b)]{if read(root,COMPLETE)?.is_none(){claim.current()?;root.publish_new(COMPLETE,expected)?;}}
    review.ensure_current()?;staging::verify_publication(&review,&roots.payload(Area::Config)?,&roots.payload(Area::Data)?)?;
    admit(&claim,&roots,&a,&b)?;Active::from_roots(claim,roots)
}
/// Metadata state only, no native claim or data/write permission is returned.
pub(crate)fn inspect(layout:&Layout)->Result<State>{
    let a=missing(layout.config())?;let b=missing(layout.data())?;
    if a&&b{return Ok(State::Absent);}if a||b{return Ok(State::Incomplete);}
    let config=Dir::root(layout.config())?;let data=Dir::root(layout.data())?;
    if read(&config,HEADER)?.is_none()||read(&data,HEADER)?.is_none(){return Ok(State::Incomplete);}
    if read(&config,ADMITTED)?.is_some(){Ok(State::AdmissionRecordPresent)}else{Ok(State::Incomplete)}
}

#[cfg(test)]#[path="publication_tests.rs"]mod tests;
#[path="publication_resume.rs"]mod resume;
pub(crate)fn recover(claim:Claim,layout:Layout)->Result<Active>{resume::recover(claim,layout)}
