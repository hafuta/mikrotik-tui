// Package config persists non-secret application and router profile settings.
package config

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

const (
	appName    = "mikrotik-tui"
	fileName   = "profiles.json"
	fileFormat = 1
)

// Preferences contains per-router user interface preferences. String values
// keep the on-disk format stable while allowing the UI to add preferences.
type Preferences map[string]string

// Profile describes a named RouterOS connection. Passwords and other
// credentials intentionally do not belong in this type.
type Profile struct {
	Name                   string      `json:"name"`
	URL                    string      `json:"url"`
	Username               string      `json:"username"`
	CertificateFingerprint string      `json:"certificate_fingerprint,omitempty"`
	CustomCA               string      `json:"custom_ca,omitempty"`
	Preferences            Preferences `json:"preferences,omitempty"`
}

// FileStore persists named profiles in one atomically replaced JSON document.
type FileStore struct {
	path string
}

// NewFileStore creates a profile store rooted at dir.
func NewFileStore(dir string) *FileStore {
	return &FileStore{path: filepath.Join(dir, fileName)}
}

// DefaultFileStore uses XDG_CONFIG_HOME when set, otherwise os.UserConfigDir.
func DefaultFileStore() (*FileStore, error) {
	dir, err := configDir()
	if err != nil {
		return nil, err
	}
	return NewFileStore(dir), nil
}

// Path returns the JSON file used by the store.
func (s *FileStore) Path() string { return s.path }

type document struct {
	Version  int                `json:"version"`
	Profiles map[string]Profile `json:"profiles"`
}

// Load returns all profiles sorted by name. A missing store is empty.
func (s *FileStore) Load() ([]Profile, error) {
	data, err := os.ReadFile(s.path)
	if errors.Is(err, os.ErrNotExist) {
		return []Profile{}, nil
	}
	if err != nil {
		return nil, fmt.Errorf("read profiles: %w", err)
	}

	var doc document
	if err := json.Unmarshal(data, &doc); err != nil {
		return nil, fmt.Errorf("decode profiles: %w", err)
	}
	if doc.Version != fileFormat {
		return nil, fmt.Errorf("unsupported profile format version %d", doc.Version)
	}

	profiles := make([]Profile, 0, len(doc.Profiles))
	for name, profile := range doc.Profiles {
		if profile.Name == "" {
			profile.Name = name
		}
		if err := validateProfile(profile); err != nil {
			return nil, fmt.Errorf("profile %q: %w", name, err)
		}
		profiles = append(profiles, cloneProfile(profile))
	}
	sort.Slice(profiles, func(i, j int) bool { return profiles[i].Name < profiles[j].Name })
	return profiles, nil
}

// Save atomically replaces the profile document.
func (s *FileStore) Save(profiles []Profile) error {
	doc := document{Version: fileFormat, Profiles: make(map[string]Profile, len(profiles))}
	for _, profile := range profiles {
		if err := validateProfile(profile); err != nil {
			return err
		}
		if _, exists := doc.Profiles[profile.Name]; exists {
			return fmt.Errorf("duplicate profile %q", profile.Name)
		}
		doc.Profiles[profile.Name] = cloneProfile(profile)
	}

	data, err := json.MarshalIndent(doc, "", "  ")
	if err != nil {
		return fmt.Errorf("encode profiles: %w", err)
	}
	data = append(data, '\n')
	if err := atomicWriteFile(s.path, data, 0o600); err != nil {
		return fmt.Errorf("write profiles: %w", err)
	}
	return nil
}

func configDir() (string, error) {
	if dir := strings.TrimSpace(os.Getenv("XDG_CONFIG_HOME")); dir != "" {
		return filepath.Join(dir, appName), nil
	}
	dir, err := os.UserConfigDir()
	if err != nil {
		return "", fmt.Errorf("locate user config directory: %w", err)
	}
	return filepath.Join(dir, appName), nil
}

func validateProfile(profile Profile) error {
	if strings.TrimSpace(profile.Name) == "" {
		return errors.New("profile name is required")
	}
	if strings.TrimSpace(profile.URL) == "" {
		return fmt.Errorf("profile %q URL is required", profile.Name)
	}
	if strings.TrimSpace(profile.Username) == "" {
		return fmt.Errorf("profile %q username is required", profile.Name)
	}
	return nil
}

func cloneProfile(profile Profile) Profile {
	if profile.Preferences != nil {
		preferences := make(Preferences, len(profile.Preferences))
		for key, value := range profile.Preferences {
			preferences[key] = value
		}
		profile.Preferences = preferences
	}
	return profile
}

func atomicWriteFile(path string, data []byte, mode os.FileMode) (err error) {
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0o700); err != nil {
		return err
	}
	if err := os.Chmod(dir, 0o700); err != nil {
		return err
	}

	temp, err := os.CreateTemp(dir, "."+filepath.Base(path)+".tmp-*")
	if err != nil {
		return err
	}
	tempName := temp.Name()
	defer func() {
		_ = temp.Close()
		_ = os.Remove(tempName)
	}()

	if err = temp.Chmod(mode); err != nil {
		return err
	}
	if _, err = temp.Write(data); err != nil {
		return err
	}
	if err = temp.Sync(); err != nil {
		return err
	}
	if err = temp.Close(); err != nil {
		return err
	}
	if err = replaceFile(tempName, path); err != nil {
		return err
	}
	return os.Chmod(path, mode)
}
