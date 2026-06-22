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
    db::file_repository::{
        insert_files, rename_file_path, select_file_last_opened, select_file_status,
        select_file_tags, select_files, update_file_last_opened, upsert_file_last_opened,
    },
    error::AppResult,
    model::{AppConfig, AppState, FileSystemEntry, SearchEvent},
};

#[cfg(windows)]
const WINDOWS_SYSTEM_FOLDERS: &[&str] = &[
    "$Recycle.Bin",
    "$RECYCLE.BIN",
    "RECYCLER",
    "System Volume Information",
    "Recovery",
];

#[cfg(windows)]
const WINDOWS_SYSTEM_PREFIXES: &[&str] = &["$Windows.~"];

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
    let mut entries: Vec<FileSystemEntry> = read_and_process_entries(&path, true)?;

    let app_state = state.lock()?;
    let conn = &app_state.conn;

    upsert_file_last_opened(conn, &path)?;
    ensure_files_in_database(conn, &entries)?;
    update_with_tags_and_status(conn, &mut entries)?;

    Ok(entries)
}

#[tauri::command]
pub fn open_file(app: tauri::AppHandle, path: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;

    app.opener().open_path(&path, None::<&str>)?;

    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock()?;
    update_file_last_opened(&app_state.conn, &path)?;
    Ok(())
}

#[tauri::command]
pub fn start_executable(app: tauri::AppHandle, path: String) -> AppResult<()> {
    use tauri_plugin_shell::ShellExt;

    let result = app.shell().command(&path).spawn();

    match result {
        Ok(_) => {}
        Err(tauri_plugin_shell::Error::Io(io_err)) if io_err.raw_os_error() == Some(740) => {
            #[cfg(target_os = "windows")]
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
                })?;
        }
        Err(e) => return Err(e.into()),
    }

    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock()?;
    update_file_last_opened(&app_state.conn, &path)?;
    Ok(())
}

#[tauri::command]
pub async fn search_files(
    app: tauri::AppHandle,
    path: String,
    name: String,
    on_event: tauri::ipc::Channel<SearchEvent>,
) {
    let mut found_any = false;
    let mut matching_entries: Vec<FileSystemEntry> = Vec::new();
    let name_lower = name.to_lowercase();

    let flush = |entries: &mut Vec<FileSystemEntry>| {
        if entries.is_empty() {
            return;
        }
        let state = app.state::<Mutex<AppState>>();
        if let Ok(app_state) = state.lock() {
            update_with_tags_and_status(&app_state.conn, entries).ok();
        }
        let _ = on_event.send(SearchEvent::Searching {
            entries: entries.drain(..).collect(),
        });
    };

    let _ = traverse_directory(PathBuf::from(path), |entry, metadata| {
        let file_name_os = entry.file_name();
        let Some(file_name) = file_name_os.to_str() else {
            return Ok(());
        };

        if !file_name.to_lowercase().contains(&name_lower) {
            return Ok(());
        }

        if let Some(file_entry) = build_file_entry(entry, metadata) {
            matching_entries.push(file_entry);
        }

        if matching_entries.len() >= 50 {
            found_any = true;
            flush(&mut matching_entries);
        }

        Ok(())
    });

    if !matching_entries.is_empty() {
        found_any = true;
        flush(&mut matching_entries);
    }

    let _ = on_event.send(if found_any {
        SearchEvent::Done
    } else {
        SearchEvent::NotFound
    });
}

pub fn read_and_process_entries(
    path: &str,
    filter_system: bool,
) -> AppResult<Vec<FileSystemEntry>> {
    let entries: Vec<FileSystemEntry> = fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;

            if filter_system {
                let file_name = entry.file_name();
                let name = file_name.to_str()?;
                if is_system_folder(name) {
                    return None;
                }
            }

            // Follow the link for type/size/date (fs::metadata follows; DirEntry::metadata
            // does not), so a link to a folder reads as a folder. For broken links fall
            // back to the link's own metadata so it still shows up in the listing.
            let metadata = fs::metadata(entry.path())
                .or_else(|_| fs::symlink_metadata(entry.path()))
                .ok()?;
            build_file_entry(entry, metadata)
        })
        .collect();

    Ok(entries)
}

/// The path we store/display for a filesystem item.
///
/// A symlink/junction is treated as its own item, so we keep the link's own
/// location rather than resolving it to its target (canonicalizing only the
/// parent). Non-links are canonicalized to normalize the path, falling back to
/// the raw path so unresolvable entries still appear.
fn stored_path(raw: &Path) -> Option<String> {
    let is_symlink = fs::symlink_metadata(raw)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);

    if is_symlink {
        let name = raw.file_name()?;
        let resolved = raw
            .parent()
            .and_then(|parent| dunce::canonicalize(parent).ok())
            .map(|parent| parent.join(name))
            .unwrap_or_else(|| raw.to_path_buf());
        return resolved.to_str().map(str::to_string);
    }

    match dunce::canonicalize(raw) {
        Ok(canonical) => canonical.to_str().map(str::to_string),
        Err(_) => raw.to_str().map(str::to_string),
    }
}

fn build_file_entry(entry: fs::DirEntry, metadata: fs::Metadata) -> Option<FileSystemEntry> {
    let name = entry.file_name().into_string().ok()?;

    let is_symlink = entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false);

    let path = stored_path(&entry.path())?;

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

    Some(FileSystemEntry {
        name,
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file(),
        is_symlink,
        size: metadata.is_file().then(|| metadata.len()),
        path,
        original_path: None,
        date_modified: date_modified.to_rfc3339(),
        file_type,
        tag_ids: Vec::new(),
        status_ids: Vec::new(),
        last_opened: None,
    })
}

fn is_system_folder(name: &str) -> bool {
    #[cfg(windows)]
    {
        return WINDOWS_SYSTEM_FOLDERS
            .iter()
            .any(|&folder| name.eq_ignore_ascii_case(folder))
            || WINDOWS_SYSTEM_PREFIXES
                .iter()
                .any(|&prefix| name.to_ascii_lowercase().starts_with(prefix));
    }
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
    let path_to_status = select_file_status(conn, paths.clone())?;
    let path_to_last_opened = select_file_last_opened(conn, paths)?;

    for entry in entries {
        if let Some(tag_ids) = path_to_tags.get(&entry.path) {
            entry.tag_ids = tag_ids.clone();
        }
        if let Some(status_ids) = path_to_status.get(&entry.path) {
            entry.status_ids = status_ids.clone();
        }
        if let Some(last_opened) = path_to_last_opened.get(&entry.path) {
            entry.last_opened = Some(last_opened.clone());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn create_folder(parent: String, name: String) -> AppResult<String> {
    use crate::error::AppError;

    let base = PathBuf::from(&parent);
    let mut candidate = base.join(&name);

    if candidate.exists() {
        let mut counter = 2u32;
        loop {
            candidate = base.join(format!("{} ({})", name, counter));
            if !candidate.exists() {
                break;
            }
            counter += 1;
        }
    }

    fs::create_dir(&candidate)?;

    let path = dunce::canonicalize(&candidate)
        .map_err(|e| AppError::FileSystemError(e.to_string()))?
        .to_string_lossy()
        .to_string();

    Ok(path)
}

#[tauri::command]
pub fn rename_entry(
    state: tauri::State<Mutex<AppState>>,
    old_path: String,
    new_name: String,
) -> AppResult<()> {
    use crate::error::AppError;

    if new_name.is_empty() {
        return Err(AppError::InvalidInput("Name cannot be empty".to_string()));
    }
    if new_name.contains('/') || new_name.contains('\\') {
        return Err(AppError::InvalidInput(
            "Name cannot contain path separators".to_string(),
        ));
    }

    let old = PathBuf::from(&old_path);
    let new = old.with_file_name(&new_name);

    if new.exists() {
        return Err(AppError::AlreadyExists(format!(
            "'{}' already exists",
            new_name
        )));
    }

    fs::rename(&old, &new)?;

    // Keep the link's own path for symlinks (canonicalizing would resolve to the
    // target and desync the DB row from the listing).
    let new_path = stored_path(&new)
        .ok_or_else(|| AppError::FileSystemError("Failed to resolve renamed path".to_string()))?;

    let app_state = state.lock()?;
    rename_file_path(&app_state.conn, &old_path, &new_path)?;

    Ok(())
}

pub fn traverse_directory<F>(root: PathBuf, mut callback: F) -> AppResult<()>
where
    F: FnMut(fs::DirEntry, fs::Metadata) -> AppResult<()>,
{
    let mut dirs_to_read = vec![root];

    while let Some(dir) = dirs_to_read.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let is_symlink = entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false);

            // Follow the link for type (fs::metadata follows; DirEntry::metadata does not),
            // falling back to the link's own metadata for broken links.
            let Some(metadata) = fs::metadata(entry.path())
                .or_else(|_| fs::symlink_metadata(entry.path()))
                .ok()
            else {
                continue;
            };

            // Never recurse into symlinked/junction directories — a link pointing at
            // an ancestor would otherwise cause an infinite traversal loop.
            if metadata.is_dir() && !is_symlink {
                dirs_to_read.push(entry.path());
            }

            callback(entry, metadata)?;
        }
    }

    Ok(())
}
