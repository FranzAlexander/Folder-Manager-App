use std::{path::Path, sync::Mutex};

use crate::model::AppState;
use notify::{EventKind, RecursiveMode, Watcher};

use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn watch_directory(
    path: String,
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;

    app_state.watcher = None;

    let app_handle = app.clone();
    let watched_path = path.clone();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let relevant = matches!(
                event.kind,
                EventKind::Create(_)
                    | EventKind::Remove(_)
                    | EventKind::Modify(notify::event::ModifyKind::Name(_))
            );
            if relevant {
                app_handle.emit("dir-changed", watched_path.clone()).ok();
            }
        }
    })
    .map_err(|e| e.to_string())?;

    watcher
        .watch(Path::new(&path), RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    app_state.watcher = Some(watcher);

    Ok(())
}

#[tauri::command]
pub fn unwatch_directory(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.watcher = None;
    Ok(())
}
