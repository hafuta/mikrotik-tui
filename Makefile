APP := mikrotik-tui
VERSION ?= dev
LDFLAGS := -s -w -X main.version=$(VERSION)

.PHONY: build test race vet lint check run clean release

build:
	go build -ldflags "$(LDFLAGS)" -o bin/$(APP) ./cmd/$(APP)

run:
	go run ./cmd/$(APP)

test:
	go test ./...

race:
	go test -race ./...

vet:
	go vet ./...

lint:
	staticcheck ./...

check: vet race

release:
	GOOS=linux GOARCH=amd64 go build -trimpath -ldflags "$(LDFLAGS)" -o dist/$(APP)-linux-amd64 ./cmd/$(APP)
	GOOS=linux GOARCH=arm64 go build -trimpath -ldflags "$(LDFLAGS)" -o dist/$(APP)-linux-arm64 ./cmd/$(APP)

clean:
	rm -rf bin dist coverage.out
