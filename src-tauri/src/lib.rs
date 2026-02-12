// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod db;
mod error;
mod migration;
mod model;
mod platform;
mod utils;

use std::sync::Mutex;

use tauri::Manager;

use crate::{
    commands::{
        file::{
            get_root_directory, read_directory, search_files, set_root_directory, start_executable,
        },
        operation::{cancel_operation, execute_operation, prepare_operation},
        status::{assign_status, create_status, get_statuses},
        tag::{assign_tag, create_tag, get_tags},
        trash::build_trash_paths,
    },
    db::db_init,
    model::AppState,
    platform::windows::{get_available_drives, get_current_user_sid},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let connection = db_init(app.handle());
            let current_user_id = get_current_user_sid()?;
            let trash_paths = build_trash_paths(&current_user_id);

            let app_data = AppState {
                conn: connection,
                file_op_entries: Vec::new(),
                operation_type: None,
                current_user_id,
                trash_paths,
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
            get_statuses,
            create_status,
            assign_status,
            start_executable,
            search_files,
            prepare_operation,
            execute_operation,
            cancel_operation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
