use std::sync::Mutex;

use crate::{
    db::status_repository::{insert_file_status, insert_status, select_all_statuses},
    error::AppResult,
    model::{AppState, Status},
};

#[tauri::command]
pub fn get_status(state: tauri::State<Mutex<AppState>>) -> AppResult<Vec<Status>> {
    let app_state = state.lock().unwrap();
    let conn = &app_state.conn;

    let status = select_all_statuses(conn)?;

    Ok(status)
}

#[tauri::command]
pub fn create_status(state: tauri::State<Mutex<AppState>>, status: String) -> AppResult<Status> {
    let app_state = state.lock().unwrap();
    let conn = &app_state.conn;

    let status = insert_status(conn, status)?;

    Ok(status)
}

#[tauri::command]
pub fn assign_status(
    state: tauri::State<Mutex<AppState>>,
    path: String,
    status_id: i64,
) -> AppResult<()> {
    let app_state = state.lock().unwrap();
    let conn = &app_state.conn;

    insert_file_status(conn, path, status_id)?;

    Ok(())
}
