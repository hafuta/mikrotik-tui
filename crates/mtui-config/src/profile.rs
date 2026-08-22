//! Non-secret router profile persistence.
//!
//! Mirrors `internal/config/config.go`: profiles never carry passwords, the
//! whole document is versioned, and writes are atomic.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, Result};
use crate::fsutil::atomic_write_file;
use crate::paths;

/// File name for the profile document, relative to the config directory.
pub const PROFILE_FILE_NAME: &str = "profiles.json";
const PROFILE_FILE_VERSION: u32 = 1;

/// Preference key for the active UI theme. Reserved for future theme
/// switching: the UI resolves this id against
/// `mtui_core::theme::ThemeRegistry`; an absent key means "use the
/// application default" (currently `mtui_core::theme::DefaultTheme::ID`,
/// i.e. `"default"`).
pub const THEME_PREFERENCE_KEY: &str = "theme";

/// Preference key for sidebar items the operator has tucked away.
/// Value is a comma-separated list of navigation ids (groups or resources).
pub const HIDDEN_NAV_PREFERENCE_KEY: &str = "hidden_nav";

/// Per-profile UI preferences. String values keep the on-disk format stable
/// while letting the UI add new preferences without a schema migration.
///
/// Well-known keys:
/// - [`THEME_PREFERENCE_KEY`] (`"theme"`): theme id to activate for this
///   profile (e.g. `"default"`).
/// - [`HIDDEN_NAV_PREFERENCE_KEY`] (`"hidden_nav"`): comma-separated nav ids
///   to omit from the sidebar until the operator reveals them.
pub type Preferences = HashMap<String, String>;

/// A named `RouterOS` connection. Passwords and other secrets intentionally
/// do not belong here; see [`crate::credentials`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub url: String,
    pub username: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub certificate_fingerprint: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub custom_ca: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub preferences: Preferences,
}

impl Profile {
    /// Reads this profile's preferred theme id, if set. See
    /// [`THEME_PREFERENCE_KEY`].
    #[must_use]
    pub fn theme_id(&self) -> Option<&str> {
        self.preferences
            .get(THEME_PREFERENCE_KEY)
            .map(String::as_str)
    }

    /// Sets this profile's preferred theme id. See [`THEME_PREFERENCE_KEY`].
    pub fn set_theme_id(&mut self, theme_id: impl Into<String>) {
        self.preferences
            .insert(THEME_PREFERENCE_KEY.to_string(), theme_id.into());
    }

    /// Reads tucked-away navigation ids. See [`HIDDEN_NAV_PREFERENCE_KEY`].
    #[must_use]
    pub fn hidden_nav_ids(&self) -> Vec<String> {
        parse_hidden_nav(
            self.preferences
                .get(HIDDEN_NAV_PREFERENCE_KEY)
                .map_or("", String::as_str),
        )
    }

    /// Replaces tucked-away navigation ids. An empty list removes the key.
    pub fn set_hidden_nav_ids(&mut self, ids: impl IntoIterator<Item = impl AsRef<str>>) {
        let mut ids: Vec<String> = ids
            .into_iter()
            .map(|id| id.as_ref().trim().to_string())
            .filter(|id| !id.is_empty())
            .collect();
        ids.sort();
        ids.dedup();
        if ids.is_empty() {
            self.preferences.remove(HIDDEN_NAV_PREFERENCE_KEY);
        } else {
            self.preferences
                .insert(HIDDEN_NAV_PREFERENCE_KEY.to_string(), ids.join(","));
        }
    }

    /// Validates required fields. Called by [`ProfileStore::load`] and
    /// [`ProfileStore::save`] so callers cannot persist an incomplete
    /// profile.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(ConfigError::ProfileNameRequired);
        }
        if self.url.trim().is_empty() {
            return Err(ConfigError::ProfileUrlRequired(self.name.clone()));
        }
        if self.username.trim().is_empty() {
            return Err(ConfigError::ProfileUsernameRequired(self.name.clone()));
        }
        Ok(())
    }
}

fn parse_hidden_nav(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    if raw.starts_with('[')
        && let Ok(ids) = serde_json::from_str::<Vec<String>>(raw)
    {
        return ids
            .into_iter()
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .collect();
    }
    raw.split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ProfileDocument {
    version: u32,
    #[serde(default)]
    profiles: BTreeMap<String, Profile>,
}

/// Persists named profiles in one atomically replaced JSON document.
pub struct ProfileStore {
    path: PathBuf,
}

impl ProfileStore {
    /// Creates a profile store rooted at `dir`.
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            path: dir.as_ref().join(PROFILE_FILE_NAME),
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

    /// Returns all profiles sorted by name. A missing store is empty.
    pub fn load(&self) -> Result<Vec<Profile>> {
        let data = match fs::read(&self.path) {
            Ok(data) => data,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(ConfigError::Read {
                    path: self.path.clone(),
                    source,
                });
            }
        };

        let doc: ProfileDocument =
            serde_json::from_slice(&data).map_err(|source| ConfigError::Decode {
                path: self.path.clone(),
                source,
            })?;
        if doc.version != PROFILE_FILE_VERSION {
            return Err(ConfigError::UnsupportedVersion {
                what: "profile",
                version: doc.version,
            });
        }

        let mut profiles = Vec::with_capacity(doc.profiles.len());
        for (key, mut profile) in doc.profiles {
            if profile.name.is_empty() {
                profile.name = key;
            }
            profile.validate()?;
            profiles.push(profile);
        }
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(profiles)
    }

    /// Atomically replaces the profile document with `profiles`.
    pub fn save(&self, profiles: &[Profile]) -> Result<()> {
        let mut doc = ProfileDocument {
            version: PROFILE_FILE_VERSION,
            profiles: BTreeMap::new(),
        };
        for profile in profiles {
            profile.validate()?;
            if doc.profiles.contains_key(&profile.name) {
                return Err(ConfigError::DuplicateProfile(profile.name.clone()));
            }
            doc.profiles.insert(profile.name.clone(), profile.clone());
        }

        let mut data = serde_json::to_vec_pretty(&doc).map_err(|source| ConfigError::Encode {
            what: "profiles",
            source,
        })?;
        data.push(b'\n');
        atomic_write_file(&self.path, &data, 0o600)
    }

    /// Adds or replaces a single profile by name, preserving the others.
    pub fn upsert(&self, profile: Profile) -> Result<()> {
        profile.validate()?;
        let mut profiles = self.load()?;
        match profiles.iter_mut().find(|p| p.name == profile.name) {
            Some(existing) => *existing = profile,
            None => profiles.push(profile),
        }
        self.save(&profiles)
    }

    /// Removes the profile named `name`, if present. Deleting a missing
    /// profile succeeds.
    pub fn delete(&self, name: &str) -> Result<()> {
        let mut profiles = self.load()?;
        let before = profiles.len();
        profiles.retain(|p| p.name != name);
        if profiles.len() != before {
            self.save(&profiles)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn sample(name: &str) -> Profile {
        Profile {
            name: name.to_string(),
            url: "https://192.168.88.1".to_string(),
            username: "admin".to_string(),
            ..Profile::default()
        }
    }

    #[test]
    fn missing_store_loads_empty() {
        let dir = TempDir::new("missing");
        let store = ProfileStore::new(dir.path());
        assert_eq!(store.load().unwrap(), Vec::new());
    }

    #[test]
    fn save_then_load_roundtrips_and_sorts() {
        let dir = TempDir::new("roundtrip");
        let store = ProfileStore::new(dir.path());
        let mut b = sample("bravo");
        b.set_theme_id("default");
        let a = sample("alpha");
        store.save(&[b.clone(), a.clone()]).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "alpha");
        assert_eq!(loaded[1].name, "bravo");
        assert_eq!(loaded[1].theme_id(), Some("default"));
    }

    #[test]
    fn upsert_replaces_existing_by_name() {
        let dir = TempDir::new("upsert");
        let store = ProfileStore::new(dir.path());
        store.upsert(sample("router1")).unwrap();
        let mut updated = sample("router1");
        updated.username = "backup-admin".to_string();
        store.upsert(updated).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].username, "backup-admin");
    }

    #[test]
    fn delete_removes_profile_and_is_idempotent() {
        let dir = TempDir::new("delete");
        let store = ProfileStore::new(dir.path());
        store.upsert(sample("router1")).unwrap();
        store.upsert(sample("router2")).unwrap();

        store.delete("router1").unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "router2");

        // Deleting an already-missing profile succeeds.
        store.delete("router1").unwrap();
    }

    #[test]
    fn save_rejects_missing_required_fields() {
        let dir = TempDir::new("invalid");
        let store = ProfileStore::new(dir.path());
        let mut bad = sample("router1");
        bad.url.clear();
        let err = store.save(&[bad]).unwrap_err();
        assert!(matches!(err, ConfigError::ProfileUrlRequired(_)));
    }

    #[test]
    fn save_rejects_duplicate_names() {
        let dir = TempDir::new("duplicate");
        let store = ProfileStore::new(dir.path());
        let err = store
            .save(&[sample("router1"), sample("router1")])
            .unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateProfile(_)));
    }

    #[test]
    fn rejects_unsupported_version() {
        let dir = TempDir::new("version");
        let path = dir.path().join(PROFILE_FILE_NAME);
        fs::write(&path, br#"{"version":99,"profiles":{}}"#).unwrap();
        let store = ProfileStore::new(dir.path());
        let err = store.load().unwrap_err();
        assert!(matches!(err, ConfigError::UnsupportedVersion { .. }));
    }

    #[test]
    fn hidden_nav_roundtrips_sorted_unique_ids() {
        let mut profile = sample("router1");
        profile.set_hidden_nav_ids(["vlan", "ppp-group", "vlan", ""]);
        assert_eq!(profile.hidden_nav_ids(), ["ppp-group", "vlan"]);
        assert_eq!(
            profile
                .preferences
                .get(HIDDEN_NAV_PREFERENCE_KEY)
                .map(String::as_str),
            Some("ppp-group,vlan")
        );
        profile.set_hidden_nav_ids(Vec::<String>::new());
        assert!(profile.hidden_nav_ids().is_empty());
        assert!(!profile.preferences.contains_key(HIDDEN_NAV_PREFERENCE_KEY));
    }

    #[test]
    fn hidden_nav_accepts_json_array_from_hand_edited_files() {
        let mut profile = sample("router1");
        profile.preferences.insert(
            HIDDEN_NAV_PREFERENCE_KEY.into(),
            r#"["bridge-group","arp"]"#.into(),
        );
        assert_eq!(profile.hidden_nav_ids(), ["bridge-group", "arp"]);
    }
}
