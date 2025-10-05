use std::sync::Mutex;

use crate::{
    db::tag_repository::{insert_file_tag, insert_tag, select_all_tags},
    error::AppResult,
    model::{AppState, Tag},
};

#[tauri::command]
pub fn get_tags(state: tauri::State<Mutex<AppState>>) -> AppResult<Vec<Tag>> {
    let app_state = state.lock().unwrap();
    let conn = &app_state.conn;

    let tags = select_all_tags(conn)?;

    Ok(tags)
}

#[tauri::command]
pub fn create_tag(state: tauri::State<Mutex<AppState>>, tag: String) -> AppResult<Tag> {
    let app_state = state.lock().unwrap();
    let conn = &app_state.conn;

    let tag = insert_tag(conn, tag)?;

    Ok(tag)
}

#[tauri::command]
pub fn assign_tag(
    state: tauri::State<Mutex<AppState>>,
    path: String,
    tag_id: i64,
) -> AppResult<()> {
    let app_state = state.lock().unwrap();
    let conn = &app_state.conn;

    insert_file_tag(conn, path, tag_id)?;

    Ok(())
}
