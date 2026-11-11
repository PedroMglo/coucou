//! The same typed literal input is used at admission, review, delivery and
//! receipt verification. Parsing data never resolves a source or grants access.
use super::{Arguments,ReceivedReference,Envelope,Kind};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReplyArguments{pub to_pet_id:String,pub source:ReceivedReference,pub text:String,pub expires_at:u64}
impl ReplyArguments{
 pub(crate)fn parse(v:&serde_json::Value)->Result<Self,()>{let a:Self=serde_json::from_value(v.clone()).map_err(|_|())?;
  if !super::known(&a.to_pet_id)||!a.source.valid()||!super::body(&a.text)||a.expires_at==0||a.expires_at>8_640_000_000_000_000{return Err(());}Ok(a)}
}
pub(in crate::pet_runtime)enum Input{ResultReply(crate::pet_runtime::derived_result_plan::Arguments),Message(Arguments),Reply(ReplyArguments),FundedReply(ReplyArguments),WorkReport(crate::pet_runtime::work_report::Arguments),WorkDisclosure(crate::pet_runtime::work_disclosure::Arguments),Delegate(crate::pet_runtime::delegation::Arguments)}
impl Input{
 pub(in crate::pet_runtime)fn parse(op:&crate::pet_runtime::plans::OperationProposal)->Result<Self,()>{if op.operation_id=="pet.resultReply"&&op.version==1{return crate::pet_runtime::derived_result_plan::Arguments::parse(op).map(Self::ResultReply).map_err(|_|());}if op.operation_id=="pet.reply"&&op.version==2{return ReplyArguments::parse(&op.arguments).map(Self::FundedReply);}if op.operation_id=="pet.workReport"&&op.version==2{return crate::pet_runtime::work_disclosure::Arguments::parse(&op.arguments).map(Self::WorkDisclosure);}if op.version!=1{return Err(());}match op.operation_id.as_str(){"pet.delegate"=>crate::pet_runtime::delegation::Arguments::parse(&op.arguments).map(Self::Delegate),"pet.message"=>Arguments::parse(&op.arguments).map(Self::Message),"pet.reply"=>ReplyArguments::parse(&op.arguments).map(Self::Reply),"pet.workReport"=>crate::pet_runtime::work_report::Arguments::parse(&op.arguments).map(Self::WorkReport),_=>Err(())}}
 pub(in crate::pet_runtime)fn to_pet(&self)->&str{match self{Self::ResultReply(a)=>&a.to_pet_id,Self::Message(a)=>&a.to_pet_id,Self::Reply(a)|Self::FundedReply(a)=>&a.to_pet_id,Self::WorkReport(a)=>&a.to_pet_id,Self::WorkDisclosure(a)=>&a.to_pet_id,Self::Delegate(a)=>&a.to_pet_id}}
 pub(in crate::pet_runtime)fn text(&self)->&str{match self{Self::ResultReply(a)=>&a.text,Self::Message(a)=>&a.text,Self::Reply(a)|Self::FundedReply(a)=>&a.text,Self::WorkReport(_)|Self::WorkDisclosure(_)=>crate::pet_runtime::work_report::TEXT,Self::Delegate(a)=>&a.subgoal}}
 pub(in crate::pet_runtime)fn deadline(&self)->u64{match self{Self::ResultReply(a)=>a.expires_at,Self::Message(a)=>a.expires_at,Self::Reply(a)|Self::FundedReply(a)=>a.expires_at,Self::WorkReport(a)=>a.expires_at,Self::WorkDisclosure(a)=>a.expires_at,Self::Delegate(a)=>a.expires_at}}
 pub(in crate::pet_runtime)fn kind(&self)->Kind{match self{Self::ResultReply(_)=>Kind::Reply,Self::Message(a)=>a.kind,Self::Reply(_)|Self::FundedReply(_)=>Kind::Reply,Self::WorkReport(_)|Self::WorkDisclosure(_)=>Kind::WorkEvent,Self::Delegate(_)=>Kind::Delegation}}
 pub(in crate::pet_runtime)fn source(&self)->Option<&ReceivedReference>{match self{Self::ResultReply(a)=>Some(&a.source),Self::Reply(a)|Self::FundedReply(a)=>Some(&a.source),Self::WorkReport(a)=>Some(&a.source),Self::WorkDisclosure(a)=>Some(&a.source),_=>None}}
 pub(in crate::pet_runtime)fn matches(&self,e:&Envelope)->bool{
  self.to_pet()==e.recipient.pet_id&&self.text()==e.text&&self.deadline()==e.deadline_at&&self.kind()==e.kind&&(if matches!(self,Self::Delegate(_))&&e.schema_version==7{e.reply_to.is_some()}else{self.source()==e.reply_to.as_ref()})
   &&match self{Self::Message(_)if matches!(e.schema_version,10|11|13|15)=>e.delegation.is_none()&&e.work_report.is_none()&&e.derived_result.is_none()&&e.planned_data.as_ref().is_some_and(|p|p.matches_envelope(e)),Self::ResultReply(a)=>matches!(e.schema_version,6|9)&&e.delegation.is_none()&&e.work_report.is_none()&&e.derived_result.as_ref().is_some_and(|d|d.valid()&&d.result==a.result),Self::FundedReply(_)=>matches!(e.schema_version,5|8)&&e.delegation.is_none()&&e.work_report.is_none(),Self::WorkReport(a)=>e.schema_version==3&&e.work_report.as_ref().is_some_and(|r|r.matches(a,e)),Self::WorkDisclosure(a)=>e.schema_version==3&&a.matches(e),Self::Delegate(a)=>matches!(e.schema_version,4|7|12|14|16)&&e.work_report.is_none()&&a.matches(e),_=>e.delegation.is_none()&&e.work_report.is_none()&&e.schema_version==if self.source().is_some(){2}else{1}}
 }
}
