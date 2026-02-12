use std::path::PathBuf;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub conn: Connection,
    pub file_op_entries: Vec<FileOperationEntry>,
    pub operation_type: Option<OperationType>,
    pub current_user_id: String,
    pub trash_paths: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SourceEntry {
    pub path: String,
    pub is_dir: bool,
}

pub struct FileOperationEntry {
    pub src: PathBuf,
    pub dest: PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictingEntry {
    pub name: String,
    pub src: PathBuf,
    pub dest: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub root_directory: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
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

#[derive(Serialize, Clone)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum SearchEvent {
    Searching { entries: Vec<FileSystemEntry> },
    Done,
    NotFound,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ConflictResolution {
    Skip,
    Keep,
    Replace,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum OperationType {
    Copy,
    Move,
}
