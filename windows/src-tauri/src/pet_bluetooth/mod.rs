//! One bounded, read-only native Bluetooth query, scoped to the original
//! accepted System context. No discovery, pairing, writes or command handles.
mod model;
use crate::pet_native_binding::{self as binding, ipc::{Checked, ContextRequest}};
use serde::Serialize;
use tauri::{AppHandle, WebviewWindow};

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum Failure {
    Context { reason: binding::ipc::Failure },
    Busy,
    Unavailable,
    UntrustedPeer,
    OriginChanged,
    Timeout,
    Unsupported,
    InvalidData,
}

impl From<binding::ipc::Failure> for Failure {
    fn from(reason: binding::ipc::Failure) -> Self { Self::Context { reason } }
}

#[cfg(target_os = "linux")]
static READ_SLOT: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

#[cfg(target_os = "linux")]
fn native_failure(error: crate::transport::Failure<binding::ipc::Failure>) -> Failure {
    use crate::transport::{Error, Failure as Native};
    match error {
        Native::Guard(reason) => Failure::Context { reason },
        Native::Native { reason, .. } => match reason {
            Error::Unavailable | Error::NativeFailure | Error::PreferencesUnconfirmed => Failure::Unavailable,
            Error::UntrustedPeer => Failure::UntrustedPeer,
            Error::OriginChanged => Failure::OriginChanged,
            Error::Timeout => Failure::Timeout,
            Error::Unsupported => Failure::Unsupported,
            Error::Protocol => Failure::InvalidData,
        },
    }
}

#[tauri::command]
pub(crate) async fn pet_system_bluetooth_read(app: AppHandle, window: WebviewWindow,
    request: Checked<ContextRequest>) -> Result<model::Snapshot, Failure> {
    #[cfg(target_os = "linux")]
    {
        let client = binding::manager(&app)?.client()?;
        let prepared = binding::prepare(&window, request.0.expected_revision, client).await?;
        prepared.validate()?;
        let _permit = READ_SLOT.try_acquire().map_err(|_| Failure::Busy)?;
        let config = crate::transport::Config::from_environment().map_err(|_| Failure::Unavailable)?;
        let reply = crate::transport::bluetooth_snapshot_guarded(&config, || prepared.validate()).await.map_err(native_failure)?;
        let snapshot = model::project(&reply.result, reply.api_version, &reply.capabilities,
            crate::assistant_memory::now(), request.0.expected_revision).map_err(|error| match error {
                model::Invalid::Unsupported => Failure::Unsupported,
                model::Invalid::Shape | model::Invalid::Capacity => Failure::InvalidData,
            })?;
        // Do not replace the original context with a newly resolved window.
        // A stale result is refused even when the native read itself succeeded.
        prepared.validate()?;
        Ok(snapshot)
    }
    #[cfg(not(target_os = "linux"))]
    { let _ = (app, window, request); Err(Failure::Unsupported) }
}
