//! Portable private copy candidates for legacy data, not live profiles. Reopen
//! verifies data only: it cannot recreate Prepared, a native review or consent.
use super::{inventory::{self, Area, Entry}, private_fs::{self, Node, Location}, staging_fs::{self, Dir}, Error, Layout, LegacyReview, Result, Summary};
use serde::{Deserialize, Serialize};
use std::{ffi::{OsStr, OsString}, io::{Read, Write}, os::unix::{ffi::{OsStrExt, OsStringExt}, fs::MetadataExt}, path::{Component, Path, PathBuf}};
use ring::{digest::{Context, SHA256}, rand::{SecureRandom, SystemRandom}};

const PROTOCOL: &[u8] = b"{\"version\":1,\"kind\":\"coucou-legacy30-copy-candidate\"}\n";
const MAX_MANIFEST: usize = 16 * 1024 * 1024;
/// An ID locates possible partial work; it is not proof of ownership, success,
/// durability or a right to adopt/remove an existing directory.
#[derive(Debug)] pub(crate) struct Failure { pub error: Error, pub candidate_id: Option<String> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub(crate) enum Phase { Incomplete, VerifiedData }
pub(crate) struct Inspection { pub phase: Phase, pub expected: Summary, source: String }
/// Issued only by this live preparation. A verified on-disk candidate cannot
/// reconstruct this object, authorize publication or reactivate native work.
pub(crate) struct Prepared { review: LegacyReview, job: Dir, id: String, proof: String }
#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct Intent { version: u32, id: String, source: String, summary: Summary }
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct Record { area: Area, path: Vec<u8>, bytes: u64, sha256: Option<String> }
#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct Manifest { version: u32, id: String, source: String, entries: Vec<Record> }
#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct Ready { version: u32, id: String, manifest: String }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] enum Step { Intent, Node(usize), Chunk(usize, u64), Manifest, Ready }
#[derive(Clone, Copy, PartialEq, Eq)] enum Completion { Required, BeforeReady }

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b|format!("{b:02x}")).collect() }
fn digest(bytes: &[u8]) -> String { hex(ring::digest::digest(&SHA256, bytes).as_ref()) }
fn valid_hex(s: &str, len: usize) -> bool { s.len() == len && s.bytes().all(|c|c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) }
fn job_name(id: &str) -> Result<String> { if valid_hex(id,32) { Ok(format!("legacy-profile-{id}")) } else { Err(Error::UnsafePath) } }
fn area_name(area: Area) -> &'static str { match area { Area::Config=>"config", Area::Data=>"data" } }
fn source_root(layout: &Layout, area: Area) -> &Path { match area { Area::Config=>&layout.legacy_config, Area::Data=>&layout.legacy_data } }
fn disjoint_location(workspace:Location,source:Location)->Result<()>{
    // On the same filesystem, distinct lexical paths are only usable when
    // they share a mount and are different directories. Refuse bind aliases
    // rather than assuming that a different path proves physical separation.
    if workspace.device==source.device&&(workspace.mount!=source.mount||workspace.inode==source.inode){Err(Error::UnsafePath)}else{Ok(())}
}
fn record(entry: &Entry) -> Record {
    Record { area: entry.area, path: entry.relative.as_os_str().as_bytes().to_vec(), bytes: if entry.digest.is_some() { entry.stamp.len() } else { 0 }, sha256: entry.digest.as_ref().map(|h| hex(h)) }
}
pub(super) fn content_fingerprint(review:&LegacyReview)->Result<String>{
    Ok(digest(&serde_json::to_vec(&review.snapshot.entries.iter().map(record).collect::<Vec<_>>()).map_err(|_|Error::Unavailable)?))
}
pub(super) fn copy_publication(review:&LegacyReview,config:&Dir,data:&Dir,mut current:impl FnMut(usize)->Result<()>)->Result<()> {
    for(index,entry)in review.snapshot.entries.iter().enumerate(){
        current(index)?;copy(review,entry,index,match entry.area{Area::Config=>config,Area::Data=>data},&mut |_|Ok(()))?;
    }
    config.sync()?;data.sync()?;review.ensure_current()
}
pub(super) fn verify_publication(review:&LegacyReview,config:&Dir,data:&Dir)->Result<Summary>{
    let mut actual=vec![];let mut summary=Summary{files:0,directories:0,bytes:0,excluded_roots:review.summary().excluded_roots};let mut paths=0;let mut proof=Context::new(&SHA256);
    for(area,root)in[(Area::Config,config),(Area::Data,data)]{
        payload(&root.node()?,area,Path::new(""),&mut actual,&mut summary,&mut paths,&mut proof)?;
    }
    if actual!=review.snapshot.entries.iter().map(record).collect::<Vec<_>>()||summary!=review.summary(){return Err(Error::SourceChanged);}
    review.ensure_current()?;Ok(summary)
}
fn area_key(area: Area) -> u8 { match area { Area::Config => 0, Area::Data => 1 } }

fn source_prefix(review: &LegacyReview, entry: &Entry, len: u64) -> Result<String> {
    let node = source(review, entry)?;
    if node.directory || len > node.len { return Err(Error::SourceChanged); }
    let mut input = (&node.file).take(len);
    let mut hash = Context::new(&SHA256); let mut total = 0;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = input.read(&mut buffer).map_err(|_| Error::Unavailable)?;
        if n == 0 { break; } total += n as u64; hash.update(&buffer[..n]);
    }
    node.current()?;
    if total != len { return Err(Error::SourceChanged); }
    Ok(hex(hash.finish().as_ref()))
}

/// Validate the entire existing subset before writing any missing data. A
/// persisted content intent is checked by the caller under its native Claim.
pub(super) fn resume_publication(review: &LegacyReview, config: &Dir, data: &Dir,
    mut current: impl FnMut(usize) -> Result<()>) -> Result<()> {
    let mut actual = vec![]; let mut paths = 0; let mut proof = Context::new(&SHA256);
    let mut summary = Summary { files: 0, directories: 0, bytes: 0, excluded_roots: 0 };
    for (area, root) in [(Area::Config, config), (Area::Data, data)] {
        payload(&root.node()?, area, Path::new(""), &mut actual, &mut summary, &mut paths, &mut proof)?;
    }
    let expected: std::collections::BTreeMap<_, _> = review.snapshot.entries.iter()
        .map(|entry| ((area_key(entry.area), entry.relative.as_os_str().as_bytes().to_vec()), entry)).collect();
    for found in actual {
        let entry = expected.get(&(area_key(found.area), found.path)).ok_or(Error::SourceChanged)?;
        match (&found.sha256, &entry.digest) {
            (None, None) => {},
            (Some(hash), Some(_)) if found.bytes <= entry.stamp.len()
                && *hash == source_prefix(review, entry, found.bytes)? => {},
            _ => return Err(Error::SourceChanged),
        }
    }
    review.ensure_current()?; current(usize::MAX)?;
    for (index, entry) in review.snapshot.entries.iter().enumerate() {
        current(index)?;
        let target = match entry.area { Area::Config => config, Area::Data => data };
        resume_entry(review, entry, index, target)?;
    }
    config.sync()?; data.sync()?; review.ensure_current()
}

fn resume_entry(review: &LegacyReview, entry: &Entry, index: usize, root: &Dir) -> Result<()> {
    use std::os::fd::AsRawFd;
    let target = parent(root, &entry.relative)?;
    let name = entry.relative.file_name().ok_or(Error::UnsafePath)?;
    let Some(existing) = target.node()?.child(name)? else {
        return copy(review, entry, index, root, &mut |_| Ok(()));
    };
    if entry.digest.is_none() {
        if !existing.directory { return Err(Error::SourceChanged); }
        return target.dir(name)?.check();
    }
    if existing.directory || !staging_fs::private_file(&existing.file.metadata().map_err(|_| Error::Unavailable)?)
        || existing.len > entry.stamp.len() { return Err(Error::SourceChanged); }
    let original = source(review, entry)?;
    let mut input = &original.file; let mut previous = &existing.file;
    let mut hash = Context::new(&SHA256); let mut done = 0;
    let mut buffer = [0u8; 64 * 1024]; let mut prefix = [0u8; 64 * 1024];
    while done < existing.len {
        let n = (existing.len - done).min(buffer.len() as u64) as usize;
        input.read_exact(&mut buffer[..n]).map_err(|_| Error::SourceChanged)?;
        previous.read_exact(&mut prefix[..n]).map_err(|_| Error::SourceChanged)?;
        if buffer[..n] != prefix[..n] { return Err(Error::SourceChanged); }
        hash.update(&buffer[..n]); done += n as u64;
    }
    original.current()?; existing.current()?; target.confirm(name, &existing.file)?;
    if done == original.len {
        if entry.digest.as_ref().is_none_or(|expected| hash.finish().as_ref() != expected) { return Err(Error::SourceChanged); }
        return Ok(());
    }
    // Open our pinned, verified inode for append. Never truncate or replace it.
    let mut output = std::fs::OpenOptions::new().append(true)
        .open(format!("/proc/self/fd/{}", existing.file.as_raw_fd())).map_err(|_| Error::Unavailable)?;
    existing.current()?; target.confirm(name, &output)?; original.current()?;
    loop {
        let n = input.read(&mut buffer).map_err(|_| Error::Unavailable)?;
        if n == 0 { break; }
        done += n as u64;
        if done > original.len { return Err(Error::SourceChanged); }
        output.write_all(&buffer[..n]).map_err(|_| Error::Unavailable)?; hash.update(&buffer[..n]);
    }
    original.current()?;
    if done != original.len || entry.digest.as_ref().is_none_or(|expected| hash.finish().as_ref() != expected) { return Err(Error::SourceChanged); }
    output.sync_all().map_err(|_| Error::Unavailable)?; target.confirm(name, &output)?; target.sync()
}
fn relative(bytes: &[u8]) -> Result<PathBuf> {
    if bytes.is_empty() || bytes.len()>inventory::MAX_PATH || bytes.contains(&0) { return Err(Error::UnsafePath); }
    let path = PathBuf::from(OsString::from_vec(bytes.to_vec()));
    if path.is_absolute() || path.components().count()>inventory::MAX_DEPTH || !path.components().all(|c|matches!(c,Component::Normal(_))) { return Err(Error::UnsafePath); }
    // Reject aliases such as repeated separators rather than normalizing input.
    let rebuilt: PathBuf = path.components().map(|c| c.as_os_str()).collect();
    if rebuilt.as_os_str().as_bytes()!=bytes { return Err(Error::UnsafePath); }
    Ok(path)
}
fn source(review: &LegacyReview, entry: &Entry) -> Result<Node> {
    let mut node=Node::root(source_root(&review.layout,entry.area))?.ok_or(Error::SourceChanged)?;
    for c in entry.relative.components() {
        let Component::Normal(name)=c else {return Err(Error::UnsafePath)};
        node=node.child(name)?.ok_or(Error::SourceChanged)?;
    }
    if node.initial!=entry.stamp || node.directory!=entry.digest.is_none() {return Err(Error::SourceChanged);}
    Ok(node)
}
fn parent(root:&Dir,path:&Path)->Result<Dir> {
    let mut at=root.clone();
    for c in path.parent().ok_or(Error::UnsafePath)?.components() {
        let Component::Normal(name)=c else {return Err(Error::UnsafePath)}; at=at.dir(name)?;
    }
    Ok(at)
}
fn copy(review:&LegacyReview,entry:&Entry,index:usize,target:&Dir,step:&mut impl FnMut(Step)->Result<()>)->Result<()> {
    let source=source(review,entry)?; let target=parent(target,&entry.relative)?;
    let name=entry.relative.file_name().ok_or(Error::UnsafePath)?;
    if source.directory {target.mkdir(name)?;return Ok(());}
    let mut output=target.new_file(name)?;
    let mut input=(&source.file).take(source.len+1); let mut hash=Context::new(&SHA256);let mut bytes=0u64;let mut buffer=[0u8;64*1024];
    loop {
        let n=input.read(&mut buffer).map_err(|_|Error::Unavailable)?;if n==0{break;}
        bytes+=n as u64;if bytes>source.len{return Err(Error::SourceChanged);}
        output.write_all(&buffer[..n]).map_err(|_|Error::Unavailable)?;hash.update(&buffer[..n]);step(Step::Chunk(index,bytes))?;
    }
    source.current()?;
    if bytes!=source.len || entry.digest.as_ref().is_none_or(|expected|hash.finish().as_ref()!=expected){return Err(Error::SourceChanged);}
    output.sync_all().map_err(|_|Error::Unavailable)?;target.confirm(name,&output)?;target.sync()
}

pub(crate) fn prepare(review:LegacyReview,workspace:&Path)->std::result::Result<Prepared,Failure> {prepare_using(review,workspace,|_|Ok(()))}
fn prepare_using(review:LegacyReview,workspace:&Path,mut step:impl FnMut(Step)->Result<()>)->std::result::Result<Prepared,Failure> {
    let mut candidate=None;
    let result=(||->Result<Prepared>{
        if !super::absolute(workspace) || [&review.layout.legacy_config,&review.layout.legacy_data].into_iter().any(|p|super::overlaps(workspace,p)){return Err(Error::UnsafePath);}
        review.ensure_current()?;let workspace=Dir::root(workspace)?;
        let workspace_location=private_fs::location(&workspace.node()?.file)?;
        for area in [Area::Config,Area::Data]{
            if let Some(root)=Node::root(source_root(&review.layout,area))?{disjoint_location(workspace_location,private_fs::location(&root.file)?)?;}
        }
        let mut nonce=[0u8;16];SystemRandom::new().fill(&mut nonce).map_err(|_|Error::Unavailable)?;let id=hex(&nonce);
        let source=review.snapshot.fingerprint()?;
        let manifest=Manifest{version:1,id:id.clone(),source:source.clone(),entries:review.snapshot.entries.iter().map(record).collect()};
        let bytes=serde_json::to_vec(&manifest).map_err(|_|Error::UnsupportedProfile)?;
        if bytes.len()>MAX_MANIFEST{return Err(Error::Limit);}
        let intent=Intent{version:1,id:id.clone(),source,summary:review.summary()};
        let intent=serde_json::to_vec(&intent).map_err(|_|Error::UnsupportedProfile)?;
        let name=job_name(&id)?;
        // Keep the ID even when durability/acknowledgement is uncertain. Never
        // delete partial data or invent a replacement job on a failed return.
        candidate=Some(id.clone());let job=workspace.mkdir(OsStr::new(&name))?;
        job.write(".protocol.json",PROTOCOL)?;job.write("intent.json",&intent)?;step(Step::Intent)?;
        let config=job.mkdir(OsStr::new("config"))?;let data=job.mkdir(OsStr::new("data"))?;
        for(index,entry)in review.snapshot.entries.iter().enumerate(){
            copy(&review,entry,index,match entry.area{Area::Config=>&config,Area::Data=>&data},&mut step)?;step(Step::Node(index))?;
        }
        config.sync()?;data.sync()?;review.ensure_current()?;
        job.write("manifest.json",&bytes)?;step(Step::Manifest)?;
        // Read back the full private payload before publishing completion.
        // The preflight mode is native/private and never returned by inspect.
        audit_using(&job,&id,Completion::BeforeReady)?;review.ensure_current()?;
        let ready=serde_json::to_vec(&Ready{version:1,id:id.clone(),manifest:digest(&bytes)}).map_err(|_|Error::UnsupportedProfile)?;
        job.write("ready.json",&ready)?;job.sync()?;workspace.sync()?;step(Step::Ready)?;
        let(audit,proof)=audit(&job,&id)?;
        if audit.phase!=Phase::VerifiedData || audit.source!=review.snapshot.fingerprint()?{return Err(Error::SourceChanged);}
        review.ensure_current()?;
        Ok(Prepared{review,job,id,proof})
    })();
    result.map_err(|error|Failure{error,candidate_id:candidate})
}
impl Prepared {
    pub(crate) fn id(&self)->&str{&self.id}
    pub(crate) fn verify(&self)->Result<Summary>{
        self.review.ensure_current()?;let(inspection,proof)=audit(&self.job,&self.id)?;
        if inspection.phase!=Phase::VerifiedData || proof!=self.proof || inspection.source!=self.review.snapshot.fingerprint()?{return Err(Error::SourceChanged);}
        self.review.ensure_current()?;Ok(inspection.expected)
    }
}
/// Read-only data inspection after restart, with no original ownership/admission
/// object. Even VerifiedData is never a publication, restore or execution grant.
pub(crate) fn inspect(workspace:&Path,id:&str)->Result<Inspection>{
    let root=Dir::root(workspace)?;let job=root.dir(OsStr::new(&job_name(id)?))?;audit(&job,id).map(|a|a.0)
}

fn bytes(node:&Node,limit:usize)->Result<Vec<u8>>{
    if node.directory || node.len>limit as u64 || !staging_fs::private_file(&node.file.metadata().map_err(|_|Error::Unavailable)?){return Err(Error::UnsafePath);}
    let mut bytes=vec![];(&node.file).take(limit as u64+1).read_to_end(&mut bytes).map_err(|_|Error::Unavailable)?;node.current()?;
    if bytes.len()>limit || bytes.len()as u64!=node.len{return Err(Error::Limit);}Ok(bytes)
}
fn control(job:&Node,name:&str,limit:usize,proof:&mut Context)->Result<Option<Vec<u8>>>{
    let Some(node)=job.child(OsStr::new(name))?else{return Ok(None)};
    let content=bytes(&node,limit)?;proof.update(name.as_bytes());proof.update(&serde_json::to_vec(&node.initial).map_err(|_|Error::Unavailable)?);proof.update(digest(&content).as_bytes());
    job.same_child(OsStr::new(name),&node)?;Ok(Some(content))
}
fn allowed(record:&Record,path:&Path)->bool{
    let parts:Vec<_>=path.components().map(|c|c.as_os_str()).collect();let directory=record.sha256.is_none();
    match(record.area,parts[0].to_str()){
        (Area::Config,Some("settings.json"|"pets.json"|"sources.json"))=>parts.len()==1&&!directory,
        (Area::Config,Some("sounds"))=>parts.len()>1||directory,
        (Area::Data,Some("recap.json"))=>parts.len()==1&&!directory,
        (Area::Data,Some("Flow"|"Notes"))=>parts.len()>1||directory,
        (Area::Data,Some("inbox"))=>(parts.len()>1||directory)&&parts.get(1).is_none_or(|n|*n!=".coucou-purge-v1"),
        (Area::Data,Some("assistant"))=>match parts.len(){
            _ if parts.get(1).is_some_and(|name|inventory::legacy_history_backup(name))=>parts.len()>2||directory,
            1=>directory,
            2 if parts[1]=="conversations.json"=>!directory,
            2 if parts[1]=="metrics"=>directory,
            3 if parts[1]=="metrics"&&parts[2]=="ollama-usage-v1.json"=>!directory,
            _=>false,
        },
        _=>false,
    }
}
fn payload(node:&Node,area:Area,relative:&Path,records:&mut Vec<Record>,summary:&mut Summary,path_bytes:&mut usize,proof:&mut Context)->Result<()> {
    if !node.directory || node.file.metadata().map_err(|_|Error::Unavailable)?.mode()&0o7777!=0o700{return Err(Error::UnsafePath);}
    proof.update(&serde_json::to_vec(&node.initial).map_err(|_|Error::Unavailable)?);
    for name in node.names(inventory::MAX_NODES-records.len())?{
        if records.len()>=inventory::MAX_NODES{return Err(Error::Limit);}
        let path=relative.join(&name);let raw=path.as_os_str().as_bytes();relative_path(raw,path_bytes)?;
        let child=node.child(&name)?.ok_or(Error::SourceChanged)?;
        let mut record=Record{area,path:raw.to_vec(),bytes:0,sha256:None};
        if !child.directory{
            if !staging_fs::private_file(&child.file.metadata().map_err(|_|Error::Unavailable)?) || child.len>inventory::MAX_FILE || summary.bytes.checked_add(child.len).is_none_or(|n|n>inventory::MAX_BYTES){return Err(Error::Limit);}
            let mut input=(&child.file).take(child.len+1);let mut hash=Context::new(&SHA256);let mut total=0u64;let mut buffer=[0u8;64*1024];
            loop{let n=input.read(&mut buffer).map_err(|_|Error::Unavailable)?;if n==0{break;}total+=n as u64;if total>child.len{return Err(Error::SourceChanged);}hash.update(&buffer[..n]);}
            child.current()?;if total!=child.len{return Err(Error::SourceChanged);}
            record.bytes=total;record.sha256=Some(hex(hash.finish().as_ref()));summary.bytes+=total;summary.files+=1;
            proof.update(&serde_json::to_vec(&child.initial).map_err(|_|Error::Unavailable)?);
        }else{summary.directories+=1;}
        if !allowed(&record,&path){return Err(Error::UnsupportedProfile);}
        records.push(record);
        if child.directory{payload(&child,area,&path,records,summary,path_bytes,proof)?;}
        node.same_child(&name,&child)?;
    }
    node.current()
}
fn relative_path(raw:&[u8],total:&mut usize)->Result<()> {
    relative(raw)?;*total=total.checked_add(raw.len()).ok_or(Error::Limit)?;
    if *total>inventory::MAX_PATH_TOTAL{return Err(Error::Limit);}Ok(())
}
fn audit(job:&Dir,id:&str)->Result<(Inspection,String)>{
    audit_using(job,id,Completion::Required)
}
fn audit_using(job:&Dir,id:&str,completion:Completion)->Result<(Inspection,String)>{
    let node=job.node()?;let names=node.names(6)?;
    if names.iter().any(|n|!matches!(n.to_str(),Some(".protocol.json"|"intent.json"|"config"|"data"|"manifest.json"|"ready.json"))){return Err(Error::UnsupportedProfile);}
    let mut proof=Context::new(&SHA256);proof.update(b"coucou-copy-candidate-physical-v1\0");proof.update(&serde_json::to_vec(&node.initial).map_err(|_|Error::Unavailable)?);
    if control(&node,".protocol.json",1024,&mut proof)?.as_deref()!=Some(PROTOCOL){return Err(Error::UnsupportedProfile);}
    let intent:Intent=serde_json::from_slice(&control(&node,"intent.json",2048,&mut proof)?.ok_or(Error::UnsupportedProfile)?).map_err(|_|Error::UnsupportedProfile)?;
    if intent.version!=1 || intent.id!=id || !valid_hex(&intent.source,64) || intent.summary.files.checked_add(intent.summary.directories).is_none_or(|n|n>inventory::MAX_NODES) || intent.summary.bytes>inventory::MAX_BYTES || intent.summary.excluded_roots>512{return Err(Error::UnsupportedProfile);}
    let ready=control(&node,"ready.json",2048,&mut proof)?;
    if ready.is_none()&&completion==Completion::Required{node.current()?;job.check()?;return Ok((Inspection{phase:Phase::Incomplete,expected:intent.summary,source:intent.source},hex(proof.finish().as_ref())));}
    let ready:Option<Ready>=ready.map(|b|serde_json::from_slice(&b).map_err(|_|Error::UnsupportedProfile)).transpose()?;
    let manifest=control(&node,"manifest.json",MAX_MANIFEST,&mut proof)?.ok_or(Error::UnsupportedProfile)?;
    if ready.is_some_and(|r|r.version!=1 || r.id!=id || r.manifest!=digest(&manifest)){return Err(Error::UnsupportedProfile);}
    let manifest:Manifest=serde_json::from_slice(&manifest).map_err(|_|Error::UnsupportedProfile)?;
    if manifest.version!=1 || manifest.id!=id || manifest.source!=intent.source || manifest.entries.len()>inventory::MAX_NODES{return Err(Error::UnsupportedProfile);}
    let mut records=vec![];let mut summary=Summary{files:0,directories:0,bytes:0,excluded_roots:intent.summary.excluded_roots};let mut paths=0;
    for area in [Area::Config,Area::Data]{
        let name=OsStr::new(area_name(area));let root=node.child(name)?.ok_or(Error::UnsupportedProfile)?;
        payload(&root,area,Path::new(""),&mut records,&mut summary,&mut paths,&mut proof)?;node.same_child(name,&root)?;
    }
    if records!=manifest.entries || summary!=intent.summary{return Err(Error::SourceChanged);}
    node.current()?;job.check()?;
    Ok((Inspection{phase:Phase::VerifiedData,expected:summary,source:intent.source},hex(proof.finish().as_ref())))
}

#[cfg(test)] #[path="staging_tests.rs"] mod tests;
