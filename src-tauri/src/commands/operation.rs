use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::{
    commands::file::traverse_directory,
    error::AppResult,
    model::{
        AppState, ConflictResolution, ConflictingEntry, FileOperationEntry, OperationType,
        SourceEntry,
    },
};
use tauri::Manager;

#[tauri::command]
pub async fn prepare_operation(
    app: tauri::AppHandle,
    src_entries: Vec<SourceEntry>,
    dest: String,
    operation_type: OperationType,
) -> AppResult<Vec<ConflictingEntry>> {
    let mut operations: Vec<(PathBuf, PathBuf)> = Vec::new();
    let dest_path = PathBuf::from(dest.clone());

    for src in src_entries.into_iter() {
        if src.is_dir {
            let source_root = PathBuf::from(&src.path);
            let folder_name = source_root.file_name().unwrap();

            let _ = traverse_directory(source_root.clone(), |entry, metadata| {
                if !metadata.is_file() {
                    return Ok(());
                }

                let file_path = entry.path();

                let relative = file_path.strip_prefix(&source_root).unwrap();

                let destination = dest_path.join(folder_name).join(relative);

                operations.push((file_path, destination));

                Ok(())
            })?;
        } else {
            let src_file = PathBuf::from(&src.path);
            let destination = dest_path.join(src_file.file_name().unwrap());
            operations.push((src_file, destination));
        }
    }

    let mut conflicting_entries: Vec<ConflictingEntry> = Vec::new();
    let state = app.state::<Mutex<AppState>>();
    let mut app_state = state.lock().unwrap();
    app_state.operation_type = Some(operation_type);

    for (src_file, dest_file) in operations.iter() {
        if dest_file.try_exists()? && dest_file.is_file() {
            let file_name = dest_file
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();

            conflicting_entries.push(ConflictingEntry {
                name: file_name,
                src: src_file.clone(),
                dest: dest_file.clone(),
            });
        }

        app_state.file_op_entries.push(FileOperationEntry {
            src: src_file.clone(),
            dest: dest_file.clone(),
        });
    }

    Ok(conflicting_entries)
}

#[tauri::command]
pub async fn execute_operation(
    app: tauri::AppHandle,
    conflict_resolutions: HashMap<String, ConflictResolution>,
) -> AppResult<()> {
    let state = app.state::<Mutex<AppState>>();
    let mut app_state = state.lock().unwrap();
    let operations = std::mem::take(&mut app_state.file_op_entries);
    let op_type = app_state
        .operation_type
        .take()
        .unwrap_or(OperationType::Move);
    drop(app_state);

    let mut unique_dirs: HashSet<PathBuf> = HashSet::new();

    for entry in operations.iter() {
        if let Some(parent_dir) = entry.dest.parent() {
            unique_dirs.insert(parent_dir.to_path_buf());
        };
    }

    for dir in unique_dirs.iter() {
        fs::create_dir_all(dir)?;
    }

    for file_entry in operations.iter() {
        let src_string = file_entry.src.to_string_lossy().to_string();
        if let Some(resolution) = conflict_resolutions.get(&src_string) {
            match resolution {
                ConflictResolution::Skip => {
                    continue;
                }
                ConflictResolution::Replace => {
                    if file_entry.dest.exists() {
                        fs::remove_file(&file_entry.dest)?;
                    }
                    perform_operation(&file_entry.src, &file_entry.dest, op_type)?;
                }
                ConflictResolution::Keep => {
                    let new_dest = generate_unique_name(&file_entry.dest)?;
                    perform_operation(&file_entry.src, &new_dest, op_type)?;
                }
            }
        } else {
            perform_operation(&file_entry.src, &file_entry.dest, op_type)?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn cancel_operation(app: tauri::AppHandle) -> AppResult<()> {
    let state = app.state::<Mutex<AppState>>();
    let mut app_state = state.lock().unwrap();
    app_state.file_op_entries.clear();
    Ok(())
}

fn perform_operation(src: &Path, dest: &Path, op_type: OperationType) -> AppResult<()> {
    match op_type {
        OperationType::Copy => {
            fs::copy(src, dest)?;
            Ok(())
        }
        OperationType::Move => match fs::rename(src, dest) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
                fs::copy(src, dest)?;
                fs::remove_file(src)?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        },
    }
}

fn generate_unique_name(dest: &Path) -> AppResult<PathBuf> {
    if !dest.exists() {
        return Ok(dest.to_path_buf());
    }

    let parent = dest.parent().unwrap();
    let file_stem = dest.file_stem().unwrap();
    let extension = dest.extension();

    let mut counter = 1;

    loop {
        let new_name = if let Some(ext) = extension {
            format!(
                "{} ({}).{}",
                file_stem.to_string_lossy(),
                counter,
                ext.to_string_lossy()
            )
        } else {
            format!("{} ({})", file_stem.to_string_lossy(), counter)
        };

        let new_path = parent.join(new_name);

        if !new_path.exists() {
            return Ok(new_path);
        }

        counter += 1;
    }
}
