// OS keychain wrapper. libsecret on Linux, Credential Manager on Windows,
// Keychain on macOS — all behind the keyring crate's unified API.
//
// Secrets are NEVER returned to the JavaScript frontend. The frontend can
// only ask whether a secret is set (`has_secret`), set/clear it
// (`set_secret`, `clear_secret`), and trigger features that consume it
// server-side (Phase 7 chat, Phase 8 auto-tag, Voyage embeddings).

use anyhow::{anyhow, Result};
use keyring::Entry;

const SERVICE: &str = "skein";

pub fn allowed(name: &str) -> bool {
    matches!(name, "anthropic_api_key" | "voyage_api_key")
}

fn entry(name: &str) -> Result<Entry> {
    if !allowed(name) {
        return Err(anyhow!("unknown secret name: {}", name));
    }
    Entry::new(SERVICE, name).map_err(|e| anyhow!("opening keychain entry {}: {}", name, e))
}

pub fn has(name: &str) -> bool {
    let Ok(e) = entry(name) else { return false };
    e.get_password().is_ok()
}

pub fn set(name: &str, value: &str) -> Result<()> {
    let e = entry(name)?;
    e.set_password(value)
        .map_err(|err| anyhow!("storing keychain entry {}: {}", name, err))?;
    Ok(())
}

pub fn clear(name: &str) -> Result<()> {
    let e = entry(name)?;
    match e.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(anyhow!("clearing keychain entry {}: {}", name, err)),
    }
}

/// Internal-only — used by Phase 7 (chat) and Phase 8 (auto-tag) to fetch
/// API keys for outbound calls. Never exposed to the frontend.
#[allow(dead_code)]
pub fn read(name: &str) -> Option<String> {
    entry(name).ok().and_then(|e| e.get_password().ok())
}
