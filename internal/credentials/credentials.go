// Package credentials provides replaceable secret storage and headless
// environment/file-secret overrides.
package credentials

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"unicode"
)

const (
	appName         = "mikrotik-tui"
	credentialsFile = "credentials.json"
	fileFormat      = 1
	defaultPrefix   = "MIKROTIK_TUI"
)

// ErrNotFound indicates that no credentials exist for a profile.
var ErrNotFound = errors.New("credentials not found")

// Credential contains secrets used to authenticate a router.
type Credential struct {
	Password string `json:"password"`
}

// Store abstracts credential persistence so a platform keychain can replace
// the JSON backend without changing callers.
type Store interface {
	Get(context.Context, string) (Credential, error)
	Put(context.Context, string, Credential) error
	Delete(context.Context, string) error
}

// FileStore is a permission-hardened JSON credential backend.
type FileStore struct {
	path string
	mu   sync.Mutex
}

// NewFileStore creates a credential store rooted at dir.
func NewFileStore(dir string) *FileStore {
	return &FileStore{path: filepath.Join(dir, credentialsFile)}
}

// DefaultFileStore uses XDG_CONFIG_HOME when set, otherwise os.UserConfigDir.
func DefaultFileStore() (*FileStore, error) {
	dir, err := configDir()
	if err != nil {
		return nil, err
	}
	return NewFileStore(dir), nil
}

// Path returns the secret JSON file used by the store.
func (s *FileStore) Path() string { return s.path }

type document struct {
	Version     int                   `json:"version"`
	Credentials map[string]Credential `json:"credentials"`
}

// Get retrieves credentials for profile.
func (s *FileStore) Get(_ context.Context, profile string) (Credential, error) {
	if strings.TrimSpace(profile) == "" {
		return Credential{}, errors.New("profile name is required")
	}
	s.mu.Lock()
	defer s.mu.Unlock()

	doc, err := s.load()
	if err != nil {
		return Credential{}, err
	}
	credential, ok := doc.Credentials[profile]
	if !ok {
		return Credential{}, ErrNotFound
	}
	return credential, nil
}

// Put creates or replaces credentials for profile.
func (s *FileStore) Put(_ context.Context, profile string, credential Credential) error {
	if strings.TrimSpace(profile) == "" {
		return errors.New("profile name is required")
	}
	s.mu.Lock()
	defer s.mu.Unlock()

	doc, err := s.load()
	if err != nil {
		return err
	}
	doc.Credentials[profile] = credential
	return s.save(doc)
}

// Delete removes credentials for profile. Deleting a missing profile succeeds.
func (s *FileStore) Delete(_ context.Context, profile string) error {
	if strings.TrimSpace(profile) == "" {
		return errors.New("profile name is required")
	}
	s.mu.Lock()
	defer s.mu.Unlock()

	doc, err := s.load()
	if err != nil {
		return err
	}
	delete(doc.Credentials, profile)
	return s.save(doc)
}

func (s *FileStore) load() (document, error) {
	doc := document{Version: fileFormat, Credentials: make(map[string]Credential)}
	info, err := os.Stat(s.path)
	if errors.Is(err, os.ErrNotExist) {
		return doc, nil
	}
	if err != nil {
		return document{}, fmt.Errorf("stat credential store: %w", err)
	}
	if runtime.GOOS != "windows" && info.Mode().Perm()&0o077 != 0 {
		return document{}, fmt.Errorf("credential store %q has insecure permissions %o; want 0600", s.path, info.Mode().Perm())
	}

	data, err := os.ReadFile(s.path)
	if err != nil {
		return document{}, fmt.Errorf("read credential store: %w", err)
	}
	if err := json.Unmarshal(data, &doc); err != nil {
		return document{}, fmt.Errorf("decode credential store: %w", err)
	}
	if doc.Version != fileFormat {
		return document{}, fmt.Errorf("unsupported credential format version %d", doc.Version)
	}
	if doc.Credentials == nil {
		doc.Credentials = make(map[string]Credential)
	}
	return doc, nil
}

func (s *FileStore) save(doc document) error {
	data, err := json.MarshalIndent(doc, "", "  ")
	if err != nil {
		return fmt.Errorf("encode credential store: %w", err)
	}
	data = append(data, '\n')
	if err := atomicWriteFile(s.path, data, 0o600); err != nil {
		return fmt.Errorf("write credential store: %w", err)
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

// OverrideStore reads Docker-style *_FILE secrets and environment variables
// before falling back to its base Store. File overrides have highest priority.
type OverrideStore struct {
	Base   Store
	Prefix string
	Lookup func(string) (string, bool)
	Read   func(string) ([]byte, error)
}

// NewOverrideStore wraps base with MIKROTIK_TUI_* environment and file
// overrides.
func NewOverrideStore(base Store) *OverrideStore {
	return &OverrideStore{
		Base:   base,
		Prefix: defaultPrefix,
		Lookup: os.LookupEnv,
		Read:   os.ReadFile,
	}
}

// Get resolves MIKROTIK_TUI_<PROFILE>_PASSWORD_FILE,
// MIKROTIK_TUI_<PROFILE>_PASSWORD, MIKROTIK_TUI_PASSWORD_FILE, then
// MIKROTIK_TUI_PASSWORD by default, and finally delegates to the base Store.
func (s *OverrideStore) Get(ctx context.Context, profile string) (Credential, error) {
	if strings.TrimSpace(profile) == "" {
		return Credential{}, errors.New("profile name is required")
	}
	prefix := sanitizeEnv(s.Prefix)
	if prefix == "" {
		prefix = defaultPrefix
	}
	profilePrefix := prefix + "_" + sanitizeEnv(profile)

	for _, key := range []string{profilePrefix + "_PASSWORD_FILE", prefix + "_PASSWORD_FILE"} {
		if path, ok := s.lookup()(key); ok {
			data, err := s.read()(path)
			if err != nil {
				return Credential{}, fmt.Errorf("read secret file from %s: %w", key, err)
			}
			return Credential{Password: trimSecretNewline(string(data))}, nil
		}
	}
	for _, key := range []string{profilePrefix + "_PASSWORD", prefix + "_PASSWORD"} {
		if value, ok := s.lookup()(key); ok {
			return Credential{Password: value}, nil
		}
	}
	if s.Base == nil {
		return Credential{}, ErrNotFound
	}
	return s.Base.Get(ctx, profile)
}

// Put delegates to the base store; environment overrides are read-only.
func (s *OverrideStore) Put(ctx context.Context, profile string, credential Credential) error {
	if s.Base == nil {
		return errors.New("credential override has no writable base store")
	}
	return s.Base.Put(ctx, profile, credential)
}

// Delete delegates to the base store.
func (s *OverrideStore) Delete(ctx context.Context, profile string) error {
	if s.Base == nil {
		return errors.New("credential override has no writable base store")
	}
	return s.Base.Delete(ctx, profile)
}

func (s *OverrideStore) lookup() func(string) (string, bool) {
	if s.Lookup != nil {
		return s.Lookup
	}
	return os.LookupEnv
}

func (s *OverrideStore) read() func(string) ([]byte, error) {
	if s.Read != nil {
		return s.Read
	}
	return os.ReadFile
}

func sanitizeEnv(value string) string {
	var builder strings.Builder
	lastUnderscore := false
	for _, r := range strings.ToUpper(value) {
		if unicode.IsLetter(r) || unicode.IsDigit(r) {
			builder.WriteRune(r)
			lastUnderscore = false
		} else if !lastUnderscore {
			builder.WriteByte('_')
			lastUnderscore = true
		}
	}
	return strings.Trim(builder.String(), "_")
}

func trimSecretNewline(value string) string {
	value = strings.TrimSuffix(value, "\n")
	return strings.TrimSuffix(value, "\r")
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
