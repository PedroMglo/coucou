//! No native identity/transport adapter exists on these platforms. Fail closed.
use crate::{lifecycle as l,authority_fence::WindowRef};
#[derive(Clone)] pub(crate) struct Client;
pub(crate) struct Executor;
pub(crate) struct Prepared;
pub(crate) struct Pending<T>(std::marker::PhantomData<T>);
impl Executor {pub(crate) fn client(&self)->Client{Client}}
impl Prepared {pub(crate) fn validate(&self)->Result<(),l::Reason>{Err(l::Reason::Unsupported)}}
impl<T> Pending<T>{pub(crate) async fn wait(self)->Result<T,l::Reason>{Err(l::Reason::Unsupported)}}
impl Client {
 pub(crate) fn prepare(&self,_:&WindowRef)->Result<Prepared,l::Reason>{Err(l::Reason::Unsupported)}
 pub(crate) fn enqueue_prepared_read(&self,_:Prepared,_:bool)->Result<Pending<l::Ticket>,l::Reason>{Err(l::Reason::Unsupported)}
 pub(crate) fn enqueue_prepared_view(&self,_:Prepared)->Result<Pending<l::View>,l::Reason>{Err(l::Reason::Unsupported)}
 pub(crate) fn enqueue_prepared_scan(&self,_:Prepared,_:&str,_:&str)->Result<Pending<l::Ticket>,l::Reason>{Err(l::Reason::Unsupported)}
 pub(crate) fn enqueue_prepared_enable_wifi(&self,_:Prepared,_:&str,_:&str)->Result<Pending<l::Ticket>,l::Reason>{Err(l::Reason::Unsupported)}
 pub(crate) fn enqueue_prepared_ticket(&self,_:Prepared,_:&str)->Result<Pending<l::Ticket>,l::Reason>{Err(l::Reason::Unsupported)}
 pub(crate) fn enqueue_prepared_cancel(&self,_:Prepared,_:&str)->Result<Pending<l::Ticket>,l::Reason>{Err(l::Reason::Unsupported)}
}
