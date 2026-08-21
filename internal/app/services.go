package app

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"strings"

	"github.com/hafuta/mikrotik-tui/internal/config"
	"github.com/hafuta/mikrotik-tui/internal/credentials"
	"github.com/hafuta/mikrotik-tui/internal/routeros"
)

type ProfileStore interface {
	Load() ([]config.Profile, error)
	Save([]config.Profile) error
}

type Services struct {
	Profiles    ProfileStore
	Credentials credentials.Store
	Logger      *slog.Logger
	NewClient   func(config.Profile, string) (routeros.Client, error)
	Probe       func(context.Context, string) (string, error)
}

func DefaultClientFactory(profile config.Profile, password string) (routeros.Client, error) {
	options := routeros.ClientOptions{
		BaseURL:        profile.URL,
		Username:       profile.Username,
		Password:       password,
		CertificatePin: profile.CertificateFingerprint,
	}
	if path := strings.TrimSpace(profile.CustomCA); path != "" {
		pem, err := os.ReadFile(path)
		if err != nil {
			return nil, fmt.Errorf("read custom CA: %w", err)
		}
		pool, err := routeros.CertPoolFromPEM(pem)
		if err != nil {
			return nil, err
		}
		options.RootCAs = pool
	}
	return routeros.NewClient(options)
}
