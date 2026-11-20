//! The ElevenLabs key, voices and models. Nothing here spends credits.

use crate::error::{off_thread, AppError, AppResult};
use crate::AppState;
use pvc_engine::elevenlabs::{Client, Credits, ModelInfo, VoiceInfo};
use pvc_engine::secrets::{self, KeySource};
use pvc_engine::speech;
use tauri::State;

/// The real service, unless a test has pointed the app at a stand-in.
fn client(key: &str) -> AppResult<Client> {
    Ok(Client::from_env(key)?)
}

#[tauri::command]
pub async fn key_status() -> AppResult<Option<KeySource>> {
    off_thread(|| Ok(secrets::source())).await
}

/// Save the key in this computer's password store, after checking that it works.
#[tauri::command]
pub async fn key_set(key: String) -> AppResult<Credits> {
    off_thread(move || {
        let credits = client(key.trim())?.credits()?;
        secrets::set_key(&key)?;
        log::info!("ElevenLabs key saved to the password store");
        Ok(credits)
    })
    .await
}

#[tauri::command]
pub async fn key_clear() -> AppResult<()> {
    off_thread(|| Ok(secrets::clear_key()?)).await
}

#[tauri::command]
pub async fn voices(state: State<'_, AppState>, refresh: bool) -> AppResult<Vec<VoiceInfo>> {
    let paths = state.paths.clone();
    off_thread(move || {
        let Some((key, _)) = secrets::key()? else {
            return Err(AppError::Other("No ElevenLabs API key is saved yet. Add it under Settings.".into()));
        };
        Ok(speech::voices(&paths, &client(&key)?, refresh)?)
    })
    .await
}

/// The ElevenLabs models that turn text into speech, and what each charges.
/// Fetching the list also brings the costs the editor shows up to date.
#[tauri::command]
pub async fn models(state: State<'_, AppState>, refresh: bool) -> AppResult<Vec<ModelInfo>> {
    let paths = state.paths.clone();
    off_thread(move || {
        let Some((key, _)) = secrets::key()? else {
            return Err(AppError::Other("No ElevenLabs API key is saved yet. Add it under Settings.".into()));
        };
        Ok(speech::models(&paths, &client(&key)?, refresh)?)
    })
    .await
}

#[tauri::command]
pub async fn credits() -> AppResult<Option<Credits>> {
    off_thread(|| {
        Ok(match secrets::key()? {
            Some((key, _)) => Some(client(&key)?.credits()?),
            None => None,
        })
    })
    .await
}
