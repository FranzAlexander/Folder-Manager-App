// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod database;
mod filemanager;
mod migration;
mod model;

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

use crate::{
    database::{assign_tag, create_tag, db_init, get_tags},
    filemanager::{get_root_directory, read_directory, set_root_directory},
};

pub struct AppData {
    conn: Connection,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let connection = db_init(app.handle());
            let app_data = AppData { conn: connection };
            app.manage(Mutex::new(app_data));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_root_directory,
            set_root_directory,
            read_directory,
            create_tag,
            get_tags,
            assign_tag
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
