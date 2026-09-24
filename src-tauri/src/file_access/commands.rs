use std::fs;
use std::path::{Path, PathBuf};

use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

use super::error::FileAccessError;
use super::model::{MarkdownEntry, RecentEntry};
use super::recent;
use super::walk::walk_markdown_folder;

const RECENT_FILES_CONFIG_FILE: &str = "recent-files.json";

const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown"];

fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| MARKDOWN_EXTENSIONS.iter().any(|allowed| ext.eq_ignore_ascii_case(allowed)))
        .unwrap_or(false)
}

/// Opens the native "open file" dialog filtered to markdown files.
/// Returns `None` if the user cancels the dialog.
///
/// Must stay `async`: the blocking dialog call below hands the actual
/// panel off to the main thread and waits on it, so this command has to
/// run off the main thread itself or the two deadlock each other.
#[tauri::command]
pub async fn open_markdown_file(app: tauri::AppHandle) -> Result<Option<String>, FileAccessError> {
    let Some(picked) = app
        .dialog()
        .file()
        .add_filter("Markdown", MARKDOWN_EXTENSIONS)
        .blocking_pick_file()
    else {
        return Ok(None);
    };

    let path = picked
        .into_path()
        .map_err(|source| FileAccessError::Dialog(source.to_string()))?;

    Ok(Some(path.display().to_string()))
}

/// Opens the native "open folder" dialog, then walks the chosen folder for
/// markdown files. Returns `None` if the user cancels the dialog.
///
/// Must stay `async` for the same reason as [`open_markdown_file`].
#[tauri::command]
pub async fn open_markdown_folder(app: tauri::AppHandle) -> Result<Option<MarkdownEntry>, FileAccessError> {
    let Some(picked) = app.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };

    let path = picked
        .into_path()
        .map_err(|source| FileAccessError::Dialog(source.to_string()))?;

    Ok(Some(walk_markdown_folder(&path)?))
}

/// Reads a markdown file's contents as UTF-8 text.
#[tauri::command]
pub fn read_markdown_file(path: String) -> Result<String, FileAccessError> {
    let path = Path::new(&path);

    if !is_markdown_path(path) {
        return Err(FileAccessError::NotMarkdown {
            path: path.display().to_string(),
        });
    }

    fs::read_to_string(path).map_err(|source| FileAccessError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn recent_config_path(app: &tauri::AppHandle) -> Result<PathBuf, FileAccessError> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(RECENT_FILES_CONFIG_FILE))
        .map_err(|source| FileAccessError::Config(source.to_string()))
}

/// Reads the persisted recent-files list. A missing or corrupt config
/// file yields an empty list rather than an error.
#[tauri::command]
pub fn read_recent_files(app: tauri::AppHandle) -> Result<Vec<RecentEntry>, FileAccessError> {
    recent::read_recent_files(&recent_config_path(&app)?)
}

/// Overwrites the persisted recent-files list with `paths` (most-recent-first).
/// Directories are dropped, duplicates collapse to their frontmost
/// occurrence, and the list is capped, dropping the oldest entries — see
/// [`recent::write_recent_files`].
#[tauri::command]
pub fn write_recent_files(app: tauri::AppHandle, paths: Vec<String>) -> Result<(), FileAccessError> {
    recent::write_recent_files(&recent_config_path(&app)?, &paths)
}
