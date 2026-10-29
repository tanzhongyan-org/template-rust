#!/usr/bin/env bash
set -euo pipefail
# `make security`: secret scan, OSV scan and new-dependency check.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if command -v gitleaks >/dev/null 2>&1; then
  gitleaks detect --redact --no-banner --verbose
else
  echo "security: gitleaks is not installed, so the secret scan was skipped. Install it from https://github.com/gitleaks/gitleaks#installing" >&2
fi

OSV_VERSION="${OSV_SCANNER_VERSION:?OSV_SCANNER_VERSION is not set; run via 'make security'}"
case "$(uname -s)" in
  Linux*) os=linux ;;
  Darwin*) os=darwin ;;
  MINGW*|MSYS*|CYGWIN*) os=windows ;;
  *) echo "security: unsupported OS $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) arch=amd64 ;;
  arm64|aarch64) arch=arm64 ;;
  *) echo "security: unsupported architecture $(uname -m)" >&2; exit 1 ;;
esac
exe=""
if [ "$os" = "windows" ]; then exe=".exe"; fi
BIN="$ROOT/target/bin/osv-scanner-$OSV_VERSION$exe"
if [ ! -x "$BIN" ]; then
  mkdir -p "$ROOT/target/bin"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  base="https://github.com/google/osv-scanner/releases/download/v$OSV_VERSION"
  asset="osv-scanner_${os}_${arch}$exe"
  curl -sSfL -o "$tmp/$asset" "$base/$asset"
  curl -sSfL -o "$tmp/SHA256SUMS" "$base/osv-scanner_SHA256SUMS"
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$tmp" && grep "[* ]${asset}$" SHA256SUMS | sha256sum --check --strict -)
  else
    (cd "$tmp" && grep "[* ]${asset}$" SHA256SUMS | shasum -a 256 -c -)
  fi
  mv "$tmp/$asset" "$BIN"
  chmod +x "$BIN"
fi
"$BIN" scan source --config "$ROOT/osv-scanner.toml" --lockfile "$ROOT/Cargo.lock" --verbosity warn "$ROOT"

node "$ROOT/scripts/new-deps.mjs"
