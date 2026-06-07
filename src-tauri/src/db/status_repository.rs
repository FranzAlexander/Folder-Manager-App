use rusqlite::{params, Connection};

use crate::{error::AppResult, model::Status};

pub fn select_all_statuses(conn: &Connection) -> AppResult<Vec<Status>> {
    let mut stmt = conn.prepare("SELECT id, name FROM status ORDER BY name")?;
    let statuses: Vec<Status> = stmt
        .query_map([], |row| {
            Ok(Status {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(statuses)
}

pub fn insert_status(conn: &Connection, status: String) -> AppResult<Status> {
    conn.execute("INSERT INTO status (name) VALUES (?1)", [&status])?;

    let id = conn.last_insert_rowid();

    Ok(Status { id, name: status })
}

pub fn insert_file_status(conn: &Connection, path: String, status_id: i64) -> AppResult<()> {
    let file_id: i64 = conn.query_row(
        "SELECT id FROM user_file_data WHERE path = ?1",
        [&path],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO file_status(file_id, status_id) VALUES(?1, ?2)",
        [file_id, status_id],
    )?;

    Ok(())
}

pub fn delete_file_status(conn: &Connection, path: &str, status_id: i64) -> AppResult<()> {
    conn.execute(
        "DELETE FROM file_status
         WHERE file_id = (SELECT id FROM user_file_data WHERE path = ?1)
         AND status_id = ?2",
        params![path, status_id],
    )?;
    Ok(())
}
