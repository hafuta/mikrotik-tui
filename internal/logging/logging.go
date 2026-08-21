// Package logging creates structured, rotating application logs with
// mandatory secret redaction.
package logging

import (
	"fmt"
	"io"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

const (
	appName         = "mikrotik-tui"
	defaultFilename = "mikrotik-tui.log"
	defaultMaxBytes = 5 << 20
	defaultBackups  = 3
)

// Config controls structured file logging.
type Config struct {
	Dir        string
	Filename   string
	Level      slog.Level
	MaxBytes   int64
	MaxBackups int
}

// DefaultConfig locates logs below XDG_STATE_HOME, XDG_CACHE_HOME, or
// os.UserCacheDir, in that order.
func DefaultConfig() (Config, error) {
	dir, err := defaultLogDir()
	if err != nil {
		return Config{}, err
	}
	return Config{
		Dir:        dir,
		Filename:   defaultFilename,
		Level:      slog.LevelInfo,
		MaxBytes:   defaultMaxBytes,
		MaxBackups: defaultBackups,
	}, nil
}

// New creates a JSON slog logger and its rotating destination. The caller must
// close the returned closer before exiting.
func New(config Config) (*slog.Logger, io.Closer, error) {
	if strings.TrimSpace(config.Dir) == "" {
		return nil, nil, fmt.Errorf("log directory is required")
	}
	if config.Filename == "" {
		config.Filename = defaultFilename
	}
	if filepath.Base(config.Filename) != config.Filename {
		return nil, nil, fmt.Errorf("log filename must not contain a path")
	}
	if config.MaxBytes <= 0 {
		config.MaxBytes = defaultMaxBytes
	}
	if config.MaxBackups < 0 {
		return nil, nil, fmt.Errorf("log backup count must not be negative")
	}

	writer, err := newRotatingWriter(
		filepath.Join(config.Dir, config.Filename),
		config.MaxBytes,
		config.MaxBackups,
	)
	if err != nil {
		return nil, nil, err
	}
	base := slog.NewJSONHandler(writer, &slog.HandlerOptions{Level: config.Level})
	return slog.New(NewRedactingHandler(base)), writer, nil
}

// Default creates a logger using DefaultConfig.
func Default() (*slog.Logger, io.Closer, error) {
	config, err := DefaultConfig()
	if err != nil {
		return nil, nil, err
	}
	return New(config)
}

func defaultLogDir() (string, error) {
	if dir := strings.TrimSpace(os.Getenv("XDG_STATE_HOME")); dir != "" {
		return filepath.Join(dir, appName), nil
	}
	if dir := strings.TrimSpace(os.Getenv("XDG_CACHE_HOME")); dir != "" {
		return filepath.Join(dir, appName), nil
	}
	dir, err := os.UserCacheDir()
	if err != nil {
		return "", fmt.Errorf("locate user cache directory: %w", err)
	}
	return filepath.Join(dir, appName), nil
}

type rotatingWriter struct {
	mu         sync.Mutex
	path       string
	maxBytes   int64
	maxBackups int
	file       *os.File
	size       int64
}

func newRotatingWriter(path string, maxBytes int64, maxBackups int) (*rotatingWriter, error) {
	if err := os.MkdirAll(filepath.Dir(path), 0o700); err != nil {
		return nil, fmt.Errorf("create log directory: %w", err)
	}
	if err := os.Chmod(filepath.Dir(path), 0o700); err != nil {
		return nil, fmt.Errorf("secure log directory: %w", err)
	}
	writer := &rotatingWriter{path: path, maxBytes: maxBytes, maxBackups: maxBackups}
	if err := writer.open(); err != nil {
		return nil, err
	}
	return writer, nil
}

func (w *rotatingWriter) Write(data []byte) (int, error) {
	w.mu.Lock()
	defer w.mu.Unlock()

	if w.file == nil {
		return 0, os.ErrClosed
	}
	if w.size > 0 && w.size+int64(len(data)) > w.maxBytes {
		if err := w.rotate(); err != nil {
			return 0, err
		}
	}
	n, err := w.file.Write(data)
	w.size += int64(n)
	return n, err
}

func (w *rotatingWriter) Close() error {
	w.mu.Lock()
	defer w.mu.Unlock()
	if w.file == nil {
		return nil
	}
	err := w.file.Close()
	w.file = nil
	return err
}

func (w *rotatingWriter) open() error {
	file, err := os.OpenFile(w.path, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0o600)
	if err != nil {
		return fmt.Errorf("open log file: %w", err)
	}
	if err := file.Chmod(0o600); err != nil {
		_ = file.Close()
		return fmt.Errorf("secure log file: %w", err)
	}
	info, err := file.Stat()
	if err != nil {
		_ = file.Close()
		return fmt.Errorf("stat log file: %w", err)
	}
	w.file = file
	w.size = info.Size()
	return nil
}

func (w *rotatingWriter) rotate() error {
	if err := w.file.Close(); err != nil {
		return fmt.Errorf("close log for rotation: %w", err)
	}
	w.file = nil

	if w.maxBackups == 0 {
		if err := os.Remove(w.path); err != nil && !os.IsNotExist(err) {
			return fmt.Errorf("remove rotated log: %w", err)
		}
	} else {
		oldest := fmt.Sprintf("%s.%d", w.path, w.maxBackups)
		if err := os.Remove(oldest); err != nil && !os.IsNotExist(err) {
			return fmt.Errorf("remove oldest log: %w", err)
		}
		for index := w.maxBackups - 1; index >= 1; index-- {
			source := fmt.Sprintf("%s.%d", w.path, index)
			destination := fmt.Sprintf("%s.%d", w.path, index+1)
			if err := os.Rename(source, destination); err != nil && !os.IsNotExist(err) {
				return fmt.Errorf("rotate log backup: %w", err)
			}
		}
		if err := os.Rename(w.path, w.path+".1"); err != nil && !os.IsNotExist(err) {
			return fmt.Errorf("rotate current log: %w", err)
		}
	}
	return w.open()
}
