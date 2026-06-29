# Glyphs / Icons

Tabel referensi glyph yang dipakai UI. Warna = default theme (lihat `06-ui.md` color scheme &
`ui/theme.rs`). Konsisten dengan emoji di `abstraction/design/06-ui.md`.

## Resource (per tier)

| Glyph | Makna | Warna |
|-------|-------|-------|
| ⚡ | Energy | Green |
| 🪨 | Minerals / Ore (iron, dst) | Green |
| 💧 | Water / Liquid | Green |
| ⚙️ | Alloys / Ingot | Blue |
| 📦 | Consumer Goods / Component | Blue |
| 💎 | Rare Crystals | Magenta |
| ⚛️ | Dark Matter / Antimatter | Magenta |
| 🔬 | Data (riset) | Cyan |
| ✦ | Warp Core / prestige | Yellow (Bold) |

## Building / Slot

| Glyph | Makna | Warna |
|-------|-------|-------|
| ⛏️ | Extractor / Mining Drill | White |
| 🌾 | (legacy farm/extractor) | White |
| 🏭 | Factory / Refinery | White |
| 🔬 | Research Lab | Cyan |
| 🚀 | Spaceship / Spaceport | Green (Bold) |
| 🛒 | Merchant | Yellow |
| [ ] | Empty slot | Dim |

## Status / Navigasi

| Glyph | Makna | Warna |
|-------|-------|-------|
| ► | Focus / selected | reverse bg |
| ◆ | Active / blinking | Blink |
| ○ | Locked / inactive | Dim |
| ● / * | Unlocked / active | White (Bold) |
| ✓ | Done / surplus | Green |
| ✗ / ! | Deficit / alert | Red (Bold) |
| │█░ | Progress bar fill/empty | per konteks |

## Map (galaxy view)

| Glyph | Makna | Warna |
|-------|-------|-------|
| ■ | Planet | White (Bold) |
| · | Star (kecil) | Cyan (Dim) |
| ★ / ✦ | Star terang | Yellow |
| ═ ─ │ | Route | Cyan (Dim) |
| ≈ | Nebula / gas | Magenta |

> Catatan portabilitas: emoji lebar 2 sel — hitung pakai display width (lihat
> [`../FORMAT.md`](../FORMAT.md)). Bila terminal target tidak mendukung emoji, theme `mono`
> menyediakan fallback ASCII (mis. `⚡`→`E`, `🚀`→`^`). Mapping fallback diimplementasi di `theme.rs`.
