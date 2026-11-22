pub(crate) mod lifecycle;
#[cfg(target_os="linux")] pub(crate) mod transport;
pub(crate) mod authority_fence;
#[cfg(target_os="linux")] pub(crate) mod executor;
#[cfg(not(target_os="linux"))]
#[path="unsupported_executor.rs"] pub(crate) mod executor;
