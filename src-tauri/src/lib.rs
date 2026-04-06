mod commands;
mod db;
mod error;
mod fetcher;
mod models;
pub mod xmltv;

use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_channels,
            commands::get_programs,
            commands::get_favourites,
            commands::toggle_favourite,
            commands::set_channel_visibility,
            commands::get_setting,
            commands::set_setting,
            commands::refresh_data,
        ])
        .setup(|app| {
            use tauri::Manager;
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("tv-tabla.db");
            let conn = db::init_db(&db_path)?;
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
