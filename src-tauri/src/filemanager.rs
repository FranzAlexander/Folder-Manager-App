use std::fs;

use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    root_directory: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DirectoryEntry {
    name: String,
    is_dir: bool,
    is_file: bool,
    size: Option<u64>,
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
pub async fn read_directory(path: String) -> Result<Vec<DirectoryEntry>, String> {
    let entries = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory '{}': {}", path, e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().into_string().ok()?;

            Some(DirectoryEntry {
                name,
                is_dir: metadata.is_dir(),
                is_file: metadata.is_file(),
                size: if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                },
            })
        })
        .collect();

    Ok(entries)
}
