#!/usr/bin/env bash
# Set [workspace.package].version in Cargo.toml.
# Release builds call this with the tag minus the leading v (v0.2.1 -> 0.2.1).
set -euo pipefail

usage() {
  echo "usage: $0 <semver>" >&2
  echo "   or: $0 --self-test" >&2
  exit 2
}

set_version() {
  local file=$1
  local version=$2
  local tmp
  valid_version "$version" || {
    echo "invalid version: ${version}" >&2
    return 1
  }
  tmp=$(mktemp)
  awk -v ver="$version" '
    $0 == "[workspace.package]" { in_pkg = 1; print; next }
    /^\[/ { in_pkg = 0 }
    in_pkg && $1 == "version" && !done {
      print "version = \"" ver "\""
      done = 1
      next
    }
    { print }
    END {
      if (!done) {
        print "workspace.package.version not found" > "/dev/stderr"
        exit 1
      }
    }
  ' "$file" > "$tmp"
  mv "$tmp" "$file"
}

valid_version() {
  [[ $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+([.+-][0-9A-Za-z.-]*)?$ ]]
}

if [ "${1:-}" = "--self-test" ]; then
  dir=$(mktemp -d)
  trap 'rm -rf "$dir"' EXIT
  cat > "$dir/Cargo.toml" <<'EOF'
[workspace]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.98"

[workspace.dependencies]
clap = { version = "4" }
EOF
  set_version "$dir/Cargo.toml" "0.2.1"
  grep -qx 'version = "0.2.1"' "$dir/Cargo.toml"
  grep -qx 'edition = "2024"' "$dir/Cargo.toml"
  grep -q 'clap = { version = "4" }' "$dir/Cargo.toml"
  grep -qx 'rust-version = "1.98"' "$dir/Cargo.toml"
  if set_version "$dir/Cargo.toml" "not-a-version" 2>/dev/null; then
    echo "accepted invalid version" >&2
    exit 1
  fi
  echo "ok"
  exit 0
fi

[ $# -eq 1 ] || usage
version=$1
valid_version "$version" || {
  echo "invalid version: ${version}" >&2
  exit 1
}

root=$(cd "$(dirname "$0")/.." && pwd)
set_version "$root/Cargo.toml" "$version"
echo "set workspace.package.version to ${version}"
