package routeros

import (
	"context"
	"crypto/rand"
	"crypto/tls"
	"crypto/x509"
	"encoding/binary"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"strings"
	"time"
)

const maxResponseBytes = 8 << 20

const (
	defaultRequestTimeout = 30 * time.Second
	defaultMaxRetries     = 2
)

type ErrorKind string

const (
	ErrorCanceled  ErrorKind = "canceled"
	ErrorTimeout   ErrorKind = "timeout"
	ErrorTransport ErrorKind = "transport"
	ErrorTLS       ErrorKind = "tls"
	ErrorAuth      ErrorKind = "authentication"
	ErrorNotFound  ErrorKind = "not_found"
	ErrorRateLimit ErrorKind = "rate_limited"
	ErrorServer    ErrorKind = "server"
	ErrorAPI       ErrorKind = "api"
	ErrorDecode    ErrorKind = "decode"
)

// APIError is the normalized error returned by the client.
type APIError struct {
	Kind       ErrorKind
	StatusCode int
	APICode    string
	Message    string
	Operation  string
	err        error
}

func (e *APIError) Error() string {
	parts := []string{"routeros"}
	if e.Operation != "" {
		parts = append(parts, e.Operation)
	}
	parts = append(parts, string(e.Kind))
	if e.StatusCode != 0 {
		parts = append(parts, fmt.Sprintf("HTTP %d", e.StatusCode))
	}
	if e.Message != "" {
		parts = append(parts, e.Message)
	}
	return strings.Join(parts, ": ")
}

func (e *APIError) Unwrap() error { return e.err }

// Client is independent of application profiles and configuration storage.
type Client interface {
	List(context.Context, ResourceDescriptor) ([]Resource, error)
	Get(context.Context, ResourceDescriptor, string) (Resource, error)
	System(context.Context, SystemResource) (Resource, error)
}

type ClientOptions struct {
	BaseURL        string
	Username       string
	Password       string
	RequestTimeout time.Duration
	MaxRetries     int
	DisableRetries bool
	RetryBaseDelay time.Duration
	RetryMaxDelay  time.Duration
	RootCAs        *x509.CertPool
	CertificatePin string
	TOFU           *TOFUPinner
	HTTPClient     *http.Client
}

type RESTClient struct {
	baseURL        string
	username       string
	password       string
	requestTimeout time.Duration
	maxRetries     int
	retryBase      time.Duration
	retryMax       time.Duration
	httpClient     *http.Client
}

var _ Client = (*RESTClient)(nil)

func NewClient(options ClientOptions) (*RESTClient, error) {
	base, err := url.Parse(strings.TrimRight(options.BaseURL, "/"))
	if err != nil || base.Scheme != "https" || base.Host == "" {
		return nil, errors.New("routeros: base URL must be a valid HTTPS URL")
	}
	if base.User != nil || base.RawQuery != "" || base.Fragment != "" {
		return nil, errors.New("routeros: base URL must not contain credentials, query, or fragment")
	}
	if options.MaxRetries < 0 {
		return nil, errors.New("routeros: maximum retries cannot be negative")
	}
	if options.RequestTimeout < 0 {
		return nil, errors.New("routeros: request timeout cannot be negative")
	}

	httpClient := options.HTTPClient
	if httpClient == nil {
		httpClient = &http.Client{}
	} else {
		clone := *httpClient
		httpClient = &clone
	}

	var transport *http.Transport
	switch existing := httpClient.Transport.(type) {
	case nil:
		defaultTransport, ok := http.DefaultTransport.(*http.Transport)
		if !ok {
			return nil, errors.New("routeros: default HTTP transport is unavailable")
		}
		transport = defaultTransport.Clone()
	case *http.Transport:
		transport = existing.Clone()
	default:
		return nil, errors.New("routeros: HTTP client must use an *http.Transport")
	}
	tlsConfig := &tls.Config{
		MinVersion: tls.VersionTLS12,
		RootCAs:    options.RootCAs,
		ServerName: base.Hostname(),
	}
	if options.CertificatePin != "" {
		if _, err := NormalizeCertificatePin(options.CertificatePin); err != nil {
			return nil, err
		}
	}
	if options.CertificatePin != "" || options.TOFU != nil {
		// An explicitly approved pin is the complete identity decision. This
		// supports RouterOS self-signed certificates that omit an IP SAN.
		tlsConfig.InsecureSkipVerify = true //nolint:gosec
		tlsConfig.VerifyConnection = func(state tls.ConnectionState) error {
			if len(state.PeerCertificates) == 0 {
				return errors.New("routeros: TLS peer sent no certificate")
			}
			leaf := state.PeerCertificates[0]
			if options.CertificatePin != "" {
				if err := VerifyCertificatePin(leaf, options.CertificatePin); err != nil {
					return err
				}
			}
			if options.TOFU != nil {
				return options.TOFU.Verify(leaf)
			}
			return nil
		}
	}
	transport.TLSClientConfig = tlsConfig
	httpClient.Transport = transport

	retryBase := options.RetryBaseDelay
	if retryBase <= 0 {
		retryBase = 100 * time.Millisecond
	}
	retryMax := options.RetryMaxDelay
	if retryMax <= 0 {
		retryMax = 2 * time.Second
	}
	if retryMax < retryBase {
		retryMax = retryBase
	}
	requestTimeout := options.RequestTimeout
	if requestTimeout == 0 {
		requestTimeout = defaultRequestTimeout
	}
	maxRetries := options.MaxRetries
	if maxRetries == 0 && !options.DisableRetries {
		maxRetries = defaultMaxRetries
	}

	return &RESTClient{
		baseURL:        strings.TrimRight(options.BaseURL, "/"),
		username:       options.Username,
		password:       options.Password,
		requestTimeout: requestTimeout,
		maxRetries:     maxRetries,
		retryBase:      retryBase,
		retryMax:       retryMax,
		httpClient:     httpClient,
	}, nil
}

func (c *RESTClient) List(ctx context.Context, resource ResourceDescriptor) ([]Resource, error) {
	var records []Resource
	if err := c.getJSON(ctx, resource.Endpoint, &records); err != nil {
		return nil, err
	}
	if records == nil {
		records = []Resource{}
	}
	return records, nil
}

func (c *RESTClient) Get(ctx context.Context, resource ResourceDescriptor, id string) (Resource, error) {
	var record Resource
	err := c.getJSON(ctx, ResourceRecordEndpoint(resource, id), &record)
	return record, err
}

func (c *RESTClient) System(ctx context.Context, resource SystemResource) (Resource, error) {
	var record Resource
	err := c.getJSON(ctx, resource.Endpoint, &record)
	return record, err
}

func (c *RESTClient) getJSON(parent context.Context, endpoint string, destination any) error {
	if !strings.HasPrefix(endpoint, "/rest/") && endpoint != "/rest" {
		return &APIError{Kind: ErrorAPI, Operation: "GET", Message: "invalid REST endpoint"}
	}
	ctx := parent
	cancel := func() {}
	if c.requestTimeout > 0 {
		ctx, cancel = context.WithTimeout(parent, c.requestTimeout)
	}
	defer cancel()

	for attempt := 0; ; attempt++ {
		request, err := http.NewRequestWithContext(ctx, http.MethodGet, c.baseURL+endpoint, nil)
		if err != nil {
			return c.normalizeError("GET", err)
		}
		request.Header.Set("Accept", "application/json")
		request.SetBasicAuth(c.username, c.password)

		response, err := c.httpClient.Do(request)
		if err != nil {
			normalized := c.normalizeError("GET", err)
			if attempt < c.maxRetries && retryableTransport(normalized) {
				if err := c.waitRetry(ctx, attempt); err != nil {
					return c.normalizeError("GET", err)
				}
				continue
			}
			return normalized
		}

		if response.StatusCode < 200 || response.StatusCode >= 300 {
			apiErr := c.readAPIError(response)
			if attempt < c.maxRetries && retryableStatus(response.StatusCode) {
				if err := c.waitRetry(ctx, attempt); err != nil {
					return c.normalizeError("GET", err)
				}
				continue
			}
			return apiErr
		}

		err = decodeResponse(response.Body, destination)
		response.Body.Close()
		if err != nil {
			return &APIError{Kind: ErrorDecode, Operation: "GET", Message: "invalid API response", err: err}
		}
		return nil
	}
}

func decodeResponse(body io.Reader, destination any) error {
	decoder := json.NewDecoder(io.LimitReader(body, maxResponseBytes+1))
	if err := decoder.Decode(destination); err != nil {
		return err
	}
	var extra json.RawMessage
	if err := decoder.Decode(&extra); err != io.EOF {
		if err == nil {
			return errors.New("multiple JSON values in response")
		}
		return err
	}
	return nil
}

type wireAPIError struct {
	Error   json.RawMessage `json:"error"`
	Message string          `json:"message"`
	Detail  string          `json:"detail"`
}

func (c *RESTClient) readAPIError(response *http.Response) error {
	defer response.Body.Close()
	data, readErr := io.ReadAll(io.LimitReader(response.Body, maxResponseBytes))
	var payload wireAPIError
	_ = json.Unmarshal(data, &payload)

	message := strings.TrimSpace(payload.Message)
	if message == "" {
		message = strings.TrimSpace(payload.Detail)
	}
	if message == "" {
		message = http.StatusText(response.StatusCode)
	}
	message = c.redact(message)

	code := ""
	if len(payload.Error) > 0 {
		var text string
		if json.Unmarshal(payload.Error, &text) == nil {
			code = text
		} else {
			var number json.Number
			if json.Unmarshal(payload.Error, &number) == nil {
				code = number.String()
			}
		}
	}
	apiErr := &APIError{
		Kind:       kindForStatus(response.StatusCode),
		StatusCode: response.StatusCode,
		APICode:    code,
		Message:    message,
		Operation:  "GET",
	}
	if readErr != nil {
		apiErr.err = readErr
	}
	return apiErr
}

func (c *RESTClient) normalizeError(operation string, err error) error {
	kind := ErrorTransport
	switch {
	case errors.Is(err, context.Canceled):
		kind = ErrorCanceled
	case errors.Is(err, context.DeadlineExceeded):
		kind = ErrorTimeout
	case errors.Is(err, ErrCertificatePinMismatch):
		kind = ErrorTLS
	default:
		var certificateError x509.UnknownAuthorityError
		var hostnameError x509.HostnameError
		var invalidCertificate x509.CertificateInvalidError
		var verificationError *tls.CertificateVerificationError
		var networkError net.Error
		if errors.As(err, &certificateError) ||
			errors.As(err, &hostnameError) ||
			errors.As(err, &invalidCertificate) ||
			errors.As(err, &verificationError) {
			kind = ErrorTLS
		} else if errors.As(err, &networkError) && networkError.Timeout() {
			kind = ErrorTimeout
		}
	}
	return &APIError{Kind: kind, Operation: operation, Message: c.redact(err.Error()), err: err}
}

func (c *RESTClient) redact(message string) string {
	for _, secret := range []string{c.password, c.username} {
		if secret != "" {
			message = strings.ReplaceAll(message, secret, "[redacted]")
		}
	}
	return message
}

func kindForStatus(status int) ErrorKind {
	switch {
	case status == http.StatusUnauthorized || status == http.StatusForbidden:
		return ErrorAuth
	case status == http.StatusNotFound:
		return ErrorNotFound
	case status == http.StatusTooManyRequests:
		return ErrorRateLimit
	case status >= 500:
		return ErrorServer
	default:
		return ErrorAPI
	}
}

func retryableStatus(status int) bool {
	return status == http.StatusTooManyRequests ||
		status == http.StatusBadGateway ||
		status == http.StatusServiceUnavailable ||
		status == http.StatusGatewayTimeout
}

func retryableTransport(err error) bool {
	var apiErr *APIError
	return errors.As(err, &apiErr) && apiErr.Kind == ErrorTransport
}

func (c *RESTClient) waitRetry(ctx context.Context, attempt int) error {
	limit := c.retryBase
	for i := 0; i < attempt && limit < c.retryMax/2; i++ {
		limit *= 2
	}
	if limit > c.retryMax {
		limit = c.retryMax
	}
	var random [8]byte
	if _, err := rand.Read(random[:]); err != nil {
		return err
	}
	delay := time.Duration(binary.LittleEndian.Uint64(random[:]) % uint64(limit+1))
	timer := time.NewTimer(delay)
	defer timer.Stop()
	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-timer.C:
		return nil
	}
}
