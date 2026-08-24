use std::path::Path;

use walkdir::WalkDir;

use super::error::FileAccessError;
use super::model::MarkdownEntry;

fn has_markdown_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
        .unwrap_or(false)
}

/// Walks `root` with `walkdir`, keeping only `.md` / `.markdown` files, and
/// returns the result as a tree rooted at `root`. Directories that contain
/// no markdown file (directly or nested) are omitted.
pub fn walk_markdown_folder(root: &Path) -> Result<MarkdownEntry, FileAccessError> {
    if !root.is_dir() {
        return Err(FileAccessError::NotADirectory {
            path: root.display().to_string(),
        });
    }

    let mut root_node = MarkdownEntry {
        name: root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.display().to_string()),
        path: root.display().to_string(),
        is_dir: true,
        children: Some(Vec::new()),
    };

    let entries = WalkDir::new(root).min_depth(1).into_iter();

    for entry in entries {
        let entry = entry.map_err(|source| FileAccessError::Io {
            path: source
                .path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| root.display().to_string()),
            source: source.into_io_error().unwrap_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "failed to walk directory")
            }),
        })?;

        if !entry.file_type().is_file() || !has_markdown_extension(entry.path()) {
            continue;
        }

        let relative = entry.path().strip_prefix(root).map_err(|_| FileAccessError::Io {
            path: entry.path().display().to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::Other,
                "entry path is not inside the walked root",
            ),
        })?;

        insert_file(&mut root_node, root, relative);
    }

    sort_tree(&mut root_node);

    Ok(root_node)
}

/// Inserts a markdown file at `relative` (relative to `root`) into the tree,
/// creating any intermediate directory nodes along the way.
fn insert_file(root_node: &mut MarkdownEntry, root: &Path, relative: &Path) {
    let mut current = root_node;
    let mut accumulated = root.to_path_buf();
    let components: Vec<_> = relative.components().collect();

    for (i, component) in components.iter().enumerate() {
        accumulated.push(component.as_os_str());
        let is_last = i == components.len() - 1;
        let name = component.as_os_str().to_string_lossy().to_string();

        let children = current.children.get_or_insert_with(Vec::new);
        let existing = children.iter().position(|child| child.name == name);

        let index = existing.unwrap_or_else(|| {
            children.push(MarkdownEntry {
                name: name.clone(),
                path: accumulated.display().to_string(),
                is_dir: !is_last,
                children: if is_last { None } else { Some(Vec::new()) },
            });
            children.len() - 1
        });

        current = &mut children[index];
    }
}

/// Sorts each directory's children (directories first, then alphabetically
/// by name) so the tree renders in a stable order.
fn sort_tree(node: &mut MarkdownEntry) {
    if let Some(children) = &mut node.children {
        children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        for child in children {
            sort_tree(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn filters_to_markdown_only_and_prunes_empty_dirs() {
        let dir = tempdir();

        fs::write(dir.join("notes.md"), "# hi").unwrap();
        fs::write(dir.join("skip.txt"), "nope").unwrap();
        fs::create_dir(dir.join("docs")).unwrap();
        fs::write(dir.join("docs/guide.markdown"), "# guide").unwrap();
        fs::create_dir(dir.join("empty")).unwrap();

        let tree = walk_markdown_folder(&dir).unwrap();
        let children = tree.children.unwrap();

        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "docs");
        assert!(children[0].is_dir);
        assert_eq!(children[0].children.as_ref().unwrap()[0].name, "guide.markdown");
        assert_eq!(children[1].name, "notes.md");
        assert!(!children[1].is_dir);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn errors_when_root_is_not_a_directory() {
        let dir = tempdir();
        let file = dir.join("file.md");
        fs::write(&file, "# hi").unwrap();

        let result = walk_markdown_folder(&file);

        assert!(matches!(result, Err(FileAccessError::NotADirectory { .. })));
        fs::remove_dir_all(&dir).ok();
    }

    fn tempdir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("filaro-walk-test-{}", uuid_like()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn uuid_like() -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    }
}
