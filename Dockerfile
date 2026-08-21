# syntax=docker/dockerfile:1
FROM golang:1.25-alpine AS build
ARG VERSION=dev
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -trimpath \
    -ldflags="-s -w -X main.version=${VERSION}" \
    -o /out/mikrotik-tui ./cmd/mikrotik-tui

FROM alpine:3.22
RUN apk add --no-cache ca-certificates \
    && addgroup -S routerdeck \
    && adduser -S -G routerdeck routerdeck \
    && mkdir -p /data \
    && chown routerdeck:routerdeck /data
COPY --from=build /out/mikrotik-tui /usr/local/bin/mikrotik-tui
USER routerdeck
ENV XDG_CONFIG_HOME=/data/config \
    XDG_STATE_HOME=/data/state
VOLUME ["/data"]
ENTRYPOINT ["mikrotik-tui"]
