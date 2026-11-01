//! Portable private copy candidates for legacy data, not live profiles. Reopen
//! verifies data only: it cannot recreate Prepared, a native review or consent.
use super::{inventory::{self, Area, Entry}, private_fs::{self, Node, Location}, staging_fs::{self, Dir}, Error, LegacyReview, Result, Summary};
use serde::{Deserialize, Serialize};
use std::{ffi::{OsStr, OsString}, io::{Read, Write}, os::unix::{ffi::{OsStrExt, OsStringExt}, fs::MetadataExt}, path::{Component, Path, PathBuf}};
use ring::{digest::{Context, SHA256}, rand::{SecureRandom, SystemRandom}};

const PROTOCOL: &[u8] = b"{\"version\":1,\"kind\":\"coucou-legacy30-copy-candidate\"}\n";
const MAX_MANIFEST: usize = 16 * 1024 * 1024;
/// An ID locates possible partial work; it is not proof of ownership, success,
/// durability or a right to adopt/remove an existing directory.
#[derive(Debug)] pub(crate) struct Failure { pub error: Error, pub candidate_id: Option<String> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub(crate) enum Phase { Incomplete, VerifiedData }
pub(crate) struct Inspection { pub phase: Phase, pub expected: Summary, pub archive_version:u32, pub includes_recovery_metadata:bool, pub source_payload_depth:Option<u8>, source: String }
/// Issued only by this live preparation. A verified on-disk candidate cannot
/// reconstruct this object, authorize publication or reactivate native work.
pub(super) struct Prepared<R:Source=LegacyReview> { review: R, job: Dir, id: String, proof: String }
#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct Intent { version: u32, id: String, source: String, summary: Summary, #[serde(default,skip_serializing_if="Option::is_none")]origin_hash:Option<String> }
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct Record { area: Area, path: Vec<u8>, bytes: u64, sha256: Option<String> }
#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct Manifest { version: u32, id: String, source: String, entries: Vec<Record>, #[serde(default,skip_serializing_if="Option::is_none")]origin:Option<super::publication::BackupOrigin> }
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
/// Native source values retain their ownership/currentness; manifest JSON
/// never implements this interface or creates a publication/execution permit.
pub(super)trait Source {
    fn snapshot(&self)->&inventory::Snapshot;
    fn source_root(&self,area:Area)->&Path;
    fn ensure_current(&self)->Result<()>;
    fn kind(&self)->Kind{Kind::Legacy}
    fn origin(&self)->Option<super::publication::BackupOrigin>{None}
    fn workspace(&self,path:&Path)->Result<Dir>{Dir::root(path)}
    fn summary(&self)->Summary{self.snapshot().summary}
}
impl Source for LegacyReview {
    fn snapshot(&self)->&inventory::Snapshot{&self.snapshot}
    fn source_root(&self,area:Area)->&Path{match area{Area::Config=>&self.layout.legacy_config,Area::Data=>&self.layout.legacy_data}}
    fn ensure_current(&self)->Result<()>{LegacyReview::ensure_current(self)}
}
#[derive(Clone,Copy,PartialEq,Eq)]pub(super)enum Kind{Legacy,Current,Restored}
impl Kind {
    pub(super)fn version(self)->u32{if self==Self::Restored{2}else{1}}
    fn protocol(self)->&'static[u8]{match self{Self::Legacy=>PROTOCOL,Self::Current=>b"{\"version\":1,\"kind\":\"coucou-current-profile-backup\"}\n",Self::Restored=>b"{\"version\":2,\"kind\":\"coucou-current-profile-backup\"}\n"}}
    fn name(self,id:&str)->Result<String>{if !valid_hex(id,32){return Err(Error::UnsafePath);}Ok(format!("{}-{id}",match self{Self::Legacy=>"legacy-profile",Self::Current|Self::Restored=>"current-profile"}))}
}
fn disjoint_location(workspace:Location,source:Location)->Result<()>{
    // On the same filesystem, distinct lexical paths are only usable when
    // they share a mount and are different directories. Refuse bind aliases
    // rather than assuming that a different path proves physical separation.
    if workspace.device==source.device&&(workspace.mount!=source.mount||workspace.inode==source.inode){Err(Error::UnsafePath)}else{Ok(())}
}
fn record(entry: &Entry) -> Record {
    Record { area: entry.area, path: entry.relative.as_os_str().as_bytes().to_vec(), bytes: if entry.digest.is_some() { entry.stamp.len() } else { 0 }, sha256: entry.digest.as_ref().map(|h| hex(h)) }
}
pub(super) fn content_fingerprint(review:&impl Source)->Result<String>{
    snapshot_content_fingerprint(review.snapshot())
}
pub(super)fn snapshot_content_fingerprint(snapshot:&inventory::Snapshot)->Result<String>{Ok(digest(&serde_json::to_vec(&snapshot.entries.iter().map(record).collect::<Vec<_>>()).map_err(|_|Error::Unavailable)?))}
pub(super)fn verify_original_content(config:&Path,data:&Path,expected:&str)->Result<()>{let snapshot=inventory::inspect_current(config,data)?;if snapshot_content_fingerprint(&snapshot)?!=expected{return Err(Error::SourceChanged);}if inventory::inspect_current(config,data)?!=snapshot{return Err(Error::SourceChanged);}Ok(())}
pub(super) fn copy_publication(review:&impl Source,config:&Dir,data:&Dir,mut current:impl FnMut(usize)->Result<()>)->Result<()> {
    for(index,entry)in review.snapshot().entries.iter().enumerate(){
        current(index)?;copy(review,entry,index,match entry.area{Area::Config=>config,Area::Data=>data},&mut |_|Ok(()))?;
    }
    config.sync()?;data.sync()?;review.ensure_current()
}
pub(super) fn verify_publication(review:&impl Source,config:&Dir,data:&Dir)->Result<Summary>{
    verify_publication_delta(review,config,data,&[])
}
/// Only the typed native controller supplies exact metadata deltas. An old
/// content receipt or caller JSON never chooses files to ignore/replace.
pub(super)fn verify_publication_delta(review:&impl Source,config:&Dir,data:&Dir,delta:&[(Area,PathBuf,Vec<u8>)])->Result<Summary>{
    verify_publication_records(review,config,data,delta,vec![])
}
pub(super)fn verify_publication_independent(review:&impl Source,config:&Dir,data:&Dir,delta:&[(Area,PathBuf,Vec<u8>)],addition:&super::publication::restore::independent::operational::PreparedAuthority)->Result<Summary>{
 let added=addition.records(config)?.into_iter().map(|(path,b)|Record{area:Area::Config,path:path.as_os_str().as_bytes().to_vec(),bytes:b.as_ref().map_or(0,|b|b.len()as u64),sha256:b.as_deref().map(digest)}).collect();verify_publication_records(review,config,data,delta,added)
}
fn verify_publication_records(review:&impl Source,config:&Dir,data:&Dir,delta:&[(Area,PathBuf,Vec<u8>)],added:Vec<Record>)->Result<Summary>{
    let mut actual=vec![];let mut summary=Summary{files:0,directories:0,bytes:0,excluded_roots:review.summary().excluded_roots};let mut paths=0;let mut proof=Context::new(&SHA256);
    for(area,root)in[(Area::Config,config),(Area::Data,data)]{
        payload(&root.node()?,area,Path::new(""),&mut actual,&mut summary,&mut paths,&mut proof,review.kind())?;
    }
    let mut expected=review.snapshot().entries.iter().map(record).collect::<Vec<_>>();let mut seen=std::collections::BTreeSet::new();
    for(area,path,bytes)in delta{
        let raw=path.as_os_str().as_bytes();relative(raw)?;if !seen.insert((area_key(*area),raw.to_vec()))||bytes.len()>inventory::MAX_FILE as usize{return Err(Error::UnsafePath);}
        let value=Record{area:*area,path:raw.to_vec(),bytes:bytes.len()as u64,sha256:Some(digest(bytes))};
        if let Some(old)=expected.iter_mut().find(|r|r.area==*area&&r.path==raw){if old.sha256.is_none(){return Err(Error::UnsafePath);}*old=value;}else{expected.push(value);}
    }
    for r in added{if expected.iter().any(|e|e.area==r.area&&e.path==r.path){return Err(Error::SourceChanged);}expected.push(r);}
    let sort=|v:&mut Vec<Record>|v.sort_by(|a,b|(area_key(a.area),&a.path).cmp(&(area_key(b.area),&b.path)));sort(&mut actual);sort(&mut expected);
    if actual!=expected{return Err(Error::SourceChanged);}
    let expected_files=expected.iter().filter(|r|r.sha256.is_some()).count();let expected_bytes=expected.iter().map(|r|r.bytes).sum::<u64>();
    if summary.files!=expected_files||summary.directories!=expected.len()-expected_files||summary.bytes!=expected_bytes{return Err(Error::SourceChanged);}
    review.ensure_current()?;Ok(summary)
}
fn area_key(area: Area) -> u8 { match area { Area::Config => 0, Area::Data => 1 } }

fn source_prefix(review: &impl Source, entry: &Entry, len: u64) -> Result<String> {
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
pub(super) fn resume_publication(review: &impl Source, config: &Dir, data: &Dir,
    mut current: impl FnMut(usize) -> Result<()>) -> Result<()> {
    let mut actual = vec![]; let mut paths = 0; let mut proof = Context::new(&SHA256);
    let mut summary = Summary { files: 0, directories: 0, bytes: 0, excluded_roots: 0 };
    for (area, root) in [(Area::Config, config), (Area::Data, data)] {
        payload(&root.node()?, area, Path::new(""), &mut actual, &mut summary, &mut paths, &mut proof,review.kind())?;
    }
    let expected: std::collections::BTreeMap<_, _> = review.snapshot().entries.iter()
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
    for (index, entry) in review.snapshot().entries.iter().enumerate() {
        current(index)?;
        let target = match entry.area { Area::Config => config, Area::Data => data };
        resume_entry(review, entry, index, target)?;
    }
    config.sync()?; data.sync()?; review.ensure_current()
}

fn resume_entry(review: &impl Source, entry: &Entry, index: usize, root: &Dir) -> Result<()> {
    use std::os::fd::AsRawFd;
    let target = parent(root, &entry.relative)?;
    let name = entry.relative.file_name().ok_or(Error::UnsafePath)?;
    let Some(existing) = target.node()?.child(name)? else {
        return copy(review, entry, index, root, &mut |_| Ok(()));
    };
    if entry.digest.is_none() {
        if !existing.directory { return Err(Error::SourceChanged); }
        // A killed writer may have created this directory before its sync.
        let parent=target.node()?;parent.same_child(name,&existing)?;target.dir(name)?.sync()?;parent.same_child(name,&existing)?;return target.sync();
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
        // Full matching bytes can belong to a writer killed before fsync.
        // Do not publish completion from page-cache verification alone.
        existing.file.sync_all().map_err(|_|Error::Unavailable)?;existing.current()?;target.confirm(name,&existing.file)?;return target.sync();
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
fn source(review: &impl Source, entry: &Entry) -> Result<Node> {
    let mut node=Node::root(review.source_root(entry.area))?.ok_or(Error::SourceChanged)?;
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
fn copy(review:&impl Source,entry:&Entry,index:usize,target:&Dir,step:&mut impl FnMut(Step)->Result<()>)->Result<()> {
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

pub(super) fn prepare<R:Source>(review:R,workspace:&Path)->std::result::Result<Prepared<R>,Failure> {prepare_using(review,workspace,|_|Ok(()))}
fn prepare_using<R:Source>(review:R,workspace:&Path,mut step:impl FnMut(Step)->Result<()>)->std::result::Result<Prepared<R>,Failure> {
    let mut candidate=None;
    let result=(||->Result<Prepared<R>>{
        if !super::absolute(workspace) || [review.source_root(Area::Config),review.source_root(Area::Data)].into_iter().any(|p|super::overlaps(workspace,p)){return Err(Error::UnsafePath);}
        review.ensure_current()?;let workspace=review.workspace(workspace)?;
        let workspace_location=private_fs::location(&workspace.node()?.file)?;
        for area in [Area::Config,Area::Data]{
            if let Some(root)=Node::root(review.source_root(area))?{disjoint_location(workspace_location,private_fs::location(&root.file)?)?;}
        }
        let mut nonce=[0u8;16];SystemRandom::new().fill(&mut nonce).map_err(|_|Error::Unavailable)?;let id=hex(&nonce);
        let source=review.snapshot().fingerprint()?;
        let origin=review.origin();let origin_hash=origin.as_ref().map(|o|serde_json::to_vec(o).map(|b|digest(&b)).map_err(|_|Error::Unavailable)).transpose()?;
        let manifest=Manifest{version:review.kind().version(),id:id.clone(),source:source.clone(),entries:review.snapshot().entries.iter().map(record).collect(),origin};
        let bytes=serde_json::to_vec(&manifest).map_err(|_|Error::UnsupportedProfile)?;
        if bytes.len()>MAX_MANIFEST{return Err(Error::Limit);}
        let intent=Intent{version:review.kind().version(),id:id.clone(),source,summary:review.summary(),origin_hash};
        let intent=serde_json::to_vec(&intent).map_err(|_|Error::UnsupportedProfile)?;
        let name=review.kind().name(&id)?;
        // Keep the ID even when durability/acknowledgement is uncertain. Never
        // delete partial data or invent a replacement job on a failed return.
        candidate=Some(id.clone());let job=workspace.mkdir(OsStr::new(&name))?;
        job.write(".protocol.json",review.kind().protocol())?;job.write("intent.json",&intent)?;step(Step::Intent)?;
        let config=job.mkdir(OsStr::new("config"))?;let data=job.mkdir(OsStr::new("data"))?;
        for(index,entry)in review.snapshot().entries.iter().enumerate(){
            copy(&review,entry,index,match entry.area{Area::Config=>&config,Area::Data=>&data},&mut step)?;step(Step::Node(index))?;
        }
        config.sync()?;data.sync()?;review.ensure_current()?;
        job.write("manifest.json",&bytes)?;step(Step::Manifest)?;
        // Read back the full private payload before publishing completion.
        // The preflight mode is native/private and never returned by inspect.
        audit_for(&job,&id,Completion::BeforeReady,review.kind())?;review.ensure_current()?;
        let ready=serde_json::to_vec(&Ready{version:review.kind().version(),id:id.clone(),manifest:digest(&bytes)}).map_err(|_|Error::UnsupportedProfile)?;
        job.write("ready.json",&ready)?;job.sync()?;workspace.sync()?;step(Step::Ready)?;
        let(audit,proof)=audit_for(&job,&id,Completion::Required,review.kind())?;
        if audit.phase!=Phase::VerifiedData || audit.source!=review.snapshot().fingerprint()?{return Err(Error::SourceChanged);}
        review.ensure_current()?;
        Ok(Prepared{review,job,id,proof})
    })();
    result.map_err(|error|Failure{error,candidate_id:candidate})
}
impl<R:Source> Prepared<R> {
    pub(crate) fn id(&self)->&str{&self.id}
    pub(crate) fn verify(&self)->Result<Summary>{
        self.review.ensure_current()?;let(inspection,proof)=audit_for(&self.job,&self.id,Completion::Required,self.review.kind())?;
        if inspection.phase!=Phase::VerifiedData || proof!=self.proof || inspection.source!=self.review.snapshot().fingerprint()?{return Err(Error::SourceChanged);}
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
fn allowed(record:&Record,path:&Path,kind:Kind)->bool{
    if kind!=Kind::Legacy{return true;}
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
fn payload(node:&Node,area:Area,relative:&Path,records:&mut Vec<Record>,summary:&mut Summary,path_bytes:&mut usize,proof:&mut Context,kind:Kind)->Result<()> {
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
        if !allowed(&record,&path,kind){return Err(Error::UnsupportedProfile);}
        records.push(record);
        if child.directory{payload(&child,area,&path,records,summary,path_bytes,proof,kind)?;}
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
    audit_for(job,id,completion,Kind::Legacy)
}
fn audit_for(job:&Dir,id:&str,completion:Completion,kind:Kind)->Result<(Inspection,String)>{
    let node=job.node()?;let names=node.names(6)?;
    if names.iter().any(|n|!matches!(n.to_str(),Some(".protocol.json"|"intent.json"|"config"|"data"|"manifest.json"|"ready.json"))){return Err(Error::UnsupportedProfile);}
    let mut proof=Context::new(&SHA256);proof.update(b"coucou-copy-candidate-physical-v1\0");proof.update(&serde_json::to_vec(&node.initial).map_err(|_|Error::Unavailable)?);
    if control(&node,".protocol.json",1024,&mut proof)?.as_deref()!=Some(kind.protocol()){return Err(Error::UnsupportedProfile);}
    let intent:Intent=serde_json::from_slice(&control(&node,"intent.json",2048,&mut proof)?.ok_or(Error::UnsupportedProfile)?).map_err(|_|Error::UnsupportedProfile)?;
    if intent.version!=kind.version() || intent.id!=id || !valid_hex(&intent.source,64) || intent.summary.files.checked_add(intent.summary.directories).is_none_or(|n|n>inventory::MAX_NODES) || intent.summary.bytes>inventory::MAX_BYTES || intent.summary.excluded_roots>512{return Err(Error::UnsupportedProfile);}
    if (kind!=Kind::Legacy)!=intent.origin_hash.is_some()||intent.origin_hash.as_ref().is_some_and(|h|!valid_hex(h,64)){return Err(Error::UnsupportedProfile);}
    let ready=control(&node,"ready.json",2048,&mut proof)?;
    if ready.is_none()&&completion==Completion::Required{node.current()?;job.check()?;return Ok((Inspection{phase:Phase::Incomplete,expected:intent.summary,archive_version:kind.version(),includes_recovery_metadata:kind==Kind::Restored,source_payload_depth:None,source:intent.source},hex(proof.finish().as_ref())));}
    let ready:Option<Ready>=ready.map(|b|serde_json::from_slice(&b).map_err(|_|Error::UnsupportedProfile)).transpose()?;
    let manifest=control(&node,"manifest.json",MAX_MANIFEST,&mut proof)?.ok_or(Error::UnsupportedProfile)?;
    if ready.is_some_and(|r|r.version!=kind.version() || r.id!=id || r.manifest!=digest(&manifest)){return Err(Error::UnsupportedProfile);}
    let manifest:Manifest=serde_json::from_slice(&manifest).map_err(|_|Error::UnsupportedProfile)?;
    if manifest.version!=kind.version() || manifest.id!=id || manifest.source!=intent.source || manifest.entries.len()>inventory::MAX_NODES{return Err(Error::UnsupportedProfile);}
    let origin_hash=manifest.origin.as_ref().map(|o|serde_json::to_vec(o).map(|b|digest(&b)).map_err(|_|Error::Unavailable)).transpose()?;
    if origin_hash!=intent.origin_hash||(kind!=Kind::Legacy)!=manifest.origin.is_some()||manifest.origin.as_ref().is_some_and(|o|!o.bounded()||o.version()!=kind.version()){return Err(Error::UnsupportedProfile);}
    let mut records=vec![];let mut summary=Summary{files:0,directories:0,bytes:0,excluded_roots:intent.summary.excluded_roots};let mut paths=0;
    for area in [Area::Config,Area::Data]{
        let name=OsStr::new(area_name(area));let root=node.child(name)?.ok_or(Error::UnsupportedProfile)?;
        payload(&root,area,Path::new(""),&mut records,&mut summary,&mut paths,&mut proof,kind)?;node.same_child(name,&root)?;
    }
    if records!=manifest.entries || summary!=intent.summary{return Err(Error::SourceChanged);}
    if kind==Kind::Restored{super::publication::restored_backup::verify(&job.dir(OsStr::new("config"))?,&job.dir(OsStr::new("data"))?,manifest.origin.as_ref().ok_or(Error::UnsupportedProfile)?)?;}
    let source_payload_depth=manifest.origin.as_ref().map(|o|o.source_depth()).transpose()?;
    node.current()?;job.check()?;
    Ok((Inspection{phase:Phase::VerifiedData,expected:summary,archive_version:kind.version(),includes_recovery_metadata:kind==Kind::Restored,source_payload_depth,source:intent.source},hex(proof.finish().as_ref())))
}

#[cfg(test)] #[path="staging_tests.rs"] mod tests;

/// Current archives are data only; the legacy inspector cannot admit them.
fn current_archive_kind(job:&Dir)->Result<Kind>{
 let node=job.node()?;let p=node.child(OsStr::new(".protocol.json"))?.ok_or(Error::UnsupportedProfile)?;let protocol=bytes(&p,1024)?;node.same_child(OsStr::new(".protocol.json"),&p)?;
 if protocol==Kind::Current.protocol(){Ok(Kind::Current)}else if protocol==Kind::Restored.protocol(){Ok(Kind::Restored)}else{Err(Error::UnsupportedProfile)}
}
pub(super)fn inspect_current(workspace:&Path,id:&str)->Result<Inspection>{
 let root=Dir::root(workspace)?;let job=root.dir(OsStr::new(&Kind::Current.name(id)?))?;let kind=current_archive_kind(&job)?;audit_for(&job,id,Completion::Required,kind).map(|a|a.0)
}
/// Fresh descriptor-bound review of a verified current archive. This is a
/// data source only, not the original live profile or its authority/FD lease.
pub(super)struct CurrentArchive{kind:Kind,workspace:Dir,job:Dir,id:String,proof:String,config:PathBuf,data:PathBuf,snapshot:inventory::Snapshot,origin:super::publication::BackupOrigin}
pub(super)struct CopyRevision{workspace:Dir,job:Dir,top:Node,controls:super::revision_fence::Controls,tree:super::revision_fence::Tree}
impl CopyRevision{pub(super)fn current(&self)->Result<()>{self.workspace.check()?;self.job.check()?;self.top.current()?;self.controls.current()?;self.tree.current()}}
impl CurrentArchive{
 /// Compatibility reader for consumers that require a flat payload pair.
 pub(super)fn open(workspace:&Path,id:&str)->Result<Self>{Self::open_kind(workspace,id,Kind::Current)}
 /// Explicit data-restoration reader. The caller must preserve the kind and
 /// publish roots2 under an inactive epoch wrapper, never treat them as payloads1.
 pub(super)fn open_restore(workspace:&Path,id:&str)->Result<Self>{let root=Dir::root(workspace)?;let job=root.dir(OsStr::new(&Kind::Current.name(id)?))?;Self::open_kind(workspace,id,current_archive_kind(&job)?)}
 fn open_kind(workspace:&Path,id:&str,kind:Kind)->Result<Self>{
  let root=Dir::root(workspace)?;let job=root.dir(OsStr::new(&Kind::Current.name(id)?))?;
  let(inspection,proof)=audit_for(&job,id,Completion::Required,kind)?;
  if inspection.phase!=Phase::VerifiedData{return Err(Error::UnsupportedProfile);}
  let node=job.node()?;let raw=bytes(&node.child(OsStr::new("manifest.json"))?.ok_or(Error::SourceChanged)?,MAX_MANIFEST)?;
  let manifest:Manifest=serde_json::from_slice(&raw).map_err(|_|Error::UnsupportedProfile)?;let origin=manifest.origin.ok_or(Error::UnsupportedProfile)?;
  let base=workspace.join(Kind::Current.name(id)?);let config=base.join("config");let data=base.join("data");
  if kind==Kind::Current&&origin.settings_io_protocol.is_none()&&private_fs::Node::root(&config.join(crate::settings::durable::FILE))?.is_some(){return Err(Error::UnsupportedProfile);}
  let mut snapshot=inventory::inspect_current(&config,&data)?;snapshot.summary.excluded_roots=inspection.expected.excluded_roots;
  if snapshot.summary!=inspection.expected{return Err(Error::SourceChanged);}
  let source=Self{kind,workspace:root,job,id:id.into(),proof,config,data,snapshot,origin};source.ensure_current()?;Ok(source)
 }
 /// A revision witness is not an archive, native owner or permission. It is
 /// usable only while these immutable, fully verified source bytes stay current.
 pub(super)fn copy_revision(&self)->Result<CopyRevision>{self.ensure_current()?;let tree=super::revision_fence::Tree::from_snapshot(Dir::root(&self.config)?,Dir::root(&self.data)?,&self.snapshot)?;let controls=super::revision_fence::Controls::capture(&self.job,&[".protocol.json","intent.json","manifest.json","ready.json"])?;let top=self.job.node()?;self.ensure_current()?;let r=CopyRevision{workspace:self.workspace.clone(),job:self.job.clone(),top,controls,tree};r.current()?;Ok(r)}
 pub(super)fn source_depth(&self)->Result<u8>{self.origin.source_depth()}
 pub(super)fn archive_version(&self)->u32{self.kind.version()}
 pub(super)fn fence(&self)->Result<String>{
  self.ensure_current()?;let origin=serde_json::to_vec(&self.origin).map_err(|_|Error::Unavailable)?;
  Ok(digest(&serde_json::to_vec(&(if self.kind==Kind::Current{"coucou-restore-archive-source-v1"}else{"coucou-restore-root-pair-archive-source-v2"},&self.id,&self.proof,self.config.as_os_str().as_bytes(),self.data.as_os_str().as_bytes(),digest(&origin),self.snapshot.fingerprint()?)).map_err(|_|Error::Unavailable)?))
 }
 pub(super)fn id(&self)->&str{&self.id}
 pub(super)fn origin_bytes(&self)->Result<Vec<u8>>{serde_json::to_vec(&self.origin).map_err(|_|Error::Unavailable)}
 pub(super)fn disjoint(&self,path:&Path)->Result<()>{
  if super::overlaps(path,&self.config)||super::overlaps(path,&self.data){return Err(Error::UnsafePath);}
  let target=Dir::parent(path)?;for area in[Area::Config,Area::Data]{let source=Node::root(self.source_root(area))?.ok_or(Error::SourceChanged)?;disjoint_location(private_fs::location(&target.node()?.file)?,private_fs::location(&source.file)?)?;}Ok(())
 }
}
impl Source for CurrentArchive{
 fn snapshot(&self)->&inventory::Snapshot{&self.snapshot}
 fn source_root(&self,area:Area)->&Path{match area{Area::Config=>&self.config,Area::Data=>&self.data}}
 fn ensure_current(&self)->Result<()>{
  self.workspace.check()?;self.job.check()?;let(inspection,proof)=audit_for(&self.job,&self.id,Completion::Required,self.kind)?;
  if inspection.phase!=Phase::VerifiedData||proof!=self.proof||inspection.expected!=self.snapshot.summary{return Err(Error::SourceChanged);}
  let mut current=inventory::inspect_current(&self.config,&self.data)?;current.summary.excluded_roots=self.snapshot.summary.excluded_roots;
  if current!=self.snapshot{return Err(Error::SourceChanged);}self.job.check()
 }
 fn kind(&self)->Kind{self.kind}
 fn origin(&self)->Option<super::publication::BackupOrigin>{Some(self.origin.clone())}
}

/// Re-attest the retained original payload without an external backup mounted.
/// This is data verification only and cannot create native/execution authority.
pub(super)fn verify_retained_delta(original_config:&Path,original_data:&Path,config:&Dir,data:&Dir,delta:&[(Area,PathBuf,Vec<u8>)],expected:&str)->Result<()>{
 verify_retained_with(original_config,original_data,config,data,delta,expected,None)
}
pub(super)fn verify_retained_independent(original_config:&Path,original_data:&Path,config:&Dir,data:&Dir,delta:&[(Area,PathBuf,Vec<u8>)],expected:&str,addition:&super::publication::restore::independent::operational::PreparedAuthority)->Result<()>{verify_retained_with(original_config,original_data,config,data,delta,expected,Some(addition))}
fn verify_retained_with(original_config:&Path,original_data:&Path,config:&Dir,data:&Dir,delta:&[(Area,PathBuf,Vec<u8>)],expected:&str,addition:Option<&super::publication::restore::independent::operational::PreparedAuthority>)->Result<()>{
 struct RetainedSource{config:PathBuf,data:PathBuf,snapshot:inventory::Snapshot}
 impl Source for RetainedSource{
  fn snapshot(&self)->&inventory::Snapshot{&self.snapshot}
  fn kind(&self)->Kind{Kind::Current}
  fn source_root(&self,a:Area)->&Path{match a{Area::Config=>&self.config,Area::Data=>&self.data}}
  fn ensure_current(&self)->Result<()>{if inventory::inspect_current(&self.config,&self.data)?!=self.snapshot{return Err(Error::SourceChanged);}Ok(())}
 }
 let source=RetainedSource{config:original_config.into(),data:original_data.into(),snapshot:inventory::inspect_current(original_config,original_data)?};
 if content_fingerprint(&source)?!=expected{return Err(Error::SourceChanged);}match addition{None=>verify_publication_delta(&source,config,data,delta),Some(p)=>verify_publication_independent(&source,config,data,delta,p)}.map(|_|())
}

#[cfg(test)]pub(super)fn interrupt_for_fixture<R:Source>(review:R,workspace:&Path,after:&str)->std::result::Result<Prepared<R>,Failure>{prepare_using(review,workspace,|s|if matches!((after,s),("manifest",Step::Manifest)|("ready",Step::Ready)){Err(Error::Unavailable)}else{Ok(())})}
