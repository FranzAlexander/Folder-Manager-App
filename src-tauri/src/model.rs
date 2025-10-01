use serde::{Deserialize, Serialize};

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
