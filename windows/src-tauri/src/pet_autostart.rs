//! A versioned Linux install must retain the reviewed environment launcher.
//! Registering the ELF directly would skip DMS discovery at the next login.
use tauri_plugin_autostart::ManagerExt;

pub(crate) fn set(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let home = crate::platform::home_dir();
        let exe = std::env::current_exe().map_err(|_| "Não foi possível identificar a instalação do Coucou.")?;
        let uid = unsafe { libc::geteuid() };
        if linux::managed(&home, &exe, env!("CARGO_PKG_VERSION"), uid) {
            let config = std::env::var_os("XDG_CONFIG_HOME").map(std::path::PathBuf::from).unwrap_or_else(|| home.join(".config"));
            return linux::save(&config, &home.join(".local/bin/coucou"), enabled, uid)
                .map_err(|_| "Não foi possível atualizar o arranque automático através do launcher do Coucou.".to_string());
        }
    }
    let manager = app.autolaunch();
    (if enabled { manager.enable() } else { manager.disable() }).map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
mod linux {
    use std::{fs, io::{self, Write}, os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt}, path::Path, sync::atomic::{AtomicU64, Ordering}};

    pub(super) fn managed(home: &Path, exe: &Path, version: &str, uid: u32) -> bool {
        if exe != home.join(".local/opt/coucou").join(version).join("usr/bin/coucou") { return false; }
        fs::symlink_metadata(home.join(".local/bin/coucou")).is_ok_and(|m|
            m.is_file() && m.uid() == uid && m.mode() & 0o022 == 0 && m.mode() & 0o111 != 0)
    }

    fn entry(launcher: &Path) -> io::Result<String> {
        let path = launcher.to_str().filter(|s| !s.chars().any(char::is_control)).ok_or(io::ErrorKind::InvalidInput)?;
        if !launcher.is_absolute() || path.contains('=') { return Err(io::ErrorKind::InvalidInput.into()); }
        let safe = path.chars().all(|c| c.is_ascii_alphanumeric() || "/._-".contains(c));
        let command = if safe { path.to_string() } else {
            let escaped = path.chars().map(|c| match c {
                '%' => "%%".into(),
                '"' | '`' | '$' | '\\' => format!("\\{c}"),
                _ => c.to_string(),
            }).collect::<String>();
            // Desktop Entry Exec has a value-unescape pass before argument
            // unquoting: https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html
            format!("\"{}\"", escaped.replace('\\', "\\\\"))
        };
        Ok(format!("[Desktop Entry]\nType=Application\nVersion=1.0\nName=Coucou\nComment=Coucoustartup script\nExec={command} --autostart\nStartupNotify=false\nTerminal=false\n"))
    }

    fn regular_owned(path: &Path, uid: u32) -> io::Result<bool> {
        match fs::symlink_metadata(path) {
            Ok(m) if m.is_file() && m.uid() == uid && m.len() <= 16*1024 => Ok(true),
            Ok(_) => Err(io::ErrorKind::PermissionDenied.into()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub(super) fn save(config: &Path, launcher: &Path, enabled: bool, uid: u32) -> io::Result<()> {
        if !config.is_absolute() { return Err(io::ErrorKind::InvalidInput.into()); }
        // Do not write through external configuration parents or file symlinks.
        for parent in config.ancestors() {
            if fs::symlink_metadata(parent).is_ok_and(|m| m.file_type().is_symlink()) {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
        }
        let dir = config.join("autostart");
        if !enabled && !dir.exists() { return Ok(()); }
        fs::create_dir_all(&dir)?;
        let parent = fs::symlink_metadata(&dir)?;
        if !parent.is_dir() || parent.uid() != uid { return Err(io::ErrorKind::PermissionDenied.into()); }
        let target = dir.join("Coucou.desktop");
        let exists = regular_owned(&target, uid)?;
        if !enabled {
            if exists { fs::remove_file(target)?; fs::File::open(&dir)?.sync_all()?; }
            return Ok(());
        }
        let contents = entry(launcher)?;
        if exists && fs::read(&target)? == contents.as_bytes() { return Ok(()); }
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let temporary = dir.join(format!(".Coucou-{}-{}.tmp", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        let result = (|| {
            let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary)?;
            file.write_all(contents.as_bytes())?;
            file.set_permissions(fs::Permissions::from_mode(0o644))?;
            file.sync_all()?;
            regular_owned(&target, uid)?;
            fs::rename(&temporary, &target)?;
            fs::File::open(&dir)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() { let _ = fs::remove_file(temporary); }
        result
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn fixture() -> std::path::PathBuf {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!("coucou-autostart-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
            fs::create_dir(&p).unwrap(); p
        }
        #[test]
        fn managed_install_requires_exact_version_and_owned_executable_wrapper() {
            let h=fixture(); let p=h.join(".local/bin/coucou"); fs::create_dir_all(p.parent().unwrap()).unwrap(); fs::write(&p,"#!/bin/sh\n").unwrap();
            fs::set_permissions(&p,fs::Permissions::from_mode(0o755)).unwrap(); let uid=unsafe{libc::geteuid()};
            let exe=h.join(".local/opt/coucou/v1/usr/bin/coucou"); assert!(managed(&h,&exe,"v1",uid));
            assert!(!managed(&h,&exe,"v2",uid)); assert!(!managed(&h,&exe,"v1",uid+1));
            fs::set_permissions(&p,fs::Permissions::from_mode(0o775)).unwrap(); assert!(!managed(&h,&exe,"v1",uid));
            fs::remove_file(&p).unwrap(); std::os::unix::fs::symlink("/usr/bin/true",&p).unwrap(); assert!(!managed(&h,&exe,"v1",uid));
            fs::remove_dir_all(h).unwrap();
        }
        #[test]
        fn enabling_preserves_launcher_and_an_identical_entry_is_not_replaced() {
            let h=fixture(); let config=h.join("config"); let launcher=h.join(".local/bin/coucou"); let uid=unsafe{libc::geteuid()};
            save(&config,&launcher,true,uid).unwrap(); let p=config.join("autostart/Coucou.desktop"); let before=fs::metadata(&p).unwrap();
            assert!(fs::read_to_string(&p).unwrap().contains(&format!("Exec={} --autostart\n",launcher.display())));
            save(&config,&launcher,true,uid).unwrap(); assert_eq!(before.ino(),fs::metadata(&p).unwrap().ino());
            save(&config,&launcher,false,uid).unwrap(); assert!(!p.exists());
            save(&config,&launcher,true,uid).unwrap(); assert!(p.exists()); fs::remove_dir_all(h).unwrap();
        }
        #[test]
        fn symlinked_entry_or_parent_is_not_followed() {
            let h=fixture(); let config=h.join("config"); fs::create_dir_all(config.join("autostart")).unwrap(); let uid=unsafe{libc::geteuid()};
            let other=h.join("other"); fs::write(&other,"keep").unwrap();
            std::os::unix::fs::symlink(&other,config.join("autostart/Coucou.desktop")).unwrap();
            assert!(save(&config,&h.join("launcher"),true,uid).is_err()); assert_eq!(fs::read_to_string(&other).unwrap(),"keep");
            let link=h.join("linked"); std::os::unix::fs::symlink(&config,&link).unwrap();
            assert!(save(&link,&h.join("launcher"),true,uid).is_err()); fs::remove_dir_all(h).unwrap();
        }
        #[test]
        fn desktop_exec_escapes_control_and_field_codes_without_shell_evaluation() {
            assert!(entry(Path::new("/home/a\n/launcher")).is_err());
            assert!(entry(Path::new("relative")).is_err());
            assert!(entry(Path::new("/home/a=b/launcher")).is_err());
            let text=entry(Path::new("/home/a b/%x/$(`q`)/launcher")).unwrap();
            assert!(text.contains("%%x")); assert!(text.contains("\\\\$")); assert!(text.contains("\\\\`"));
            let slash=entry(Path::new("/home/a\\b/launcher")).unwrap(); assert!(slash.contains("\\\\\\\\"));
            assert_eq!(text.matches("\nExec=").count(),1);
        }
    }
}
