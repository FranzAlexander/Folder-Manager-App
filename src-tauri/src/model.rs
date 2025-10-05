use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub conn: Connection,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub root_directory: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_file: bool,
    pub size: Option<u64>,
    pub path: String,
    pub date_modified: String,
    pub file_type: String,
    pub tag_ids: Vec<i64>,
    pub status_ids: Vec<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct UserFileData {
    pub id: i64,
    pub path: String,
    pub last_opened: Option<String>,
    pub last_updated: Option<String>,
    pub opened_since_update: bool,
    pub user_notes: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Status {
    pub id: i64,
    pub name: String,
}
