use std::{collections::HashSet, fs, sync::Mutex};

use chrono::{DateTime, Local};
use tauri::Manager;

use crate::{
    db::file_repository::{insert_files, select_file_status, select_file_tags, select_files},
    error::AppResult,
    model::{AppConfig, AppState, FileSystemEntry},
};

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
pub fn read_directory(
    state: tauri::State<Mutex<AppState>>,
    path: String,
) -> Result<Vec<FileSystemEntry>, String> {
    let app_state = state.lock().unwrap();
    let conn = &app_state.conn;

    let mut entries: Vec<FileSystemEntry> = fs::read_dir(&path)
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
                String::from("DIR")
            } else {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|s| s.to_uppercase())
                    .unwrap_or_else(|| String::from("File"))
            };

            Some(FileSystemEntry {
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
                status_ids: Vec::new(),
            })
        })
        .collect();

    let files_in_db = select_files(conn, entries.iter().map(|e| e.path.as_str()).collect())?;

    let db_paths: HashSet<String> = files_in_db.iter().map(|f| f.path.clone()).collect();

    let files_to_insert: Vec<&FileSystemEntry> = entries
        .iter()
        .filter(|entry| !db_paths.contains(&entry.path))
        .collect();

    if !files_to_insert.is_empty() {
        insert_files(conn, files_to_insert)?;
    }

    let path_to_tags = select_file_tags(conn, entries.iter().map(|e| e.path.as_str()).collect())?;

    let path_to_status =
        select_file_status(conn, entries.iter().map(|e| e.path.as_str()).collect())?;

    for entry in &mut entries {
        if let Some(tag_ids) = path_to_tags.get(&entry.path) {
            entry.tag_ids = tag_ids.clone();
        }
        if let Some(status_ids) = path_to_status.get(&entry.path) {
            entry.status_ids = status_ids.clone();
        }
    }
    Ok(entries)
}

#[tauri::command]
pub fn start_executable(app: tauri::AppHandle, path: String) -> AppResult<()> {
    use tauri_plugin_shell::ShellExt;

    let result = app.shell().command(&path).spawn();

    match result {
        Ok(_) => Ok(()),
        Err(tauri_plugin_shell::Error::Io(io_err)) if io_err.raw_os_error() == Some(740) => {
            #[cfg(target_os = "windows")]
            {
                app.shell()
                    .command("powershell")
                    .args([
                        "-Command",
                        &format!("Start-Process -FilePath '{}' -Verb RunAs", path),
                    ])
                    .spawn()
                    .map_err(|_| {
                        use crate::error::AppError;

                        AppError::permission_denied(
                            "Failed to launch with elevation. User may have denied UAC prompt.",
                        )
                    })
                    .map(|_| ())
            }
        }
        Err(e) => Err(e.into()),
    }
}
