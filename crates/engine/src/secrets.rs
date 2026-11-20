//! The ElevenLabs API key.
//!
//! It is kept in the operating system's own password store (Windows Credential
//! Manager, macOS Keychain, the Secret Service on Linux). It is never written to
//! a setup, a log, a video or this repo, and nothing in this file prints it.

use crate::error::{Error, Result};
use crate::paths::APP_ID;

const ACCOUNT: &str = "elevenlabs-api-key";
pub const ENV_VAR: &str = "ELEVENLABS_API_KEY";

fn entry() -> Result<keyring::Entry> {
    keyring::Entry::new(APP_ID, ACCOUNT).map_err(|e| Error::other(format!("Could not reach this computer's password store: {e}")))
}

/// Where the key was found, for telling the person. Never the key itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeySource {
    /// The `ELEVENLABS_API_KEY` environment variable.
    Environment,
    /// This computer's password store.
    PasswordStore,
}

/// The key, if there is one. The environment variable wins, so a key can be
/// lent for one run without saving it.
pub fn key() -> Result<Option<(String, KeySource)>> {
    if let Ok(k) = std::env::var(ENV_VAR) {
        if !k.trim().is_empty() {
            return Ok(Some((k.trim().to_string(), KeySource::Environment)));
        }
    }
    match entry()?.get_password() {
        Ok(k) if !k.trim().is_empty() => Ok(Some((k.trim().to_string(), KeySource::PasswordStore))),
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(Error::other(format!("Could not read the key from this computer's password store: {e}"))),
    }
}

pub fn source() -> Option<KeySource> {
    key().ok().flatten().map(|(_, s)| s)
}

pub fn set_key(key: &str) -> Result<()> {
    let key = key.trim();
    if key.is_empty() {
        return Err(Error::other("That key is empty."));
    }
    entry()?.set_password(key).map_err(|e| Error::other(format!("Could not save the key: {e}")))
}

pub fn clear_key() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(Error::other(format!("Could not remove the key: {e}"))),
    }
}
