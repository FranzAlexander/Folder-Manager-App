// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod db;
mod error;
mod migration;
mod model;

use std::sync::Mutex;

use tauri::Manager;

use crate::{
    commands::{
        file::{
            get_root_directory, move_files, prepare_operation, read_directory, search_files,
            set_root_directory, start_executable,
        },
        status::{assign_status, create_status, get_status},
        tag::{assign_tag, create_tag, get_tags},
    },
    db::db_init,
    model::{AppState, FileOperationEntry},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let connection = db_init(app.handle());
            let app_data = AppState {
                conn: connection,
                file_op_entries: Vec::new(),
            };
            app.manage(Mutex::new(app_data));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_root_directory,
            set_root_directory,
            read_directory,
            create_tag,
            get_tags,
            assign_tag,
            get_status,
            create_status,
            assign_status,
            start_executable,
            search_files,
            move_files,
            prepare_operation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
