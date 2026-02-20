use iroh::SecretKey;
use iroh_gossip::proto::TopicId;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Derive a gossip TopicId from channel name.
pub fn derive_topic(channel: &str) -> TopicId {
    let mut hasher = Sha256::new();
    hasher.update(b"talkd:topic:");
    hasher.update(channel.as_bytes());
    TopicId::from_bytes(hasher.finalize().into())
}

/// Load or create a persistent iroh identity keypair.
pub fn load_or_create_identity(dir: &Path) -> anyhow::Result<SecretKey> {
    let key_path = dir.join("identity");
    if key_path.exists() {
        let bytes = std::fs::read(&key_path)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| {
            anyhow::anyhow!("Invalid identity file — expected 32 bytes")
        })?;
        Ok(SecretKey::from_bytes(&arr))
    } else {
        let mut seed = [0u8; 32];
        getrandom::fill(&mut seed).expect("getrandom failed");
        let key = SecretKey::from_bytes(&seed);
        std::fs::create_dir_all(dir)?;
        std::fs::write(&key_path, key.to_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(key)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Contact {
    pub id: iroh::PublicKey,
    /// Description / note about this contact (for agent search)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
}

/// Load contacts from ~/.talkd/contacts.json
pub fn load_contacts(dir: &Path) -> std::collections::HashMap<String, Contact> {
    let path = dir.join("contacts.json");
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(map) = serde_json::from_str(&data) {
                return map;
            }
        }
    }
    std::collections::HashMap::new()
}

/// Save contacts to ~/.talkd/contacts.json
pub fn save_contacts(
    dir: &Path,
    contacts: &std::collections::HashMap<String, Contact>,
) -> anyhow::Result<()> {
    let path = dir.join("contacts.json");
    let data = serde_json::to_string_pretty(contacts)?;
    std::fs::write(path, data)?;
    Ok(())
}

/// Resolve a target: could be a hex address string or a contact name.
pub fn resolve_target(dir: &Path, target: &str) -> Option<iroh::PublicKey> {
    // Try parsing as hex address
    if let Ok(key) = target.parse::<iroh::PublicKey>() {
        return Some(key);
    }
    // Try contact lookup
    let contacts = load_contacts(dir);
    contacts.get(target).map(|c| c.id)
}

/// Get the client/subscriber ID.
pub fn client_id() -> String {
    if let Ok(id) = std::env::var("TALKD_ID") {
        return id;
    }
    let session_hint = std::env::var("TERM_SESSION_ID")
        .or_else(|_| std::env::var("ITERM_SESSION_ID"))
        .or_else(|_| std::env::var("WEZTERM_PANE"))
        .or_else(|_| std::env::var("TMUX_PANE"))
        .or_else(|_| std::env::var("WINDOWID"))
        .ok();
    if let Some(hint) = session_hint {
        let mut h = Sha256::new();
        h.update(hint.as_bytes());
        return hex::encode(&h.finalize()[..4]);
    }
    "default".into()
}

/// Generate a random 8-hex-char daemon ID.
pub fn generate_daemon_id() -> String {
    let mut bytes = [0u8; 4];
    getrandom::fill(&mut bytes).expect("getrandom failed");
    hex::encode(bytes)
}
