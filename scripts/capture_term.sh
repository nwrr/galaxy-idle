#!/usr/bin/env bash
# Screenshot dari terminal SUNGGUHAN (bukan rasterizer sel) — audit warna/font asli + grafis
# sixel/kitty yang TAK muncul di scripts/screenshot.sh. Skip-aman bila lingkungan tak mendukung
# (JANGAN gagalkan scripts/verify.sh).
#
# Usage: scripts/capture_term.sh <view> <w> <h>
#
# M03.1–M03.3: deteksi lingkungan + jalankan binari asli offscreen + tangkap layar OS.
# Skip-aman M03.4; audit banding rasterizer M03.5.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Detik tunggu setelah start agar frame pertama TUI (event loop ~13fps) sempat dirender.
TUI_STARTUP_WAIT="${TUI_STARTUP_WAIT:-1.5}"

# Emulator terminal berkemampuan grafis (sixel/kitty) yang bisa dijalankan utk capture.
detect_graphics_terminal() {
  if command -v kitty >/dev/null 2>&1; then
    echo "kitty"
    return 0
  fi
  if command -v wezterm >/dev/null 2>&1; then
    echo "wezterm"
    return 0
  fi
  echo ""
  return 1
}

# Alat screenshot level-OS utk menangkap frame emulator (X11: import/scrot; Wayland: grim).
detect_screenshot_tool() {
  if command -v grim >/dev/null 2>&1 && [ -n "${WAYLAND_DISPLAY:-}" ]; then
    echo "grim"
    return 0
  fi
  if command -v import >/dev/null 2>&1 && [ -n "${DISPLAY:-}" ]; then
    echo "import"
    return 0
  fi
  if command -v scrot >/dev/null 2>&1 && [ -n "${DISPLAY:-}" ]; then
    echo "scrot"
    return 0
  fi
  echo ""
  return 1
}

# Jalankan binari TUI asli (`galaxy-idle`) di emulator `$1` (kitty|wezterm), ukuran sel
# `$2`x`$3` (kolom x baris), offscreen (background, tak butuh layar terlihat user). Cetak PID
# proses emulator ke stdout; view awal selalu `main_menu` (start state game, sama spt M03.5 audit).
launch_tui_offscreen() {
  local method="$1" w="$2" h="$3"
  case "$method" in
    kitty)
      kitty -o "initial_window_width=${w}c" -o "initial_window_height=${h}c" \
        -o remember_window_size=no --detach -- cargo run --quiet --bin galaxy-idle \
        >/dev/null 2>&1 &
      ;;
    wezterm)
      wezterm start --always-new-process --class galaxy-idle-capture \
        --config "initial_cols=${w}" --config "initial_rows=${h}" \
        -- cargo run --quiet --bin galaxy-idle >/dev/null 2>&1 &
      ;;
    *)
      return 1
      ;;
  esac
  echo $!
}

# Tangkap layar penuh (frame emulator yang baru start diasumsikan fokus/di depan) via alat `$1`
# ke path `$2`. Best-effort: tak menargetkan window spesifik (butuh xdotool/wlr-tools tambahan,
# di luar scope M03.3) — cukup utk audit visual manual (M03.5).
capture_screen() {
  local tool="$1" out="$2"
  case "$tool" in
    grim) grim "$out" ;;
    import) import -window root "$out" ;;
    scrot) scrot -o "$out" ;;
    *) return 1 ;;
  esac
}

main() {
  local view="${1:-main_menu}" w="${2:-120}" h="${3:-40}"
  local term_method shot_method
  term_method="$(detect_graphics_terminal)" || true
  shot_method="$(detect_screenshot_tool)" || true

  echo "[capture_term] terminal grafis  : ${term_method:-tidak ditemukan (kitty/wezterm)}"
  echo "[capture_term] alat screenshot  : ${shot_method:-tidak ditemukan (grim/import/scrot)}"

  if [ -z "$term_method" ] || [ -z "$shot_method" ]; then
    echo "[capture_term] SKIP — lingkungan tak mendukung capture terminal nyata di sini."
    echo "[capture_term] (ini normal di sandbox/CI headless; audit galaxy pixel M17 via mesin lain.)"
    exit 0
  fi

  # Binari real hanya punya view awal main_menu (bukan arg pilih view spt bin/screenshot.rs);
  # audit M03.5 memang cuma butuh main_menu. View lain dicatat, tak dipakai launch.
  echo "[capture_term] view diminta     : $view (binari asli mulai di main_menu; abaikan bila beda)"

  local pid
  pid="$(launch_tui_offscreen "$term_method" "$w" "$h")"
  sleep "$TUI_STARTUP_WAIT"

  if ! kill -0 "$pid" 2>/dev/null; then
    echo "[capture_term] SKIP — proses emulator ($term_method, pid $pid) gagal start/keluar dini." >&2
    exit 0
  fi
  echo "[capture_term] emulator jalan (pid $pid); menangkap layar via $shot_method..."

  local out_dir="agent/test/screens/term"
  mkdir -p "$out_dir"
  local out="$out_dir/${view}_${w}x${h}.png"

  if ! capture_screen "$shot_method" "$out" || [ ! -s "$out" ]; then
    echo "[capture_term] GAGAL menangkap layar ($shot_method) — proses emulator dibersihkan." >&2
    kill "$pid" 2>/dev/null || true
    exit 1
  fi

  kill "$pid" 2>/dev/null || true
  echo "[capture_term] tersimpan: $out"
}

main "$@"
