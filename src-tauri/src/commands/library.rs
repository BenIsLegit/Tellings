//! Setups: listing, opening, checking, saving and sharing them.

use crate::error::{off_thread, AppError, AppResult};
use crate::AppState;
use pvc_engine::library::{self, Entry, Kind};
use pvc_engine::make::{self, Look};
use pvc_engine::plan::Context;
use pvc_engine::setup::{self, Setup, SCREENS};
use pvc_engine::speech::Forgotten;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::State;

#[derive(Serialize)]
pub struct LibraryList {
    presets: Vec<Entry>,
    saved: Vec<Entry>,
}

#[tauri::command]
pub async fn library_list(state: State<'_, AppState>) -> AppResult<LibraryList> {
    let paths = state.paths.clone();
    off_thread(move || Ok(LibraryList { presets: library::presets(), saved: library::saved(&paths) })).await
}

#[derive(Serialize)]
pub struct Opened {
    entry: Entry,
    /// The sparse setup to edit. A preset opens as "start from this preset", so
    /// saving keeps only what was changed.
    doc: Value,
}

#[tauri::command]
pub async fn setup_open(state: State<'_, AppState>, id: String) -> AppResult<Opened> {
    let paths = state.paths.clone();
    off_thread(move || {
        let found = library::find(&paths, &id)?;
        let doc = match found.entry.kind {
            Kind::Preset => json!({ "extends": found.entry.id }),
            _ => found.doc,
        };
        Ok(Opened { entry: found.entry, doc })
    })
    .await
}

#[derive(Serialize)]
pub struct Checked {
    /// The setup with every default filled in.
    setup: Setup,
    look: Look,
}

/// Fill in a setup, plan its video and say what it would cost. Called on every
/// change in the editor, so it has to be quick. It spends nothing.
#[tauri::command]
pub async fn setup_look(state: State<'_, AppState>, doc: Value) -> AppResult<Checked> {
    let paths = state.paths.clone();
    off_thread(move || {
        let setup = setup::from_doc(&library::resolve(&paths, &doc)?)?;
        let ctx = Context::now(paths.search(Some(&paths.setups())));
        let look = make::look(&paths, &setup, &ctx)?;
        Ok(Checked { setup, look })
    })
    .await
}

/// Throw away the kept recordings heard on one scene, so the next video records
/// them afresh. For a recording that came out wrong. Spends nothing itself: the
/// next video asks before it records, like any other.
#[tauri::command]
pub async fn record_again(state: State<'_, AppState>, doc: Value, scene: usize) -> AppResult<Forgotten> {
    let paths = state.paths.clone();
    off_thread(move || {
        let setup = setup::from_doc(&library::resolve(&paths, &doc)?)?;
        let ctx = Context::now(paths.search(Some(&paths.setups())));
        Ok(make::record_again(&paths, &setup, &ctx, scene)?)
    })
    .await
}

#[tauri::command]
pub async fn setup_save(state: State<'_, AppState>, name: String, doc: Value) -> AppResult<Entry> {
    let paths = state.paths.clone();
    off_thread(move || {
        let entry = library::save(&paths, &name, &doc)?;
        log::info!("saved setup \"{}\"", entry.id);
        Ok(entry)
    })
    .await
}

#[tauri::command]
pub async fn setup_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let paths = state.paths.clone();
    off_thread(move || Ok(library::delete(&paths, &id)?)).await
}

/// Bring in a setup file someone shared.
#[tauri::command]
pub async fn setup_import(state: State<'_, AppState>, path: String) -> AppResult<Entry> {
    let paths = state.paths.clone();
    off_thread(move || {
        let found = library::find(&paths, &path)?;
        let name = found.doc["name"].as_str().filter(|n| !n.trim().is_empty()).map(String::from).unwrap_or_else(|| found.entry.id.clone());
        Ok(library::save(&paths, &name, &found.doc)?)
    })
    .await
}

/// Write a setup to a file to share. Whatever it starts from is folded in, so
/// the file works on its own on someone else's computer.
#[tauri::command]
pub async fn setup_export(state: State<'_, AppState>, doc: Value, path: String) -> AppResult<()> {
    let paths = state.paths.clone();
    off_thread(move || {
        let whole = library::resolve(&paths, &doc)?;
        setup::from_doc(&whole)?;
        std::fs::write(&path, serde_json::to_string_pretty(&whole)? + "\n")?;
        Ok(())
    })
    .await
}

/// The lists the editor's menus are built from.
#[tauri::command]
pub async fn options() -> AppResult<Value> {
    Ok(json!({
        "screens": SCREENS.iter().map(|s| json!({ "key": s.key, "name": s.name, "width": s.width, "height": s.height })).collect::<Vec<_>>(),
        "prayers": pvc_engine::prayers::PRAYERS.iter().map(|p| json!({ "key": p.key, "title": p.title, "lines": p.lines })).collect::<Vec<_>>(),
        "sets": pvc_engine::prayers::SETS.iter().map(|s| json!({
            "key": s.key, "name": s.name, "days": s.days,
            "items": s.items.iter().map(|m| json!({ "name": m.name, "reference": m.reference, "note": m.note })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    }))
}

/// A picture as text the preview can show. The window never reads files itself.
#[tauri::command]
pub async fn picture_data(state: State<'_, AppState>, path: String) -> AppResult<String> {
    let paths = state.paths.clone();
    off_thread(move || {
        let search = paths.search(Some(&paths.setups()));
        let file: PathBuf = pvc_engine::plan::find_file(&path, &search, &pvc_engine::plan::PICTURE_ENDINGS).ok_or_else(|| AppError::Other(format!("Picture not found: {path}")))?;
        Ok(pvc_engine::frames::data_url(&file)?)
    })
    .await
}
