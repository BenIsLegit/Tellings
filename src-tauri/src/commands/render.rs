//! Making the video. Progress goes to the window as "render-progress" events.

use crate::error::{off_thread, AppError, AppResult};
use crate::AppState;
use pvc_engine::make::{self, Made, Progress, Request};
use pvc_engine::plan::Context;
use pvc_engine::{library, setup};
use serde_json::Value;
use std::sync::atomic::Ordering;
use tauri::{Emitter, State};

/// `approved_letters` is how many letters of new speech the person agreed to in
/// the dialog. The engine refuses to record more than that.
#[tauri::command]
pub async fn render_start(state: State<'_, AppState>, window: tauri::Window, doc: Value, approved_letters: u64) -> AppResult<Made> {
    if state.busy.swap(true, Ordering::SeqCst) {
        return Err(AppError::Other("A video is already being made. Wait for it to finish, or stop it.".into()));
    }
    state.cancel.store(false, Ordering::SeqCst);
    let (paths, cancel, busy) = (state.paths.clone(), state.cancel.clone(), state.busy.clone());
    let result = off_thread(move || {
        let setup = setup::from_doc(&library::resolve(&paths, &doc)?)?;
        let ctx = Context::now(paths.search(Some(&paths.setups())));
        log::info!("making \"{}\" ({} letters of new speech approved)", setup.name, approved_letters);
        let req = Request { paths: &paths, setup: &setup, ctx: &ctx, approved_letters, voice_service: std::env::var("PVC_ELEVENLABS_URL").ok() };
        let mut on_progress = |p: Progress| {
            let _ = window.emit("render-progress", &p);
        };
        Ok(make::make(&req, &mut on_progress, &cancel)?)
    })
    .await;
    busy.store(false, Ordering::SeqCst);
    match &result {
        Ok(made) => log::info!("made {} file(s) in {:.0} s", made.files.len(), made.took),
        Err(e) => log::warn!("the video was not made: {e}"),
    }
    result
}

#[tauri::command]
pub async fn render_cancel(state: State<'_, AppState>) -> AppResult<()> {
    state.cancel.store(true, Ordering::SeqCst);
    Ok(())
}
