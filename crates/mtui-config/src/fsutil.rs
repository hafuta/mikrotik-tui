//! Atomic, permission-hardened file writes shared by the profile and
//! credential stores.

use std::fs::{self, File};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{ConfigError, Result};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Writes `data` to `path` by creating a sibling temp file, syncing it, and
/// renaming it over the destination. The rename is atomic on both POSIX
/// (`rename(2)`) and Windows (`MoveFileExW` with replace semantics), so
/// readers never observe a partially written file.
///
/// `mode` is applied to the temp file (and therefore the final file) on Unix
/// only; other platforms ignore it, matching the requirement that
/// permissions are hardened "on Unix when possible".
pub fn atomic_write_file(path: &Path, data: &[u8], mode: u32) -> Result<()> {
    let dir = path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    fs::create_dir_all(&dir).map_err(|source| ConfigError::Write {
        path: dir.clone(),
        source,
    })?;
    harden_dir_permissions(&dir)?;

    let temp_path = unique_temp_path(&dir, path);
    if let Err(err) = write_temp_file(&temp_path, data, mode) {
        let _ = fs::remove_file(&temp_path);
        return Err(err);
    }

    if let Err(source) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(ConfigError::Write {
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

fn write_temp_file(temp_path: &Path, data: &[u8], mode: u32) -> Result<()> {
    let mut file = create_with_mode(temp_path, mode).map_err(|source| ConfigError::Write {
        path: temp_path.to_path_buf(),
        source,
    })?;
    file.write_all(data).map_err(|source| ConfigError::Write {
        path: temp_path.to_path_buf(),
        source,
    })?;
    file.sync_all().map_err(|source| ConfigError::Write {
        path: temp_path.to_path_buf(),
        source,
    })
}

fn unique_temp_path(dir: &Path, target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("mtui-config");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    dir.join(format!(
        ".{file_name}.tmp-{}-{nanos}-{counter}",
        std::process::id()
    ))
}

#[cfg(unix)]
fn create_with_mode(path: &Path, mode: u32) -> std::io::Result<File> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(path)
}

#[cfg(not(unix))]
fn create_with_mode(path: &Path, _mode: u32) -> std::io::Result<File> {
    File::create(path)
}

#[cfg(unix)]
fn harden_dir_permissions(dir: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).map_err(|source| {
        ConfigError::Write {
            path: dir.to_path_buf(),
            source,
        }
    })
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn harden_dir_permissions(_dir: &Path) -> Result<()> {
    Ok(())
}
