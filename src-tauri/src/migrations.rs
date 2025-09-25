use tauri_plugin_sql::{Migration, MigrationKind};

pub fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create initial tables",
        sql: r#"
                CREATE TABLE user_file_data (
                id INTEGER PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                last_opened DATETIME,
                last_updated DATETIME,
                opened_since_update BOOLEAN DEFAULT 0,
                user_notes TEXT
                );
            
                CREATE INDEX idx_user_file_data_path ON user_file_data(path);
                CREATE INDEX idx_user_file_data_last_opened ON user_file_data(last_opened);
            "#,
        kind: MigrationKind::Up,
    }]
}
