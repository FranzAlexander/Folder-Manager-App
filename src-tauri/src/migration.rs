use chrono::Utc;
use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Create migration tracking table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migration_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
    )",
        [],
    )?;

    // Get current version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM migration_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        conn.execute_batch(
            r#"
            CREATE TABLE user_file_data (
                id INTEGER PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                last_opened TEXT,
                last_updated TEXT,
                opened_since_update BOOLEAN DEFAULT 0, 
                user_notes TEXT
            );
            
            CREATE INDEX idx_user_file_data_path ON user_file_data(path);
            CREATE INDEX idx_user_file_data_last_opened ON user_file_data(last_opened);
            "#,
        )?;

        // Record migration
        conn.execute(
            "INSERT INTO migration_version (version, applied_at) VALUES (?1, ?2)",
            [&1.to_string(), &Utc::now().to_rfc3339()],
        )?;
    }

    if current_version < 2 {
        conn.execute_batch(
            r#"
            CREATE TABLE tag (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            );
            
            CREATE INDEX idx_tag_name ON tag(name);

            CREATE TABLE status (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            );
            
            CREATE INDEX idx_status_name ON status(name);

            CREATE TABLE file_tags (
                file_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (file_id, tag_id),
                FOREIGN KEY (file_id) REFERENCES user_file_data(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tag(id) ON DELETE CASCADE
            );

            CREATE TABLE file_status (
                file_id INTEGER NOT NULL,
                status_id INTEGER NOT NULL,
                PRIMARY KEY (file_id, status_id),
                FOREIGN KEY (file_id) REFERENCES user_file_data(id) ON DELETE CASCADE,
                FOREIGN KEY (status_id) REFERENCES status(id) ON DELETE CASCADE
            );
            "#,
        )?;

        conn.execute(
            "INSERT INTO migration_version (version, applied_at) VALUES (?1, ?2)",
            [&2.to_string(), &Utc::now().to_rfc3339()],
        )?;
    }
    Ok(())
}
