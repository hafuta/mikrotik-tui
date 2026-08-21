package credentials

import (
	"context"
	"errors"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	profileconfig "github.com/hafuta/mikrotik-tui/internal/config"
)

const markerSecret = "MARKER_SECRET_7d912f_must_not_escape"

func TestFileStoreRoundTripAndSecretIsolation(t *testing.T) {
	root := t.TempDir()
	profiles := profileconfig.NewFileStore(root)
	if err := profiles.Save([]profileconfig.Profile{{
		Name:     "lab",
		URL:      "https://router.example",
		Username: "admin",
	}}); err != nil {
		t.Fatal(err)
	}

	store := NewFileStore(root)
	ctx := context.Background()
	if err := store.Put(ctx, "lab", Credential{Password: markerSecret}); err != nil {
		t.Fatal(err)
	}
	got, err := store.Get(ctx, "lab")
	if err != nil {
		t.Fatal(err)
	}
	if got.Password != markerSecret {
		t.Fatalf("password = %q, want marker", got.Password)
	}

	err = filepath.WalkDir(root, func(path string, entry os.DirEntry, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		if entry.IsDir() {
			return nil
		}
		data, readErr := os.ReadFile(path)
		if readErr != nil {
			return readErr
		}
		if strings.Contains(string(data), markerSecret) && path != store.Path() {
			t.Errorf("secret marker escaped credential file into %s", path)
		}
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}

	if err := store.Delete(ctx, "lab"); err != nil {
		t.Fatal(err)
	}
	if _, err := store.Get(ctx, "lab"); !errors.Is(err, ErrNotFound) {
		t.Fatalf("Get after Delete error = %v, want ErrNotFound", err)
	}
}

func TestEnvironmentAndFileOverrides(t *testing.T) {
	ctx := context.Background()
	base := NewFileStore(t.TempDir())
	if err := base.Put(ctx, "branch-office", Credential{Password: "stored"}); err != nil {
		t.Fatal(err)
	}
	store := NewOverrideStore(base)

	t.Setenv("MIKROTIK_TUI_BRANCH_OFFICE_PASSWORD", "from-profile-env")
	got, err := store.Get(ctx, "branch-office")
	if err != nil {
		t.Fatal(err)
	}
	if got.Password != "from-profile-env" {
		t.Fatalf("profile env override = %q", got.Password)
	}

	secretFile := filepath.Join(t.TempDir(), "password")
	if err := os.WriteFile(secretFile, []byte("from-docker-secret\r\n"), 0o600); err != nil {
		t.Fatal(err)
	}
	t.Setenv("MIKROTIK_TUI_BRANCH_OFFICE_PASSWORD_FILE", secretFile)
	got, err = store.Get(ctx, "branch-office")
	if err != nil {
		t.Fatal(err)
	}
	if got.Password != "from-docker-secret" {
		t.Fatalf("file override = %q", got.Password)
	}
}

func TestGlobalOverrideAndBaseFallback(t *testing.T) {
	ctx := context.Background()
	base := NewFileStore(t.TempDir())
	if err := base.Put(ctx, "lab", Credential{Password: "stored"}); err != nil {
		t.Fatal(err)
	}
	store := NewOverrideStore(base)
	overrides := map[string]string{}
	store.Lookup = func(key string) (string, bool) {
		value, ok := overrides[key]
		return value, ok
	}

	got, err := store.Get(ctx, "lab")
	if err != nil {
		t.Fatal(err)
	}
	if got.Password != "stored" {
		t.Fatalf("base password = %q", got.Password)
	}

	overrides["MIKROTIK_TUI_PASSWORD"] = "global"
	got, err = store.Get(ctx, "other")
	if err != nil {
		t.Fatal(err)
	}
	if got.Password != "global" {
		t.Fatalf("global password = %q", got.Password)
	}
}

func TestCredentialPermissions(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("POSIX permission bits are not meaningful on Windows")
	}
	dir := filepath.Join(t.TempDir(), "private")
	store := NewFileStore(dir)
	if err := store.Put(context.Background(), "lab", Credential{Password: markerSecret}); err != nil {
		t.Fatal(err)
	}
	info, err := os.Stat(store.Path())
	if err != nil {
		t.Fatal(err)
	}
	if got := info.Mode().Perm(); got != 0o600 {
		t.Fatalf("credential mode = %o, want 600", got)
	}
	dirInfo, err := os.Stat(dir)
	if err != nil {
		t.Fatal(err)
	}
	if got := dirInfo.Mode().Perm(); got != 0o700 {
		t.Fatalf("credential directory mode = %o, want 700", got)
	}
	if err := os.Chmod(store.Path(), 0o644); err != nil {
		t.Fatal(err)
	}
	if _, err := store.Get(context.Background(), "lab"); err == nil ||
		!strings.Contains(err.Error(), "insecure permissions") {
		t.Fatalf("Get with insecure mode error = %v", err)
	}
}
