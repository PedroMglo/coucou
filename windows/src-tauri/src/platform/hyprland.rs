use std::io::{Read, Write};
use std::os::unix::{
    fs::{FileTypeExt, MetadataExt},
    net::UnixStream,
};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(super) fn socket_path() -> Result<PathBuf, String> {
    let signature =
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| "Hyprland indisponível")?;
    if signature.is_empty()
        || signature.len() > 256
        || matches!(signature.as_str(), "." | "..")
        || !signature.bytes().all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
    {
        return Err("Identidade do compositor inválida".into());
    }
    let uid = unsafe { libc::geteuid() };
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{uid}")));
    if !runtime.is_absolute() {
        return Err("Diretório da sessão inválido".into());
    }
    let meta = std::fs::metadata(&runtime).map_err(|_| "Sessão do compositor indisponível")?;
    if !meta.is_dir() || meta.uid() != uid || meta.mode() & 0o077 != 0 {
        return Err("Diretório da sessão inválido".into());
    }
    let path = runtime.join("hypr").join(signature).join(".socket.sock");
    let meta = std::fs::symlink_metadata(&path).map_err(|_| "Socket do compositor indisponível")?;
    if !meta.file_type().is_socket() || meta.uid() != uid || meta.mode() & 0o022 != 0 {
        return Err("Socket do compositor não pertence à sessão".into());
    }
    Ok(path)
}

pub(super) fn request(socket: &Path, command: &str) -> Result<Vec<u8>, String> {
    const LIMIT: u64 = 256 * 1024;
    let mut stream = UnixStream::connect(socket).map_err(|_| "Compositor indisponível")?;
    let timeout = Some(Duration::from_millis(100));
    stream.set_read_timeout(timeout).map_err(|_| "Prazo de leitura indisponível")?;
    stream.set_write_timeout(timeout).map_err(|_| "Prazo de escrita indisponível")?;
    stream.write_all(command.as_bytes()).map_err(|_| "Falha a escrever para o compositor")?;
    let mut bytes = Vec::new();
    stream.take(LIMIT + 1).read_to_end(&mut bytes).map_err(|_| "O compositor não respondeu")?;
    if bytes.len() as u64 > LIMIT {
        return Err("Resposta do compositor demasiado grande".into());
    }
    Ok(bytes)
}
