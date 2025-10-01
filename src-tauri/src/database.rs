use std::{collections::HashMap, sync::Mutex};

use rusqlite::{params, params_from_iter, Connection};
use tauri::Manager;

use crate::{
    filemanager::DirectoryEntry,
    migration::run_migrations,
    model::{Tag, UserFileData},
    AppData,
};

pub fn db_init(app: &tauri::AppHandle) -> Connection {
    let app_db_path = app
        .path()
        .resolve("local.db", tauri::path::BaseDirectory::AppLocalData)
        .unwrap();

    let conn = Connection::open(app_db_path).unwrap();

    run_migrations(&conn).unwrap();

    conn
}

pub fn select_files(app: &tauri::AppHandle, paths: Vec<&str>) -> Result<Vec<UserFileData>, String> {
    let state = app.state::<Mutex<AppData>>();
    let app_data = state.lock().unwrap();

    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = paths.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    let query = format!(
        "SELECT id, path, last_opened, last_updated, opened_since_update, user_notes
    FROM user_file_data
    WHERE PATH IN ({})
    ",
        placeholders
    );

    let mut stmt = app_data.conn.prepare(&query).map_err(|e| e.to_string())?;

    let metadata = stmt
        .query_map(params_from_iter(paths), |row| {
            Ok({
                UserFileData {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    last_opened: row.get(2)?,
                    last_updated: row.get(3)?,
                    opened_since_update: row.get(4)?,
                    user_notes: row.get(5)?,
                }
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(metadata)
}

pub fn insert_files(app: &tauri::AppHandle, entries: Vec<&DirectoryEntry>) -> Result<(), String> {
    let state = app.state::<Mutex<AppData>>();
    let mut app_data = state.lock().unwrap();

    let tx = app_data.conn.transaction().map_err(|e| e.to_string())?;

    {
        let mut stmt = tx.prepare(
        "INSERT INTO user_file_data (path, last_opened, last_updated, opened_since_update, user_notes)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(path) DO NOTHING").map_err(|e| e.to_string())?;

        for entry in entries {
            stmt.execute(params![
                entry.path,
                Option::<i64>::None,
                Option::<i64>::None,
                false,
                Option::<String>::None
            ])
            .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_tags(app: tauri::AppHandle) -> Result<Vec<Tag>, String> {
    let state = app.state::<Mutex<AppData>>();
    let app_data = state.lock().unwrap();

    let mut stmt = app_data
        .conn
        .prepare("SELECT id, name FROM tag")
        .map_err(|e| e.to_string())?;

    let tags = stmt
        .query_map([], |row| {
            Ok({
                Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                }
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tags)
}

#[tauri::command]
pub fn create_tag(app: tauri::AppHandle, tag: String) -> Result<Tag, String> {
    let state = app.state::<Mutex<AppData>>();

    let app_data = state.lock().unwrap();

    app_data
        .conn
        .execute(r#"INSERT INTO tag(name) VALUES (?1)"#, [&tag])
        .map_err(|e| e.to_string())?;

    let id = app_data.conn.last_insert_rowid();
    Ok(Tag { id, name: tag })
}

#[tauri::command]
pub fn assign_tag(app: tauri::AppHandle, path: String, tag_id: i64) -> Result<(), String> {
    let state = app.state::<Mutex<AppData>>();

    let app_data = state.lock().unwrap();

    let file_id = app_data
        .conn
        .query_one(
            "SELECT id FROM user_file_data WHERE path = ?1",
            [&path],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    app_data
        .conn
        .execute(
            "INSERT OR IGNORE INTO file_tags(file_id, tag_id) VALUES (?1, ?2)",
            [&file_id, &tag_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn select_file_tags(
    app: &tauri::AppHandle,
    paths: Vec<&str>,
) -> Result<HashMap<String, Vec<i64>>, String> {
    let state = app.state::<Mutex<AppData>>();
    let app_data = state.lock().unwrap();

    if paths.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = paths.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    let query = format!(
        "SELECT ufd.path, ft.tag_id
        FROM user_file_data ufd
        LEFT JOIN file_tags ft ON ufd.id = ft.file_id
        WHERE ufd.path IN ({})",
        placeholders
    );

    let mut stmt = app_data.conn.prepare(&query).map_err(|e| e.to_string())?;

    let mut path_to_tags: HashMap<String, Vec<i64>> = HashMap::new();

    let rows = stmt
        .query_map(params_from_iter(paths), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (path, tag_id) = row.map_err(|e| e.to_string())?;
        if let Some(tag_id) = tag_id {
            path_to_tags
                .entry(path)
                .or_insert_with(Vec::new)
                .push(tag_id);
        }
    }

    Ok(path_to_tags)
}
