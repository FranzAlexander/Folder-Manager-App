use std::{collections::HashSet, fs};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::database::{insert_files, select_file_tags, select_files};

#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    root_directory: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_file: bool,
    pub size: Option<u64>,
    pub path: String,
    pub date_modified: String,
    pub file_type: String,
    pub tag_ids: Vec<i64>,
}

#[tauri::command]
pub async fn get_root_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let config_path = app
        .path()
        .resolve("config.json", tauri::path::BaseDirectory::AppConfig)
        .map_err(|e| format!("Failed to resolve config path: {}", e))?;

    if !config_path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig =
        serde_json::from_str(&contents).map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config.root_directory) // Return the Option directly
}

#[tauri::command]
pub async fn set_root_directory(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let config_path = app
        .path()
        .resolve("config.json", tauri::path::BaseDirectory::AppConfig)
        .map_err(|e| format!("Failed to resolve config path: {}", e))?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config = AppConfig {
        root_directory: Some(path),
    };

    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, config_json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn read_directory(app: tauri::AppHandle, path: String) -> Result<Vec<DirectoryEntry>, String> {
    let mut entries: Vec<DirectoryEntry> = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory '{}': {}", path, e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().into_string().ok()?;
            let path = dunce::canonicalize(entry.path())
                .ok()?
                .to_str()?
                .to_string();

            let date_modified: DateTime<Local> = metadata.modified().ok()?.into();
            let file_type = if metadata.is_dir() {
                String::from("Folder")
            } else {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|s| s.to_uppercase())
                    .unwrap_or_else(|| String::from("File"))
            };

            Some(DirectoryEntry {
                name,
                is_dir: metadata.is_dir(),
                is_file: metadata.is_file(),
                size: if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                },
                path,
                date_modified: date_modified.to_rfc3339(),
                file_type,
                tag_ids: Vec::new(),
            })
        })
        .collect();

    let files_in_db = select_files(&app, entries.iter().map(|e| e.path.as_str()).collect())?;

    let db_paths: HashSet<String> = files_in_db.iter().map(|f| f.path.clone()).collect();

    let files_to_insert: Vec<&DirectoryEntry> = entries
        .iter()
        .filter(|entry| !db_paths.contains(&entry.path))
        .collect();

    if !files_to_insert.is_empty() {
        insert_files(&app, files_to_insert)?;
    }

    let path_to_tags = select_file_tags(&app, entries.iter().map(|e| e.path.as_str()).collect())?;

    for entry in &mut entries {
        if let Some(tag_ids) = path_to_tags.get(&entry.path) {
            entry.tag_ids = tag_ids.clone();
        }
    }
    Ok(entries)
}
