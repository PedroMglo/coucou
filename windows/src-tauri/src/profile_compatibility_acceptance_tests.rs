//! Private, opt-in read-only acceptance of the actual native codecs. Reports
//! counts only; no paths, preference values or conversation text are printed.
use super::*;
use serde::Serialize;
use std::{fs, io::Read, os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt}};

#[derive(Default,Serialize)]
#[serde(rename_all="camelCase")]
struct Delta { added_fields_or_values:usize, removed_stored_fields_or_values:usize, changed_stored_values:usize }
impl Delta { fn unchanged(&self)->bool{self.removed_stored_fields_or_values==0&&self.changed_stored_values==0} }
fn compare(before:&Value,after:&Value)->Delta{
    fn visit(before:&Value,after:&Value,d:&mut Delta){
        match(before,after){
            (Value::Object(a),Value::Object(b))=>{
                for(key,value)in a{if let Some(next)=b.get(key){visit(value,next,d)}else{d.removed_stored_fields_or_values+=1;}}
                d.added_fields_or_values+=b.keys().filter(|key|!a.contains_key(*key)).count();
            }
            (Value::Array(a),Value::Array(b))=>{
                for(value,next)in a.iter().zip(b){visit(value,next,d);}
                d.removed_stored_fields_or_values+=a.len().saturating_sub(b.len());
                d.added_fields_or_values+=b.len().saturating_sub(a.len());
            }
            _=>if before!=after{d.changed_stored_values+=1;},
        }
    }
    let mut delta=Delta::default();visit(before,after,&mut delta);delta
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct Report{
    settings:Delta,pets:Delta,memory_reader:Delta,memory_runtime:Delta,
    inputs_unchanged:bool,compatible_without_review:bool,
}
fn bytes(path:&Path)->Result<Vec<u8>,&'static str>{
    if !path.is_absolute(){return Err("input path must be absolute");}
    for part in path.ancestors(){if fs::symlink_metadata(part).map_err(|_|"input unavailable")?.file_type().is_symlink(){return Err("input symlink refused");}}
    let mut options=fs::OpenOptions::new();options.read(true).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC);
    let file=options.open(path).map_err(|_|"input unreadable")?;
    let before=file.metadata().map_err(|_|"input metadata unavailable")?;
    if !before.is_file()||before.uid()!=unsafe{libc::geteuid()}||before.mode()&0o7777!=0o600||before.nlink()!=1||before.len()>crate::assistant_memory::MAX_BYTES{return Err("input not private bounded data");}
    let mut result=vec![];(&file).take(crate::assistant_memory::MAX_BYTES+1).read_to_end(&mut result).map_err(|_|"input read failed")?;
    let after=file.metadata().map_err(|_|"input metadata unavailable")?;let named=fs::symlink_metadata(path).map_err(|_|"input moved")?;
    let stamp=|m:&fs::Metadata|(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec(),m.mode(),m.nlink());
    if stamp(&before)!=stamp(&after)||stamp(&after)!=stamp(&named)||result.len()as u64!=before.len(){return Err("input changed");}Ok(result)
}
fn value(raw:&[u8])->Result<Value,&'static str>{serde_json::from_slice(raw.strip_prefix(&[0xef,0xbb,0xbf]).unwrap_or(raw)).map_err(|_|"input JSON invalid")}
fn report(root:&Path)->Result<Report,&'static str>{
    let paths=[root.join("config/settings.json"),root.join("config/pets.json"),root.join("data/assistant/conversations.json")];
    let original:Vec<_>=paths.iter().map(|p|bytes(p)).collect::<Result<_,_>>()?;
    let raw_settings=value(&original[0])?;let raw_pets=value(&original[1])?;let raw_memory=value(&original[2])?;
    let loaded_settings=super::parse(&original[0]).ok_or("native settings rejected")?;
    let loaded_pets=crate::pet_config::read_existing_at(&paths[1]).map_err(|_|"native pets rejected")?;
    let library=crate::assistant_memory::load(&paths[2]).map_err(|_|"native history reader rejected")?;
    let memory=crate::assistant_memory::native_codec_projection(&library).map_err(|_|"native history projection rejected")?;
    let runtime=crate::chat::Chat::persistent(paths[2].clone()).native_codec_projection().map_err(|_|"native history runtime rejected")?;
    let settings=compare(&raw_settings,&serde_json::to_value(loaded_settings).map_err(|_|"native settings projection rejected")?);
    let pets=compare(&raw_pets,&serde_json::to_value(loaded_pets).map_err(|_|"native pets projection rejected")?);
    let memory_reader=compare(&raw_memory,&memory);let memory_runtime=compare(&raw_memory,&runtime);
    let unchanged=paths.iter().zip(&original).all(|(p,old)|bytes(p).is_ok_and(|now|&now==old));
    let compatible=unchanged&&settings.unchanged()&&pets.unchanged()&&memory_reader.unchanged()&&memory_runtime.unchanged();
    Ok(Report{settings,pets,memory_reader,memory_runtime,inputs_unchanged:unchanged,compatible_without_review:compatible})
}

#[test]
fn native_codecs_report_normalized_preferences_without_writing_inputs(){
    let root=std::env::temp_dir().join(format!("coucou-native-profile-codecs-{}",crate::assistant_memory::new_id()));
    for part in [root.clone(),root.join("config"),root.join("data"),root.join("data/assistant")]{fs::create_dir(&part).unwrap();fs::set_permissions(part,fs::Permissions::from_mode(0o700)).unwrap();}
    let settings=root.join("config/settings.json");let pets=root.join("config/pets.json");let memory=root.join("data/assistant/conversations.json");
    let write=|path:&Path,v:&Value|{fs::write(path,serde_json::to_vec(v).unwrap()).unwrap();fs::set_permissions(path,fs::Permissions::from_mode(0o600)).unwrap();};
    let original=serde_json::to_value(Settings::default()).unwrap();write(&settings,&original);write(&pets,&serde_json::to_value(crate::pet_config::defaults()).unwrap());
    write(&memory,&serde_json::json!({"version":4,"pets":{"codex":crate::chat::SavedConversation{id:"test-cid".into(),created:1,updated:1,..Default::default()}},"archives":{}}));
    let good=report(&root).unwrap();assert!(good.compatible_without_review);assert!(good.inputs_unchanged);
    let mut bad=original;bad["soundVolume"]=serde_json::json!("unusable");write(&settings,&bad);
    let reviewed=report(&root).unwrap();assert!(!reviewed.compatible_without_review);assert!(reviewed.inputs_unchanged);assert_eq!(reviewed.settings.changed_stored_values,1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore="explicit opt-in private copied profile; reads JSON only, no app/native account/model operations"]
fn native_copied_profile_codec_acceptance(){
    assert_eq!(std::env::var("COUCOU_NATIVE_PROFILE_CODEC_PROBE").as_deref(),Ok("1"));
    let root=PathBuf::from(std::env::var_os("COUCOU_NATIVE_PROFILE_CODEC_ROOT").expect("private input root required"));
    let result=report(&root).unwrap_or_else(|code|panic!("{code}"));
    println!("COUCOU_NATIVE_PROFILE_CODEC_REPORT {}",serde_json::to_string(&result).unwrap());
    assert!(result.inputs_unchanged,"native readers changed copied input files");
    assert!(result.compatible_without_review,"native stored-field normalization requires review; inspect aggregate counters");
}
