//! This computer: the programs the tool needs, its folders, its settings.

use crate::error::{off_thread, AppError, AppResult};
use crate::AppState;
use pvc_engine::paths::Settings;
use pvc_engine::secrets::{self, KeySource};
use pvc_engine::{speech, tools};
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Doctor {
    ffmpeg: Option<String>,
    ffmpeg_problem: Option<String>,
    browser: Option<String>,
    browser_problem: Option<String>,
    key: Option<KeySource>,
    data_folder: String,
    pictures_folder: String,
    videos_folder: String,
    kept_speech_bytes: u64,
    version: String,
}

#[tauri::command]
pub async fn doctor(state: State<'_, AppState>) -> AppResult<Doctor> {
    let paths = state.paths.clone();
    off_thread(move || {
        let settings = paths.settings();
        let ffmpeg = tools::find_ffmpeg(&settings);
        let browser = tools::find_browser(&settings);
        Ok(Doctor {
            ffmpeg: ffmpeg.as_ref().ok().map(|p| p.to_string_lossy().into_owned()),
            ffmpeg_problem: ffmpeg.err().map(|e| e.to_string()),
            browser: browser.as_ref().ok().map(|p| p.to_string_lossy().into_owned()),
            browser_problem: browser.err().map(|e| e.to_string()),
            key: secrets::source(),
            data_folder: paths.data.to_string_lossy().into_owned(),
            pictures_folder: paths.pictures().to_string_lossy().into_owned(),
            videos_folder: paths.output_folder().to_string_lossy().into_owned(),
            kept_speech_bytes: speech::kept_bytes(&paths),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    })
    .await
}

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> AppResult<Settings> {
    Ok(state.paths.settings())
}

#[tauri::command]
pub async fn settings_set(state: State<'_, AppState>, settings: Settings) -> AppResult<()> {
    Ok(state.paths.save_settings(&settings)?)
}

/// Forget all recorded speech. The next video with a voice will have to pay to record it again.
#[tauri::command]
pub async fn clear_kept_speech(state: State<'_, AppState>) -> AppResult<()> {
    let paths = state.paths.clone();
    off_thread(move || {
        for dir in [paths.speech(), paths.clips()] {
            if dir.is_dir() {
                std::fs::remove_dir_all(&dir)?;
            }
        }
        log::warn!("all kept speech was cleared at the person's request");
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn open_path(app: AppHandle, path: String) -> AppResult<()> {
    let p = std::path::Path::new(&path);
    if p.extension().is_none() {
        std::fs::create_dir_all(p).ok();
    }
    app.opener().open_path(path, None::<&str>).map_err(|e| AppError::Other(format!("Could not open that: {e}")))
}

#[tauri::command]
pub async fn reveal_path(app: AppHandle, path: String) -> AppResult<()> {
    app.opener().reveal_item_in_dir(path).map_err(|e| AppError::Other(format!("Could not show that file: {e}")))
}
