//! A single native decision field with distinct, non-interchangeable scopes.
use serde::{Serialize,Deserialize};
#[derive(Clone,Copy,PartialEq,Eq,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Consent{BoundedWorkDisclosure,BoundedResultDisclosure,BoundedNativePeerPlan}
