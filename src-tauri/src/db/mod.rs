use rusqlite::Connection;
use tauri::Manager;

use crate::migration::run_migrations;

pub mod file_repository;
pub mod status_repository;
pub mod tag_repository;

pub fn db_init(app: &tauri::AppHandle) -> Connection {
    let app_db_path = app
        .path()
        .resolve("local.db", tauri::path::BaseDirectory::AppLocalData)
        .unwrap();

    let conn = Connection::open(app_db_path).unwrap();

    run_migrations(&conn).unwrap();

    conn
}
