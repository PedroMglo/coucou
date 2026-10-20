//! A conversation is a normal floating toplevel, not a keyboard-interactive
//! shell layer. Hyprland retains the last normal window while a layer has the
//! keyboard; clicking that same window can leave input attached to the layer.
//! Only our own chat may be positioned through the local compositor socket.
use std::{io::{Read, Write}, os::unix::{net::UnixStream, fs::{FileTypeExt, MetadataExt}}, path::PathBuf, sync::{Mutex, OnceLock, mpsc}, time::Duration};
use gtk::prelude::*;
use serde_json::Value;
use tauri::WebviewWindow;

const TITLE: &str = "Pets · Conversa";
static POSITION: Mutex<Option<(i32, i32, i32, i32)>> = Mutex::new(None);

fn socket_path() -> Result<PathBuf, String> {
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| "Hyprland indisponível")?;
    if signature.is_empty() || signature.len() > 256 || !signature.bytes().all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b)) {
        return Err("Identidade do compositor inválida".into());
    }
    let path = PathBuf::from(format!("/run/user/{}/hypr/{signature}/.socket.sock", unsafe { libc::geteuid() }));
    let meta = std::fs::symlink_metadata(&path).map_err(|_| "Socket do compositor indisponível")?;
    if !meta.file_type().is_socket() || meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o022 != 0 {
        return Err("Socket do compositor não pertence à sessão".into());
    }
    Ok(path)
}

pub(super) fn supported() -> bool { socket_path().is_ok() }

fn request(command: &str) -> Result<Vec<u8>, String> {
    let mut stream = UnixStream::connect(socket_path()?).map_err(|_| "Compositor indisponível")?;
    let timeout = Some(Duration::from_millis(100));
    stream.set_read_timeout(timeout).map_err(|_| "Prazo de leitura indisponível")?;
    stream.set_write_timeout(timeout).map_err(|_| "Prazo de escrita indisponível")?;
    stream.write_all(command.as_bytes()).map_err(|_| "Não foi possível posicionar a conversa")?;
    let mut bytes = Vec::new();
    stream.take(256 * 1024 + 1).read_to_end(&mut bytes).map_err(|_| "O compositor não respondeu")?;
    if bytes.len() > 256 * 1024 { return Err("Inventário do compositor demasiado grande".into()); }
    Ok(bytes)
}

fn own_chat(clients: &Value, pid: u32) -> Result<&Value, String> {
    let rows = clients.as_array().ok_or("Inventário do compositor inválido")?;
    let mut ours = rows.iter().filter(|w| w["pid"].as_u64() == Some(pid as u64) && w["title"] == TITLE && w["mapped"] == true);
    let window = ours.next().ok_or("A conversa ainda não foi apresentada")?;
    if ours.next().is_some() { return Err("Identidade da conversa ambígua".into()); }
    let address = window["address"].as_str().ok_or("Janela sem endereço")?;
    if address.len() < 3 || address.len() > 20 || !address.starts_with("0x") || !address[2..].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Endereço da conversa inválido".into());
    }
    Ok(window)
}

fn dispatch(action: &str, address: &str, args: &str) -> Result<(), String> {
    // Both address and action are validated/internal. No shell or user text.
    let reply = request(&format!("dispatch hl.dsp.window.{action}({{{args},window=\"address:{address}\"}})"))?;
    if reply.trim_ascii() != b"ok" { return Err("O compositor recusou a posição da conversa".into()); }
    Ok(())
}

fn position(x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    let clients: Value = serde_json::from_slice(&request("j/clients")?).map_err(|_| "Inventário do compositor inválido")?;
    let window = own_chat(&clients, std::process::id())?;
    let address = window["address"].as_str().unwrap();
    if window["floating"] != true { dispatch("float", address, "action=\"on\"")?; }
    if window["pinned"] != true { dispatch("pin", address, "action=\"on\"")?; }
    // GTK's initial natural size can win over resize() before the first map.
    // Apply the requested logical bounds to this exact floating toplevel.
    if window["size"][0].as_i64() != Some(width as i64) || window["size"][1].as_i64() != Some(height as i64) {
        dispatch("resize", address, &format!("x={width},y={height}"))?;
    }
    dispatch("move", address, &format!("x={x},y={y}"))
}

fn wake() {
    static WORKER: OnceLock<mpsc::SyncSender<()>> = OnceLock::new();
    let sender = WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || while rx.recv().is_ok() {
            let desired = *POSITION.lock().unwrap();
            if let Some((x, y, width, height)) = desired {
                // GTK map precedes the compositor's first buffer. Retry only
                // placement, never visibility or focus, and abandon stale work.
                for attempt in 0..4 {
                    if *POSITION.lock().unwrap() != Some((x, y, width, height)) { break; }
                    match position(x, y, width, height) {
                        Ok(()) => break,
                        Err(error) if attempt == 3 => crate::log::line(format!("floating chat placement: {error}")),
                        Err(_) => std::thread::sleep(Duration::from_millis(25)),
                    }
                }
            }
        });
        tx
    });
    let _ = sender.try_send(());
}

pub(super) fn place(x: f64, y: f64, width: f64, height: f64) {
    if ![x, y, width, height].iter().all(|v| v.is_finite() && v.abs() <= 32768.0) || width < 320.0 || height < 280.0 { return; }
    *POSITION.lock().unwrap() = Some((x.round() as i32, y.round() as i32, width.round() as i32, height.round() as i32));
    wake();
}

pub(super) fn prepare(win: &WebviewWindow) -> bool {
    let Ok(gw) = win.gtk_window() else { return false };
    gw.set_accept_focus(true);
    gw.set_focus_on_map(false);
    // Fixed native constraints keep this helper outside the tiling layout.
    // The existing size button still sets the explicit content size.
    gw.set_resizable(false);
    gw.set_type_hint(gtk::gdk::WindowTypeHint::Utility);
    gw.set_keep_above(true);
    gw.connect_map_event(|_, _| { wake(); gtk::glib::Propagation::Proceed });
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn row() -> Value { json!({"pid":42,"title":TITLE,"mapped":true,"address":"0xab12","floating":true}) }
    #[test]
    fn only_the_exact_mapped_chat_of_this_process_can_move() {
        assert!(own_chat(&json!([row()]), 42).is_ok());
        for field in ["pid", "title", "mapped", "address"] {
            let mut w = row(); w[field] = match field { "pid"=>json!(99), "title"=>json!("Coucou · Pets"), "mapped"=>json!(false), _=>json!("0xab\";bad()") };
            assert!(own_chat(&json!([w]), 42).is_err(), "{field}");
        }
        assert!(own_chat(&json!([row(), row()]), 42).is_err());
    }
}
