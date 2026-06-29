#!/usr/bin/env bash
# Render satu view TUI ke teks (headless, deterministik) lalu cetak ke stdout.
# Agent membaca output ini langsung untuk "melihat" terminal.
#
# Usage: scripts/snapshot.sh <view> [w] [h]
#   view: main_menu | planet_view | galaxy_map | research | warp
#   w,h : ukuran terminal (default 120x40)
#
# Bergantung pada binari `snapshot` (src/bin/snapshot.rs) — dibuat di milestone M3.
set -euo pipefail

VIEW="${1:-main_menu}"
W="${2:-120}"
H="${3:-40}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! cargo run --quiet --bin snapshot -- "$VIEW" "$W" "$H" 2>/dev/null; then
  echo "[snapshot] binari 'snapshot' belum ada / gagal build." >&2
  echo "[snapshot] Buat src/bin/snapshot.rs dulu (lihat agent/test/README.md, milestone M3)." >&2
  exit 1
fi
