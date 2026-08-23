#!/bin/sh
# Install or upgrade mikrotik-tui from the latest GitHub Release.
# Linux amd64 / arm64 only. Copied to the Pages site at build time.
#
#   curl -fsSL https://raw.githubusercontent.com/hafuta/mikrotik-tui/master/scripts/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/hafuta/mikrotik-tui/master/scripts/install.sh | sh -s -- --yes
set -eu

REPO="${MIKROTIK_TUI_INSTALL_REPO:-hafuta/mikrotik-tui}"
BIN="mikrotik-tui"
RELEASES="https://github.com/${REPO}/releases"

YES=0
FORCE=0
PREFIX=""

usage() {
  cat <<EOF
Install the latest ${BIN} Linux binary from GitHub Releases.

Usage: install.sh [--yes] [--force] [--prefix DIR]

  --yes       Replace an existing install without prompting
  --force     Replace even when the installed version matches latest
  --prefix    Install directory (default: ~/.local/bin, or /usr/local/bin as root)
  -h, --help  Show this help

Architectures: x86_64/amd64, aarch64/arm64.
EOF
}

log() { printf '%s\n' "$*" >&2; }
die() { log "error: $*"; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "need '$1' on PATH"
}

http_get() {
  url=$1
  dest=$2
  if command -v curl >/dev/null 2>&1; then
    if [ -n "${GITHUB_TOKEN:-}" ]; then
      curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN}" -o "$dest" "$url"
    else
      curl -fsSL -o "$dest" "$url"
    fi
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$dest" "$url"
  else
    die "need curl or wget"
  fi
}

file_sha256() {
  f=$1
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$f" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$f" | awk '{print $1}'
  else
    die "need sha256sum or shasum"
  fi
}

bin_version() {
  # clap: "mikrotik-tui 0.1.5"
  out=$("$1" --version 2>/dev/null) || { printf 'unknown'; return 0; }
  set -- $out
  while [ "$#" -gt 0 ]; do
    case $1 in
      v*[0-9]*|[0-9]*.*)
        printf '%s' "${1#v}"
        return 0
        ;;
    esac
    shift
  done
  printf 'unknown'
}

confirm_replace() {
  msg=$1
  if [ "$YES" -eq 1 ]; then
    return 0
  fi
  if [ ! -c /dev/tty ]; then
    die "${msg} Pass --yes to replace without a prompt (stdin is not a terminal)."
  fi
  printf '%s [y/N] ' "$msg" >/dev/tty
  reply=
  read -r reply </dev/tty || true
  case $reply in
    y|Y|yes|YES) return 0 ;;
    *) log "aborted."; exit 0 ;;
  esac
}

while [ "$#" -gt 0 ]; do
  case $1 in
    --yes|-y) YES=1 ;;
    --force) FORCE=1 ;;
    --prefix)
      shift
      [ "$#" -gt 0 ] || die "--prefix needs a directory"
      PREFIX=$1
      ;;
    --prefix=*) PREFIX=${1#--prefix=} ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
  shift
done

os=$(uname -s)
[ "$os" = Linux ] || die "this installer is for Linux only (got ${os}). Download a release from ${RELEASES}"

arch=$(uname -m)
case $arch in
  x86_64|amd64) asset_arch=amd64 ;;
  aarch64|arm64) asset_arch=arm64 ;;
  *) die "unsupported architecture '${arch}'. Need x86_64 or aarch64." ;;
esac

if [ -z "$PREFIX" ]; then
  if [ "$(id -u)" -eq 0 ]; then
    PREFIX=/usr/local/bin
  else
    PREFIX="${HOME}/.local/bin"
  fi
fi

asset="${BIN}-linux-${asset_arch}.tar.gz"
dest="${PREFIX}/${BIN}"
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

log "downloading latest ${asset} from ${REPO}…"
http_get "${RELEASES}/latest/download/checksums.txt" "${tmpdir}/checksums.txt"
http_get "${RELEASES}/latest/download/${asset}" "${tmpdir}/${asset}"

expected=$(awk -v f="$asset" '$2 == f || $2 == ("*" f) {print $1; exit}' "${tmpdir}/checksums.txt")
[ -n "$expected" ] || die "no checksum for ${asset} in checksums.txt"
got=$(file_sha256 "${tmpdir}/${asset}")
[ "$got" = "$expected" ] || die "checksum mismatch for ${asset} (got ${got}, expected ${expected})"

need_cmd tar
tar -xzf "${tmpdir}/${asset}" -C "$tmpdir"
tmpbin="${tmpdir}/${BIN}"
[ -f "$tmpbin" ] || die "archive did not contain ${BIN}"
chmod 755 "$tmpbin"
new_ver=$(bin_version "$tmpbin")

existing=""
existing_ver="none"
if [ -e "$dest" ]; then
  existing=$dest
  existing_ver=$(bin_version "$dest")
elif command -v "$BIN" >/dev/null 2>&1; then
  existing=$(command -v "$BIN")
  existing_ver=$(bin_version "$existing")
fi

if [ -n "$existing" ]; then
  if [ "$existing_ver" = "$new_ver" ] && [ "$new_ver" != unknown ] && [ "$FORCE" -eq 0 ]; then
    log "${BIN} ${new_ver} is already installed at ${existing}"
    exit 0
  fi
  confirm_replace "${BIN} ${existing_ver} is installed at ${existing}. Replace with ${new_ver}?"
fi

mkdir -p "$PREFIX"
cp "$tmpbin" "${dest}.tmp"
chmod 755 "${dest}.tmp"
mv -f "${dest}.tmp" "$dest"

log "installed ${BIN} ${new_ver} to ${dest}"
case :$PATH: in
  *:"$PREFIX":*) ;;
  *) log "note: ${PREFIX} is not on PATH; add it or move the binary" ;;
esac
log "run: ${BIN} --version"
