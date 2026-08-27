#!/bin/sh
# Install or upgrade routeros-tui from the latest GitHub Release.
# Linux amd64 / arm64 only. Do not run this on macOS or Windows.
#
#   curl -fsSL https://raw.githubusercontent.com/hafuta/routeros-tui/master/scripts/install-linux.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/hafuta/routeros-tui/master/scripts/install-linux.sh | sh -s -- --yes
set -eu

REPO="${ROUTEROS_TUI_INSTALL_REPO:-hafuta/routeros-tui}"
BIN="routeros-tui"
LEGACY_BIN="mikrotik-tui"
RELEASES="https://github.com/${REPO}/releases"

YES=0
FORCE=0
PREFIX="${ROUTEROS_TUI_INSTALL_PREFIX:-}"

usage() {
  cat <<EOF
Install the latest ${BIN} Linux binary from GitHub Releases.

Usage: install-linux.sh [--yes] [--force] [--prefix DIR]

  --yes       Use the default location and replace an existing install
              without prompting
  --force     Replace even when the installed version matches latest
  --prefix    Install directory (skips location prompt)
  -h, --help  Show this help

Default location is a user-owned directory (\$HOME/.local/bin, then \$HOME/bin)
so the install does not need root. System paths such as /usr/local/bin are
offered last, and only when they are writable.

When stdin is a terminal, the script lists those locations and asks which
to use.

Architectures: x86_64/amd64, aarch64/arm64.
EOF
}

log() { printf '%s\n' "$*" >&2; }
die() { log "error: $*"; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "need '$1' on PATH"
}

have_tty() {
  [ -c /dev/tty ]
}

path_has() {
  case :$PATH: in
    *:"$1":*) return 0 ;;
    *) return 1 ;;
  esac
}

dir_uid() {
  stat -c '%u' "$1" 2>/dev/null || stat -f '%u' "$1" 2>/dev/null || printf ''
}

is_user_owned() {
  _path=$1
  _uid=$(id -u)
  _probe=$_path
  while [ ! -e "$_probe" ]; do
    _next=$(dirname "$_probe")
    [ "$_next" = "$_probe" ] && return 1
    _probe=$_next
  done
  [ "$(dir_uid "$_probe")" = "$_uid" ]
}

under_home() {
  [ -n "${HOME:-}" ] || return 1
  case $1 in
    "$HOME"|"$HOME"/*) return 0 ;;
    *) return 1 ;;
  esac
}

# Root owns /usr/local/bin too; that still needs a privileged tree. Treat
# only $HOME (and other non-system dirs the user owns) as user locations.
is_system_dir() {
  case $1 in
    /usr|/usr/*|/bin|/sbin|/opt|/opt/*) return 0 ;;
    *) return 1 ;;
  esac
}

is_user_dir() {
  under_home "$1" && return 0
  is_system_dir "$1" && return 1
  is_user_owned "$1"
}

# http_get must not assign `dest`: that name is the install path, and POSIX
# sh functions share globals (this used to install into the temp tarball).
http_get() {
  _get_url=$1
  _get_out=$2
  if command -v curl >/dev/null 2>&1; then
    if [ -n "${GITHUB_TOKEN:-}" ]; then
      curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN}" -o "$_get_out" "$_get_url"
    else
      curl -fsSL -o "$_get_out" "$_get_url"
    fi
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$_get_out" "$_get_url"
  else
    die "need curl or wget"
  fi
}

file_sha256() {
  _hash_file=$1
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$_hash_file" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$_hash_file" | awk '{print $1}'
  else
    die "need sha256sum or shasum"
  fi
}

bin_version() {
  # clap: "routeros-tui 0.1.5"
  _ver_out=$("$1" --version 2>/dev/null) || { printf 'unknown'; return 0; }
  set -- $_ver_out
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

normalize_dir() {
  _dir=$1
  case $_dir in
    ~) _dir=$HOME ;;
    ~/*) _dir=${HOME}/${_dir#~/} ;;
  esac
  case $_dir in
    /) printf '%s' / ;;
    */) printf '%s' "${_dir%/}" ;;
    *) printf '%s' "$_dir" ;;
  esac
}

can_write_prefix() {
  _dir=$1
  _probe=$_dir
  while [ ! -e "$_probe" ]; do
    _next=$(dirname "$_probe")
    [ "$_next" = "$_probe" ] && return 1
    _probe=$_next
  done
  [ -d "$_probe" ] && [ -w "$_probe" ]
}

skip_auto_prefix() {
  case $1 in
    /tmp|/tmp/*|/var/tmp|/var/tmp/*|.) return 0 ;;
    /bin|/sbin|/usr/sbin|/usr/local/sbin) return 0 ;;
    "") return 0 ;;
  esac
  return 1
}

add_prefix_candidate() {
  _dir=$(normalize_dir "$1")
  [ -n "$_dir" ] || return 0
  skip_auto_prefix "$_dir" && return 0
  can_write_prefix "$_dir" || return 0
  if [ -f "$CAND_FILE" ]; then
    grep -Fxq "$_dir" "$CAND_FILE" && return 0
  fi
  printf '%s\n' "$_dir" >> "$CAND_FILE"
}

existing_bin_dir() {
  if command -v "$BIN" >/dev/null 2>&1; then
    dirname "$(command -v "$BIN")"
  elif command -v "$LEGACY_BIN" >/dev/null 2>&1; then
    dirname "$(command -v "$LEGACY_BIN")"
  fi
}

collect_prefix_candidates() {
  : > "$CAND_FILE"
  _existing=$(existing_bin_dir)

  if [ -n "$_existing" ] && is_user_dir "$_existing"; then
    add_prefix_candidate "$_existing"
  fi
  if [ -n "${HOME:-}" ]; then
    add_prefix_candidate "${HOME}/.local/bin"
    add_prefix_candidate "${HOME}/bin"
  fi
  _pathscan=$PATH
  while [ -n "$_pathscan" ]; do
    _pathdir=${_pathscan%%:*}
    if [ "$_pathdir" = "$_pathscan" ]; then
      _pathscan=
    else
      _pathscan=${_pathscan#*:}
    fi
    if is_user_dir "$_pathdir"; then
      add_prefix_candidate "$_pathdir"
    fi
  done
  if [ -n "$_existing" ]; then
    add_prefix_candidate "$_existing"
  fi
  add_prefix_candidate /usr/local/bin
  add_prefix_candidate /usr/bin
}

prefix_note() {
  if path_has "$1"; then
    printf 'on PATH'
  else
    printf 'not on PATH'
  fi
  if is_user_dir "$1"; then
    printf ', user-owned'
  else
    printf ', system'
  fi
  if [ ! -d "$1" ]; then
    printf ', will be created'
  fi
}

read_tty() {
  _prompt=$1
  _reply=
  printf '%s' "$_prompt" >/dev/tty
  read -r _reply </dev/tty || true
  printf '%s' "$_reply"
}

pick_prefix() {
  if [ -n "$PREFIX" ]; then
    PREFIX=$(normalize_dir "$PREFIX")
    [ -n "$PREFIX" ] || die "--prefix needs a directory"
    can_write_prefix "$PREFIX" || die "cannot write to ${PREFIX}"
    return 0
  fi

  collect_prefix_candidates
  if [ ! -s "$CAND_FILE" ]; then
    die "no writable install directory found. Pass --prefix DIR"
  fi

  _default=$(head -n 1 "$CAND_FILE")
  _count=$(grep -c . "$CAND_FILE")

  if [ "$YES" -eq 1 ] || ! have_tty; then
    PREFIX=$_default
    return 0
  fi

  log "Install ${BIN} ${new_ver} where?"
  log "User directories are listed first so root is not required."
  _i=1
  while IFS= read -r _dir; do
    _mark=
    [ "$_i" -eq 1 ] && _mark=' [default]'
    log "  ${_i}) ${_dir}  ($(prefix_note "$_dir"))${_mark}"
    _i=$((_i + 1))
  done < "$CAND_FILE"
  log "  ${_i}) custom path"
  _custom_n=$_i

  _choice=$(read_tty "Choice [1]: ")
  if [ -z "$_choice" ]; then
    PREFIX=$_default
  elif printf '%s' "$_choice" | grep -Eq '^[0-9]+$'; then
    if [ "$_choice" -eq "$_custom_n" ]; then
      PREFIX=
    elif [ "$_choice" -ge 1 ] && [ "$_choice" -le "$_count" ]; then
      PREFIX=$(sed -n "${_choice}p" "$CAND_FILE")
    else
      die "invalid choice: ${_choice}"
    fi
  else
    PREFIX=$(normalize_dir "$_choice")
  fi

  if [ -z "$PREFIX" ]; then
    PREFIX=$(normalize_dir "$(read_tty "Directory: ")")
  fi
  [ -n "$PREFIX" ] || die "install directory is empty"
  can_write_prefix "$PREFIX" || die "cannot write to ${PREFIX}"
}

confirm_replace() {
  _msg=$1
  if [ "$YES" -eq 1 ]; then
    return 0
  fi
  if ! have_tty; then
    die "${_msg} Pass --yes to replace without a prompt (stdin is not a terminal)."
  fi
  _reply=$(read_tty "${_msg} [y/N] ")
  case $_reply in
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
[ "$os" = Linux ] || die "install-linux.sh is for Linux only (got ${os}). Download a release from ${RELEASES}"

arch=$(uname -m)
case $arch in
  x86_64|amd64) asset_arch=amd64 ;;
  aarch64|arm64) asset_arch=arm64 ;;
  *) die "unsupported architecture '${arch}'. Need x86_64 or aarch64." ;;
esac

tmpdir=$(mktemp -d)
CAND_FILE="${tmpdir}/prefixes"
trap 'rm -rf "$tmpdir"' EXIT

asset="${BIN}-linux-${asset_arch}.tar.gz"

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
if [ "$new_ver" = unknown ]; then
  _run_err=$("$tmpbin" --version 2>&1) || true
  if [ -n "$_run_err" ]; then
    die "downloaded ${BIN} did not run: ${_run_err}"
  fi
  die "downloaded ${BIN} did not report a version"
fi

pick_prefix
dest="${PREFIX}/${BIN}"

if [ -e "$dest" ]; then
  existing_ver="unknown"
  if [ -f "$dest" ] && [ -x "$dest" ]; then
    existing_ver=$(bin_version "$dest")
  fi
  if [ "$existing_ver" = "$new_ver" ] && [ "$FORCE" -eq 0 ]; then
    log "${BIN} ${new_ver} is already installed at ${dest}"
    exit 0
  fi
  confirm_replace "${BIN} ${existing_ver} is installed at ${dest}. Replace with ${new_ver}?"
fi

if command -v "$BIN" >/dev/null 2>&1; then
  _onpath=$(command -v "$BIN")
  if [ "$_onpath" != "$dest" ]; then
    log "note: another ${BIN} is on PATH at ${_onpath}"
  fi
elif command -v "$LEGACY_BIN" >/dev/null 2>&1; then
  log "note: ${LEGACY_BIN} is still on PATH; the command is now ${BIN}"
fi

log "installing ${BIN} ${new_ver} to ${dest} ($(prefix_note "$PREFIX"))"

mkdir -p "$PREFIX"
cp "$tmpbin" "${dest}.tmp"
chmod 755 "${dest}.tmp"
mv -f "${dest}.tmp" "$dest"

log "installed ${BIN} ${new_ver} to ${dest}"
if path_has "$PREFIX"; then
  log "run: ${BIN} --version"
else
  log "note: ${PREFIX} is not on PATH. Add it with:"
  log "  export PATH=\"${PREFIX}:\$PATH\""
  log "or run: ${dest} --version"
fi
