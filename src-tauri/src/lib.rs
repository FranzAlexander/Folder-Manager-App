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
            create_folder, get_root_directory, open_file, read_directory, rename_entry,
            search_files, set_root_directory, start_executable,
        },
        operation::{cancel_operation, execute_operation, prepare_operation},
        status::{assign_status, create_status, get_statuses, unassign_status},
        tag::{assign_tag, create_tag, get_tags, unassign_tag},
        trash::{
            delete_permanently, delete_trash_entry, get_trash_count, get_trash_entries,
            move_to_trash, restore_trash_entry,
        },
        watcher::{unwatch_directory, watch_directory},
    },
    db::db_init,
    model::AppState,
    platform::windows::get_current_user_sid,
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

            let app_data = AppState {
                conn: connection,
                file_op_entries: Vec::new(),
                operation_type: None,
                current_user_id,
                watcher: None,
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
            unassign_tag,
            get_statuses,
            create_status,
            assign_status,
            unassign_status,
            start_executable,
            open_file,
            search_files,
            prepare_operation,
            execute_operation,
            cancel_operation,
            get_trash_entries,
            get_trash_count,
            restore_trash_entry,
            delete_trash_entry,
            move_to_trash,
            delete_permanently,
            watch_directory,
            unwatch_directory,
            rename_entry,
            create_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
