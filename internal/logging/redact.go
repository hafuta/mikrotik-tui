package logging

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"regexp"
	"strings"
)

const redacted = "[REDACTED]"

var (
	authorizationPattern = regexp.MustCompile(`(?i)\b(?:basic|bearer)\s+[A-Za-z0-9._~+/=-]+`)
	urlUserInfoPattern   = regexp.MustCompile(`(?i)\b([a-z][a-z0-9+.-]*://)[^/@\s]+@`)
	sensitiveKVPattern   = regexp.MustCompile(`(?i)\b(password|passwd|pwd|secret|token|api[-_]?key|authorization)\b["']?\s*[:=]\s*(?:"[^"]*"|'[^']*'|[^\s,;}\]]+)`)
)

// Redact removes common secret representations from free-form text.
func Redact(value string) string {
	value = authorizationPattern.ReplaceAllString(value, redacted)
	value = urlUserInfoPattern.ReplaceAllString(value, `${1}`+redacted+"@")
	value = sensitiveKVPattern.ReplaceAllStringFunc(value, func(match string) string {
		separator := strings.IndexAny(match, ":=")
		if separator < 0 {
			return redacted
		}
		return match[:separator+1] + redacted
	})
	return value
}

// NewRedactingHandler wraps a slog handler and sanitizes messages, values, and
// nested groups before they reach the destination.
func NewRedactingHandler(next slog.Handler) slog.Handler {
	return &redactingHandler{next: next}
}

type redactingHandler struct {
	next slog.Handler
}

func (h *redactingHandler) Enabled(ctx context.Context, level slog.Level) bool {
	return h.next.Enabled(ctx, level)
}

func (h *redactingHandler) Handle(ctx context.Context, record slog.Record) error {
	clean := slog.NewRecord(record.Time, record.Level, Redact(record.Message), record.PC)
	record.Attrs(func(attribute slog.Attr) bool {
		clean.AddAttrs(redactAttr(attribute))
		return true
	})
	return h.next.Handle(ctx, clean)
}

func (h *redactingHandler) WithAttrs(attributes []slog.Attr) slog.Handler {
	clean := make([]slog.Attr, len(attributes))
	for index, attribute := range attributes {
		clean[index] = redactAttr(attribute)
	}
	return &redactingHandler{next: h.next.WithAttrs(clean)}
}

func (h *redactingHandler) WithGroup(name string) slog.Handler {
	return &redactingHandler{next: h.next.WithGroup(name)}
}

func redactAttr(attribute slog.Attr) slog.Attr {
	attribute.Value = attribute.Value.Resolve()
	if sensitiveKey(attribute.Key) {
		return slog.String(attribute.Key, redacted)
	}

	switch attribute.Value.Kind() {
	case slog.KindString:
		return slog.String(attribute.Key, Redact(attribute.Value.String()))
	case slog.KindGroup:
		group := attribute.Value.Group()
		clean := make([]slog.Attr, len(group))
		for index, child := range group {
			clean[index] = redactAttr(child)
		}
		return slog.Group(attribute.Key, attrsToAny(clean)...)
	case slog.KindAny:
		return slog.String(attribute.Key, redactAny(attribute.Value.Any()))
	default:
		return attribute
	}
}

func redactAny(value any) string {
	if err, ok := value.(error); ok {
		return Redact(err.Error())
	}
	encoded, err := json.Marshal(value)
	if err == nil {
		return Redact(string(encoded))
	}
	return Redact(fmt.Sprint(value))
}

func attrsToAny(attributes []slog.Attr) []any {
	values := make([]any, len(attributes))
	for index, attribute := range attributes {
		values[index] = attribute
	}
	return values
}

func sensitiveKey(key string) bool {
	normalized := strings.Map(func(r rune) rune {
		switch r {
		case '-', '_', '.', ' ':
			return -1
		default:
			return r
		}
	}, strings.ToLower(key))
	for _, fragment := range []string{
		"password", "passwd", "pwd", "secret", "token", "authorization",
		"apikey", "privatekey", "credential", "cookie", "session",
	} {
		if strings.Contains(normalized, fragment) {
			return true
		}
	}
	return false
}
