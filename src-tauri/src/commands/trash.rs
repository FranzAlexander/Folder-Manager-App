use std::{
    fs,
    io::Read,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Local};

#[cfg(windows)]
use crate::platform::windows::get_available_drives;
use crate::{
    error::{AppError, AppResult},
    model::{AppState, FileSystemEntry},
};

fn build_trash_paths(user_id: &str) -> Vec<PathBuf> {
    #[cfg(windows)]
    get_available_drives()
        .iter()
        .map(|drive| {
            let mut path = PathBuf::from(drive);
            path.push("$Recycle.Bin");
            path.push(user_id);
            path
        })
        .filter(|p| p.exists())
        .collect()
}

fn parse_recycle_bin_info(i_file_path: &PathBuf) -> AppResult<(String, SystemTime, u64)> {
    // $I files contain:
    // - Bytes 0-7: Header
    // - Bytes 8-15: Original file size
    // - Bytes 16-23: Deletion timestamp
    // - Bytes 24+: Original file path (UTF-16)

    let mut file = fs::File::open(i_file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.len() < 28 {
        return Err(AppError::FileSystemError("$I file too small".into()));
    }

    let version = u64::from_le_bytes(buffer[0..8].try_into()?);

    let size = u64::from_le_bytes(buffer[8..16].try_into()?);

    let deletion_timestamp = u64::from_le_bytes(buffer[16..24].try_into()?);
    let deletion_time = filetime_to_system_time(deletion_timestamp).unwrap_or_else(SystemTime::now);

    let path_start = if version == 2 { 28 } else { 24 };

    if buffer.len() < path_start {
        return Err(AppError::FileSystemError("$I file truncated".into()));
    }

    let path_bytes = &buffer[path_start..];
    let original_path = decode_utf16_le(path_bytes)?;

    Ok((original_path, deletion_time, size))
}

fn decode_utf16_le(bytes: &[u8]) -> AppResult<String> {
    let u16_vec: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|&c| c != 0)
        .collect();

    Ok(String::from_utf16(&u16_vec)?)
}

#[tauri::command]
pub fn get_trash_entries(state: tauri::State<Mutex<AppState>>) -> AppResult<Vec<FileSystemEntry>> {
    let app_state = state
        .lock()
        .map_err(|_| "Failed to acquire lock on app state")?;

    let current_user_id = &app_state.current_user_id;
    let trash_paths = build_trash_paths(current_user_id);

    let mut all_entries = Vec::new();

    for trash_path in trash_paths {
        let Ok(dir_iter) = fs::read_dir(&trash_path) else {
            continue;
        };

        for entry in dir_iter {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();

            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            if file_name.starts_with("$I") {
                let Ok((original_path, deletion_date, original_size)) =
                    parse_recycle_bin_info(&path)
                else {
                    eprintln!("Failed to parse trash metadata: {:?}", path);
                    continue;
                };

                let path_buf = PathBuf::from(&original_path);

                let name = path_buf
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let date_modified: DateTime<Local> = deletion_date.into();

                let file_type = path_buf
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|s| s.to_uppercase())
                    .unwrap_or_else(|| String::from("File"));

                let unique_path = path.to_string_lossy().to_string();

                all_entries.push(FileSystemEntry {
                    name,
                    is_dir: false,
                    is_file: true,
                    size: Some(original_size),
                    path: unique_path,
                    original_path: Some(original_path),
                    date_modified: date_modified.to_rfc3339(),
                    file_type,
                    tag_ids: Vec::new(),
                    status_ids: Vec::new(),
                });
            }
        }
    }

    Ok(all_entries)
}

fn filetime_to_system_time(filetime: u64) -> Option<SystemTime> {
    const FILETIME_TO_UNIX_SECONDS: u64 = 11_644_473_600;

    // Convert 100-nanosecond intervals to seconds
    let seconds = filetime.checked_div(10_000_000)?;
    let nanos = ((filetime % 10_000_000) * 100) as u32;

    // Subtract the epoch difference
    let unix_seconds = seconds.checked_sub(FILETIME_TO_UNIX_SECONDS)?;

    // Convert to SystemTime
    SystemTime::UNIX_EPOCH.checked_add(Duration::new(unix_seconds, nanos))
}
