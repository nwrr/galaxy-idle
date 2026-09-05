#!/usr/bin/env bash
# Render satu view TUI ke PNG berwarna (headless, Buffer→PNG) → agent/test/screens/.
# Agent membaca PNG ini langsung (Claude vision) untuk menilai desain — bukan lagi .txt.
#
# Usage: scripts/screenshot.sh <view> <w> <h> [theme]
#   view : main_menu | planet_view | research | galaxy_map | warp
#   w,h  : ukuran terminal (kolom x baris)
#   theme: default | high_contrast | mono (opsional, default: default)
#
# Bergantung pada binari `screenshot` (src/bin/screenshot.rs) — dibuat di milestone M02.
set -euo pipefail

VIEW="${1:?usage: scripts/screenshot.sh <view> <w> <h> [theme]}"
W="${2:?usage: scripts/screenshot.sh <view> <w> <h> [theme]}"
H="${3:?usage: scripts/screenshot.sh <view> <w> <h> [theme]}"
THEME="${4:-}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUT_DIR="agent/test/screens"
mkdir -p "$OUT_DIR"

if [ -n "$THEME" ]; then
  OUT="$OUT_DIR/${VIEW}_${W}x${H}_${THEME}.png"
  ARGS=("$VIEW" "$W" "$H" "$THEME" "$OUT")
else
  OUT="$OUT_DIR/${VIEW}_${W}x${H}.png"
  ARGS=("$VIEW" "$W" "$H" "$OUT")
fi

if ! cargo run --quiet --bin screenshot -- "${ARGS[@]}"; then
  echo "[screenshot] gagal render '$VIEW' ${W}x${H} ${THEME:-default}." >&2
  exit 1
fi

echo "$OUT"
