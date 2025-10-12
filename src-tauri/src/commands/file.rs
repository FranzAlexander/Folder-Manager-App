use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use chrono::{DateTime, Local};
use rusqlite::Connection;
use tauri::Manager;

use crate::{
    db::file_repository::{insert_files, select_file_status, select_file_tags, select_files},
    error::AppResult,
    model::{AppConfig, AppState, FileSystemEntry, SearchEvent},
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

    Ok(config.root_directory)
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
) -> AppResult<Vec<FileSystemEntry>> {
    let mut entries: Vec<FileSystemEntry> = fs::read_dir(&path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            build_file_entry(entry, metadata)
        })
        .collect();

    let app_state = state.lock().unwrap();
    let conn = &app_state.conn;

    ensure_files_in_database(conn, &entries)?;
    update_with_tags_and_status(conn, &mut entries)?;

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

#[tauri::command]
pub async fn search_files(
    app: tauri::AppHandle,
    path: String,
    name: String,
    on_event: tauri::ipc::Channel<SearchEvent>,
) {
    let mut dirs_to_read = vec![PathBuf::from(path)];
    let mut found_any = false;

    while let Some(dir) = dirs_to_read.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut matching_entries: Vec<FileSystemEntry> = Vec::new();

        for entry in entries.filter_map(|e| e.ok()) {
            let Some(metadata) = entry.metadata().ok() else {
                continue;
            };

            if metadata.is_dir() {
                dirs_to_read.push(entry.path());
            }

            let file_name_os = entry.file_name();
            let Some(file_name) = file_name_os.to_str() else {
                continue;
            };

            if !file_name.to_lowercase().contains(&name.to_lowercase()) {
                continue;
            }

            if let Some(file_entry) = build_file_entry(entry, metadata) {
                matching_entries.push(file_entry)
            }
        }
        if !matching_entries.is_empty() {
            found_any = true;
            let state = app.state::<Mutex<AppState>>();
            let app_state = state.lock().unwrap();
            let conn = &app_state.conn;

            let _ = update_with_tags_and_status(conn, &mut matching_entries);

            drop(app_state);

            let _ = on_event.send(SearchEvent::Searching {
                entries: matching_entries,
            });
        }
    }

    let _ = on_event.send(if found_any {
        SearchEvent::Done
    } else {
        SearchEvent::NotFound
    });
}

#[tauri::command]
pub async fn move_files(src: String, dest: String) {
    let src_path = Path::new(&src);
    let dest_path = Path::new(&dest);

    let dest_name = dest_path.join(src_path.file_name().unwrap());

    if dest_name.try_exists().expect("Failed to check") {
        println!("exists")
    } else {
        println!("doesn't exists")
    }
}

fn build_file_entry(entry: fs::DirEntry, metadata: fs::Metadata) -> Option<FileSystemEntry> {
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
}

fn ensure_files_in_database(conn: &Connection, entries: &[FileSystemEntry]) -> AppResult<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    let files_in_db = select_files(conn, paths)?;

    let db_paths: HashSet<String> = files_in_db.iter().map(|f| f.path.clone()).collect();

    let files_to_insert: Vec<&FileSystemEntry> = entries
        .iter()
        .filter(|entry| !db_paths.contains(&entry.path))
        .collect();

    if !files_to_insert.is_empty() {
        insert_files(conn, files_to_insert)?; // Auto-converts
    }

    Ok(())
}

fn update_with_tags_and_status(
    conn: &Connection,
    entries: &mut [FileSystemEntry],
) -> AppResult<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();

    let path_to_tags = select_file_tags(conn, paths.clone())?;
    let path_to_status = select_file_status(conn, paths)?;

    for entry in entries {
        if let Some(tag_ids) = path_to_tags.get(&entry.path) {
            entry.tag_ids = tag_ids.clone();
        }
        if let Some(status_ids) = path_to_status.get(&entry.path) {
            entry.status_ids = status_ids.clone();
        }
    }
    Ok(())
}
