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
fn pet_inference_legacy_editor_cannot_clear_binding_or_downgrade_format() {
    let root=PrivateDir::new();let path=root.0.join("pets.json");let mut configured=defaults();configured.version=2;
    let choice=crate::pet_inference::Choice{schema_version:1,backend:crate::pet_inference::Backend::Lmstudio,endpoint:"http://127.0.0.1:1234".into(),model:"synthetic/model".into(),credential:crate::pet_inference::CredentialReference::None};
    configured.pets.get_mut("ai_ollama").unwrap().inference=Some(choice.clone());configured.pets.get_mut("ai_ollama").unwrap().inference_revision=Some("a".repeat(32));write(&path,&configured).unwrap();
    let doc=read_at_policy(&path,false).unwrap();let before=std::fs::read(&path).unwrap();
    let mut old=doc.config.clone();old.version=1;old.pets.get_mut("ai_ollama").unwrap().inference=None;old.pets.get_mut("ai_ollama").unwrap().inference_revision=None;
    old.pets.get_mut("ai_ollama").unwrap().name="Synthetic owner".into();
    assert!(save_at(&path,&old,"stale-revision").is_err());assert_eq!(std::fs::read(&path).unwrap(),before);
    let saved=save_at(&path,&old,&doc.revision).unwrap();assert_eq!(saved.config.version,2);
    assert_eq!(saved.config.pets["ai_ollama"].inference,Some(choice));assert_eq!(saved.config.pets["ai_ollama"].name,"Synthetic owner");
    assert_eq!(saved.config.pets["ai_ollama"].inference_revision,Some("a".repeat(32)));
    assert_eq!(read_at_policy(&path,false).unwrap().revision,saved.revision);
}

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
    assert_eq!(pet.provider,"codex"); assert_eq!(pet.name, "Som pessoal sintético");
    assert!(pet.aliases.iter().any(|a| a == "som de teste"));
    assert_eq!(pet.fast, fast); assert_eq!(pet.permissions, grants);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
}
