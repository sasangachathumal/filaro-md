use std::fmt;

/// Errors surfaced from the file-access layer to the frontend.
///
/// Implements `Serialize` (as its `Display` string) so `Result<T, FileAccessError>`
/// can be returned directly from a `#[tauri::command]`.
#[derive(Debug)]
pub enum FileAccessError {
    Io { path: String, source: std::io::Error },
    Dialog(String),
    NotMarkdown { path: String },
    NotADirectory { path: String },
}

impl fmt::Display for FileAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileAccessError::Io { path, source } => {
                write!(f, "couldn't access \"{path}\": {source}")
            }
            FileAccessError::Dialog(message) => write!(f, "dialog error: {message}"),
            FileAccessError::NotMarkdown { path } => {
                write!(f, "\"{path}\" is not a markdown file")
            }
            FileAccessError::NotADirectory { path } => {
                write!(f, "\"{path}\" is not a directory")
            }
        }
    }
}

impl std::error::Error for FileAccessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FileAccessError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl serde::Serialize for FileAccessError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
