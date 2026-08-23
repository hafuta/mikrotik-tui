//! Headless environment/file secret overrides.
//!
//! Mirrors the `OverrideStore` behavior in `internal/credentials/credentials.go`,
//! scoped to the generic (non-profile-prefixed) `MIKROTIK_TUI_*` variables:
//! `HOST` (or deprecated `URL`), `USERNAME`, `PASSWORD`, `PASSWORD_FILE`,
//! `CA_FILE`, and `CERT_FINGERPRINT`.

use std::path::PathBuf;

use crate::credentials::CredentialStore;
use crate::error::{ConfigError, Result};
use crate::profile::Profile;

/// Prefix shared by every override variable.
pub const ENV_PREFIX: &str = "MIKROTIK_TUI";

/// Snapshot of `MIKROTIK_TUI_*` environment variables that override profile
/// and credential fields for headless/CI use. Populated once via
/// [`EnvOverrides::from_env`] (or [`EnvOverrides::from_lookup`] in tests).
#[derive(Debug, Clone, Default)]
pub struct EnvOverrides {
    /// `MIKROTIK_TUI_HOST`, or deprecated `MIKROTIK_TUI_URL`.
    pub url: Option<String>,
    /// `MIKROTIK_TUI_USERNAME`
    pub username: Option<String>,
    /// `MIKROTIK_TUI_PASSWORD`
    pub password: Option<String>,
    /// `MIKROTIK_TUI_PASSWORD_FILE`
    pub password_file: Option<String>,
    /// `MIKROTIK_TUI_CA_FILE`
    pub ca_file: Option<String>,
    /// `MIKROTIK_TUI_CERT_FINGERPRINT`
    pub cert_fingerprint: Option<String>,
}

impl EnvOverrides {
    /// Reads overrides from the process environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Reads overrides via a custom lookup function. Useful for tests that
    /// must not depend on (or mutate) the real process environment.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let get = |suffix: &str| lookup(&format!("{ENV_PREFIX}_{suffix}"));
        Self {
            url: get("HOST").or_else(|| get("URL")),
            username: get("USERNAME"),
            password: get("PASSWORD"),
            password_file: get("PASSWORD_FILE"),
            ca_file: get("CA_FILE"),
            cert_fingerprint: get("CERT_FINGERPRINT"),
        }
    }

    /// Applies the URL, username, certificate fingerprint, and CA overrides
    /// onto `profile` in place. `MIKROTIK_TUI_CA_FILE`, when set, is read
    /// and its contents replace `profile.custom_ca`.
    pub fn apply_to_profile(&self, profile: &mut Profile) -> Result<()> {
        if let Some(url) = &self.url {
            profile.url.clone_from(url);
        }
        if let Some(username) = &self.username {
            profile.username.clone_from(username);
        }
        if let Some(fingerprint) = &self.cert_fingerprint {
            profile.certificate_fingerprint.clone_from(fingerprint);
        }
        if let Some(path) = &self.ca_file {
            let data = std::fs::read_to_string(path).map_err(|source| ConfigError::SecretFile {
                path: PathBuf::from(path),
                source,
            })?;
            profile.custom_ca = data;
        }
        Ok(())
    }

    /// Resolves the router password using, in order:
    /// 1. `MIKROTIK_TUI_PASSWORD_FILE` (file contents, trailing newline
    ///    trimmed),
    /// 2. `MIKROTIK_TUI_PASSWORD`,
    /// 3. `store.get(profile_name)`, if `store` is given.
    ///
    /// Returns `Ok(None)` when no source has a password, rather than
    /// treating "not found" in the credential store as an error.
    pub fn resolve_password(
        &self,
        profile_name: &str,
        store: Option<&dyn CredentialStore>,
    ) -> Result<Option<String>> {
        if let Some(path) = &self.password_file {
            let data = std::fs::read_to_string(path).map_err(|source| ConfigError::SecretFile {
                path: PathBuf::from(path),
                source,
            })?;
            return Ok(Some(trim_secret_newline(&data)));
        }
        if let Some(password) = &self.password {
            return Ok(Some(password.clone()));
        }
        if let Some(store) = store {
            return match store.get(profile_name) {
                Ok(credential) if credential.password.is_empty() => Ok(None),
                Ok(credential) => Ok(Some(credential.password)),
                Err(ConfigError::CredentialsNotFound(_)) => Ok(None),
                Err(err) => Err(err),
            };
        }
        Ok(None)
    }
}

fn trim_secret_newline(value: &str) -> String {
    let value = value.strip_suffix('\n').unwrap_or(value);
    let value = value.strip_suffix('\r').unwrap_or(value);
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::Credential;
    use std::collections::HashMap;
    use std::fs;
    use std::sync::Mutex;

    fn lookup_from(map: HashMap<&'static str, &'static str>) -> impl Fn(&str) -> Option<String> {
        move |key| map.get(key).map(ToString::to_string)
    }

    #[test]
    fn from_lookup_reads_all_fields() {
        let mut map = HashMap::new();
        map.insert("MIKROTIK_TUI_HOST", "10.0.0.1:8729");
        map.insert("MIKROTIK_TUI_USERNAME", "admin");
        map.insert("MIKROTIK_TUI_PASSWORD", "hunter2");
        map.insert("MIKROTIK_TUI_CERT_FINGERPRINT", "aa:bb");
        let overrides = EnvOverrides::from_lookup(lookup_from(map));

        assert_eq!(overrides.url, Some("10.0.0.1:8729".to_string()));
        assert_eq!(overrides.username, Some("admin".to_string()));
        assert_eq!(overrides.password, Some("hunter2".to_string()));
        assert_eq!(overrides.cert_fingerprint, Some("aa:bb".to_string()));
        assert_eq!(overrides.password_file, None);
        assert_eq!(overrides.ca_file, None);
    }

    #[test]
    fn host_overrides_deprecated_url() {
        let mut map = HashMap::new();
        map.insert("MIKROTIK_TUI_HOST", "new.lan:8729");
        map.insert("MIKROTIK_TUI_URL", "https://old.lan");
        let overrides = EnvOverrides::from_lookup(lookup_from(map));
        assert_eq!(overrides.url, Some("new.lan:8729".to_string()));
    }

    #[test]
    fn deprecated_url_is_used_without_host() {
        let mut map = HashMap::new();
        map.insert("MIKROTIK_TUI_URL", "https://10.0.0.1");
        let overrides = EnvOverrides::from_lookup(lookup_from(map));
        assert_eq!(overrides.url, Some("https://10.0.0.1".to_string()));
    }

    #[test]
    fn apply_to_profile_overrides_fields() {
        let mut map = HashMap::new();
        map.insert("MIKROTIK_TUI_HOST", "override.lan:8729");
        map.insert("MIKROTIK_TUI_USERNAME", "override-user");
        let overrides = EnvOverrides::from_lookup(lookup_from(map));

        let mut profile = Profile {
            name: "r1".to_string(),
            url: "192.168.88.1:8729".to_string(),
            username: "original-user".to_string(),
            ..Profile::default()
        };
        overrides.apply_to_profile(&mut profile).unwrap();

        assert_eq!(profile.url, "override.lan:8729");
        assert_eq!(profile.username, "override-user");
    }

    struct StubStore(Mutex<HashMap<String, Credential>>);

    impl CredentialStore for StubStore {
        fn get(&self, profile: &str) -> Result<Credential> {
            self.0
                .lock()
                .unwrap()
                .get(profile)
                .cloned()
                .ok_or_else(|| ConfigError::CredentialsNotFound(profile.to_string()))
        }
        fn put(&self, profile: &str, credential: Credential) -> Result<()> {
            self.0
                .lock()
                .unwrap()
                .insert(profile.to_string(), credential);
            Ok(())
        }
        fn delete(&self, profile: &str) -> Result<()> {
            self.0.lock().unwrap().remove(profile);
            Ok(())
        }
    }

    #[test]
    fn resolve_password_prefers_password_file_over_password_env() {
        let dir = std::env::temp_dir().join(format!(
            "mtui-config-test-envfile-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let secret_path = dir.join("password.txt");
        fs::write(&secret_path, "from-file\n").unwrap();

        let mut map = HashMap::new();
        map.insert("MIKROTIK_TUI_PASSWORD", "from-env");
        let mut overrides = EnvOverrides::from_lookup(lookup_from(map));
        overrides.password_file = Some(secret_path.to_string_lossy().to_string());

        let resolved = overrides.resolve_password("router1", None).unwrap();
        assert_eq!(resolved, Some("from-file".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_password_falls_back_to_credential_store() {
        let overrides = EnvOverrides::default();
        let store = StubStore(Mutex::new(HashMap::from([(
            "router1".to_string(),
            Credential {
                password: "from-store".to_string(),
            },
        )])));

        let resolved = overrides.resolve_password("router1", Some(&store)).unwrap();
        assert_eq!(resolved, Some("from-store".to_string()));
    }

    #[test]
    fn resolve_password_returns_none_when_nothing_matches() {
        let overrides = EnvOverrides::default();
        let store = StubStore(Mutex::new(HashMap::new()));

        let resolved = overrides.resolve_password("router1", Some(&store)).unwrap();
        assert_eq!(resolved, None);
    }
}
