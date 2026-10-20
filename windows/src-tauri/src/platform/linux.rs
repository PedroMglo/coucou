// Linux: XDG directories for files, xdg-open for links and folders, and
// gtk-layer-shell for the island window.
//
// Wayland gives an app no global cursor position and no say over where its
// window goes, so the island works differently from Windows:
//   * it is a layer-shell surface anchored to the top edge, above everything,
//     on compositors that support it (COSMIC, KDE, wlroots — not GNOME);
//   * click-through is the window's input region, set to the island shape, so
//     the compositor itself sends every other click to whatever is underneath;
//   * the cursor comes from the page's own mouse events, which only fire over
//     the island — Mochi's eyes follow the pointer there, not across the screen.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::*;
use tauri::{WebviewWindow,Manager as _};

use super::{home_dir, LocalTime};

/// File name of the Claude Code relay.
pub const HOOK_EXE: &str = "coucou-hook";

/// Environment variable holding the home directory.
pub const HOME_VAR: &str = "HOME";

// ── Files ─────────────────────────────────────────────────────────────────────

/// An XDG base directory (`$XDG_CONFIG_HOME` …), or its fallback under the home
/// directory when it is unset or not absolute.
fn xdg(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home_dir().join(fallback))
}

/// ~/.config/coucou — preferences.
pub fn config_dir() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("coucou")
}

/// ~/.local/share/coucou — where coucou-hook, the inbox and the log live. The
/// relay has to sit at a stable path: an AppImage is mounted somewhere new on
/// every launch.
pub fn local_dir() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share").join("coucou")
}

/// Where a saved image goes, best first: the XDG pictures folder named in
/// ~/.config/user-dirs.dirs, then ~/Pictures, ~/Downloads and the home folder.
/// The caller takes the first one that exists.
pub fn picture_dirs() -> Vec<PathBuf> {
    let home = home_dir();
    let mut dirs = Vec::new();
    let user_dirs = xdg("XDG_CONFIG_HOME", ".config").join("user-dirs.dirs");
    if let Ok(text) = std::fs::read_to_string(user_dirs) {
        if let Some(dir) = xdg_user_dir(&text, "XDG_PICTURES_DIR", &home) {
            dirs.push(dir);
        }
    }
    dirs.push(home.join("Pictures"));
    dirs.push(home.join("Downloads"));
    dirs.push(home);
    dirs
}

/// `XDG_PICTURES_DIR="$HOME/Pictures"` → /home/me/Pictures. Only the two forms
/// xdg-user-dirs writes are understood: "$HOME/…" and an absolute path.
fn xdg_user_dir(text: &str, key: &str, home: &Path) -> Option<PathBuf> {
    let line = text.lines().map(str::trim).find(|l| l.starts_with(key))?;
    let value = line.strip_prefix(key)?.trim_start().strip_prefix('=')?.trim();
    let value = value.strip_prefix('"')?.strip_suffix('"')?;
    let path = match value.strip_prefix("$HOME") {
        Some(rest) => home.join(rest.trim_start_matches('/')),
        None => PathBuf::from(value),
    };
    // "$HOME/" alone means "no pictures folder", by the spec.
    (path.is_absolute() && path != home).then_some(path)
}

/// Environment the webview must inherit, set before any thread or process
/// starts.
///
/// Inside an AppImage, WebKit uses the GStreamer bundled with it, and GStreamer
/// keeps its plugin registry in ~/.cache/gstreamer-1.0 by default — the same
/// file the system's GStreamer uses. The AppImage is mounted somewhere new on
/// every launch, so each launch would rewrite the system's registry with
/// plugin paths that vanish once Coucou quits. Give ours its own file.
pub fn prepare_environment() {
    prefer_x11_on_gnome();
    follow_gnome_text_scaling();
    if std::env::var_os("APPIMAGE").is_none() || std::env::var_os("GST_REGISTRY").is_some() {
        return;
    }
    let cache = xdg("XDG_CACHE_HOME", ".cache").join("coucou");
    if std::fs::create_dir_all(&cache).is_ok() {
        std::env::set_var("GST_REGISTRY", cache.join("gstreamer-registry.bin"));
    }
}

/// GNOME on Wayland has no layer-shell and ignores where a regular window asks
/// to go, so the island lands in the middle of the screen. Through XWayland it
/// can be placed, and a Dock window survives "show desktop" (Super+D).
/// An inherited GDK_BACKEND=wayland is overridden too: editors and terminals
/// pass theirs down to every child. COUCOU_X11=0 keeps the Wayland window.
fn prefer_x11_on_gnome() {
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    if should_prefer_x11(
        &env("XDG_SESSION_TYPE"),
        &env("XDG_CURRENT_DESKTOP"),
        &env("COUCOU_X11"),
        &env("DISPLAY"),
    ) {
        std::env::set_var("GDK_BACKEND", "x11");
    }
}

/// Only with XWayland actually there (`DISPLAY` set): forcing x11 without it
/// would leave GTK with no display at all.
fn should_prefer_x11(session: &str, desktop: &str, opt: &str, display: &str) -> bool {
    session.eq_ignore_ascii_case("wayland")
        && desktop.split(':').any(|d| d.eq_ignore_ascii_case("gnome"))
        && opt != "0"
        && !display.trim().is_empty()
}

fn is_gnome(desktop: &str) -> bool {
    desktop.split(':').any(|d| d.eq_ignore_ascii_case("gnome"))
}

/// GNOME's "Large text" (text-scaling-factor > 1) makes WebKitGTK draw every
/// font larger inside a window sized in fixed pixels, so the island's content
/// was cut off (#122). The island's layout is its own, so it asks GDK to undo
/// that factor for this process only. An explicit GDK_DPI_SCALE always wins.
fn follow_gnome_text_scaling() {
    if std::env::var_os("GDK_DPI_SCALE").is_some()
        || !is_gnome(&std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default())
    {
        return;
    }
    let Ok(out) = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "text-scaling-factor"])
        .output()
    else {
        return;
    };
    if let Some(scale) = dpi_scale_for(&String::from_utf8_lossy(&out.stdout)) {
        std::env::set_var("GDK_DPI_SCALE", scale);
    }
}

/// `"1.25\n"` → `Some("0.8")`; `None` for 1.0 or anything unreadable.
fn dpi_scale_for(gsettings_output: &str) -> Option<String> {
    let factor: f64 = gsettings_output.trim().parse().ok()?;
    if !(0.5..=3.0).contains(&factor) || (factor - 1.0).abs() < 0.01 {
        return None;
    }
    Some(format!("{:.4}", 1.0 / factor).trim_end_matches('0').trim_end_matches('.').to_string())
}

pub fn local_time() -> LocalTime {
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        libc::localtime_r(&now, &mut tm);
    }
    LocalTime {
        year: (tm.tm_year + 1900) as u32,
        month: (tm.tm_mon + 1) as u32,
        day: tm.tm_mday as u32,
        hour: tm.tm_hour as u32,
        minute: tm.tm_min as u32,
        second: tm.tm_sec as u32,
    }
}

/// Creates `dir` and closes it to other users. The log, the inbox of dropped
/// files and the relay binary live under these directories; with the default
/// umask they would come out 0755 and readable by anyone on the machine.
pub fn ensure_private_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

/// True when `dir` is a real directory (not a symlink), owned by us, with no
/// access for group or others: what `$XDG_RUNTIME_DIR` promises, checked
/// rather than assumed, since the socket in it decides who can answer a
/// permission request.
fn is_private_dir(dir: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    std::fs::symlink_metadata(dir)
        .map(|m| {
            m.file_type().is_dir() && m.uid() == unsafe { libc::getuid() } && m.mode() & 0o077 == 0
        })
        .unwrap_or(false)
}

/// Where coucou-hook finds us: `$XDG_RUNTIME_DIR/coucou.sock`, or
/// `/run/user/<uid>/coucou.sock` when the variable is missing. A directory
/// that is not ours and private means no relay at all — never a fallback to a
/// shared place like /tmp. Must match `socket_path()` in hook/src/unix.rs
/// exactly.
pub fn relay_socket_path() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() })));
    is_private_dir(&dir).then(|| dir.join("coucou.sock"))
}

// ── Who we are ────────────────────────────────────────────────────────────────

/// The account's full name: the first field of the passwd GECOS entry
/// ("Louis Raille,,,"), or nothing when it is unset.
pub fn user_full_name() -> Option<String> {
    unsafe {
        let mut pwd: libc::passwd = std::mem::zeroed();
        let mut result: *mut libc::passwd = std::ptr::null_mut();
        let mut buf = vec![0 as libc::c_char; 16 * 1024];
        let rc = libc::getpwuid_r(libc::getuid(), &mut pwd, buf.as_mut_ptr(), buf.len(), &mut result);
        if rc != 0 || result.is_null() || pwd.pw_gecos.is_null() {
            return None;
        }
        let gecos = std::ffi::CStr::from_ptr(pwd.pw_gecos).to_string_lossy().into_owned();
        let name = gecos.split(',').next().unwrap_or("").trim().to_string();
        (!name.is_empty()).then_some(name)
    }
}

// ── Processes ─────────────────────────────────────────────────────────────────

/// Nothing to hide: a spawned process only gets a terminal if it asks for one.
pub fn no_console(cmd: &mut Command) -> &mut Command {
    cmd
}

pub fn open_url(url: &str) {
    let _ = Command::new("xdg-open").arg(url).spawn();
}

pub fn reveal_folder(path: &str) {
    let _ = Command::new("xdg-open").arg(path).spawn();
}

/// Our own `which`: the first executable file named `stem` on $PATH.
pub fn find_on_path(stem: &str) -> Option<PathBuf> {
    let dirs = std::env::var_os("PATH")?;
    std::env::split_paths(&dirs)
        .map(|dir| dir.join(stem))
        .find(|p| {
            std::fs::metadata(p)
                .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        })
}

// ── Session windows ("Open terminal") ─────────────────────────────────────────
//
// Finding and raising a session's window lives in linux_focus.rs. There is no
// Claude desktop app for Linux.

pub fn open_claude_desktop() -> bool {
    false
}

/// Where the Codex CLI may be, best first: $PATH, then the usual per-user
/// install folders, which a desktop launch often leaves out of $PATH (npm's
/// global prefix, Volta, Bun, pnpm, and nvm with its newest Node first).
pub fn codex_candidates() -> Vec<PathBuf> {
    let home = home_dir();
    let mut out: Vec<PathBuf> = find_on_path("codex").into_iter().collect();
    for dir in [".local/bin", ".npm-global/bin", ".volta/bin", ".bun/bin", ".local/share/pnpm"] {
        out.push(home.join(dir).join("codex"));
    }
    out.push(PathBuf::from("/usr/local/bin/codex"));
    out.push(PathBuf::from("/usr/bin/codex"));
    let nvm = home.join(".nvm/versions/node");
    if let Ok(entries) = std::fs::read_dir(&nvm) {
        let mut versions: Vec<String> =
            entries.filter_map(|e| e.ok()?.file_name().into_string().ok()).collect();
        versions.sort_by(|a, b| crate::codex_plan::compare_versions(b, a));
        out.extend(versions.iter().map(|v| nvm.join(v).join("bin/codex")));
    }
    out.retain(|p| {
        std::fs::metadata(p)
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    });
    out
}

// ── Cursor ────────────────────────────────────────────────────────────────────

/// Nothing polls the cursor here: the page reports it over the island, and the
/// input region decides click-through (see the top of this file).
pub const CURSOR_POLL: bool = false;

pub fn cursor_physical() -> Option<(f64, f64)> {
    None
}

pub fn left_button_down() -> bool {
    false
}

// ── Island window ─────────────────────────────────────────────────────────────

/// The few gtk-layer-shell calls we need, straight from the C library.
mod layer {
    use gtk::ffi::GtkWindow;
    use std::os::raw::{c_char, c_int};

    pub const LAYER_TOP: c_int = 2;
    pub const LAYER_OVERLAY: c_int = 3;
    pub const EDGE_LEFT: c_int = 0;
    pub const EDGE_RIGHT: c_int = 1;
    pub const EDGE_TOP: c_int = 2;
    pub const EDGE_BOTTOM: c_int = 3;
    pub const KEYBOARD_NONE: c_int = 0;
    pub const KEYBOARD_ON_DEMAND: c_int = 2;

    #[link(name = "gtk-layer-shell")]
    extern "C" {
        pub fn gtk_layer_is_supported() -> c_int;
        pub fn gtk_layer_is_layer_window(window: *mut GtkWindow) -> c_int;
        pub fn gtk_layer_init_for_window(window: *mut GtkWindow);
        pub fn gtk_layer_set_namespace(window: *mut GtkWindow, name_space: *const c_char);
        pub fn gtk_layer_set_layer(window: *mut GtkWindow, layer: c_int);
        pub fn gtk_layer_set_anchor(window: *mut GtkWindow, edge: c_int, anchor: c_int);
        pub fn gtk_layer_set_margin(window: *mut GtkWindow, edge: c_int, margin: c_int);
        pub fn gtk_layer_set_monitor(window: *mut GtkWindow, monitor: *mut gtk::gdk::ffi::GdkMonitor);
        pub fn gtk_layer_set_exclusive_zone(window: *mut GtkWindow, zone: c_int);
        pub fn gtk_layer_set_keyboard_mode(window: *mut GtkWindow, mode: c_int);
    }
}

/// COUCOU_LAYER_SHELL=0 is the way out on a compositor where it misbehaves.
fn layer_shell_wanted() -> bool {
    std::env::var("COUCOU_LAYER_SHELL").map(|v| v != "0").unwrap_or(true)
}

/// True once the island window is a layer-shell surface.
static LAYER_SURFACE: AtomicBool = AtomicBool::new(false);

/// The input region last asked for, re-applied whenever the window is mapped:
/// GTK resets it to the whole window on map. Until the page reports the island
/// shape it is empty, so nothing takes the mouse.
type Region = Option<(f64, f64, f64, f64)>;
static INPUT_REGION: Mutex<Region> = Mutex::new(Some((0.0, 0.0, 0.0, 0.0)));

fn gtk_window_ptr(win: &gtk::ApplicationWindow) -> *mut gtk::ffi::GtkWindow {
    let w: &gtk::Window = win.upcast_ref();
    w.to_glib_none().0
}

/// Turns the island into an overlay surface on the top edge that never takes
/// the keyboard. Must run before the window is first shown: a layer surface
/// cannot be made out of a window the compositor already knows.
///
/// Without layer-shell (GNOME, X11, or COUCOU_LAYER_SHELL=0) the window stays
/// an ordinary always-on-top window that refuses focus; where it lands is then
/// up to the window manager.
pub fn make_non_activating(win: &WebviewWindow) {
    let Ok(gw) = win.gtk_window() else { return };
    // The pointer watch reports the pointer leaving on every kind of window:
    // a regular one (GNOME, X11) misses `mouseout` just as a layer surface does.
    ISLAND_GTK.with(|cell| *cell.borrow_mut() = Some(gw.clone()));
    watch_pointer_leave(win);
    let wanted = layer_shell_wanted();
    let supported = unsafe { layer::gtk_layer_is_supported() } != 0;
    if !wanted || !supported || gw.is_realized() {
        let why = if !wanted {
            "COUCOU_LAYER_SHELL=0"
        } else if supported {
            "window already shown"
        } else {
            "compositor has no layer-shell"
        };
        crate::log::line(format!("island is a regular window ({why})"));
        gw.set_accept_focus(false);
        if !gw.is_realized() {
            // On X11 a Dock is kept above everything and is the only kind of
            // window, besides the desktop, that "show desktop" leaves alone.
            // COUCOU_DOCK=0 falls back to a utility window.
            let dock = std::env::var("COUCOU_DOCK").map(|v| v != "0").unwrap_or(true);
            gw.set_type_hint(if dock {
                gtk::gdk::WindowTypeHint::Dock
            } else {
                gtk::gdk::WindowTypeHint::Utility
            });
        }
        gw.set_keep_above(true);
        return;
    }
    // tao gives undecorated Wayland windows an empty titlebar to force
    // client-side decorations. A layer surface has none, and a client-decorated
    // GtkWindow recomputes its own input region (shadow margins included) on
    // every map, over ours.
    gw.set_titlebar(None::<&gtk::Widget>);
    let ptr = gtk_window_ptr(&gw);
    unsafe {
        layer::gtk_layer_init_for_window(ptr);
        layer::gtk_layer_set_namespace(ptr, c"coucou".as_ptr());
        layer::gtk_layer_set_layer(ptr, layer::LAYER_OVERLAY);
        // Top edge only: the compositor centres the surface horizontally.
        layer::gtk_layer_set_anchor(ptr, layer::EDGE_TOP, 1);
        // -1: sit right against the screen edge, over any top panel, the way
        // the Mac island sits in the notch.
        layer::gtk_layer_set_exclusive_zone(ptr, -1);
        layer::gtk_layer_set_keyboard_mode(ptr, layer::KEYBOARD_NONE);
    }
    // WebKitGTK in a freshly mapped layer surface never paints its first frame
    // (seen on COSMIC, and reproduced with a bare GTK window + WebKitGTK, no
    // Tauri involved): the surface stays empty. Unmapping and mapping it once,
    // right after the first map, gets it drawing for good.
    let remapped = std::cell::Cell::new(false);
    gw.connect_map_event(move |w, _| {
        apply_input_region(w, *INPUT_REGION.lock().unwrap());
        if !remapped.replace(true) {
            let w = w.clone();
            gtk::glib::idle_add_local_once(move || {
                w.hide();
                w.show_all();
                apply_input_region(&w, *INPUT_REGION.lock().unwrap());
            });
        }
        gtk::glib::Propagation::Proceed
    });
    LAYER_SURFACE.store(true, Ordering::Relaxed);
    crate::log::line("island is a layer-shell overlay");
}

/// In companion mode the panel is a popup near Mochi, not a top-edge notch.
pub fn place_island(win: &WebviewWindow, origin: Option<(f64, f64)>) {
    if !LAYER_SURFACE.load(Ordering::Relaxed) { return; }
    let Ok(gw) = win.gtk_window() else { return };
    let ptr = gtk_window_ptr(&gw);
    let (x, y) = origin.unwrap_or((0.0, 0.0));
    unsafe {
        layer::gtk_layer_set_anchor(ptr, layer::EDGE_LEFT, origin.is_some() as i32);
        layer::gtk_layer_set_margin(ptr, layer::EDGE_LEFT, x.round() as i32);
        layer::gtk_layer_set_margin(ptr, layer::EDGE_TOP, y.round() as i32);
    }
}

thread_local! {
    /// The island's GTK window, for the pointer watch (GTK main thread only).
    static ISLAND_GTK: std::cell::RefCell<Option<gtk::ApplicationWindow>> =
        const { std::cell::RefCell::new(None) };
}

/// Reports the pointer leaving the island, which the page cannot see by itself.
///
/// On a layer surface WebKitGTK does not send `mouseout` when the pointer leaves
/// for good, and GTK's leave-notify fires spuriously while the pointer is still
/// over the island (each followed by a re-enter), so neither tells "really
/// gone". GDK's pointer focus does: it follows the compositor's wl_pointer
/// enter/leave. It is read every 33 ms while the island is shown; both edges go
/// to the page as `pointer-inside`, and leaving also emits the far-away cursor
/// the page uses. While the island is hidden the timer is removed altogether
/// (set_pointer_watch), so a hidden island costs nothing.
static POINTER_WIN: std::sync::OnceLock<WebviewWindow> = std::sync::OnceLock::new();
static WATCH_WANTED: AtomicBool = AtomicBool::new(true);
static WATCH_RUNNING: AtomicBool = AtomicBool::new(false);

fn watch_pointer_leave(win: &WebviewWindow) {
    let _ = POINTER_WIN.set(win.clone());
    start_pointer_watch();
}

/// Starts or parks the pointer watch. Called with the island's collapsed state;
/// safe from any thread (the timer itself lives on the GTK main loop).
pub fn set_pointer_watch(active: bool) {
    WATCH_WANTED.store(active, Ordering::Relaxed);
    if active {
        gtk::glib::MainContext::default().invoke(start_pointer_watch);
    }
}

/// Main thread only.
fn start_pointer_watch() {
    if !WATCH_WANTED.load(Ordering::Relaxed) || WATCH_RUNNING.swap(true, Ordering::Relaxed) {
        return;
    }
    use tauri::Emitter;
    let Some(win) = POINTER_WIN.get().cloned() else {
        WATCH_RUNNING.store(false, Ordering::Relaxed);
        return;
    };
    let inside = std::cell::Cell::new(false);
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(33), move || {
        if !WATCH_WANTED.load(Ordering::Relaxed) {
            WATCH_RUNNING.store(false, Ordering::Relaxed);
            return gtk::glib::ControlFlow::Break;
        }
        let now_inside = ISLAND_GTK
            .with(|cell| {
                let gw = cell.borrow().clone()?;
                let ours = gw.window()?;
                let pointer = gtk::gdk::Display::default()?.default_seat()?.pointer()?;
                let (under, _, _) = pointer.window_at_position();
                Some(under?.toplevel() == ours.toplevel())
            })
            .unwrap_or(false);
        if inside.replace(now_inside) != now_inside {
            // The page gates its own (late, sometimes stale) mouse events on
            // this state, so a mousemove queued before the pointer left cannot
            // pull the island back to "inside".
            let _ = win.emit("pointer-inside", now_inside);
            if !now_inside {
                let _ = win.emit("cursor", crate::island::CursorPayload { x: -10_000.0, y: -10_000.0 });
            }
        }
        gtk::glib::ControlFlow::Continue
    });
}

/// Puts the layer surface on the display whose logical origin is (`x`, `y`).
///
/// A layer surface ignores `set_position`: without an explicit output the
/// compositor maps it on whichever display has focus at that moment, so on a
/// multi-monitor Hyprland or Sway desktop the island wandered from one screen
/// to the other every time it was remapped. A no-op for a regular window, which
/// `set_position` already places.
pub fn pin_to_monitor(win: &WebviewWindow, x: i32, y: i32) {
    let _=pin_layer_to_monitor(win,x,y);
}

/// Select this layer's output by the requested logical origin. A child pet
/// surface must not inherit the output of the hidden card window: that card
/// can still be on the previous display while its geometry is parked.
pub fn pin_layer_to_monitor(win:&WebviewWindow,x:i32,y:i32)->bool {
    let Ok(gw) = win.gtk_window() else { return false };
    if unsafe{layer::gtk_layer_is_layer_window(gtk_window_ptr(&gw))}==0{return false;}
    let display = gw.display();
    let monitor = (0..display.n_monitors())
        .filter_map(|i| display.monitor(i))
        .find(|m| {
            let g = m.geometry();
            g.x() == x && g.y() == y
        });
    let Some(monitor) = monitor else { return false };
    let mon_ptr: *mut gtk::gdk::ffi::GdkMonitor = monitor.to_glib_none().0;
    unsafe { layer::gtk_layer_set_monitor(gtk_window_ptr(&gw), mon_ptr) };
    true
}

/// Temporarily allow keyboard focus so a text field inside the island can be
/// typed in.
pub fn set_activating(win: &WebviewWindow, activating: bool) {
    let Ok(gw) = win.gtk_window() else { return };
    // The island is created `focusable: false` (tauri.linux.conf.json), so GTK
    // refuses focus until we say otherwise — on a layer surface too.
    gw.set_accept_focus(activating);
    if unsafe { layer::gtk_layer_is_layer_window(gtk_window_ptr(&gw)) } != 0 {
        let mode = if activating { layer::KEYBOARD_ON_DEMAND } else { layer::KEYBOARD_NONE };
        unsafe { layer::gtk_layer_set_keyboard_mode(gtk_window_ptr(&gw), mode) };
    }
}

/// Only this rectangle (window-logical pixels) takes the mouse; `None` means
/// the whole window does. Everything outside goes to the window underneath.
pub fn set_input_region(win: &WebviewWindow, rect: Region) {
    *INPUT_REGION.lock().unwrap() = rect;
    let Ok(gw) = win.gtk_window() else { return };
    apply_input_region(&gw, rect);
}

/// Reset a card's native logical allocation after output/scale changes. This
/// touches only the actual captured island objects, with no show/remap/focus.
/// Each write checks the current geometry and native owner; no old mask is kept.
pub(crate) fn resize_island_native_view(win:&WebviewWindow,width:f64,height:f64,stamp:crate::island::GeometryStamp){
    if win.label()!=crate::island::WINDOW_LABEL{return;}
    let Some((width,height))=crate::island::native_logical_size(width,height)else{return;};
    let app=win.app_handle().clone();
    let _=win.with_webview(move|platform|{
        let view=platform.inner().upcast::<gtk::Widget>();
        let Some(gw)=view.toplevel().and_then(|w|w.downcast::<gtk::ApplicationWindow>().ok())else{return;};
        let current=||{
            !gw.in_destruction()&&!view.in_destruction()
                &&ISLAND_GTK.with(|slot|slot.borrow().as_ref().is_some_and(|registered|registered==&gw))
                &&view.toplevel().is_some_and(|top|top==gw.clone().upcast::<gtk::Widget>())
                &&stamp.current(&app)
        };
        if !current(){return;}view.set_size_request(width,height);
        if !current(){return;}gw.set_resizable(true);
        if !current(){return;}gw.set_default_size(width,height);
        if !current(){return;}gw.resize(width,height);
        if !current(){return;}view.queue_resize();
        if !current(){return;}view.queue_draw();
        if !current(){return;}gw.queue_draw();
        // Resize may invalidate the GDK shape. Reapply only CURRENT input, with
        // no region mutex across native writes or synchronous callbacks.
        refresh_island_region(current,||*INPUT_REGION.lock().unwrap(),|region|apply_input_region(&gw,region));
    });
}

fn refresh_island_region(mut current:impl FnMut()->bool,mut read:impl FnMut()->Region,mut apply:impl FnMut(Region)){
    for _ in 0..8{
        if !current(){return;}let region=read();
        if !current(){return;}apply(region);
        if !current(){return;}if read()==region{return;}
    }
    // Nonconvergent callbacks cannot leave an old expanded mask behind. Clear
    // this actual current surface only; never hide/show it or grant focus.
    if current(){apply(Some((0.,0.,0.,0.)));}
}

#[cfg(test)]mod island_resize_tests{
    use super::*;
    use std::cell::{Cell,RefCell};
    #[test]fn resize_shape_repair_uses_latest_policy_after_a_causal_close(){
        let expanded=Some((10.,10.,600.,500.));let closed=Some((0.,0.,0.,0.));
        let policy=Cell::new(expanded);let writes=RefCell::new(Vec::new());
        refresh_island_region(||true,||policy.get(),|r|{writes.borrow_mut().push(r);policy.set(closed);});
        assert_eq!(*writes.borrow(),vec![expanded,closed]);
    }
    #[test]fn resize_shape_repair_never_writes_after_owner_or_geometry_is_lost(){
        let valid=Cell::new(true);let writes=Cell::new(0);
        refresh_island_region(||false,||panic!("read after loss"),|_|panic!("write after loss"));
        refresh_island_region(||valid.get(),||Some((1.,1.,2.,2.)),|_|{writes.set(writes.get()+1);valid.set(false);});
        assert_eq!(writes.get(),1);assert!(!valid.get());
    }
    #[test]fn resize_shape_repair_revalidates_after_policy_read_before_native_write(){
        let valid=Cell::new(true);let reads=Cell::new(0);let writes=Cell::new(0);
        refresh_island_region(||valid.get(),||{
            reads.set(reads.get()+1);valid.set(false);Some((10.,10.,600.,500.))
        },|_|writes.set(writes.get()+1));
        assert_eq!(reads.get(),1);assert_eq!(writes.get(),0);
    }
    #[test]fn resize_shape_repair_keeps_closed_input_and_does_not_reveal_a_card(){
        let closed=Some((0.,0.,0.,0.));let writes=RefCell::new(Vec::new());
        refresh_island_region(||true,||closed,|r|writes.borrow_mut().push(r));
        assert_eq!(*writes.borrow(),vec![closed]);
    }
    #[test]fn resize_shape_repair_bounds_churn_and_clears_only_current_surface(){
        let policy=Cell::new(Some((1.,1.,2.,2.)));let writes=RefCell::new(Vec::new());
        refresh_island_region(||true,||policy.get(),|r|{writes.borrow_mut().push(r);let mut next=r.unwrap();next.0+=1.;policy.set(Some(next));});
        let writes=writes.borrow();assert_eq!(writes.len(),9);assert_eq!(writes.last(),Some(&Some((0.,0.,0.,0.))));
    }
}

fn apply_input_region(gw: &impl IsA<gtk::Widget>, rect: Region) {
    match rect {
        None => gw.input_shape_combine_region(None),
        Some((x, y, w, h)) => {
            let Some(gdk_window) = gw.window() else { return };
            let region = gtk::cairo::Region::create_rectangle(&gtk::cairo::RectangleInt::new(
                x.floor() as i32,
                y.floor() as i32,
                w.ceil().max(0.0) as i32,
                h.ceil().max(0.0) as i32,
            ));
            gdk_window.input_shape_combine_region(&region, 0, 0);
        }
    }
}

// ── Global shortcuts ──────────────────────────────────────────────────────────

/// Global shortcuts are X11 key grabs. A Wayland session has no such thing: a
/// grab made through XWayland only sees keys typed into other X11 windows, so
/// it would look registered and never fire. On Wayland the shortcuts go to the
/// desktop through the XDG GlobalShortcuts portal instead (portal.rs), and
/// where there is none Settings explains how to bind `coucou --shortcut <id>`
/// in the desktop's own keyboard settings.
pub fn global_shortcuts_blocked() -> Option<&'static str> {
    let set = |var: &str| std::env::var_os(var).is_some_and(|v| !v.is_empty());
    let wayland = set("WAYLAND_DISPLAY")
        || std::env::var("XDG_SESSION_TYPE").is_ok_and(|v| v.eq_ignore_ascii_case("wayland"));
    if wayland {
        Some("wayland")
    } else if !set("DISPLAY") {
        Some("no-display")
    } else {
        None
    }
}

/// On X11, AltGr is a modifier of its own (ISO_Level3_Shift): Ctrl+Alt+key
/// never stands for it, so no combination takes a character away.
pub fn ctrl_alt_types(_vk: u16, _shift: bool) -> Option<String> {
    None
}

// ── Desktop Mochi window ──────────────────────────────────────────────────────
//
// Where the window may go decides everything here:
//   * X11 places any window where it is asked: an ordinary always-on-top
//     window, moved with set_position;
//   * a layer-shell compositor won't move a toplevel, but anchors a layer
//     surface wherever its margins say, on the island's display;
//   * GNOME on Wayland has neither, so the feature is off there.
// In every case the input region is Mochi's body, so clicks anywhere else go
// to the desktop underneath without any cursor polling.

/// Which of the three applies. Must run on the GTK main thread.
pub fn desktop_mode() -> super::DesktopMode {
    use gtk::glib::prelude::ObjectExt;
    let wayland = gtk::gdk::Display::default()
        .map(|d| d.type_().name().contains("Wayland"))
        .unwrap_or(false);
    let layer = layer_shell_wanted() && unsafe { layer::gtk_layer_is_supported() } != 0;
    if layer {
        super::DesktopMode::Layer
    } else if !wayland {
        super::DesktopMode::Window
    } else {
        super::DesktopMode::Off
    }
}

/// The input shape last asked for the desktop Mochi, re-applied on every map.
static MOCHI_SHAPE: Mutex<super::MouseShape> = Mutex::new(super::MouseShape::Empty);

/// Sets the window up for `mode` before it is ever shown. False means it can't
/// be used (a layer surface can't be made from a window already shown).
pub fn prepare_desktop_window(win: &WebviewWindow, mode: super::DesktopMode) -> bool {
    let Ok(gw) = win.gtk_window() else { return false };
    gw.set_accept_focus(false);
    match mode {
        super::DesktopMode::Layer => {
            if gw.is_realized() {
                crate::log::line("desktop Mochi: window already shown, no layer surface");
                return false;
            }
            gw.set_titlebar(None::<&gtk::Widget>);
            let ptr = gtk_window_ptr(&gw);
            unsafe {
                layer::gtk_layer_init_for_window(ptr);
                layer::gtk_layer_set_namespace(ptr, c"coucou-mochi".as_ptr());
                // Above windows, below fullscreen video — like a floating panel.
                layer::gtk_layer_set_layer(ptr, layer::LAYER_TOP);
                layer::gtk_layer_set_anchor(ptr, layer::EDGE_TOP, 1);
                layer::gtk_layer_set_anchor(ptr, layer::EDGE_LEFT, 1);
                // -1: margins count from the display's corner, panels or not,
                // the same origin the island uses.
                layer::gtk_layer_set_exclusive_zone(ptr, -1);
                layer::gtk_layer_set_keyboard_mode(ptr, layer::KEYBOARD_NONE);
            }
            // Same first-frame problem as the island (see make_non_activating).
            let remapped = std::cell::Cell::new(false);
            gw.connect_map_event(move |w, _| {
                apply_mouse_shape(w, *MOCHI_SHAPE.lock().unwrap());
                if !remapped.replace(true) {
                    let w = w.clone();
                    gtk::glib::idle_add_local_once(move || {
                        w.hide();
                        w.show_all();
                        apply_mouse_shape(&w, *MOCHI_SHAPE.lock().unwrap());
                    });
                }
                gtk::glib::Propagation::Proceed
            });
            true
        }
        super::DesktopMode::Window => {
            gw.connect_map_event(|w, _| {
                apply_mouse_shape(w, *MOCHI_SHAPE.lock().unwrap());
                gtk::glib::Propagation::Proceed
            });
            true
        }
        _ => false,
    }
}

/// Which part of the desktop Mochi takes the mouse. Main thread.
pub fn set_desktop_shape(win: &WebviewWindow, shape: super::MouseShape) {
    *MOCHI_SHAPE.lock().unwrap() = shape;
    let Ok(gw) = win.gtk_window() else { return };
    apply_mouse_shape(&gw, shape);
}

fn apply_mouse_shape(gw: &impl IsA<gtk::Widget>, shape: super::MouseShape) {
    use gtk::cairo::{RectangleInt, Region};
    if shape == super::MouseShape::Whole {
        gw.input_shape_combine_region(None);
        return;
    }
    let Some(gdk_window) = gw.window() else { return };
    match shape {
        super::MouseShape::Whole => {}
        super::MouseShape::Empty => {
            let region = Region::create_rectangle(&RectangleInt::new(0, 0, 0, 0));
            gdk_window.input_shape_combine_region(&region, 0, 0);
        }
        super::MouseShape::Disc { cx, cy, r } => {
            // A region is made of rectangles: stack 2 px rows into a disc.
            let region = Region::create_rectangle(&RectangleInt::new(0, 0, 0, 0));
            let mut y = (cy - r).floor();
            while y < cy + r {
                let mid = y + 1.0 - cy;
                let half = (r * r - mid * mid).max(0.0).sqrt();
                let row = RectangleInt::new(
                    (cx - half).floor() as i32,
                    y as i32,
                    (2.0 * half).ceil() as i32,
                    2,
                );
                let _ = region.union_rectangle(&row);
                y += 2.0;
            }
            gdk_window.input_shape_combine_region(&region, 0, 0);
        }
    }
}

/// Layer surface only: puts the top-left corner at (x, y), logical pixels from
/// the display's corner. Main thread.
pub fn set_layer_margins(win: &WebviewWindow, x: f64, y: f64) {
    let Ok(gw) = win.gtk_window() else { return };
    let ptr = gtk_window_ptr(&gw);
    unsafe {
        layer::gtk_layer_set_margin(ptr, layer::EDGE_LEFT, x.round() as i32);
        layer::gtk_layer_set_margin(ptr, layer::EDGE_TOP, y.round() as i32);
    }
}

/// Layer surface only: `on` stretches it over the whole display — how a drag
/// gets exact pointer positions on Wayland, where a surface that moves under
/// the pointer can't tell where it is — and `off` folds it back to Mochi's
/// size in the corner. Main thread.
pub fn set_layer_overlay(win: &WebviewWindow, on: bool) {
    let Ok(gw) = win.gtk_window() else { return };
    let ptr = gtk_window_ptr(&gw);
    unsafe {
        layer::gtk_layer_set_anchor(ptr, layer::EDGE_RIGHT, on as i32);
        layer::gtk_layer_set_anchor(ptr, layer::EDGE_BOTTOM, on as i32);
        if on {
            layer::gtk_layer_set_margin(ptr, layer::EDGE_LEFT, 0);
            layer::gtk_layer_set_margin(ptr, layer::EDGE_TOP, 0);
        }
    }
}

/// Layer surface only: puts the desktop Mochi on the island's display and
/// returns that display's logical size. Main thread.
pub fn layer_display(island: &WebviewWindow, mochi: &WebviewWindow) -> Option<(f64, f64)> {
    let island_gw = island.gtk_window().ok()?;
    let display = gtk::gdk::Display::default()?;
    let monitor = island_gw
        .window()
        .and_then(|w| display.monitor_at_window(&w))
        .or_else(|| display.primary_monitor())
        .or_else(|| display.monitor(0))?;
    if let Ok(gw) = mochi.gtk_window() {
        unsafe { layer::gtk_layer_set_monitor(gtk_window_ptr(&gw), monitor.to_glib_none().0) };
    }
    let g = monitor.geometry();
    Some((g.width() as f64, g.height() as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_private_directory_of_ours_can_hold_the_relay_socket() {
        let base = std::env::temp_dir().join(format!("coucou-rt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("runtime");
        std::fs::create_dir_all(&dir).unwrap();
        let set = |mode| std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(mode)).unwrap();

        set(0o700);
        assert!(is_private_dir(&dir));

        // Readable or reachable by group or others: no.
        for open in [0o750, 0o705, 0o755, 0o777, 0o1777] {
            set(open);
            assert!(!is_private_dir(&dir), "{open:o} must be refused");
        }

        // A symlink to a private directory: no, the link itself is what we got.
        set(0o700);
        let link = base.join("link");
        std::os::unix::fs::symlink(&dir, &link).unwrap();
        assert!(!is_private_dir(&link));

        // Missing: no.
        assert!(!is_private_dir(&base.join("missing")));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn x11_is_only_preferred_on_gnome_wayland_unless_opted_out() {
        assert!(should_prefer_x11("wayland", "ubuntu:GNOME", "", ":0"));
        assert!(!should_prefer_x11("x11", "ubuntu:GNOME", "", ":0"));
        assert!(!should_prefer_x11("wayland", "KDE", "", ":0"));
        assert!(!should_prefer_x11("wayland", "GNOME", "0", ":0"));
        // No XWayland: keep the Wayland window rather than no display at all.
        assert!(!should_prefer_x11("wayland", "GNOME", "", ""));
    }

    #[test]
    fn text_scaling_is_undone_for_the_island() {
        assert_eq!(dpi_scale_for("1.25\n").as_deref(), Some("0.8"));
        assert_eq!(dpi_scale_for("1.5").as_deref(), Some("0.6667"));
        assert_eq!(dpi_scale_for("1.0").as_deref(), None);
        assert_eq!(dpi_scale_for("").as_deref(), None);
        assert_eq!(dpi_scale_for("nonsense").as_deref(), None);
        assert_eq!(dpi_scale_for("40").as_deref(), None);
    }

    #[test]
    fn private_dirs_are_closed_to_everyone_else() {
        use std::os::unix::fs::MetadataExt;
        let dir = std::env::temp_dir().join(format!("coucou-priv-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        ensure_private_dir(&dir).unwrap();
        assert_eq!(std::fs::metadata(&dir).unwrap().mode() & 0o777, 0o700);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_pictures_folder_comes_from_user_dirs() {
        let home = Path::new("/home/me");
        let text = "# written by xdg-user-dirs-update\nXDG_DESKTOP_DIR=\"$HOME/Desktop\"\nXDG_PICTURES_DIR=\"$HOME/Images\"\n";
        assert_eq!(xdg_user_dir(text, "XDG_PICTURES_DIR", home), Some(PathBuf::from("/home/me/Images")));
        let absolute = "XDG_PICTURES_DIR=\"/data/pics\"";
        assert_eq!(xdg_user_dir(absolute, "XDG_PICTURES_DIR", home), Some(PathBuf::from("/data/pics")));
        // "$HOME/" means the folder is disabled; relative paths are not paths.
        assert_eq!(xdg_user_dir("XDG_PICTURES_DIR=\"$HOME/\"", "XDG_PICTURES_DIR", home), None);
        assert_eq!(xdg_user_dir("XDG_PICTURES_DIR=\"pics\"", "XDG_PICTURES_DIR", home), None);
        assert_eq!(xdg_user_dir("", "XDG_PICTURES_DIR", home), None);
    }
}

// ── Flow dictation's pill ─────────────────────────────────────────────────────
//
// The pill rests at the bottom (or on the left or right edge) of the screen.
// On X11 it is an ordinary always-on-top window placed with set_position; on a
// layer-shell compositor a layer surface anchored to that edge. Either way the
// input region is the pill and its bubble, so clicks anywhere else go through.

/// The pill's input rectangles, re-applied on every map.
static PILL_SHAPE: Mutex<Vec<(f64, f64, f64, f64)>> = Mutex::new(Vec::new());

/// Sets the pill's window up before it is ever shown. Main thread.
pub fn prepare_pill_window(win: &WebviewWindow, mode: super::DesktopMode) -> bool {
    let Ok(gw) = win.gtk_window() else { return false };
    gw.set_accept_focus(false);
    let reapply = |w: &gtk::ApplicationWindow| apply_rects(w, &PILL_SHAPE.lock().unwrap());
    match mode {
        super::DesktopMode::Layer => {
            if gw.is_realized() {
                return false;
            }
            gw.set_titlebar(None::<&gtk::Widget>);
            let ptr = gtk_window_ptr(&gw);
            unsafe {
                layer::gtk_layer_init_for_window(ptr);
                layer::gtk_layer_set_namespace(ptr, c"coucou-pill".as_ptr());
                layer::gtk_layer_set_layer(ptr, layer::LAYER_OVERLAY);
                layer::gtk_layer_set_anchor(ptr, layer::EDGE_BOTTOM, 1);
                layer::gtk_layer_set_exclusive_zone(ptr, 0);
                layer::gtk_layer_set_keyboard_mode(ptr, layer::KEYBOARD_NONE);
            }
            let remapped = std::cell::Cell::new(false);
            gw.connect_map_event(move |w, _| {
                reapply(w);
                if !remapped.replace(true) {
                    let w = w.clone();
                    gtk::glib::idle_add_local_once(move || {
                        w.hide();
                        w.show_all();
                        apply_rects(&w, &PILL_SHAPE.lock().unwrap());
                    });
                }
                gtk::glib::Propagation::Proceed
            });
            true
        }
        super::DesktopMode::Window => {
            if !gw.is_realized() {
                gw.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
            }
            gw.set_keep_above(true);
            gw.connect_map_event(move |w, _| {
                reapply(w);
                gtk::glib::Propagation::Proceed
            });
            true
        }
        _ => false,
    }
}

/// Layer surface only: anchors the pill to its edge, centred along it. Main thread.
pub fn set_pill_edge(win: &WebviewWindow, edge: &str) {
    let Ok(gw) = win.gtk_window() else { return };
    let ptr = gtk_window_ptr(&gw);
    unsafe {
        layer::gtk_layer_set_anchor(ptr, layer::EDGE_BOTTOM, (edge == "bottom") as i32);
        layer::gtk_layer_set_anchor(ptr, layer::EDGE_LEFT, (edge == "left") as i32);
        layer::gtk_layer_set_anchor(ptr, layer::EDGE_RIGHT, (edge == "right") as i32);
        layer::gtk_layer_set_anchor(ptr, layer::EDGE_TOP, 0);
    }
}

/// The parts of the pill's window that take the mouse. Main thread.
pub fn set_pill_input(win: &WebviewWindow, rects: Vec<(f64, f64, f64, f64)>) {
    *PILL_SHAPE.lock().unwrap() = rects.clone();
    let Ok(gw) = win.gtk_window() else { return };
    apply_rects(&gw, &rects);
}

fn apply_rects(gw: &impl IsA<gtk::Widget>, rects: &[(f64, f64, f64, f64)]) {
    use gtk::cairo::{RectangleInt, Region};
    let Some(gdk_window) = gw.window() else { return };
    let region = Region::create_rectangle(&RectangleInt::new(0, 0, 0, 0));
    for &(x, y, w, h) in rects {
        let _ = region.union_rectangle(&RectangleInt::new(
            x.floor() as i32,
            y.floor() as i32,
            w.ceil().max(0.0) as i32,
            h.ceil().max(0.0) as i32,
        ));
    }
    gdk_window.input_shape_combine_region(&region, 0, 0);
}

// The companions habitat has its own shape; never share the microphone pill's
// region or the single desktop Mochi's region with this surface.
static COMPANIONS_SHAPE: Mutex<Vec<(f64, f64, f64, f64)>> = Mutex::new(Vec::new());
thread_local! {static COMPANIONS_POINTER_WATCH:std::cell::RefCell<Option<gtk::glib::SourceId>>=const{std::cell::RefCell::new(None)};}

/// A Wayland input mask can lose DOM pointerleave entirely. Report native
/// pointer ownership while the habitat has visible hit regions, and park the
/// observer when hidden/empty. This carries no keyboard or pet authority.
fn companions_pointer_coordinates(ours:&gtk::gdk::Window,mut under:gtk::gdk::Window,mut x:f64,mut y:f64)->Option<(f64,f64)>{
    // Follow GDK's child/embedder transforms; coordinates are already logical.
    // A malformed or unexpectedly detached hierarchy cannot keep us looping.
    for _ in 0..64 {
        if &under==ours{return Some((x,y));}
        (x,y)=under.coords_to_parent(x,y);
        under=under.effective_parent()?;
    }
    None
}

fn watch_companions_pointer(win:&WebviewWindow,gw:&gtk::ApplicationWindow,active:bool){
    use tauri::Emitter;
    COMPANIONS_POINTER_WATCH.with(|slot|{
        if !active {
            if let Some(source)=slot.borrow_mut().take(){
                source.remove();
                let _=win.emit("companions-pointer-inside",false);
            }
            return;
        }
        if slot.borrow().is_some(){return;}
        let win=win.clone();let gw=gw.clone();let inside=std::cell::Cell::new(None);
        let source=gtk::glib::timeout_add_local(std::time::Duration::from_millis(33),move||{
            let position=(||{
                let ours=gw.window()?;let pointer=gw.display().default_seat()?.pointer()?;
                // Query the pointer's actual window, not the queried habitat's
                // child hit-test. On Wayland the latter can be None even when
                // the pointer focus is the habitat toplevel itself.
                let(under,x,y)=pointer.window_at_position_double();
                let under=under?;
                if !gw.is_visible()||under.toplevel()!=ours.toplevel(){return None;}
                // GDK may return a WebKit child with child-relative coordinates.
                // Keep ownership even if its transform cannot reach our window;
                // unusable coordinates must never recover a pet's pointer hold.
                Some(companions_pointer_coordinates(&ours,under,x,y)
                    .map(|(x,y)|(x,y,ours.width(),ours.height())))
            })();
            let now_inside=position.is_some();
            if inside.replace(Some(now_inside))!=Some(now_inside){
                let _=win.emit("companions-pointer-inside",now_inside);
                // This observation alone cannot open a pet. A fresh opening
                // also requires the page's matching trusted pointer movement.
                if let Some(Some((x,y,width,height)))=position{
                    if x.is_finite()&&y.is_finite()&&x>=0.0&&y>=0.0&&x<f64::from(width)&&y<f64::from(height){
                        let _=win.emit("companions-pointer-position",crate::island::CursorPayload{x,y});
                    }
                }
            }
            gtk::glib::ControlFlow::Continue
        });
        *slot.borrow_mut()=Some(source);
    });
}

pub fn prepare_companions_window(win: &WebviewWindow, mode: super::DesktopMode) -> bool {
    use tauri::Emitter;
    let Ok(gw) = win.gtk_window() else { return false };
    gw.set_accept_focus(false);
    if mode == super::DesktopMode::Layer {
        if gw.is_realized() { return false; }
        gw.set_titlebar(None::<&gtk::Widget>);
        let ptr = gtk_window_ptr(&gw);
        unsafe {
            layer::gtk_layer_init_for_window(ptr);
            layer::gtk_layer_set_namespace(ptr, c"coucou-companions".as_ptr());
            // The DMS bar has an invisible full-width scroll strip. The pet
            // discs must receive pointer/drop events above it; the native mask
            // still passes through every pixel outside the visible pets.
            layer::gtk_layer_set_layer(ptr, layer::LAYER_OVERLAY);
            layer::gtk_layer_set_anchor(ptr, layer::EDGE_TOP, 1);
            layer::gtk_layer_set_anchor(ptr, layer::EDGE_LEFT, 1);
            layer::gtk_layer_set_exclusive_zone(ptr, -1);
            layer::gtk_layer_set_keyboard_mode(ptr, layer::KEYBOARD_NONE);
        }
    } else if mode == super::DesktopMode::Window {
        gw.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
        gw.set_keep_above(true);
    } else { return false; }
    let hidden = win.clone();
    gw.connect_hide(move |w| { watch_companions_pointer(&hidden,w,false);let _ = hidden.emit("companions-visible", false); });
    let shown = win.clone();
    gw.connect_show(move |w| { watch_companions_pointer(&shown,w,!COMPANIONS_SHAPE.lock().unwrap().is_empty());let _ = shown.emit("companions-visible", true); });
    let remapped = std::cell::Cell::new(false);
    gw.connect_map_event(move |w, _| {
        apply_rects(w, &COMPANIONS_SHAPE.lock().unwrap());
        if !remapped.replace(true) {
            let w = w.clone();
            gtk::glib::idle_add_local_once(move || {
                w.hide();
                w.show_all();
                apply_rects(&w, &COMPANIONS_SHAPE.lock().unwrap());
            });
        }
        gtk::glib::Propagation::Proceed
    });
    true
}

pub fn set_companions_input(win: &WebviewWindow, rects: Vec<(f64, f64, f64, f64)>) {
    *COMPANIONS_SHAPE.lock().unwrap() = rects.clone();
    if let Ok(gw) = win.gtk_window() {
        apply_rects(&gw, &rects);
        watch_companions_pointer(win,&gw,gw.is_visible()&&!rects.is_empty());
    }
}

/// An explicitly opened shelter popout can take keyboard focus for search.
static PET_CONTROL_SHAPE:Mutex<Vec<(f64,f64,f64,f64)>>=Mutex::new(Vec::new());
static PET_CONTROL_KEYBOARD:AtomicBool=AtomicBool::new(false);
thread_local! {static PET_CONTROL_POINTER_WATCH:std::cell::RefCell<Option<gtk::glib::SourceId>>=const{std::cell::RefCell::new(None)};}
/// WebKitGTK loses DOM pointerleave when a Wayland input region is left.
/// Observe compositor focus only while expanded; compact/hidden uses no timer.
fn watch_pet_control_pointer(win:&WebviewWindow,gw:&gtk::ApplicationWindow,expanded:bool){
 PET_CONTROL_POINTER_WATCH.with(|slot|{
  if !expanded {if let Some(source)=slot.borrow_mut().take(){source.remove();}return;}
  if slot.borrow().is_some(){return;}
  let win=win.clone();let gw=gw.clone();let inside=std::cell::Cell::new(false);
  let source=gtk::glib::timeout_add_local(std::time::Duration::from_millis(33),move||{
   let now_inside=(||{
    let ours=gw.window()?;let pointer=gw.display().default_seat()?.pointer()?;
    let(under,_,_)=pointer.window_at_position();Some(under?.toplevel()==ours.toplevel())
   })().unwrap_or(false);
   if inside.replace(now_inside)!=now_inside {use tauri::Emitter;let _=win.emit("pet-control-pointer-inside",now_inside);}
   gtk::glib::ControlFlow::Continue
  });
  *slot.borrow_mut()=Some(source);
 });
}
thread_local! {static PET_CONTROL_NATIVE_WINDOW:std::cell::RefCell<Option<gtk::glib::WeakRef<gtk::ApplicationWindow>>>=const{std::cell::RefCell::new(None)};}
fn pet_control_current_native(gw:&gtk::ApplicationWindow)->bool{
 PET_CONTROL_NATIVE_WINDOW.with(|s|s.borrow().as_ref().and_then(|w|w.upgrade()).is_some_and(|w|w==*gw))
}
pub fn set_pet_control_input(win:&WebviewWindow,rects:Vec<(f64,f64,f64,f64)>,expanded:bool,keyboard:bool){
 if let Ok(gw)=win.gtk_window(){set_pet_control_native_input(win,&gw,rects,expanded,keyboard);}
}
/// Scoped actual GTK target; a late callback cannot retarget a same-label window.
/// The binding sequencer resamples policy after this unlocked native operation.
pub fn set_pet_control_native_input(win:&WebviewWindow,gw:&gtk::ApplicationWindow,rects:Vec<(f64,f64,f64,f64)>,expanded:bool,keyboard:bool){
 if !pet_control_current_native(gw){return;}
 *PET_CONTROL_SHAPE.lock().unwrap()=rects.clone();apply_rects(gw,&rects);
 if !pet_control_current_native(gw){return;}
 watch_pet_control_pointer(win,gw,expanded&&gw.is_visible()&&gw.is_mapped());
 if !pet_control_current_native(gw){return;}
 let activating=expanded&&keyboard&&gw.is_visible()&&gw.is_mapped()&&rects.iter().any(|r|r.2>0.0&&r.3>0.0);
 if PET_CONTROL_KEYBOARD.swap(activating,Ordering::Relaxed)!=activating {
  gw.set_accept_focus(activating);
  if !pet_control_current_native(gw){return;}
  if desktop_mode()==super::DesktopMode::Layer{unsafe{layer::gtk_layer_set_keyboard_mode(gtk_window_ptr(gw),if activating{layer::KEYBOARD_ON_DEMAND}else{layer::KEYBOARD_NONE});}}
  if activating&&pet_control_current_native(gw){gw.present();}
 }
}
/// Synchronous native loss notification. No Tauri lookup and no Core lock.
fn revoke_pet_control_activity(gw:&gtk::ApplicationWindow){
 if !pet_control_current_native(gw){return;}
 PET_CONTROL_POINTER_WATCH.with(|slot|{if let Some(source)=slot.borrow_mut().take(){source.remove();}});
 PET_CONTROL_KEYBOARD.store(false,Ordering::Relaxed);gw.set_accept_focus(false);
 if pet_control_current_native(gw)&&desktop_mode()==super::DesktopMode::Layer{unsafe{layer::gtk_layer_set_keyboard_mode(gtk_window_ptr(gw),layer::KEYBOARD_NONE);}}
}
pub fn prepare_pet_control(win:&WebviewWindow)->bool {
 let Ok(gw)=win.gtk_window() else{return false};
 let old=PET_CONTROL_NATIVE_WINDOW.with(|s|s.borrow().as_ref().and_then(|w|w.upgrade()));if let Some(old)=old{revoke_pet_control_activity(&old);}
 PET_CONTROL_NATIVE_WINDOW.with(|s|*s.borrow_mut()=Some(gw.downgrade()));
 *PET_CONTROL_SHAPE.lock().unwrap()=Vec::new();PET_CONTROL_KEYBOARD.store(false,Ordering::Relaxed);gw.set_accept_focus(false);
 gw.connect_hide(revoke_pet_control_activity);gw.connect_unmap(revoke_pet_control_activity);gw.connect_destroy(|w|{
  if pet_control_current_native(w){revoke_pet_control_activity(w);PET_CONTROL_NATIVE_WINDOW.with(|s|{s.borrow_mut().take();});}
 });
 if desktop_mode()==super::DesktopMode::Layer {
  if gw.is_realized(){return false;}
  gw.set_titlebar(None::<&gtk::Widget>);let ptr=gtk_window_ptr(&gw);
  unsafe{layer::gtk_layer_init_for_window(ptr);layer::gtk_layer_set_namespace(ptr,c"coucou-pet-control".as_ptr());layer::gtk_layer_set_layer(ptr,layer::LAYER_OVERLAY);layer::gtk_layer_set_anchor(ptr,layer::EDGE_TOP,1);layer::gtk_layer_set_anchor(ptr,layer::EDGE_LEFT,1);layer::gtk_layer_set_exclusive_zone(ptr,-1);layer::gtk_layer_set_keyboard_mode(ptr,layer::KEYBOARD_NONE);}
  // Initial layer remap is sequenced by the native binding after construction,
  // with actual-object ownership and current Core policy at every native step.
 }else{gw.set_type_hint(gtk::gdk::WindowTypeHint::Dock);gw.set_keep_above(true);}
 true
}
#[path="linux_chat.rs"]mod chat_window;
static PET_CHAT_NORMAL:AtomicBool=AtomicBool::new(false);
pub fn pet_chat_normal_window()->bool { PET_CHAT_NORMAL.load(Ordering::Relaxed) }
pub fn place_pet_chat(x:f64,y:f64,width:f64,height:f64){chat_window::place(x,y,width,height);}
pub fn prepare_pet_chat(win:&WebviewWindow)->bool {
 let normal=chat_window::supported();PET_CHAT_NORMAL.store(normal,Ordering::Relaxed);
 if normal{chat_window::prepare(win)}else{prepare_pet_popup(win,c"coucou-pet-chat")}
}
fn prepare_pet_popup(win:&WebviewWindow,namespace:&std::ffi::CStr)->bool {
 let Ok(gw)=win.gtk_window() else{return false};
 gw.set_accept_focus(true);
 gw.set_focus_on_map(false);
 if desktop_mode()==super::DesktopMode::Layer{
  if gw.is_realized(){return false;}
  gw.set_titlebar(None::<&gtk::Widget>);let ptr=gtk_window_ptr(&gw);
  unsafe{layer::gtk_layer_init_for_window(ptr);layer::gtk_layer_set_namespace(ptr,namespace.as_ptr());layer::gtk_layer_set_layer(ptr,layer::LAYER_OVERLAY);layer::gtk_layer_set_anchor(ptr,layer::EDGE_TOP,1);layer::gtk_layer_set_anchor(ptr,layer::EDGE_LEFT,1);layer::gtk_layer_set_exclusive_zone(ptr,-1);layer::gtk_layer_set_keyboard_mode(ptr,layer::KEYBOARD_ON_DEMAND);}
  // WebKitGTK needs the same initial remap as the island and habitat. Without
  // it the DOM is accessible but the first layer buffer can remain invisible.
  let remapped=std::cell::Cell::new(false);
  gw.connect_map_event(move|w,_|{
   if !remapped.replace(true){let w=w.clone();gtk::glib::idle_add_local_once(move||{if w.is_visible(){w.hide();w.show_all();}});}
   gtk::glib::Propagation::Proceed
  });
 }else{gw.set_type_hint(gtk::gdk::WindowTypeHint::Utility);gw.set_keep_above(true);}
 true
}
pub fn place_pet_panel(app:&tauri::AppHandle,win:&WebviewWindow,x:f64,y:f64){
 if desktop_mode()==super::DesktopMode::Layer{if let Some(island)=crate::island::window(app){layer_display(&island,win);}set_layer_margins(win,x,y);}
 else{let _=win.set_position(tauri::LogicalPosition::new(x,y));}
}
