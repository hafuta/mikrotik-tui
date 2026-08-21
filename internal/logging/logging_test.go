package logging

import (
	"encoding/base64"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

const markerSecret = "MARKER_SECRET_7d912f_must_not_escape"

func TestLoggerRedactsSecretsEverywhere(t *testing.T) {
	dir := t.TempDir()
	logger, closer, err := New(Config{
		Dir:        dir,
		Filename:   "app.log",
		Level:      slog.LevelDebug,
		MaxBytes:   1 << 20,
		MaxBackups: 2,
	})
	if err != nil {
		t.Fatal(err)
	}

	basic := base64.StdEncoding.EncodeToString([]byte("admin:" + markerSecret))
	logger.With("password", markerSecret).Info(
		"password="+markerSecret+" Authorization: Basic "+basic,
		"authorization", "Basic "+basic,
		"endpoint", "https://admin:"+markerSecret+"@router.example/rest",
		"request", struct {
			Password string
		}{Password: markerSecret},
		"nested", slog.GroupValue(
			slog.String("api_key", markerSecret),
			slog.String("safe_url", "http://user:"+markerSecret+"@router.local"),
		),
	)
	if err := closer.Close(); err != nil {
		t.Fatal(err)
	}

	data, err := os.ReadFile(filepath.Join(dir, "app.log"))
	if err != nil {
		t.Fatal(err)
	}
	text := string(data)
	for _, forbidden := range []string{markerSecret, basic, "admin:"} {
		if strings.Contains(text, forbidden) {
			t.Fatalf("log contains forbidden secret material %q:\n%s", forbidden, text)
		}
	}
	if count := strings.Count(text, redacted); count < 7 {
		t.Fatalf("expected comprehensive redaction, got %d markers:\n%s", count, text)
	}
}

func TestRotationIsBoundedAndAllFilesAreRedacted(t *testing.T) {
	dir := t.TempDir()
	logger, closer, err := New(Config{
		Dir:        dir,
		Filename:   "app.log",
		MaxBytes:   300,
		MaxBackups: 2,
	})
	if err != nil {
		t.Fatal(err)
	}
	for index := 0; index < 30; index++ {
		logger.Info("rotation entry",
			"index", index,
			"password", markerSecret,
			"padding", strings.Repeat("x", 80),
		)
	}
	if err := closer.Close(); err != nil {
		t.Fatal(err)
	}

	files, err := filepath.Glob(filepath.Join(dir, "app.log*"))
	if err != nil {
		t.Fatal(err)
	}
	if len(files) != 3 {
		t.Fatalf("log file count = %d (%v), want current plus two backups", len(files), files)
	}
	for _, path := range files {
		data, readErr := os.ReadFile(path)
		if readErr != nil {
			t.Fatal(readErr)
		}
		if strings.Contains(string(data), markerSecret) {
			t.Fatalf("secret marker emitted in %s", path)
		}
		if int64(len(data)) > 300 {
			t.Fatalf("%s size = %d, want at most 300", path, len(data))
		}
	}
}

func TestLogPermissions(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("POSIX permission bits are not meaningful on Windows")
	}
	dir := filepath.Join(t.TempDir(), "private")
	_, closer, err := New(Config{Dir: dir, Filename: "app.log", MaxBytes: 1024})
	if err != nil {
		t.Fatal(err)
	}
	if err := closer.Close(); err != nil {
		t.Fatal(err)
	}

	for path, want := range map[string]os.FileMode{
		dir:                           0o700,
		filepath.Join(dir, "app.log"): 0o600,
	} {
		info, statErr := os.Stat(path)
		if statErr != nil {
			t.Fatal(statErr)
		}
		if got := info.Mode().Perm(); got != want {
			t.Errorf("%s mode = %o, want %o", path, got, want)
		}
	}
}

func TestDefaultConfigLocationPrecedence(t *testing.T) {
	state := filepath.Join(t.TempDir(), "state")
	cache := filepath.Join(t.TempDir(), "cache")
	t.Setenv("XDG_STATE_HOME", state)
	t.Setenv("XDG_CACHE_HOME", cache)

	config, err := DefaultConfig()
	if err != nil {
		t.Fatal(err)
	}
	want := filepath.Join(state, appName)
	if config.Dir != want {
		t.Fatalf("log dir = %q, want %q", config.Dir, want)
	}
}

func TestRedactFreeFormRepresentations(t *testing.T) {
	basic := base64.StdEncoding.EncodeToString([]byte("admin:" + markerSecret))
	input := fmt.Sprintf(
		`password: "%s", token=%s Basic %s https://admin:%s@router`,
		markerSecret, markerSecret, basic, markerSecret,
	)
	got := Redact(input)
	for _, forbidden := range []string{markerSecret, basic, "admin:"} {
		if strings.Contains(got, forbidden) {
			t.Fatalf("Redact left %q in %q", forbidden, got)
		}
	}
}
