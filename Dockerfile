# syntax=docker/dockerfile:1
FROM rust:1.98-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p mikrotik-tui

FROM alpine:3.22
ARG VERSION=dev
LABEL org.opencontainers.image.version="${VERSION}"
RUN apk add --no-cache ca-certificates \
    && addgroup -S routerdeck \
    && adduser -S -G routerdeck routerdeck \
    && mkdir -p /data \
    && chown routerdeck:routerdeck /data
COPY --from=build /src/target/release/mikrotik-tui /usr/local/bin/mikrotik-tui
USER routerdeck
ENV XDG_CONFIG_HOME=/data/config \
    XDG_STATE_HOME=/data/state
VOLUME ["/data"]
ENTRYPOINT ["mikrotik-tui"]
