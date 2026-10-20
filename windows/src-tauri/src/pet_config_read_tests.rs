//! Read-only configuration fixtures never use the user's configured path.
use super::*;
use std::os::unix::fs::PermissionsExt;

struct PrivateDir(std::path::PathBuf);
impl PrivateDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("mochi-config-read-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap(); Self(path)
    }
}
impl Drop for PrivateDir { fn drop(&mut self) { std::fs::remove_dir_all(&self.0).unwrap(); } }

#[test]
fn read_only_missing_config_cannot_create_defaults_or_parent_directories() {
    let root = PrivateDir::new(); let path = root.0.join("missing/config/pets.json");
    assert!(read_at_policy(&path, false).is_err());
    assert_eq!(std::fs::read_dir(&root.0).unwrap().count(), 0);
    // Existing explicit configuration access retains its creation behavior.
    read_at(&path).unwrap(); assert!(path.is_file());
}

#[test]
fn read_only_legacy_upgrade_preserves_exact_file_bytes_identity_and_grants() {
    let root = PrivateDir::new(); let path = root.0.join("pets.json"); let mut config = defaults();
    let old = config.pets.get_mut("integration_spotify").unwrap(); old.provider = "codex".into();
    old.name = "Som pessoal sintético".into(); old.aliases.push("som de teste".into());
    old.permissions.as_mut().unwrap().documents = true;
    let fast = old.fast.clone(); let grants = old.permissions.clone();
    let bytes = serde_json::to_vec(&config).unwrap(); std::fs::write(&path, &bytes).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let doc = read_at_policy(&path, false).unwrap(); let pet = &doc.config.pets["integration_spotify"];
    assert!(pet.provider.is_empty()); assert_eq!(pet.name, "Som pessoal sintético");
    assert!(pet.aliases.iter().any(|a| a == "som de teste"));
    assert_eq!(pet.fast, fast); assert_eq!(pet.permissions, grants);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
}
