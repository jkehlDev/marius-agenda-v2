#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# Default outside repo — screenshots are not versioned (see .gitignore).
OUT="${1:-/tmp/marius-ui-screenshots}"
mkdir -p "$OUT"
# Captures use GSK widget snapshot (Wayland-safe). scrot is optional fallback only.
cd "$ROOT"
if command -v xvfb-run >/dev/null 2>&1 && [[ -z "${DISPLAY:-}" ]]; then
  xvfb-run -a cargo run -p marius-agenda --features gtk --quiet -- --root "$ROOT" --gui-screenshot-tour "$OUT"
else
  cargo run -p marius-agenda --features gtk --quiet -- --root "$ROOT" --gui-screenshot-tour "$OUT"
fi
echo "Captures demandées dans $OUT"
