//! Local filesystem helpers for Files upload/download. Used only from workers.

use std::path::Path;

pub const MAX_REST_UPLOAD_BYTES: u64 = 1024 * 1024;

pub fn read_utf8_upload(path: &Path) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|err| err.to_string())?;
    if meta.len() > MAX_REST_UPLOAD_BYTES {
        return Err("file too large for REST contents upload — use Fetch URL".into());
    }
    let bytes = std::fs::read(path).map_err(|err| err.to_string())?;
    String::from_utf8(bytes).map_err(|_| "binary upload not supported; use /tool/fetch".into())
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
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("mtui-files-{stamp}-{name}"))
    }

    #[test]
    fn oversized_upload_is_rejected() {
        let path = temp_path("big.txt");
        let len = usize::try_from(MAX_REST_UPLOAD_BYTES.saturating_add(1)).expect("size");
        let data = vec![b'a'; len];
        fs::write(&path, data).expect("write");
        let err = read_utf8_upload(&path).expect_err("too large");
        let _ = fs::remove_file(&path);
        assert!(err.contains("file too large"));
        assert!(err.contains("Fetch URL"));
    }

    #[test]
    fn binary_upload_is_rejected() {
        let path = temp_path("bin.dat");
        fs::write(&path, [0xff, 0xfe, 0x00]).expect("write");
        let err = read_utf8_upload(&path).expect_err("binary");
        let _ = fs::remove_file(&path);
        assert!(err.contains("binary upload not supported"));
    }

    #[test]
    fn missing_parent_dir_is_rejected() {
        let path = temp_path("missing-dir").join("out.txt");
        let err = write_download(&path, "hi").expect_err("missing dir");
        assert!(err.contains("directory does not exist"));
    }
}
