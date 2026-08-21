package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"io"
	"log/slog"
	"os"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/hafuta/mikrotik-tui/internal/app"
	"github.com/hafuta/mikrotik-tui/internal/config"
	"github.com/hafuta/mikrotik-tui/internal/credentials"
	applog "github.com/hafuta/mikrotik-tui/internal/logging"
	"github.com/hafuta/mikrotik-tui/internal/routeros"
)

var version = "dev"

func main() {
	if err := run(os.Args[1:], os.Stdout, os.Stderr); err != nil {
		fmt.Fprintln(os.Stderr, "mikrotik-tui:", err)
		os.Exit(1)
	}
}

func run(args []string, stdout, stderr io.Writer) error {
	flags := flag.NewFlagSet("mikrotik-tui", flag.ContinueOnError)
	flags.SetOutput(stderr)
	showVersion := flags.Bool("version", false, "print version and exit")
	noAltScreen := flags.Bool("no-alt-screen", false, "render without the terminal alternate screen")
	if err := flags.Parse(args); err != nil {
		return err
	}
	if *showVersion {
		_, err := fmt.Fprintln(stdout, version)
		return err
	}

	logger, closer, err := applog.Default()
	if err != nil {
		logger = slog.New(slog.NewJSONHandler(io.Discard, nil))
		closer = io.NopCloser(strings.NewReader(""))
		fmt.Fprintln(stderr, "warning: application logging disabled:", err)
	}
	defer closer.Close()

	profiles, err := config.DefaultFileStore()
	if err != nil {
		return err
	}
	secretFile, err := credentials.DefaultFileStore()
	if err != nil {
		return err
	}
	secrets := credentials.NewOverrideStore(secretFile)

	profile, credential, err := initialConnection(context.Background(), profiles, secrets)
	if err != nil {
		logger.Warn("saved connection unavailable", "error", err)
	}
	options := app.Options{
		Services: app.Services{
			Profiles:    profiles,
			Credentials: secrets,
			Logger:      logger,
			NewClient:   app.DefaultClientFactory,
			Probe:       routeros.ProbeCertificate,
		},
		Profile:    profile,
		Credential: credential,
	}
	programOptions := []tea.ProgramOption{tea.WithOutput(stdout)}
	if !*noAltScreen {
		programOptions = append(programOptions, tea.WithAltScreen())
	}
	_, err = tea.NewProgram(app.New(options), programOptions...).Run()
	if err != nil {
		logger.Error("terminal program stopped", "error", err)
		return err
	}
	return nil
}

func initialConnection(ctx context.Context, profiles *config.FileStore, secrets credentials.Store) (*config.Profile, credentials.Credential, error) {
	saved, err := profiles.Load()
	if err != nil {
		return nil, credentials.Credential{}, err
	}
	var profile *config.Profile
	if len(saved) > 0 {
		value := saved[0]
		profile = &value
	}
	if url := strings.TrimSpace(os.Getenv("MIKROTIK_TUI_URL")); url != "" {
		if profile == nil {
			profile = &config.Profile{Name: "default"}
		}
		profile.URL = url
	}
	if username := strings.TrimSpace(os.Getenv("MIKROTIK_TUI_USERNAME")); username != "" {
		if profile == nil {
			profile = &config.Profile{Name: "default"}
		}
		profile.Username = username
	}
	if profile == nil || profile.URL == "" || profile.Username == "" {
		return nil, credentials.Credential{}, nil
	}
	if ca := strings.TrimSpace(os.Getenv("MIKROTIK_TUI_CA_FILE")); ca != "" {
		profile.CustomCA = ca
	}
	if pin := strings.TrimSpace(os.Getenv("MIKROTIK_TUI_CERT_FINGERPRINT")); pin != "" {
		profile.CertificateFingerprint = pin
	}
	credential, err := secrets.Get(ctx, profile.Name)
	if errors.Is(err, credentials.ErrNotFound) {
		return profile, credentials.Credential{}, nil
	}
	return profile, credential, err
}
