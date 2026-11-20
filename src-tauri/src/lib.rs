//! The app. A thin shell: every command hands straight to the engine, which is
//! the same engine the command-line tool uses.

mod commands;
mod error;

use pvc_engine::paths::Paths;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

pub struct AppState {
    pub paths: Paths,
    /// Set when the person presses Stop.
    pub cancel: Arc<AtomicBool>,
    /// True while a video is being made. Only one at a time.
    pub busy: Arc<AtomicBool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // For checking the app while the person has it open: PVC_SECOND_WINDOW lets a
    // second copy start. That copy keeps its hands off the first one's window. It
    // does not take over as the only copy, and it does not save where its window is.
    // Give it its own PVC_DATA_DIR too, so it shares nothing.
    let second = std::env::var_os("PVC_SECOND_WINDOW").is_some();
    let mut builder = tauri::Builder::default();
    if !second {
        // Always first: opening the app twice shows the window that is already there.
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }));
        builder = builder.plugin(tauri_plugin_window_state::Builder::new().build());
    }
    builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([Target::new(TargetKind::LogDir { file_name: None }), Target::new(TargetKind::Stdout), Target::new(TargetKind::Webview)])
                .max_file_size(5_000_000)
                .rotation_strategy(RotationStrategy::KeepAll)
                .level(log::LevelFilter::Info)
                .format(|out, message, record| {
                    out.finish(format_args!("[{}] [{}] [{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), record.level(), record.target(), message))
                })
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            log::info!("Prayer Video Creator v{} starting up", env!("CARGO_PKG_VERSION"));
            // The same folder the command-line tool uses, so setups and recorded speech are shared.
            let paths = if std::env::var_os("PVC_DATA_DIR").is_some() { Paths::standard() } else { Paths::new(app.handle().path().app_data_dir()?) };
            std::fs::create_dir_all(&paths.data).ok();
            std::fs::create_dir_all(paths.pictures()).ok();
            log::info!("Data folder: {}", paths.data.display());
            app.manage(AppState { paths, cancel: Arc::new(AtomicBool::new(false)), busy: Arc::new(AtomicBool::new(false)) });

            // Remember the window's size and place even if the app is closed abruptly.
            if let Some(window) = app.get_webview_window("main").filter(|_| !second) {
                use tauri_plugin_window_state::{AppHandleExt, StateFlags};
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if matches!(event, tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_)) {
                        let _ = handle.save_window_state(StateFlags::all());
                    }
                });
            }
            log::info!("Startup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::library_list,
            commands::library::setup_open,
            commands::library::setup_look,
            commands::library::record_again,
            commands::library::setup_save,
            commands::library::setup_delete,
            commands::library::setup_import,
            commands::library::setup_export,
            commands::library::options,
            commands::library::picture_data,
            commands::voice::key_status,
            commands::voice::key_set,
            commands::voice::key_clear,
            commands::voice::voices,
            commands::voice::voice_settings,
            commands::voice::models,
            commands::voice::credits,
            commands::render::render_start,
            commands::render::render_cancel,
            commands::system::doctor,
            commands::system::settings_get,
            commands::system::settings_set,
            commands::system::clear_kept_speech,
            commands::system::open_path,
            commands::system::reveal_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Prayer Video Creator");
}
