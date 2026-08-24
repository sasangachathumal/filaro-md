use std::fs;
use std::path::Path;

use tauri_plugin_dialog::DialogExt;

use super::error::FileAccessError;
use super::model::MarkdownEntry;
use super::walk::walk_markdown_folder;

const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown"];

fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| MARKDOWN_EXTENSIONS.iter().any(|allowed| ext.eq_ignore_ascii_case(allowed)))
        .unwrap_or(false)
}

/// Opens the native "open file" dialog filtered to markdown files.
/// Returns `None` if the user cancels the dialog.
#[tauri::command]
pub fn open_markdown_file(app: tauri::AppHandle) -> Result<Option<String>, FileAccessError> {
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
#[tauri::command]
pub fn open_markdown_folder(app: tauri::AppHandle) -> Result<Option<MarkdownEntry>, FileAccessError> {
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
