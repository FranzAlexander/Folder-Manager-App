use rusqlite::Connection;

use crate::{error::AppResult, model::Tag};

pub fn select_all_tags(conn: &Connection) -> AppResult<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT id, name FROM tag")?;

    let tags = stmt
        .query_map([], |row| {
            Ok({
                Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                }
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tags)
}

pub fn insert_tag(conn: &Connection, tag: String) -> AppResult<Tag> {
    conn.execute(r#"INSERT INTO tag(name) VALUES (?1)"#, [&tag])?;

    let id = conn.last_insert_rowid();

    Ok(Tag { id, name: tag })
}

pub fn insert_file_tag(conn: &Connection, path: String, tag_id: i64) -> AppResult<()> {
    let file_id = conn.query_one(
        "SELECT id FROM user_file_data WHERE path = ?1",
        [&path],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO file_tags(file_id, tag_id) VALUES (?1, ?2)",
        [&file_id, &tag_id],
    )?;

    Ok(())
}
