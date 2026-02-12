use std::path::PathBuf;

#[cfg(windows)]
use crate::platform::windows::get_available_drives;

pub fn build_trash_paths(user_id: &str) -> Vec<PathBuf> {
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
