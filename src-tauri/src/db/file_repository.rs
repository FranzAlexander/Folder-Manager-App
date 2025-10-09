use std::collections::HashMap;

use rusqlite::{params, params_from_iter, Connection};

use crate::{
    error::AppResult,
    model::{FileSystemEntry, UserFileData},
};

pub fn select_files(conn: &Connection, paths: Vec<&str>) -> AppResult<Vec<UserFileData>> {
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

    let mut stmt = conn.prepare(&query)?;

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
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(metadata)
}

pub fn insert_files(conn: &Connection, entries: Vec<&FileSystemEntry>) -> AppResult<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO user_file_data (path, last_opened, last_updated, opened_since_update, user_notes)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(path) DO NOTHING")?;

    for entry in entries {
        stmt.execute(params![
            entry.path,
            Option::<i64>::None,
            Option::<i64>::None,
            false,
            Option::<String>::None
        ])?;
    }

    Ok(())
}

pub fn select_file_tags(
    conn: &Connection,
    paths: Vec<&str>,
) -> AppResult<HashMap<String, Vec<i64>>> {
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

    let mut stmt = conn.prepare(&query)?;

    let mut path_to_tags: HashMap<String, Vec<i64>> = HashMap::new();

    let rows = stmt.query_map(params_from_iter(paths), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?))
    })?;

    for row in rows {
        let (path, tag_id) = row?;
        if let Some(tag_id) = tag_id {
            path_to_tags
                .entry(path)
                .or_insert_with(Vec::new)
                .push(tag_id);
        }
    }

    Ok(path_to_tags)
}

pub fn select_file_status(
    conn: &Connection,
    paths: Vec<&str>,
) -> AppResult<HashMap<String, Vec<i64>>> {
    if paths.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = paths.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    let query = format!(
        "SELECT ufd.path, fs.status_id
        FROM user_file_data ufd
        LEFT JOIN file_status fs ON ufd.id = fs.file_id
        WHERE ufd.path IN ({})",
        placeholders
    );

    let mut stmt = conn.prepare(&query)?;

    let mut path_to_status: HashMap<String, Vec<i64>> = HashMap::new();

    let rows = stmt.query_map(params_from_iter(paths), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?))
    })?;

    for row in rows {
        let (path, status_id) = row?;
        if let Some(status_id) = status_id {
            path_to_status
                .entry(path)
                .or_insert_with(Vec::new)
                .push(status_id);
        }
    }

    Ok(path_to_status)
}
