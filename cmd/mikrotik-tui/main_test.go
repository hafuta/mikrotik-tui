package main

import (
	"bytes"
	"context"
	"strings"
	"testing"

	"github.com/hafuta/mikrotik-tui/internal/config"
	"github.com/hafuta/mikrotik-tui/internal/credentials"
)

func TestVersionFlag(t *testing.T) {
	original := version
	version = "test-version"
	t.Cleanup(func() { version = original })
	var output bytes.Buffer
	if err := run([]string{"--version"}, &output, &output); err != nil {
		t.Fatal(err)
	}
	if strings.TrimSpace(output.String()) != "test-version" {
		t.Fatalf("version output = %q", output.String())
	}
}

func TestInitialConnectionFromEnvironment(t *testing.T) {
	t.Setenv("MIKROTIK_TUI_URL", "https://192.0.2.1:8443")
	t.Setenv("MIKROTIK_TUI_USERNAME", "reader")
	t.Setenv("MIKROTIK_TUI_PASSWORD", "secret")
	profiles := config.NewFileStore(t.TempDir())
	secrets := credentials.NewOverrideStore(credentials.NewFileStore(t.TempDir()))

	profile, credential, err := initialConnection(context.Background(), profiles, secrets)
	if err != nil {
		t.Fatal(err)
	}
	if profile == nil || profile.Name != "default" || profile.URL != "https://192.0.2.1:8443" || profile.Username != "reader" {
		t.Fatalf("profile = %#v", profile)
	}
	if credential.Password != "secret" {
		t.Fatalf("credential was not resolved from environment")
	}
}
