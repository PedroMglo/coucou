//! Civil-time DATA for rule review, using the existing sealed source-free worker.
use chrono::{Datelike,NaiveDate,NaiveTime,NaiveDateTime};
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Schedule{pub kind:Kind,pub first_date:String,pub days:u8,pub local_time:String,pub time_zone:String,pub weekdays:Vec<u8>,pub overlap:Overlap,pub gap:Gap}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Kind{Daily}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Overlap{Earlier,Later,Skip,Decision}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Gap{Skip,Decision}
impl Schedule{
 pub(crate)fn validate(&self)->Result<(),&'static str>{self.locals().map(|_|())}
 fn locals(&self)->Result<Vec<String>,&'static str>{
  let bad="invalidSchedule";let date=NaiveDate::parse_from_str(&self.first_date,"%Y-%m-%d").map_err(|_|bad)?;
  let clock=NaiveTime::parse_from_str(&self.local_time,"%H:%M").map_err(|_|bad)?;
  if date.format("%Y-%m-%d").to_string()!=self.first_date||!(1970..=9999).contains(&date.year())||clock.format("%H:%M").to_string()!=self.local_time||!(1..=64).contains(&self.days)||!super::query::valid_zone(&self.time_zone)||self.weekdays.is_empty()||self.weekdays.len()>7||self.weekdays.iter().any(|n|!(1..=7).contains(n))||self.weekdays.windows(2).any(|w|w[0]>=w[1]){return Err(bad);}
  let mut out=vec![];for n in 0..self.days{let d=date.checked_add_days(chrono::Days::new(n.into())).ok_or(bad)?;if d.year()>9999{return Err(bad);}if self.weekdays.contains(&(d.weekday().number_from_monday()as u8)){out.push(d.and_time(clock).format("%Y-%m-%dT%H:%M:%S").to_string());}}
  if out.is_empty(){return Err(bad);}Ok(out)
 }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Slot{pub local_date_time:String,pub resolution:Resolution,pub at:Option<u64>,pub offset_seconds:Option<i64>}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Resolution{Ordinary,Earlier,Later,SkippedGap,SkippedOverlap,DecisionGap,DecisionOverlap}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Reply{schema_version:u32,kind:String,status:String,request_id:String,schedule:Schedule,time_zone_data_version:String,slots:Vec<Slot>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Preview{pub schema_version:u32,pub schedule:Schedule,pub time_zone_data_version:String,pub resolver_revision:String,pub slots:Vec<Slot>,pub execution_authorized:bool}
fn bound(schedule:&Schedule,request_id:&str,value:serde_json::Value)->Result<Preview,&'static str>{
 let bad="scheduleResponseMismatch";let locals=schedule.locals()?;
 let r:Reply=serde_json::from_value(value).map_err(|_|bad)?;
 if r.schema_version!=1||r.kind!="petRuleSchedule"||r.status!="ready"||r.request_id!=request_id||&r.schedule!=schedule||r.slots.len()!=locals.len()||r.time_zone_data_version.len()!=5||!r.time_zone_data_version.as_bytes()[..4].iter().all(u8::is_ascii_digit)||!r.time_zone_data_version.as_bytes()[4].is_ascii_lowercase(){return Err(bad);}
 let mut previous=None;for(s,local)in r.slots.iter().zip(locals){
  if s.local_date_time!=local{return Err(bad);}
  let planned=match s.resolution{Resolution::Ordinary=>true,Resolution::Earlier=>schedule.overlap==Overlap::Earlier,Resolution::Later=>schedule.overlap==Overlap::Later,Resolution::SkippedGap=>schedule.gap==Gap::Skip,Resolution::DecisionGap=>schedule.gap==Gap::Decision,Resolution::SkippedOverlap=>schedule.overlap==Overlap::Skip,Resolution::DecisionOverlap=>schedule.overlap==Overlap::Decision};if !planned{return Err(bad);}
  if matches!(s.resolution,Resolution::Ordinary|Resolution::Earlier|Resolution::Later){
   let at=s.at.filter(|n|*n>0&&*n<=8_640_000_000_000_000).ok_or(bad)?;let offset=s.offset_seconds.filter(|n|n.unsigned_abs()<=86400).ok_or(bad)?;
   let civil=NaiveDateTime::parse_from_str(&local,"%Y-%m-%dT%H:%M:%S").map_err(|_|bad)?.and_utc().timestamp_millis();
   if i64::try_from(at).ok().and_then(|n|n.checked_add(offset*1000))!=Some(civil)||previous.is_some_and(|n|at<=n){return Err(bad);}previous=Some(at);
  }else if s.at.is_some()||s.offset_seconds.is_some(){return Err(bad);}
 }
 let resolver=super::sha(&[include_bytes!("runtime.zip").as_slice(),include_bytes!("worker.py").as_slice(),r.time_zone_data_version.as_bytes()].concat());
 Ok(Preview{schema_version:1,schedule:r.schedule,time_zone_data_version:r.time_zone_data_version,resolver_revision:resolver,slots:r.slots,execution_authorized:false})
}
#[cfg(target_os="linux")]
pub(crate)fn resolve(schedule:&Schedule,current:impl Fn()->bool)->Result<Preview,&'static str>{
 schedule.validate()?;let mut bytes=[0u8;16];use ring::rand::SecureRandom;ring::rand::SystemRandom::new().fill(&mut bytes).map_err(|_|"runtimeUnavailable")?;let request_id=super::sha(&bytes);
 let request=serde_json::json!({"schemaVersion":1,"kind":"petRuleSchedule","requestId":request_id,"schedule":schedule});let value=super::process::run(&request,current)?;
 if value["status"]=="unavailable"{return Err(if value["errorCode"]=="unknownTimeZone"{"unknownTimeZone"}else{"invalidSchedule"});}bound(schedule,&request_id,value)
}
#[cfg(test)]mod tests{
 use super::*;use serde_json::json;
 pub(crate)fn schedule(date:&str,zone:&str,time:&str)->Schedule{serde_json::from_value(json!({"kind":"daily","firstDate":date,"days":3,"localTime":time,"timeZone":zone,"weekdays":[1,2,3,4,5,6,7],"overlap":"earlier","gap":"skip"})).unwrap()}
 #[test]fn rule_contract_strict_daily_schedule_and_response_binding(){
  let mut s=schedule("2026-10-24","Europe/Lisbon","01:30");assert!(s.validate().is_ok());for zone in["../UTC","/etc/passwd",""]{s.time_zone=zone.into();assert!(s.validate().is_err());}s=schedule("2026-10-24","Europe/Lisbon","1:30");assert!(s.validate().is_err());s=schedule("2026-10-24","Europe/Lisbon","01:30");s.weekdays=vec![1,1];assert!(s.validate().is_err());
  s=schedule("2026-10-25","Europe/Lisbon","01:30");s.days=1;let request="request";let response=json!({"schemaVersion":1,"kind":"petRuleSchedule","status":"ready","requestId":request,"schedule":s,"timeZoneDataVersion":"2026a","slots":[{"localDateTime":"2026-10-25T01:30:00","resolution":"earlier","at":1792888200000_u64,"offsetSeconds":3600}]});assert!(bound(&s,request,response.clone()).is_ok());
  for key in["requestId","schedule","timeZoneDataVersion"]{let mut bad=response.clone();bad[key]=json!("other");assert!(bound(&s,request,bad).is_err());}let mut bad=response;bad["slots"][0]["at"]=json!(1792891800000_u64);assert!(bound(&s,request,bad).is_err());
 }
 #[cfg(target_os="linux")]
 #[test]fn rule_contract_sealed_iana_preview_handles_overlap_gap_skipped_date_and_revocation_without_source_or_model(){
  let _serial=super::super::process::TEST_SERIAL.lock().unwrap();let mut s=schedule("2026-10-24","Europe/Lisbon","01:30");let early=resolve(&s,||true).unwrap();assert_eq!(early.slots.len(),3);assert_eq!(early.slots[1].resolution,Resolution::Earlier);s.overlap=Overlap::Later;let late=resolve(&s,||true).unwrap();assert_eq!(late.slots[1].at.unwrap()-early.slots[1].at.unwrap(),3600000);assert!(!late.execution_authorized);s.overlap=Overlap::Decision;assert_eq!(resolve(&s,||true).unwrap().slots[1].resolution,Resolution::DecisionOverlap);
  s=schedule("2026-03-28","Europe/Lisbon","01:30");let gap=resolve(&s,||true).unwrap();assert_eq!(gap.slots[1].resolution,Resolution::SkippedGap);assert!(gap.slots[1].at.is_none());s.gap=Gap::Decision;assert_eq!(resolve(&s,||true).unwrap().slots[1].resolution,Resolution::DecisionGap);
  s=schedule("2011-12-29","Pacific/Apia","09:00");assert_eq!(resolve(&s,||true).unwrap().slots[1].resolution,Resolution::SkippedGap);assert!(resolve(&s,||false).is_err());s.time_zone="Unknown/Nowhere".into();assert!(resolve(&s,||true).is_err());
 }
}


/// Structural/current-resolver verification of an already captured native plan.
pub(crate)fn valid_preview(p:&Preview)->bool{
 if p.schema_version!=1||p.execution_authorized{return false;}
 let value=serde_json::json!({"schemaVersion":1,"kind":"petRuleSchedule","status":"ready","requestId":"stored-native-rule-plan","schedule":p.schedule,"timeZoneDataVersion":p.time_zone_data_version,"slots":p.slots});
 bound(&p.schedule,"stored-native-rule-plan",value).is_ok_and(|same|same==*p)
}


/// Historical plans retain their captured timezone facts after an app/tzdata update.
pub(crate)fn valid_preview_shape(p:&Preview)->bool{
 if p.schema_version!=1||p.execution_authorized||p.resolver_revision.len()!=64||!p.resolver_revision.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f')){return false;}
 let value=serde_json::json!({"schemaVersion":1,"kind":"petRuleSchedule","status":"ready","requestId":"stored-native-rule-plan","schedule":p.schedule,"timeZoneDataVersion":p.time_zone_data_version,"slots":p.slots});
 bound(&p.schedule,"stored-native-rule-plan",value).is_ok_and(|mut same|{same.resolver_revision=p.resolver_revision.clone();same==*p})
}
