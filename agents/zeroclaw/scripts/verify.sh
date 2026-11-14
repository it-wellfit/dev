#!/usr/bin/env bash
# Verify a ZeroClaw install on Linux, macOS or WSL2.
#
# Non-interactive checks. Prints PASS / FAIL / UNKNOWN per check. Exit code is the
# number of failed checks; UNKNOWN never counts as a pass.
#
# Usage: ./verify.sh [--version 0.9.0] [--full]
set -uo pipefail

VERSION="0.9.0"
FULL=0
while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --full)    FULL=1; shift ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

FAILURES=0
pass()    { printf '[PASS]    %s :: %s\n' "$1" "$2"; }
fail()    { printf '[FAIL]    %s :: %s\n' "$1" "$2"; FAILURES=$((FAILURES + 1)); }
unknown() { printf '[UNKNOWN] %s :: %s\n' "$1" "$2"; }

CONFIG_DIR="${ZEROCLAW_CONFIG_DIR:-$HOME/.zeroclaw}"
CONFIG_PATH="$CONFIG_DIR/config.toml"
BIN_DIR="${ZEROCLAW_HOME:-$HOME/.zeroclaw}/bin"
CLI=""
if command -v zeroclaw >/dev/null 2>&1; then
  CLI="zeroclaw"
elif [ -x "$BIN_DIR/zeroclaw" ]; then
  CLI="$BIN_DIR/zeroclaw"
fi

echo "== ZeroClaw verification =="

# V1 version -----------------------------------------------------------------
if [ -z "$CLI" ]; then
  fail V1 "zeroclaw not found on PATH and not at $BIN_DIR/zeroclaw"
else
  RAW="$("$CLI" --version 2>/dev/null || true)"
  FOUND="$(printf '%s' "$RAW" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1)"
  if [ -z "$FOUND" ]; then
    fail V1 "could not parse version from '$RAW'"
  elif [ "$FOUND" != "$VERSION" ]; then
    fail V1 "expected $VERSION, found $FOUND"
  else
    pass V1 "zeroclaw $VERSION"
  fi
fi

# V2 config present ----------------------------------------------------------
if [ -f "$CONFIG_PATH" ]; then pass V2 "config at $CONFIG_PATH"; else fail V2 "missing $CONFIG_PATH"; fi

# V3 config validates --------------------------------------------------------
if [ -n "$CLI" ] && [ -f "$CONFIG_PATH" ]; then
  JSON="$("$CLI" config migrate --json 2>&1 || true)"
  case "$JSON" in
    *'"valid":true'*)  pass V3 "config validates" ;;
    *'"valid":false'*) fail V3 "config invalid: $JSON" ;;
    *)                 unknown V3 "could not read validity: $JSON" ;;
  esac
else
  unknown V3 "skipped (need CLI and config)"
fi

# V4 config hygiene: no inline secrets ---------------------------------------
if [ -f "$CONFIG_PATH" ]; then
  if grep -Eq '^[[:space:]]*api_key[[:space:]]*=[[:space:]]*"' "$CONFIG_PATH"; then
    fail V4 "inline api_key found in config.toml"
  else
    pass V4 "no inline api_key in config.toml"
  fi
else
  unknown V4 "skipped (no config)"
fi

# V5 PATH --------------------------------------------------------------------
case ":$PATH:" in
  *":$BIN_DIR:"*) pass V5 "bin on PATH: $BIN_DIR" ;;
  *)              fail V5 "bin missing from PATH: $BIN_DIR" ;;
esac

# V6 ollama reachable --------------------------------------------------------
if command -v curl >/dev/null 2>&1; then
  if TAGS="$(curl -fsS --max-time 5 http://127.0.0.1:11434/api/tags 2>/dev/null)"; then
    COUNT="$(printf '%s' "$TAGS" | grep -o '"name"' | wc -l | tr -d ' ')"
    if [ "$COUNT" -gt 0 ]; then
      pass V6 "ollama reachable, $COUNT model(s)"
      PRIMARY="$(grep -E '^[[:space:]]*model[[:space:]]*=' "$CONFIG_PATH" 2>/dev/null | head -n1 | sed -E 's/.*"(.*)".*/\1/')"
      if [ -n "$PRIMARY" ]; then
        if printf '%s' "$TAGS" | grep -q "\"$PRIMARY\""; then
          pass V7 "primary model present: $PRIMARY"
        else
          fail V7 "primary model '$PRIMARY' not found in ollama"
        fi
      else
        unknown V7 "no primary model parsed from config"
      fi
    else
      fail V6 "ollama reachable but returned no models"
      unknown V7 "skipped"
    fi
  else
    unknown V6 "ollama not reachable on 127.0.0.1:11434"
    unknown V7 "skipped (ollama unreachable)"
  fi
else
  unknown V6 "curl not available"
  unknown V7 "skipped"
fi

# V8 gateway health ----------------------------------------------------------
if command -v curl >/dev/null 2>&1; then
  CODE="$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 http://127.0.0.1:42617/health 2>/dev/null || true)"
  if [ "$CODE" = "200" ]; then
    pass V8 "gateway health OK"
  else
    unknown V8 "gateway not running (start it with: zeroclaw daemon)"
  fi
else
  unknown V8 "curl not available"
fi

# V9 optional end-to-end completion -----------------------------------------
if [ "$FULL" -eq 1 ]; then
  if [ -n "$CLI" ]; then
    ANSWER="$("$CLI" agent -a assistant -m "Reply with the single word Ready. Do not use tools." 2>&1 || true)"
    case "$ANSWER" in
      *[Rr]eady*) pass V9 "completion via configured model" ;;
      *)          fail V9 "unexpected answer: $ANSWER" ;;
    esac
  else
    unknown V9 "skipped (no CLI)"
  fi
else
  unknown V9 "skipped (use --full for the live completion test)"
fi

echo
if [ "$FAILURES" -gt 0 ]; then
  echo "RESULT: $FAILURES check(s) failed."
  exit "$FAILURES"
fi
echo "RESULT: no failures (review UNKNOWN items; UNKNOWN is not a pass)."