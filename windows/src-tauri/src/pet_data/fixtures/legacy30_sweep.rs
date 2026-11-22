const KEEP_FOR: Duration = Duration::from_secs(7 * 24 * 60 * 60);

fn sweep(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(copied) = meta.modified() else { continue };
        if now.duration_since(copied).map(|age| age > KEEP_FOR).unwrap_or(false) {
            if meta.is_dir(){let _=std::fs::remove_dir_all(entry.path());}else{let _ = std::fs::remove_file(entry.path());}
        }
    }
}
