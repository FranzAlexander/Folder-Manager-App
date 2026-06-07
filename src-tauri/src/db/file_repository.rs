use std::collections::HashMap;

use chrono::Utc;
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

pub fn update_file_paths(conn: &Connection, path_updates: &[(String, String)]) -> AppResult<()> {
    if path_updates.is_empty() {
        return Ok(());
    }

    let mut stmt = conn.prepare("UPDATE user_file_data SET path = ?1 WHERE path = ?2")?;

    for (new_path, old_path) in path_updates {
        stmt.execute(params![new_path, old_path])?;
    }

    Ok(())
}

pub fn rename_file_path(conn: &Connection, old_path: &str, new_path: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE user_file_data SET path = ?1 WHERE path = ?2",
        params![new_path, old_path],
    )?;
    Ok(())
}

pub fn update_file_last_opened(conn: &Connection, path: &str) -> AppResult<()> {
    let mut stmt = conn.prepare("UPDATE user_file_data SET last_opened = ?1 WHERE path = ?2")?;

    stmt.execute(params![Utc::now().to_rfc3339(), path])?;
    Ok(())
}

pub fn upsert_file_last_opened(conn: &Connection, path: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO user_file_data (path, last_opened) VALUES (?1, ?2)
         ON CONFLICT(path) DO UPDATE SET last_opened = excluded.last_opened",
        params![path, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn select_file_last_opened(
    conn: &Connection,
    paths: Vec<&str>,
) -> AppResult<HashMap<String, String>> {
    if paths.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = paths.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    let query = format!(
        "SELECT path, last_opened FROM user_file_data WHERE path IN ({}) AND last_opened IS NOT NULL",
        placeholders
    );

    let mut stmt = conn.prepare(&query)?;

    let mut path_to_last_opened: HashMap<String, String> = HashMap::new();

    let rows = stmt.query_map(params_from_iter(paths), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for row in rows {
        let (path, last_opened) = row?;
        path_to_last_opened.insert(path, last_opened);
    }

    Ok(path_to_last_opened)
}

pub fn copy_file_metadata(conn: &Connection, copies: &[(String, String)]) -> AppResult<()> {
    if copies.is_empty() {
        return Ok(());
    }

    for (src_path, dest_path) in copies {
        let src_files = select_files(conn, vec![src_path.as_str()])?;

        let Some(src) = src_files.into_iter().next() else {
            continue;
        };

        conn.execute(
            "INSERT INTO user_file_data (path, user_notes) 
            VALUES (?1, ?2) 
            ON CONFLICT(path) DO NOTHING",
            params![dest_path, src.user_notes],
        )?;

        let dest_id: i64 = conn.query_row(
            "SELECT id FROM user_file_data WHERE path = ?1",
            params![dest_path],
            |row| row.get(0),
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id)
            SELECT ?1, tag_id FROM file_tags WHERE file_id = ?2",
            params![dest_id, src.id],
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO file_status (file_id, status_id)
            SELECT ?1, status_id FROM file_status WHERE file_id = ?2",
            params![dest_id, src.id],
        )?;
    }

    Ok(())
}
