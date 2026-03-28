use std::{
    ffi::OsStr,
    fs,
    io::Read,
    os::windows::ffi::OsStrExt,
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

fn parse_recycle_bin_info(i_file_path: &std::path::Path) -> AppResult<(String, SystemTime, u64)> {
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
    let app_state = state.lock()?;

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

                // The $R counterpart holds the actual deleted content; use it to
                // determine whether the original item was a directory or a file.
                let r_file_name = file_name.replacen("$I", "$R", 1);
                let r_path = path.with_file_name(&r_file_name);

                if !r_path.exists() {
                    continue;
                }

                let is_dir = r_path.is_dir();

                let path_buf = PathBuf::from(&original_path);

                let name = path_buf
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let date_modified: DateTime<Local> = deletion_date.into();

                let file_type = if is_dir {
                    String::from("Folder")
                } else {
                    path_buf
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|s| s.to_uppercase())
                        .unwrap_or_else(|| String::from("File"))
                };

                let unique_path = path.to_string_lossy().to_string();

                all_entries.push(FileSystemEntry {
                    name,
                    is_dir,
                    is_file: !is_dir,
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

fn i_path_to_r_path(i_path: &std::path::Path) -> AppResult<PathBuf> {
    let file_name = i_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::InvalidInput("Invalid $I file path".into()))?;
    let r_file_name = file_name.replacen("$I", "$R", 1);
    Ok(i_path.with_file_name(r_file_name))
}

#[tauri::command]
pub fn restore_trash_entry(i_file_path: String) -> AppResult<()> {
    let i_path = PathBuf::from(&i_file_path);
    let (original_path, _, _) = parse_recycle_bin_info(&i_path)?;
    let r_path = i_path_to_r_path(&i_path)?;

    let dest_path = PathBuf::from(&original_path);

    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if dest_path.exists() {
        return Err(AppError::AlreadyExists(original_path));
    }

    fs::rename(&r_path, &dest_path)?;
    fs::remove_file(&i_path)?;

    Ok(())
}

#[tauri::command]
pub fn delete_trash_entry(i_file_path: String) -> AppResult<()> {
    let i_path = PathBuf::from(&i_file_path);
    let r_path = i_path_to_r_path(&i_path)?;

    if r_path.is_dir() {
        fs::remove_dir_all(&r_path)?;
    } else if r_path.exists() {
        fs::remove_file(&r_path)?;
    }

    fs::remove_file(&i_path)?;

    Ok(())
}

fn system_time_to_filetime(time: SystemTime) -> u64 {
    const UNIX_TO_FILETIME_SECONDS: u64 = 11_644_473_600;
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    (duration.as_secs() + UNIX_TO_FILETIME_SECONDS) * 10_000_000
        + duration.subsec_nanos() as u64 / 100
}

fn write_recycle_bin_info(i_path: &std::path::Path, original_path: &str, file_size: u64) -> AppResult<()> {
    let deletion_time = system_time_to_filetime(SystemTime::now());

    // UTF-16 LE path with null terminator
    let wide_path: Vec<u16> = OsStr::new(original_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let path_char_count = wide_path.len() as u32;

    let mut buffer = Vec::new();
    buffer.extend_from_slice(&2u64.to_le_bytes()); // version 2
    buffer.extend_from_slice(&file_size.to_le_bytes());
    buffer.extend_from_slice(&deletion_time.to_le_bytes());
    buffer.extend_from_slice(&path_char_count.to_le_bytes());
    for word in &wide_path {
        buffer.extend_from_slice(&word.to_le_bytes());
    }

    fs::write(i_path, &buffer)?;
    Ok(())
}

fn generate_unique_id(recycle_bin_path: &std::path::Path) -> AppResult<String> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let mut n = seed;
    for _ in 0..10_000 {
        let mut id = String::with_capacity(6);
        let mut v = n;
        for _ in 0..6 {
            id.push(CHARS[(v % CHARS.len() as u64) as usize] as char);
            v /= CHARS.len() as u64;
        }

        let i_prefix = format!("$I{}", id);
        let r_prefix = format!("$R{}", id);

        let taken = fs::read_dir(recycle_bin_path)
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    let name = e.file_name().to_string_lossy().to_uppercase();
                    name.starts_with(&i_prefix) || name.starts_with(&r_prefix)
                })
            })
            .unwrap_or(false);

        if !taken {
            return Ok(id);
        }
        n = n.wrapping_add(1);
    }

    Err(AppError::FileSystemError(
        "Could not generate unique recycle bin ID".into(),
    ))
}

#[tauri::command]
pub fn delete_permanently(paths: Vec<String>) -> AppResult<()> {
    for path_str in &paths {
        let path = PathBuf::from(path_str);
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn move_to_trash(paths: Vec<String>, state: tauri::State<Mutex<AppState>>) -> AppResult<()> {
    let user_sid = {
        let app_state = state.lock()?;
        app_state.current_user_id.clone()
    };

    for path_str in &paths {
        let src = PathBuf::from(path_str);

        // "C:" → "C:\$Recycle.Bin\{SID}"
        let drive = path_str
            .get(..2)
            .ok_or_else(|| AppError::InvalidInput("Path has no drive component".into()))?;
        let recycle_bin = PathBuf::from(format!("{}\\$Recycle.Bin\\{}", drive, user_sid));

        if !recycle_bin.exists() {
            fs::create_dir_all(&recycle_bin)?;
        }

        let unique_id = generate_unique_id(&recycle_bin)?;

        let extension = src.extension().and_then(|e| e.to_str()).unwrap_or("");
        let suffix = if extension.is_empty() {
            unique_id
        } else {
            format!("{}.{}", unique_id, extension)
        };

        let i_path = recycle_bin.join(format!("$I{}", suffix));
        let r_path = recycle_bin.join(format!("$R{}", suffix));

        let metadata = fs::metadata(&src)?;
        let file_size = if metadata.is_file() { metadata.len() } else { 0 };

        write_recycle_bin_info(&i_path, path_str, file_size)?;
        fs::rename(&src, &r_path)?;
    }

    Ok(())
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
