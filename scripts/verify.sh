#!/usr/bin/env bash
# Gerbang verifikasi satu iterasi loop. Harus HIJAU sebelum agent mencentang CHECKLIST.
# Usage: scripts/verify.sh
#   UPDATE_SNAPSHOTS=1 scripts/verify.sh   # regenerasi golden snapshot (keputusan sadar)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

step() { printf '\n=== %s ===\n' "$1"; }
fail=0

step "conventions (CONVENTIONS.md)"
scripts/check_conventions.sh || { echo "FAIL: conventions"; fail=1; }

step "fmt"
cargo fmt --all --check || { echo "FAIL: fmt"; fail=1; }

step "clippy"
cargo clippy --all-targets -- -D warnings || { echo "FAIL: clippy"; fail=1; }

step "test (termasuk snapshot)"
cargo test --all || { echo "FAIL: test"; fail=1; }

if [ "$fail" -ne 0 ]; then
  echo
  echo "VERIFY: MERAH — perbaiki sebelum lanjut (lihat agent/LOOP.md langkah 4)."
  exit 1
fi

echo
echo "VERIFY: HIJAU ✓"
