#!/bin/sh
# Agent hook entrypoint shared by Claude Code and Gemini CLI.
#   post-edit  format the edited Rust file; never blocks.
#   stop       run `make verify` when the tree has changes; exit 2 on failure.
# stdout stays empty: Gemini CLI parses stdout as JSON. Feedback goes to stderr.

MODE="${1:-post-edit}"
INPUT="$(cat || true)"

if ! command -v make >/dev/null 2>&1; then
  echo "agent-hook: make is not on PATH, so checks were skipped. Install GNU make (Windows: winget install ezwinports.make; macOS: xcode-select --install) and restart the agent." >&2
  exit 0
fi

case "$MODE" in
post-edit)
  FILE_PATH="${2:-}"
  if [ -z "$FILE_PATH" ]; then
    FILE_PATH="$(printf '%s' "$INPUT" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)"
  fi
  case "$FILE_PATH" in
  *.rs)
    if [ -f "$FILE_PATH" ]; then
      rustfmt --edition 2024 "$FILE_PATH" >/dev/null 2>&1 || true
    fi
    ;;
  esac
  exit 0
  ;;
stop)
  if [ "${AGENT_HOOK_SKIP:-}" = "1" ]; then
    exit 0
  fi
  if printf '%s' "$INPUT" | grep -Eq '"stop_hook_active"[[:space:]]*:[[:space:]]*true'; then
    exit 0
  fi
  if [ -z "$(git status --porcelain 2>/dev/null)" ]; then
    exit 0
  fi
  if OUTPUT="$(make verify 2>&1)"; then
    exit 0
  fi
  {
    echo "make verify failed. Fix these errors before finishing:"
    printf '%s\n' "$OUTPUT" | tail -n 40
  } >&2
  exit 2
  ;;
*)
  echo "usage: agent-hook.sh post-edit|stop" >&2
  exit 1
  ;;
esac
