//! Local filesystem helpers. Used only from workers.

use std::path::Path;

/// Contents upload is not supported on the classic API.
pub fn read_utf8_upload(_path: &Path) -> Result<String, String> {
    Err("Classic API cannot transfer file contents; use Fetch URL".into())
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
}
