package routeros

import (
	"context"
	"crypto/sha256"
	"crypto/tls"
	"crypto/x509"
	"encoding/hex"
	"errors"
	"fmt"
	"net"
	"net/url"
	"strings"
	"sync"
	"time"
)

var ErrCertificatePinMismatch = errors.New("routeros: certificate pin mismatch")

// ProbeCertificate retrieves a router's leaf certificate fingerprint without
// sending credentials. The returned fingerprint is untrusted and must be
// approved by the user before it is passed to NewClient.
func ProbeCertificate(ctx context.Context, baseURL string) (string, error) {
	parsed, err := url.Parse(strings.TrimRight(baseURL, "/"))
	if err != nil || parsed.Scheme != "https" || parsed.Host == "" {
		return "", errors.New("routeros: certificate probe requires a valid HTTPS URL")
	}
	address := parsed.Host
	if parsed.Port() == "" {
		address = net.JoinHostPort(parsed.Hostname(), "443")
	}
	dialer := &tls.Dialer{
		NetDialer: &net.Dialer{Timeout: 10 * time.Second},
		Config: &tls.Config{
			MinVersion: tls.VersionTLS12,
			ServerName: parsed.Hostname(),
			// This probe only reads the public certificate. No credentials or
			// application data are sent until the fingerprint is approved.
			InsecureSkipVerify: true, //nolint:gosec
		},
	}
	connection, err := dialer.DialContext(ctx, "tcp", address)
	if err != nil {
		return "", fmt.Errorf("routeros: probe certificate: %w", err)
	}
	defer connection.Close()
	state := connection.(*tls.Conn).ConnectionState()
	if len(state.PeerCertificates) == 0 {
		return "", errors.New("routeros: TLS peer sent no certificate")
	}
	return CertificateSHA256(state.PeerCertificates[0]), nil
}

// CertificateSHA256 returns the lowercase SHA-256 fingerprint of a DER
// certificate.
func CertificateSHA256(cert *x509.Certificate) string {
	sum := sha256.Sum256(cert.Raw)
	return hex.EncodeToString(sum[:])
}

// NormalizeCertificatePin validates a SHA-256 fingerprint and accepts common
// colon-delimited and "sha256:" display forms.
func NormalizeCertificatePin(pin string) (string, error) {
	normalized := strings.ToLower(strings.TrimSpace(pin))
	normalized = strings.TrimPrefix(normalized, "sha256:")
	normalized = strings.ReplaceAll(normalized, ":", "")
	decoded, err := hex.DecodeString(normalized)
	if err != nil || len(decoded) != sha256.Size {
		return "", fmt.Errorf("routeros: invalid SHA-256 certificate pin")
	}
	return hex.EncodeToString(decoded), nil
}

// VerifyCertificatePin verifies the leaf certificate against a fingerprint.
func VerifyCertificatePin(cert *x509.Certificate, pin string) error {
	expected, err := NormalizeCertificatePin(pin)
	if err != nil {
		return err
	}
	if CertificateSHA256(cert) != expected {
		return ErrCertificatePinMismatch
	}
	return nil
}

// CertPoolFromPEM constructs a CA pool from PEM certificates.
func CertPoolFromPEM(pemData []byte) (*x509.CertPool, error) {
	pool := x509.NewCertPool()
	if !pool.AppendCertsFromPEM(pemData) {
		return nil, errors.New("routeros: custom CA contains no valid certificates")
	}
	return pool, nil
}

// TOFUPinner implements trust-on-first-use pinning. Persist Pin() after the
// first successful connection and pass it to NewTOFUPinner on later runs.
type TOFUPinner struct {
	mu  sync.RWMutex
	pin string
}

func NewTOFUPinner(savedPin string) (*TOFUPinner, error) {
	pinner := &TOFUPinner{}
	if strings.TrimSpace(savedPin) == "" {
		return pinner, nil
	}
	pin, err := NormalizeCertificatePin(savedPin)
	if err != nil {
		return nil, err
	}
	pinner.pin = pin
	return pinner, nil
}

func (p *TOFUPinner) Pin() string {
	p.mu.RLock()
	defer p.mu.RUnlock()
	return p.pin
}

func (p *TOFUPinner) Verify(cert *x509.Certificate) error {
	actual := CertificateSHA256(cert)
	p.mu.Lock()
	defer p.mu.Unlock()
	if p.pin == "" {
		p.pin = actual
		return nil
	}
	if p.pin != actual {
		return ErrCertificatePinMismatch
	}
	return nil
}
