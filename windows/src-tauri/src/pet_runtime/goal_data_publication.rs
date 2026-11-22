//! One data-only goal publication contract, shared by restored-future and
//! received-request origins. Implementors are native owned values, not JSON.
use super::{model::{self,Spec},store::{Store,Guard}};
use std::path::Path;
pub(super)trait Certificate{
 fn spec(&self)->&Spec;
 fn authority_path(&self)->&Path;
 fn prepared_at(&self)->u64;
 fn expires_at(&self)->u64;
 fn lock_source(&self)->model::Result<Guard>;
 fn current(&self,store:&Store,now:u64)->model::Result<()>;
}
