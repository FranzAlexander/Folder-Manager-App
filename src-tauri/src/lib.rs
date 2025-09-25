// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod database;
mod filemanager;
mod migrations;
use std::path::PathBuf;

use migrations::get_migrations;

use crate::filemanager::{get_root_directory, read_directory, set_root_directory};

struct AppData {
    root_dir: Option<PathBuf>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:filemanager.db", get_migrations())
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_root_directory,
            set_root_directory,
            read_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
