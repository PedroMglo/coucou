//! Copilot's native ACP connection. Integration must supply only session IDs
//! owned by the selected Mochi conversation, plus the existing approval broker.
//! This module does not register commands, change the host, or read credentials.
pub mod rpc;

use std::path::PathBuf;

pub fn executable() -> Result<PathBuf, String> {
    let name = "copilot";
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
        .ok_or_else(|| "Copilot CLI não foi encontrado. Instala e inicia sessão no Copilot antes de usar este pet.".into())
}
