//! Replaceable secret storage.
//!
//! Mirrors `internal/credentials/credentials.go`: a small [`CredentialStore`]
//! trait so a platform keychain can later replace the JSON backend without
//! changing callers, plus a permission-hardened [`FileCredentialStore`].

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, Result};
use crate::fsutil::atomic_write_file;
use crate::paths;

/// File name for the credential document, relative to the config directory.
pub const CREDENTIALS_FILE_NAME: &str = "credentials.json";
const CREDENTIALS_FILE_VERSION: u32 = 1;

/// Secrets used to authenticate a router. Never logged; see
/// [`crate::redact`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    pub password: String,
}

/// Abstracts credential persistence by profile name so a platform keychain
/// can replace the JSON backend without changing callers.
pub trait CredentialStore: Send + Sync {
    /// Retrieves credentials for `profile`. Returns
    /// [`ConfigError::CredentialsNotFound`] when absent.
    fn get(&self, profile: &str) -> Result<Credential>;
    /// Creates or replaces credentials for `profile`.
    fn put(&self, profile: &str, credential: Credential) -> Result<()>;
    /// Removes credentials for `profile`. Deleting a missing profile
    /// succeeds.
    fn delete(&self, profile: &str) -> Result<()>;
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CredentialDocument {
    version: u32,
    #[serde(default)]
    credentials: BTreeMap<String, Credential>,
}

/// Permission-hardened JSON credential backend. Refuses to read a store with
/// group/world-readable permissions on Unix.
pub struct FileCredentialStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl FileCredentialStore {
    /// Creates a credential store rooted at `dir`.
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            path: dir.as_ref().join(CREDENTIALS_FILE_NAME),
            lock: Mutex::new(()),
        }
    }

    /// Resolves the platform config directory and returns a store rooted
    /// there. Honors `XDG_CONFIG_HOME` when set (see [`paths::config_dir`]).
    pub fn discover() -> Result<Self> {
        Ok(Self::new(paths::config_dir()?))
    }

    /// The JSON file backing this store.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn load(&self) -> Result<CredentialDocument> {
        let metadata = match fs::metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                return Ok(CredentialDocument {
                    version: CREDENTIALS_FILE_VERSION,
                    credentials: BTreeMap::new(),
                });
            }
            Err(source) => {
                return Err(ConfigError::Read {
                    path: self.path.clone(),
                    source,
                });
            }
        };
        check_permissions(&self.path, &metadata)?;

        let data = fs::read(&self.path).map_err(|source| ConfigError::Read {
            path: self.path.clone(),
            source,
        })?;
        let doc: CredentialDocument =
            serde_json::from_slice(&data).map_err(|source| ConfigError::Decode {
                path: self.path.clone(),
                source,
            })?;
        if doc.version != CREDENTIALS_FILE_VERSION {
            return Err(ConfigError::UnsupportedVersion {
                what: "credential",
                version: doc.version,
            });
        }
        Ok(doc)
    }

    fn save(&self, doc: &CredentialDocument) -> Result<()> {
        let mut data = serde_json::to_vec_pretty(doc).map_err(|source| ConfigError::Encode {
            what: "credentials",
            source,
        })?;
        data.push(b'\n');
        atomic_write_file(&self.path, &data, 0o600)
    }
}

#[cfg(unix)]
fn check_permissions(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(ConfigError::InsecurePermissions {
            path: path.to_path_buf(),
            mode,
        });
    }
    Ok(())
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn check_permissions(_path: &Path, _metadata: &fs::Metadata) -> Result<()> {
    Ok(())
}

fn require_profile_name(profile: &str) -> Result<()> {
    if profile.trim().is_empty() {
        return Err(ConfigError::CredentialProfileNameRequired);
    }
    Ok(())
}

impl CredentialStore for FileCredentialStore {
    fn get(&self, profile: &str) -> Result<Credential> {
        require_profile_name(profile)?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let doc = self.load()?;
        doc.credentials
            .get(profile)
            .cloned()
            .ok_or_else(|| ConfigError::CredentialsNotFound(profile.to_string()))
    }

    fn put(&self, profile: &str, credential: Credential) -> Result<()> {
        require_profile_name(profile)?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut doc = self.load()?;
        doc.credentials.insert(profile.to_string(), credential);
        self.save(&doc)
    }

    fn delete(&self, profile: &str) -> Result<()> {
        require_profile_name(profile)?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut doc = self.load()?;
        doc.credentials.remove(profile);
        self.save(&doc)
    }
}

const KEYRING_SERVICE: &str = crate::paths::APPLICATION;

/// OS keychain with owner-only JSON fallback.
///
/// Remembered passwords are written to the platform keyring first. If the
/// keyring is unavailable (CI, containers, missing Secret Service), the
/// existing [`FileCredentialStore`] is used. A successful keyring write
/// deletes the plaintext JSON copy for that profile.
pub struct PlatformCredentialStore {
    file: FileCredentialStore,
}

impl PlatformCredentialStore {
    /// Wraps a file store used as fallback and migration source.
    #[must_use]
    pub fn new(file: FileCredentialStore) -> Self {
        Self { file }
    }

    /// Discovers the platform config directory and uses the file store there
    /// as fallback.
    pub fn discover() -> Result<Self> {
        Ok(Self::new(FileCredentialStore::discover()?))
    }
}

fn keyring_entry(profile: &str) -> Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, profile)
        .map_err(|err| ConfigError::Keyring(err.to_string()))
}

fn keyring_get(profile: &str) -> Result<Option<String>> {
    match keyring_entry(profile)?.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(ConfigError::Keyring(err.to_string())),
    }
}

fn keyring_put(profile: &str, password: &str) -> Result<()> {
    keyring_entry(profile)?
        .set_password(password)
        .map_err(|err| ConfigError::Keyring(err.to_string()))
}

fn keyring_delete(profile: &str) -> Result<()> {
    match keyring_entry(profile)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(ConfigError::Keyring(err.to_string())),
    }
}

impl CredentialStore for PlatformCredentialStore {
    fn get(&self, profile: &str) -> Result<Credential> {
        require_profile_name(profile)?;
        match keyring_get(profile) {
            Ok(Some(password)) => return Ok(Credential { password }),
            Ok(None) => {}
            Err(err) => {
                tracing::debug!(error = %err, profile, "keyring get failed; trying file store");
            }
        }
        self.file.get(profile)
    }

    fn put(&self, profile: &str, credential: Credential) -> Result<()> {
        require_profile_name(profile)?;
        match keyring_put(profile, &credential.password) {
            Ok(()) => {
                let _ = self.file.delete(profile);
                Ok(())
            }
            Err(err) => {
                tracing::debug!(error = %err, profile, "keyring put failed; using file store");
                self.file.put(profile, credential)
            }
        }
    }

    fn delete(&self, profile: &str) -> Result<()> {
        require_profile_name(profile)?;
        if let Err(err) = keyring_delete(profile) {
            tracing::debug!(error = %err, profile, "keyring delete failed");
        }
        self.file.delete(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "mtui-config-test-{label}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn missing_store_returns_not_found() {
        let dir = TempDir::new("missing");
        let store = FileCredentialStore::new(dir.path());
        let err = store.get("router1").unwrap_err();
        assert!(matches!(err, ConfigError::CredentialsNotFound(_)));
    }

    #[test]
    fn put_then_get_roundtrips() {
        let dir = TempDir::new("roundtrip");
        let store = FileCredentialStore::new(dir.path());
        store
            .put(
                "router1",
                Credential {
                    password: "hunter2".to_string(),
                },
            )
            .unwrap();

        let credential = store.get("router1").unwrap();
        assert_eq!(credential.password, "hunter2");
    }

    #[test]
    fn delete_removes_credential_and_is_idempotent() {
        let dir = TempDir::new("delete");
        let store = FileCredentialStore::new(dir.path());
        store
            .put(
                "router1",
                Credential {
                    password: "hunter2".to_string(),
                },
            )
            .unwrap();

        store.delete("router1").unwrap();
        let err = store.get("router1").unwrap_err();
        assert!(matches!(err, ConfigError::CredentialsNotFound(_)));

        // Deleting an already-missing profile succeeds.
        store.delete("router1").unwrap();
    }

    #[test]
    fn empty_profile_name_is_rejected() {
        let dir = TempDir::new("empty-name");
        let store = FileCredentialStore::new(dir.path());
        let err = store.get("  ").unwrap_err();
        assert!(matches!(err, ConfigError::CredentialProfileNameRequired));
    }

    #[cfg(unix)]
    #[test]
    fn insecure_permissions_are_rejected() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new("perms");
        let store = FileCredentialStore::new(dir.path());
        store
            .put(
                "router1",
                Credential {
                    password: "hunter2".to_string(),
                },
            )
            .unwrap();

        fs::set_permissions(store.path(), fs::Permissions::from_mode(0o644)).unwrap();
        let err = store.get("router1").unwrap_err();
        assert!(matches!(err, ConfigError::InsecurePermissions { .. }));
    }

    #[test]
    fn platform_store_reads_file_fallback() {
        let dir = TempDir::new("platform-fallback");
        let file = FileCredentialStore::new(dir.path());
        let name = format!("fallback-{}", std::process::id());
        file.put(
            &name,
            Credential {
                password: "from-file".into(),
            },
        )
        .unwrap();
        let platform = PlatformCredentialStore::new(file);
        match platform.get(&name) {
            Ok(credential) => assert_eq!(credential.password, "from-file"),
            Err(err) => panic!("expected file fallback, got {err}"),
        }
        let _ = platform.delete(&name);
    }
}
