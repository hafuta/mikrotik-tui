//! Local filesystem helpers. Used only from workers.

use std::path::{Path, PathBuf};

use mtui_config::expand_user_path;
use mtui_ui::FilePickerEntry;

const MAX_DIR_ENTRIES: usize = 2000;

/// Contents upload is not supported on the classic API.
pub fn read_utf8_upload(_path: &Path) -> Result<String, String> {
    Err("Classic API cannot transfer file contents; use Fetch URL".into())
}

/// Starting folder for the CA file browser.
#[must_use]
pub fn default_browse_dir(current_ca: &str) -> String {
    let trimmed = current_ca.trim();
    if !trimmed.is_empty() {
        let expanded = expand_user_path(trimmed);
        if expanded.is_file()
            && let Some(parent) = expanded.parent()
            && !parent.as_os_str().is_empty()
        {
            return parent.to_string_lossy().into_owned();
        }
        if expanded.is_dir() {
            return expanded.to_string_lossy().into_owned();
        }
        if let Some(parent) = expanded.parent()
            && parent.exists()
            && !parent.as_os_str().is_empty()
        {
            return parent.to_string_lossy().into_owned();
        }
    }
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Parent folder, or an empty path on Windows so the picker can list drives.
#[must_use]
pub fn parent_browse_dir(dir: &str) -> Option<String> {
    if dir.is_empty() {
        return None;
    }
    #[cfg(windows)]
    {
        if is_windows_drive_root(dir) {
            return Some(String::new());
        }
    }
    let parent = Path::new(dir).parent()?;
    if parent.as_os_str().is_empty() {
        #[cfg(windows)]
        {
            return Some(String::new());
        }
        #[cfg(not(windows))]
        {
            return None;
        }
    }
    let shown = parent.to_string_lossy().into_owned();
    if shown == dir { None } else { Some(shown) }
}

/// Lists `path`. An empty path lists Windows drive letters, or `/` on Unix.
pub fn list_local_dir(path: &str) -> Result<(String, Vec<FilePickerEntry>), String> {
    if path.is_empty() {
        #[cfg(windows)]
        {
            return Ok((String::new(), list_windows_drives()));
        }
        #[cfg(not(windows))]
        {
            return read_dir_entries(Path::new("/"), "/");
        }
    }
    let expanded = expand_user_path(path);
    if expanded.is_file() {
        let parent = expanded
            .parent()
            .ok_or_else(|| "no parent directory".to_string())?;
        let shown = parent.to_string_lossy().into_owned();
        return read_dir_entries(parent, &shown);
    }
    let shown = expanded.to_string_lossy().into_owned();
    read_dir_entries(&expanded, &shown)
}

fn read_dir_entries(path: &Path, shown: &str) -> Result<(String, Vec<FilePickerEntry>), String> {
    let mut entries = Vec::new();
    let iter = std::fs::read_dir(path).map_err(|err| format!("{}: {err}", path.display()))?;
    for item in iter {
        if entries.len() >= MAX_DIR_ENTRIES {
            break;
        }
        let Ok(item) = item else {
            continue;
        };
        let file_type = item.file_type().ok();
        let is_dir = file_type.is_some_and(|kind| kind.is_dir());
        let name = item.file_name().to_string_lossy().into_owned();
        if name.is_empty() {
            continue;
        }
        entries.push(FilePickerEntry {
            name,
            path: item.path().to_string_lossy().into_owned(),
            is_dir,
        });
    }
    entries.sort_by(|left, right| {
        right.is_dir.cmp(&left.is_dir).then_with(|| {
            left.name
                .to_ascii_lowercase()
                .cmp(&right.name.to_ascii_lowercase())
        })
    });
    Ok((shown.to_string(), entries))
}

#[cfg(windows)]
fn is_windows_drive_root(path: &str) -> bool {
    let comps: Vec<_> = Path::new(path).components().collect();
    matches!(
        comps.as_slice(),
        [
            std::path::Component::Prefix(_),
            std::path::Component::RootDir
        ] | [std::path::Component::Prefix(_)]
    )
}

#[cfg(windows)]
fn list_windows_drives() -> Vec<FilePickerEntry> {
    (b'A'..=b'Z')
        .filter_map(|letter| {
            let path = format!("{}:\\", char::from(letter));
            if Path::new(&path).exists() {
                Some(FilePickerEntry {
                    name: path.clone(),
                    path,
                    is_dir: true,
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn write_download(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        return Err(format!("directory does not exist: {}", parent.display()));
    }
    std::fs::write(path, contents).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("mtui-files-{stamp}-{name}"))
    }

    #[test]
    fn contents_upload_is_rejected() {
        let err = read_utf8_upload(Path::new("any.txt")).expect_err("unsupported");
        assert!(err.contains("Fetch URL"));
        assert!(!err.to_ascii_lowercase().contains("rest"));
    }

    #[test]
    fn missing_parent_dir_is_rejected() {
        let path = temp_path("missing-dir").join("out.txt");
        let err = write_download(&path, "hi").expect_err("missing dir");
        assert!(err.contains("directory does not exist"));
    }

    #[test]
    fn lists_a_temp_dir_with_file_and_subdir() {
        let dir = temp_path("browse");
        std::fs::create_dir_all(dir.join("nested")).expect("dir");
        std::fs::write(dir.join("ca.pem"), "x").expect("file");
        let (shown, entries) = list_local_dir(&dir.to_string_lossy()).expect("list");
        assert!(shown.contains("browse"), "{shown}");
        assert!(
            entries
                .iter()
                .any(|entry| entry.is_dir && entry.name == "nested")
        );
        assert!(
            entries
                .iter()
                .any(|entry| !entry.is_dir && entry.name == "ca.pem")
        );
        let dirs_first = entries.iter().position(|entry| entry.name == "nested")
            < entries.iter().position(|entry| entry.name == "ca.pem");
        assert!(dirs_first);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn default_browse_dir_uses_parent_of_a_file() {
        let dir = temp_path("browse-parent");
        std::fs::create_dir_all(&dir).expect("dir");
        let file = dir.join("leaf.pem");
        std::fs::write(&file, "x").expect("file");
        let start = default_browse_dir(&file.to_string_lossy());
        assert_eq!(
            Path::new(&start).canonicalize().ok(),
            dir.canonicalize().ok()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(all(test, unix))]
mod unix_tests {
    use super::*;

    #[test]
    fn unix_root_has_no_parent() {
        assert_eq!(parent_browse_dir("/"), None);
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;

    #[test]
    fn windows_drive_root_opens_drive_list() {
        assert_eq!(parent_browse_dir(r"C:\"), Some(String::new()));
        let (shown, entries) = list_local_dir("").expect("drives");
        assert!(shown.is_empty());
        assert!(
            entries
                .iter()
                .any(|entry| entry.is_dir && entry.path.ends_with(":\\")),
            "{entries:?}"
        );
    }
}
