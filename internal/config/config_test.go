package config

import (
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"strings"
	"testing"
)

func TestAtomicProfileRoundTrip(t *testing.T) {
	dir := t.TempDir()
	store := NewFileStore(dir)
	first := []Profile{{
		Name:                   "lab",
		URL:                    "https://10.0.0.1",
		Username:               "admin",
		CertificateFingerprint: "sha256:abc",
		CustomCA:               "/run/secrets/router-ca.pem",
		Preferences:            Preferences{"theme": "dark", "refresh": "5s"},
	}}
	if err := store.Save(first); err != nil {
		t.Fatal(err)
	}

	first[0].Preferences["theme"] = "mutated"
	got, err := store.Load()
	if err != nil {
		t.Fatal(err)
	}
	first[0].Preferences["theme"] = "dark"
	if !reflect.DeepEqual(got, first) {
		t.Fatalf("round trip mismatch:\n got: %#v\nwant: %#v", got, first)
	}

	replacement := []Profile{
		{Name: "branch", URL: "http://router.local", Username: "operator"},
		{Name: "lab", URL: "https://10.0.0.2", Username: "admin"},
	}
	if err := store.Save(replacement); err != nil {
		t.Fatal(err)
	}
	got, err = store.Load()
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(got, replacement) {
		t.Fatalf("replacement mismatch:\n got: %#v\nwant: %#v", got, replacement)
	}

	temps, err := filepath.Glob(filepath.Join(dir, ".*.tmp-*"))
	if err != nil {
		t.Fatal(err)
	}
	if len(temps) != 0 {
		t.Fatalf("temporary files left behind: %v", temps)
	}
}

func TestProfileFileDoesNotPersistSecret(t *testing.T) {
	const marker = "MARKER_SECRET_must_not_escape"
	store := NewFileStore(t.TempDir())
	profile := Profile{
		Name:        "router",
		URL:         "https://router.example",
		Username:    "admin",
		Preferences: Preferences{"note": "ordinary"},
	}
	if err := store.Save([]Profile{profile}); err != nil {
		t.Fatal(err)
	}
	data, err := os.ReadFile(store.Path())
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(data), marker) {
		t.Fatal("secret marker persisted in profile file")
	}
	if strings.Contains(strings.ToLower(string(data)), "password") {
		t.Fatal("profile schema unexpectedly contains a password field")
	}
}

func TestProfilePermissions(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("POSIX permission bits are not meaningful on Windows")
	}
	store := NewFileStore(filepath.Join(t.TempDir(), "private"))
	if err := store.Save([]Profile{{Name: "r", URL: "https://r", Username: "u"}}); err != nil {
		t.Fatal(err)
	}
	info, err := os.Stat(store.Path())
	if err != nil {
		t.Fatal(err)
	}
	if got := info.Mode().Perm(); got != 0o600 {
		t.Fatalf("profile mode = %o, want 600", got)
	}
	dirInfo, err := os.Stat(filepath.Dir(store.Path()))
	if err != nil {
		t.Fatal(err)
	}
	if got := dirInfo.Mode().Perm(); got != 0o700 {
		t.Fatalf("profile directory mode = %o, want 700", got)
	}
}

func TestDefaultFileStoreHonorsXDGConfigHome(t *testing.T) {
	root := t.TempDir()
	t.Setenv("XDG_CONFIG_HOME", root)
	store, err := DefaultFileStore()
	if err != nil {
		t.Fatal(err)
	}
	want := filepath.Join(root, appName, fileName)
	if store.Path() != want {
		t.Fatalf("path = %q, want %q", store.Path(), want)
	}
}
