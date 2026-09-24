use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::error::FileAccessError;
use super::model::RecentEntry;

/// Oldest entries beyond this are dropped on write, so the list can't grow
/// without bound and the existence check on read stays cheap.
const MAX_RECENT_FILES: usize = 20;

/// Reads the recent-files list from `config_path`. A missing file means
/// "no recents yet"; a present-but-unparseable file is treated the same
/// way rather than surfacing an error, since it just means starting over
/// with an empty list is safe.
pub fn read_recent_files(config_path: &Path) -> Result<Vec<RecentEntry>, FileAccessError> {
    let contents = match fs::read_to_string(config_path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(FileAccessError::Io {
                path: config_path.display().to_string(),
                source,
            })
        }
    };

    let paths: Vec<String> = serde_json::from_str(&contents).unwrap_or_default();

    Ok(paths.into_iter().map(to_entry).collect())
}

fn to_entry(path_str: String) -> RecentEntry {
    let path = PathBuf::from(&path_str);
    let exists = path.is_file();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.clone());

    RecentEntry {
        name,
        path: path_str,
        exists,
    }
}

/// Overwrites the recent-files list at `config_path` with `paths`,
/// creating the parent config directory if it doesn't exist yet.
///
/// `paths` must be in most-recent-first order (e.g. the just-opened file
/// prepended ahead of the previous list) — this function doesn't reorder,
/// it only enforces the list's invariants: entries that currently resolve
/// to a directory are dropped (recents track individual files only — a
/// folder that was opened to populate the sidebar tree never itself
/// becomes a recent entry), duplicates collapse to their first (frontmost)
/// occurrence, and the result is capped to [`MAX_RECENT_FILES`], dropping
/// whatever falls off the end.
pub fn write_recent_files(config_path: &Path, paths: &[String]) -> Result<(), FileAccessError> {
    let paths = sanitize(paths);

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|source| FileAccessError::Io {
            path: parent.display().to_string(),
            source,
        })?;
    }

    let json = serde_json::to_string_pretty(&paths)
        .map_err(|source| FileAccessError::Config(source.to_string()))?;

    fs::write(config_path, json).map_err(|source| FileAccessError::Io {
        path: config_path.display().to_string(),
        source,
    })
}

fn sanitize(paths: &[String]) -> Vec<String> {
    let mut seen = HashSet::with_capacity(paths.len().min(MAX_RECENT_FILES));
    let mut result = Vec::with_capacity(paths.len().min(MAX_RECENT_FILES));

    for path in paths {
        if result.len() == MAX_RECENT_FILES {
            break;
        }
        // A path that no longer exists is kept (it's a legitimately missing
        // recent entry); one that currently exists as a directory is not.
        if Path::new(path).is_dir() {
            continue;
        }
        if seen.insert(path.clone()) {
            result.push(path.clone());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_config_file_returns_empty_list() {
        let dir = tempdir();
        let config_path = dir.join("does-not-exist.json");

        let entries = read_recent_files(&config_path).unwrap();

        assert!(entries.is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn corrupt_config_file_returns_empty_list() {
        let dir = tempdir();
        let config_path = dir.join("recent-files.json");
        fs::write(&config_path, "{ not valid json").unwrap();

        let entries = read_recent_files(&config_path).unwrap();

        assert!(entries.is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn round_trips_and_flags_missing_files() {
        let dir = tempdir();
        let config_path = dir.join("nested").join("recent-files.json");

        let present = dir.join("present.md");
        fs::write(&present, "# hi").unwrap();
        let missing = dir.join("missing.md");

        let paths = vec![
            present.to_string_lossy().to_string(),
            missing.to_string_lossy().to_string(),
        ];

        write_recent_files(&config_path, &paths).unwrap();
        let entries = read_recent_files(&config_path).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "present.md");
        assert!(entries[0].exists);
        assert_eq!(entries[1].name, "missing.md");
        assert!(!entries[1].exists);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reopening_a_file_moves_it_to_the_top_without_duplicating() {
        let dir = tempdir();
        let config_path = dir.join("recent-files.json");

        let a = dir.join("a.md").to_string_lossy().to_string();
        let b = dir.join("b.md").to_string_lossy().to_string();
        let c = dir.join("c.md").to_string_lossy().to_string();

        write_recent_files(&config_path, &[a.clone(), b.clone(), c.clone()]).unwrap();
        // Simulate re-opening `b`: caller prepends it ahead of the previous list.
        write_recent_files(&config_path, &[b.clone(), a.clone(), b.clone(), c.clone()]).unwrap();

        let entries = read_recent_files(&config_path).unwrap();
        let paths: Vec<_> = entries.iter().map(|e| e.path.clone()).collect();

        assert_eq!(paths, vec![b, a, c]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn caps_at_max_recent_files_dropping_the_oldest() {
        let dir = tempdir();
        let config_path = dir.join("recent-files.json");

        let paths: Vec<String> = (0..25)
            .map(|i| dir.join(format!("{i}.md")).to_string_lossy().to_string())
            .collect();

        write_recent_files(&config_path, &paths).unwrap();
        let entries = read_recent_files(&config_path).unwrap();

        assert_eq!(entries.len(), MAX_RECENT_FILES);
        assert_eq!(entries.first().unwrap().path, paths[0]);
        assert_eq!(entries.last().unwrap().path, paths[MAX_RECENT_FILES - 1]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn drops_paths_that_are_currently_directories() {
        let dir = tempdir();
        let config_path = dir.join("recent-files.json");

        let file = dir.join("notes.md");
        fs::write(&file, "# hi").unwrap();
        let folder = dir.join("some-folder");
        fs::create_dir(&folder).unwrap();

        let paths = vec![
            folder.to_string_lossy().to_string(),
            file.to_string_lossy().to_string(),
        ];

        write_recent_files(&config_path, &paths).unwrap();
        let entries = read_recent_files(&config_path).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "notes.md");
        fs::remove_dir_all(&dir).ok();
    }

    fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("filaro-recent-test-{}", uuid_like()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn uuid_like() -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    }
}
