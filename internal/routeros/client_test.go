package routeros

import (
	"context"
	"crypto/sha256"
	"crypto/x509"
	"encoding/hex"
	"encoding/pem"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync/atomic"
	"testing"
	"time"
)

func testServer(t *testing.T, handler http.Handler) (*httptest.Server, *x509.CertPool) {
	t.Helper()
	server := httptest.NewTLSServer(handler)
	t.Cleanup(server.Close)
	certificate := server.Certificate()
	pool := x509.NewCertPool()
	pool.AddCert(certificate)
	return server, pool
}

func newTestClient(t *testing.T, server *httptest.Server, roots *x509.CertPool, mutate func(*ClientOptions)) *RESTClient {
	t.Helper()
	options := ClientOptions{
		BaseURL:        server.URL,
		Username:       "admin",
		Password:       "correct horse battery staple",
		RootCAs:        roots,
		RetryBaseDelay: time.Millisecond,
		RetryMaxDelay:  2 * time.Millisecond,
	}
	if mutate != nil {
		mutate(&options)
	}
	client, err := NewClient(options)
	if err != nil {
		t.Fatalf("NewClient() error = %v", err)
	}
	return client
}

func TestListSuccessPreservesRawStringsAndBasicAuth(t *testing.T) {
	server, roots := testServer(t, http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
		username, password, ok := request.BasicAuth()
		if !ok || username != "admin" || password != "correct horse battery staple" {
			http.Error(writer, "unauthorized", http.StatusUnauthorized)
			return
		}
		if request.URL.EscapedPath() != EndpointInterfaces {
			t.Errorf("path = %q, want %q", request.URL.EscapedPath(), EndpointInterfaces)
		}
		writer.Header().Set("Content-Type", "application/json")
		fmt.Fprint(writer, `[{
			".id":"*1",
			"name":"ether1",
			"disabled":"false",
			"running":"true",
			"rx-byte":"00123"
		}]`)
	}))

	records, err := newTestClient(t, server, roots, nil).List(context.Background(), InterfacesResource)
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}
	if len(records) != 1 || records[0].ID != "*1" {
		t.Fatalf("records = %#v", records)
	}
	if value, ok := records[0].Raw("rx-byte"); !ok || value != "00123" {
		t.Fatalf("raw rx-byte = %q, %v", value, ok)
	}
}

func TestMalformedResponseIsDecodeError(t *testing.T) {
	server, roots := testServer(t, http.HandlerFunc(func(writer http.ResponseWriter, _ *http.Request) {
		fmt.Fprint(writer, `[{"name":42}]`)
	}))

	_, err := newTestClient(t, server, roots, nil).List(context.Background(), InterfacesResource)
	var apiErr *APIError
	if !errors.As(err, &apiErr) || apiErr.Kind != ErrorDecode {
		t.Fatalf("error = %#v, want decode APIError", err)
	}
}

func TestAuthenticationAndAPIErrorMapping(t *testing.T) {
	tests := []struct {
		status int
		kind   ErrorKind
	}{
		{http.StatusUnauthorized, ErrorAuth},
		{http.StatusForbidden, ErrorAuth},
		{http.StatusNotFound, ErrorNotFound},
		{http.StatusTooManyRequests, ErrorRateLimit},
		{http.StatusInternalServerError, ErrorServer},
		{http.StatusBadRequest, ErrorAPI},
	}
	for _, test := range tests {
		t.Run(http.StatusText(test.status), func(t *testing.T) {
			server, roots := testServer(t, http.HandlerFunc(func(writer http.ResponseWriter, _ *http.Request) {
				writer.WriteHeader(test.status)
				fmt.Fprint(writer, `{"error":`+fmt.Sprint(test.status)+`,"message":"request rejected"}`)
			}))
			_, err := newTestClient(t, server, roots, nil).List(context.Background(), InterfacesResource)
			var apiErr *APIError
			if !errors.As(err, &apiErr) {
				t.Fatalf("error = %v, want APIError", err)
			}
			if apiErr.Kind != test.kind || apiErr.StatusCode != test.status {
				t.Fatalf("APIError = %#v", apiErr)
			}
		})
	}
}

func TestRequestTimeoutAndCancellation(t *testing.T) {
	started := make(chan struct{}, 2)
	release := make(chan struct{})
	server, roots := testServer(t, http.HandlerFunc(func(writer http.ResponseWriter, _ *http.Request) {
		started <- struct{}{}
		<-release
		fmt.Fprint(writer, `[]`)
	}))
	t.Cleanup(func() { close(release) })

	timeoutClient := newTestClient(t, server, roots, func(options *ClientOptions) {
		options.RequestTimeout = 20 * time.Millisecond
	})
	_, err := timeoutClient.List(context.Background(), InterfacesResource)
	var apiErr *APIError
	if !errors.As(err, &apiErr) || apiErr.Kind != ErrorTimeout || !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("timeout error = %#v", err)
	}
	<-started

	ctx, cancel := context.WithCancel(context.Background())
	cancelClient := newTestClient(t, server, roots, nil)
	result := make(chan error, 1)
	go func() {
		_, callErr := cancelClient.List(ctx, InterfacesResource)
		result <- callErr
	}()
	<-started
	cancel()
	err = <-result
	if !errors.As(err, &apiErr) || apiErr.Kind != ErrorCanceled || !errors.Is(err, context.Canceled) {
		t.Fatalf("cancel error = %#v", err)
	}
}

func TestCustomCAAndCertificatePins(t *testing.T) {
	server, roots := testServer(t, http.HandlerFunc(func(writer http.ResponseWriter, _ *http.Request) {
		fmt.Fprint(writer, `[]`)
	}))
	cert := server.Certificate()
	pemData := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: cert.Raw})
	pemRoots, err := CertPoolFromPEM(pemData)
	if err != nil {
		t.Fatalf("CertPoolFromPEM() error = %v", err)
	}
	if _, err := newTestClient(t, server, pemRoots, nil).List(context.Background(), InterfacesResource); err != nil {
		t.Fatalf("custom CA List() error = %v", err)
	}

	pin := CertificateSHA256(cert)
	if _, err := newTestClient(t, server, nil, func(options *ClientOptions) {
		options.CertificatePin = "sha256:" + pin
	}).List(context.Background(), InterfacesResource); err != nil {
		t.Fatalf("matching pin List() error = %v", err)
	}

	wrong := sha256.Sum256([]byte("different certificate"))
	_, err = newTestClient(t, server, roots, func(options *ClientOptions) {
		options.CertificatePin = hex.EncodeToString(wrong[:])
	}).List(context.Background(), InterfacesResource)
	var apiErr *APIError
	if !errors.As(err, &apiErr) || apiErr.Kind != ErrorTLS || !errors.Is(err, ErrCertificatePinMismatch) {
		t.Fatalf("pin mismatch error = %#v", err)
	}
}

func TestProbeCertificateReturnsLeafFingerprintWithoutHTTPAuth(t *testing.T) {
	requests := atomic.Int32{}
	server, _ := testServer(t, http.HandlerFunc(func(writer http.ResponseWriter, _ *http.Request) {
		requests.Add(1)
		fmt.Fprint(writer, `[]`)
	}))

	pin, err := ProbeCertificate(context.Background(), server.URL)
	if err != nil {
		t.Fatalf("ProbeCertificate() error = %v", err)
	}
	if pin != CertificateSHA256(server.Certificate()) {
		t.Fatalf("pin = %q", pin)
	}
	if requests.Load() != 0 {
		t.Fatalf("certificate probe sent %d HTTP requests", requests.Load())
	}
}

func TestTOFUPinnerLearnsAndRejectsChangedCertificate(t *testing.T) {
	firstServer, _ := testServer(t, http.HandlerFunc(func(writer http.ResponseWriter, _ *http.Request) {
		fmt.Fprint(writer, `[]`)
	}))
	pinner, err := NewTOFUPinner("")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := newTestClient(t, firstServer, nil, func(options *ClientOptions) {
		options.TOFU = pinner
	}).List(context.Background(), InterfacesResource); err != nil {
		t.Fatalf("first TOFU request error = %v", err)
	}
	if pinner.Pin() != CertificateSHA256(firstServer.Certificate()) {
		t.Fatalf("learned pin = %q", pinner.Pin())
	}

	changedCertificate := *firstServer.Certificate()
	changedCertificate.Raw = append([]byte(nil), changedCertificate.Raw...)
	changedCertificate.Raw[0] ^= 0xff
	if err := pinner.Verify(&changedCertificate); !errors.Is(err, ErrCertificatePinMismatch) {
		t.Fatalf("changed certificate error = %v", err)
	}
}

func TestErrorsDoNotExposeSecrets(t *testing.T) {
	const password = "correct horse battery staple"
	server, roots := testServer(t, http.HandlerFunc(func(writer http.ResponseWriter, _ *http.Request) {
		writer.WriteHeader(http.StatusBadRequest)
		fmt.Fprintf(writer, `{"message":"bad password %s for admin"}`, password)
	}))
	client := newTestClient(t, server, roots, nil)
	_, err := client.List(context.Background(), InterfacesResource)
	if err == nil {
		t.Fatal("List() error = nil")
	}
	text := err.Error()
	if strings.Contains(text, password) || strings.Contains(text, "admin") {
		t.Fatalf("error exposes credentials: %q", text)
	}
}

func TestGETRetriesAndRecordPathEscaping(t *testing.T) {
	var attempts atomic.Int32
	server, roots := testServer(t, http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
		if attempts.Add(1) == 1 {
			writer.WriteHeader(http.StatusServiceUnavailable)
			return
		}
		if request.URL.EscapedPath() != "/rest/interface/%2A1%2Funsafe%20id" {
			t.Errorf("escaped path = %q", request.URL.EscapedPath())
		}
		fmt.Fprint(writer, `{".id":"*1/unsafe id","name":"ether1"}`)
	}))
	client := newTestClient(t, server, roots, func(options *ClientOptions) {
		options.MaxRetries = 1
	})
	record, err := client.Get(context.Background(), InterfacesResource, "*1/unsafe id")
	if err != nil {
		t.Fatalf("Get() error = %v", err)
	}
	if record.ID != "*1/unsafe id" || attempts.Load() != 2 {
		t.Fatalf("record = %#v, attempts = %d", record, attempts.Load())
	}
}

func TestStringNormalization(t *testing.T) {
	for _, value := range []string{"true", "YES", " on ", "1"} {
		if result, err := ParseBool(value); err != nil || !result {
			t.Errorf("ParseBool(%q) = %v, %v", value, result, err)
		}
	}
	for _, value := range []string{"false", "NO", " off ", "0"} {
		if result, err := ParseBool(value); err != nil || result {
			t.Errorf("ParseBool(%q) = %v, %v", value, result, err)
		}
	}
	if _, err := ParseBool("maybe"); err == nil {
		t.Error("ParseBool(maybe) error = nil")
	}
	if result, err := ParseInt(" 00123 "); err != nil || result != 123 {
		t.Errorf("ParseInt() = %d, %v", result, err)
	}
}

func TestCLIPathStripsRESTPrefix(t *testing.T) {
	tests := map[string]string{
		EndpointFirewallFilter: "/ip/firewall/filter",
		EndpointInterfaces:     "/interface",
		EndpointLog:            "/log",
		EndpointNTPClient:      "/system/ntp/client",
	}
	for endpoint, want := range tests {
		if got := CLIPath(endpoint); got != want {
			t.Errorf("CLIPath(%q) = %q, want %q", endpoint, got, want)
		}
	}
}
