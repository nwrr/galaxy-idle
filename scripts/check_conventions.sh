#!/usr/bin/env bash
# Penegak CONVENTIONS.md: (1) ≤10 file/folder, (2) baris kode ≤100 kolom.
# Usage: scripts/check_conventions.sh   (exit !=0 bila ada pelanggaran)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
fail=0

# (1) maksimal 10 file per folder (abaikan .git/target/__pycache__)
while IFS= read -r d; do
  n=$(find "$d" -maxdepth 1 -type f | wc -l)
  if [ "$n" -gt 10 ]; then
    echo "FAIL §1: $n file di '$d' (maks 10) — pakai sub-folder."
    fail=1
  fi
done < <(find . -type d \
  -not -path './.git/*' -not -path './target/*' -not -path '*/__pycache__*')

# (2) panjang baris kode ≤100 kolom (Rust/Python)
while IFS= read -r f; do
  if grep -nE '.{101,}' "$f" >/dev/null 2>&1; then
    echo "FAIL §2: baris >100 kolom di $f:"
    grep -nE '.{101,}' "$f" | sed 's/^/    /'
    fail=1
  fi
done < <(find . \( -name '*.rs' -o -name '*.py' \) \
  -not -path './.git/*' -not -path './target/*' -not -path '*/__pycache__*')

if [ "$fail" -ne 0 ]; then
  echo; echo "CONVENTIONS: MERAH — lihat CONVENTIONS.md."
  exit 1
fi
echo "CONVENTIONS: HIJAU ✓"
