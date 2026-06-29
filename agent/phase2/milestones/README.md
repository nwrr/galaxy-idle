# Milestones — Phase 2 (28 milestone, 6 kategori)

Tiap milestone = 1 file. Loop kerjakan urut A → F (lihat `depends_on` tiap file). Format file:
header (id/kategori/status/depends/gerbang/DoD) + Tujuan + File disentuh + Checklist + Selesai bila +
Verifikasi + Referensi. Status di-update agent + cermin di [`../state.json`](../state.json).

Gerbang: **V**=verify.sh · **S**=screenshot+nilai (`VISUAL_TARGETS.md`) · **T**=capture_term audit ·
**A**=verify.py/check_assets · **U**=unit test.

## A — Foundation / Harness (`A-foundation/`)
| ID | File | Fokus | Gerbang | Status |
|----|------|-------|---------|--------|
| M01 | [rasterizer](A-foundation/M01-rasterizer.md) | Buffer→PNG: font, color, glyph/box/half-block | V,U | ⬜ |
| M02 | [screenshot-bin](A-foundation/M02-screenshot-bin.md) | `bin/screenshot.rs` + `screenshot.sh` + baseline | V,S | ⬜ |
| M03 | [capture-terminal](A-foundation/M03-capture-terminal.md) | `capture_term.sh` audit + galaxy pixel | T | ⬜ |
| M04 | [wire-assets](A-foundation/M04-wire-assets.md) | Sprites/Portraits ke `App` + audit baseline | V,U,S | ⬜ |

## B — Asset Pipeline (`B-asset-pipeline/`)
| ID | File | Fokus | Gerbang | Status |
|----|------|-------|---------|--------|
| M05 | [comfyui-restructure](B-asset-pipeline/M05-comfyui-restructure.md) | prompt⟂workflow, `--set` | A | ⬜ |
| M06 | [comfyui-prompts](B-asset-pipeline/M06-comfyui-prompts.md) | set celestial/star/ship/galaxy/banner | A | ⬜ |
| M07 | [verify-similarity](B-asset-pipeline/M07-verify-similarity.md) | `verify.py` ≥90 + regen seed++ | A | ⬜ |
| M08 | [generate-verify-all](B-asset-pipeline/M08-generate-verify-all.md) | generate+regen semua aset ≥90 | S,A | ⬜ |
| M09 | [png-to-ansi](B-asset-pipeline/M09-png-to-ansi.md) | konverter PNG→ANSI, dither, depth | U,A | ⬜ |
| M10 | [ansi-fidelity](B-asset-pipeline/M10-ansi-fidelity.md) | skor ≥99 + 9 varian responsive | A,S | ⬜ |
| M11 | [convert-all](B-asset-pipeline/M11-convert-all.md) | konversi semua + check_assets + manifest | A,S | ⬜ |

## C — Shell & Core Views (`C-shell-views/`)
| ID | File | Fokus | Gerbang | Status |
|----|------|-------|---------|--------|
| M12 | [shell-skeleton](C-shell-views/M12-shell-skeleton.md) | 3-kolom + status bar + market footer | V,S | ⬜ |
| M13 | [shell-panels](C-shell-views/M13-shell-panels.md) | panel padat + responsif + golden | V,S | ⬜ |
| M14 | [planet-colony](C-shell-views/M14-planet-colony.md) | sprite body + list + aksi | V,S,U | ⬜ |
| M15 | [research](C-shell-views/M15-research.md) | tree + progress + detail | V,S | ⬜ |

## D — Galaxy (krusial) (`D-galaxy/`)
| ID | File | Fokus | Gerbang | Status |
|----|------|-------|---------|--------|
| M16 | [backdrop-procedural](D-galaxy/M16-backdrop-procedural.md) | spiral partikel ≈`galaxy_2.png` | S,U | ⬜ |
| M17 | [galaxy-pixel](D-galaxy/M17-galaxy-pixel.md) | sixel/kitty + fallback | T,U | ⬜ |
| M18 | [starmap-grid](D-galaxy/M18-starmap-grid.md) | grid tile + kursor + scroll | S,U | ⬜ |
| M19 | [starmap-detail](D-galaxy/M19-starmap-detail.md) | detail/legend/minimap/aksi | S,U | ⬜ |
| M20 | [galaxy-integration](D-galaxy/M20-galaxy-integration.md) | mode + responsif + sign-off | V,S,T | ⬜ |

## E — Secondary Views (`E-secondary-views/`)
| ID | File | Fokus | Gerbang | Status |
|----|------|-------|---------|--------|
| M21 | [warp-travel](E-secondary-views/M21-warp-travel.md) | rute, progress, ship, exhaust | V,S,U | ⬜ |
| M22 | [merchant](E-secondary-views/M22-merchant.md) | portrait + offer + timer | V,S,U | ⬜ |
| M23 | [events-log](E-secondary-views/M23-events-log.md) | queue event, log, badge | V,S,U | ⬜ |

## F — Polish & Verification (`F-polish-verify/`)
| ID | File | Fokus | Gerbang | Status |
|----|------|-------|---------|--------|
| M24 | [theme-depth](F-polish-verify/M24-theme-depth.md) | Default/HC/Mono + depth fallback | S,U | ⬜ |
| M25 | [particles-banner](F-polish-verify/M25-particles-banner.md) | exhaust/warp trail + banner | S | ⬜ |
| M26 | [responsive](F-polish-verify/M26-responsive.md) | tiap view × 3 breakpoint | S,T | ⬜ |
| M27 | [visual-checks-golden](F-polish-verify/M27-visual-checks-golden.md) | visual_checks + golden final | V,S | ⬜ |
| M28 | [signoff](F-polish-verify/M28-signoff.md) | screenshot disetujui + DoD gate | S,T,A | ⬜ |

Legenda status: ⬜ todo · 🟦 wip · ✅ done · ⛔ blocker.
