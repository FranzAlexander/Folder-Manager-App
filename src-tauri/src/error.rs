use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("File system error: {0}")]
    FileSystemError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("{0} already exists")]
    AlreadyExists(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        #[cfg(debug_assertions)]
        {
            eprintln!("🔴 SQL Error: {:?}", err);
            eprintln!("📍 Backtrace:\n{}", std::backtrace::Backtrace::capture());
        }
        match err {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound("Resource not found".to_string())
            }
            rusqlite::Error::SqliteFailure(_, Some(msg)) if msg.contains("UNIQUE") => {
                AppError::AlreadyExists("Resource".to_string())
            }
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind;

        match err.kind() {
            ErrorKind::NotFound => AppError::NotFound("File or directory not found".to_string()),
            ErrorKind::PermissionDenied => AppError::PermissionDenied(err.to_string()),
            ErrorKind::AlreadyExists => AppError::AlreadyExists("File or directory".to_string()),
            _ => {
                // Check for Windows error code 740 (requires elevation)
                if let Some(740) = err.raw_os_error() {
                    AppError::PermissionDenied(
                        "This operation requires administrator privileges".to_string(),
                    )
                } else {
                    AppError::FileSystemError(err.to_string())
                }
            }
        }
    }
}

impl From<tauri_plugin_shell::Error> for AppError {
    fn from(err: tauri_plugin_shell::Error) -> Self {
        match err {
            tauri_plugin_shell::Error::Io(io_err) => {
                // Delegate to the std::io::Error conversion
                AppError::from(io_err)
            }
            tauri_plugin_shell::Error::ProgramNotAllowed(path) => {
                AppError::PermissionDenied(format!(
                    "Program not allowed: {}. Check your tauri.conf.json shell scope configuration.",
                    path.display()
                ))
            }
            tauri_plugin_shell::Error::SidecarNotAllowed(path) => {
                AppError::ConfigError(format!(
                    "Sidecar not configured: {}. Add it to tauri.conf.json > bundle > externalBin",
                    path.display()
                ))
            }
            tauri_plugin_shell::Error::Scope(scope_err) => {
                AppError::PermissionDenied(format!("Scope error: {}", scope_err))
            }
            tauri_plugin_shell::Error::UnknownProgramName(name) => {
                AppError::NotFound(format!("Unknown program: {}", name))
            }
            _ => AppError::FileSystemError(err.to_string()),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn not_found(resource: &str) -> Self {
        AppError::NotFound(format!("{} not found", resource))
    }

    pub fn already_exists(resource: &str) -> Self {
        AppError::AlreadyExists(resource.to_string())
    }

    pub fn permission_denied(message: &str) -> Self {
        AppError::PermissionDenied(message.to_string())
    }
}
