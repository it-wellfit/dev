#!/usr/bin/env bash
# Install ZeroClaw on Linux, macOS or WSL2 from the pinned GitHub release.
#
# Verifies the release checksum before installing, then renders the portable
# config template. No administrator rights are required.
#
# Usage:
#   ./install.sh [--version 0.9.0] [--primary-model qwen3.5:4b] \
#                [--fallback-model gemini-3.8-flash] [--force] [--skip-config]
set -euo pipefail

VERSION="0.9.0"
PRIMARY_MODEL="qwen3.5:4b"
FALLBACK_MODEL="gemini-3.8-flash"
FORCE=0
SKIP_CONFIG=0

while [ $# -gt 0 ]; do
  case "$1" in
    --version)         VERSION="$2"; shift 2 ;;
    --primary-model)   PRIMARY_MODEL="$2"; shift 2 ;;
    --fallback-model)  FALLBACK_MODEL="$2"; shift 2 ;;
    --force)           FORCE=1; shift ;;
    --skip-config)     SKIP_CONFIG=1; shift ;;
    -h|--help)
      sed -n '2,12p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

say()  { printf '==> %s\n' "$*"; }
ok()   { printf '    [OK]   %s\n' "$*"; }
warn() { printf '    [WARN] %s\n' "$*" >&2; }
die()  { printf '    [FAIL] %s\n' "$*" >&2; exit 1; }

# --- target -----------------------------------------------------------------
ARCH="$(uname -m)"
OS="$(uname -s)"
case "$OS:$ARCH" in
  Linux:x86_64)   ASSET="zeroclaw-x86_64-unknown-linux-gnu.tar.gz" ;;
  Linux:aarch64)  ASSET="zeroclaw-aarch64-unknown-linux-gnu.tar.gz" ;;
  Darwin:arm64)   ASSET="zeroclaw-x86_64-apple-darwin.tar.gz" ;;  # no arm64 darwin asset published; see README
  Darwin:x86_64)  ASSET="zeroclaw-x86_64-apple-darwin.tar.gz" ;;
  *) die "unsupported platform $OS/$ARCH (see README for the supported matrix)" ;;
esac

INSTALL_ROOT="${ZEROCLAW_HOME:-$HOME/.zeroclaw}"
BIN_DIR="$INSTALL_ROOT/bin"
CONFIG_DIR="${ZEROCLAW_CONFIG_DIR:-$INSTALL_ROOT}"
CONFIG_PATH="$CONFIG_DIR/config.toml"
TEMPLATE="$(cd "$(dirname "$0")/.." && pwd)/config/zeroclaw.toml.tmpl"
BASE_URL="https://github.com/zeroclaw-labs/zeroclaw/releases/download/v$VERSION"

say "ZeroClaw installer ($OS/$ARCH, pinned v$VERSION)"

# --- already installed? -----------------------------------------------------
INSTALLED=""
if [ -x "$BIN_DIR/zeroclaw" ]; then
  INSTALLED="$("$BIN_DIR/zeroclaw" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1 || true)"
fi

if [ "$INSTALLED" = "$VERSION" ] && [ "$FORCE" -eq 0 ]; then
  ok "zeroclaw $VERSION already installed at $BIN_DIR/zeroclaw"
else
  TMP_DIR="$(mktemp -d)"
  trap 'rm -rf "$TMP_DIR"' EXIT

  say "Downloading $ASSET"
  curl -fsSL -o "$TMP_DIR/$ASSET" "$BASE_URL/$ASSET" || die "download failed"
  curl -fsSL -o "$TMP_DIR/SHA256SUMS" "$BASE_URL/SHA256SUMS" || die "checksum file download failed"

  say "Verifying SHA-256"
  if command -v sha256sum >/dev/null 2>&1; then
    ( cd "$TMP_DIR" && grep " $ASSET\$" SHA256SUMS | sha256sum -c - ) \
      || die "checksum verification failed for $ASSET"
  elif command -v shasum >/dev/null 2>&1; then
    EXPECTED="$(awk -v a="$ASSET" '$2 == a || $2 == "*"a {print $1}' "$TMP_DIR/SHA256SUMS")"
    [ -n "$EXPECTED" ] || die "no checksum entry for $ASSET"
    ACTUAL="$(shasum -a 256 "$TMP_DIR/$ASSET" | awk '{print $1}')"
    [ "$EXPECTED" = "$ACTUAL" ] || die "checksum mismatch: expected=$EXPECTED actual=$ACTUAL"
  else
    die "neither sha256sum nor shasum is available"
  fi
  ok "archive digest matches the release SHA256SUMS"

  say "Extracting"
  mkdir -p "$BIN_DIR"
  tar -xzf "$TMP_DIR/$ASSET" -C "$BIN_DIR"
  [ -x "$BIN_DIR/zeroclaw" ] || die "zeroclaw not found after extraction in $BIN_DIR"
  chmod +x "$BIN_DIR/zeroclaw" 2>/dev/null || true
  ok "installed to $BIN_DIR/zeroclaw"
fi

# --- PATH -------------------------------------------------------------------
PROFILE_FILE="$HOME/.profile"
if ! grep -qs "$BIN_DIR" "$PROFILE_FILE" 2>/dev/null; then
  printf '\n# ZeroClaw\nexport PATH="%s:$PATH"\n' "$BIN_DIR" >> "$PROFILE_FILE"
  ok "added $BIN_DIR to PATH in $PROFILE_FILE (open a new shell)"
else
  ok "$BIN_DIR already referenced in $PROFILE_FILE"
fi

# --- render config ----------------------------------------------------------
if [ "$SKIP_CONFIG" -eq 1 ]; then
  warn "skipped config rendering (--skip-config)"
else
  if [ -f "$CONFIG_PATH" ] && [ "$FORCE" -eq 0 ]; then
    warn "config exists and was left untouched: $CONFIG_PATH (use --force to re-render)"
  else
    [ -f "$TEMPLATE" ] || die "template not found: $TEMPLATE"
    say "Rendering config"
    mkdir -p "$CONFIG_DIR"

    WORKSPACE="$CONFIG_DIR/workspace"
    REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
    FORBIDDEN="$REPO_ROOT
$HOME/Documents
$HOME/Desktop
$HOME/Downloads
$HOME/.ssh
$HOME/.aws"

    TOML_LIST="$(printf '%s\n' "$FORBIDDEN" | while IFS= read -r p; do
      [ -n "$p" ] && [ -e "$p" ] && printf '    "%s",\n' "$p"
    done)"

    TMP_CFG="$(mktemp)"
    sed -e "s|__WORKSPACE__|$WORKSPACE|g" \
        -e "s|__PRIMARY_MODEL__|$PRIMARY_MODEL|g" \
        -e "s|__FALLBACK_MODEL__|$FALLBACK_MODEL|g" \
        "$TEMPLATE" > "$TMP_CFG"

    # Replace the __FORBIDDEN__ marker line with the generated TOML list.
    awk -v list="$TOML_LIST" '
      /__FORBIDDEN__/ { print list; next }
      { print }
    ' "$TMP_CFG" > "$CONFIG_PATH"
    rm -f "$TMP_CFG"
    ok "wrote $CONFIG_PATH"
  fi
fi

cat <<EOF

Installed. Next steps (run in a NEW shell):
    zeroclaw --version
    zeroclaw quickstart                # provider / risk / memory / channels
    zeroclaw agent -a assistant -m "Reply with the single word Ready. Do not use tools."
    zeroclaw daemon                    # gateway + dashboard on 127.0.0.1:42617

Set the Gemini key without writing it to disk:
    export ZEROCLAW_providers__models__gemini__free__api_key='<key>'

Then run: ./verify.sh
EOF