# GOALS — Phase 2: UI/UX Overhaul

> Prasyarat: **Phase 1 ✅ SELESAI** (mekanik DoD G1–G10, lihat [`../phase1/GOALS.md`](../phase1/GOALS.md)).
> Phase 2 = loop **improvement tampilan & pengalaman** — bukan mekanik baru. Ratusan milestone kecil
> terverifikasi-screenshot. Spec visual: [`../../abstraction/design/`](../../abstraction/design/)
> (`06`,`07`,`14`,`15`). North-star desain: **Riftborne** + `galaxy_2.png` di
> [`../../assets/source/references/`](../../assets/source/references/).

## Ide (satu kalimat)

Ubah galaxy-idle dari "view kosong + spiral kelap-kelip" → **TUI ANSI padat & playable ala Riftborne**:
asset (sprite/portrait/galaxy) benar-benar tampil, layout informatif & responsif, galaxy semirip &
se-playable mungkin — diverifikasi **screenshot langsung**, di-improve berulang sampai layak.

## Keputusan Arsitektur (terkonfirmasi user)

1. **Target = Hybrid TUI + pixel galaxy.** Final tetap terminal ANSI (Riftborne); galaxy view boleh
   pixel asli **sixel/kitty** via `ratatui-image` saat didukung, fallback ANSI half-block. (Mengganti
   non-goal lama "no sixel/kitty" **khusus galaxy**.)
2. **Verifikasi = screenshot langsung (bukan .txt).** Rasterizer **Buffer→PNG** (utama, deterministik)
   + **capture terminal nyata** (audit + galaxy pixel). Agent Read PNG → nilai → iterasi.
3. **Asset dari ComfyUI, gerbang kemiripan.** Prompt dipisah dari workflow. `verify.py` skor (SSIM+phash,
   CLIP opsional) **≥90%** ke referensi → regen seed++ bila kurang; agent Read PNG konfirmasi final.
4. **Asset → ANSI** detail, skor fidelitas **≥99%**, **responsive** (sm/md/lg × tc/256/16).
5. **Galaxy hybrid:** backdrop procedural particle spiral (anim, resize penuh) + starmap grid playable
   (Riftborne) + body sprite ComfyUI→ANSI + galaxy pixel sixel saat didukung.

## Goal Akhir (Definition of Done — Phase 2 SELESAI bila SEMUA terpenuhi)

- [ ] **U-A** Harness screenshot jalan: `scripts/screenshot.sh <view> <w> <h>` → PNG berwarna benar
      (glyph+box+half-block, fg/bg truecolor) yang bisa dibaca agent; `capture_term.sh` audit jalan.
- [ ] **U-B** Asset wired ke runtime: sprite celestial tampil di view (planet/galaxy/warp), portrait
      karakter tampil di konteks nyata (merchant/judul). Tidak ada lagi modul asset yang nganggur.
- [ ] **U-C** Pipeline ComfyUI: prompt ⟂ workflow terpisah, `generate.py --set <x>` multi-asset,
      `verify.py` gate ≥90% + regen; report tersimpan; agent konfirmasi vision.
- [ ] **U-D** Pipeline ANSI: `png_to_ansi` hasil 9 varian/base, skor fidelitas ≥99%, `check_assets.py`
      PASS; semua asset tampil tajam & responsif.
- [ ] **U-E** Shell Riftborne: layout 3-kolom persisten (RESOURCES/status · MAIN · OPTIONS · MARKET
      footer · top bar) di semua view, padat-informasi, responsif 3 breakpoint.
- [ ] **U-F** Tiap view (planet, research, galaxy, warp, merchant, events/log, settings) playable &
      enak dilihat; navigasi keybinding jelas; tak ada overflow/clipping di 3 breakpoint.
- [ ] **U-G** **Galaxy** (krusial): backdrop procedural mirip `galaxy_2.png`; starmap grid playable
      (kursor/scroll/select/detail/legend/minimap); pixel sixel saat didukung. Disetujui via screenshot.
- [ ] **U-H** Theme Default/HighContrast/Mono konsisten & terbaca; depth tc/256/16 fallback benar.
- [ ] **U-I** `scripts/verify.sh` hijau (fmt+clippy -D+test+snapshot) + golden snapshot di-update sadar;
      `../test/visual_checks.md` diperluas (assertion warna/asset) & lulus.
- [ ] **U-J** Tiap view × 3 breakpoint punya screenshot referensi di `agent/test/screens/` yang
      **disetujui** (cocok target `VISUAL_TARGETS.md`).

## Milestone (28 milestone, 6 kategori — 1 file/milestone di `milestones/`)

Detail item granular: tiap file [`milestones/<kat>/M<NN>-*.md`](milestones/README.md). Indeks status:
[`CHECKLIST.md`](CHECKLIST.md). Urutan eksekusi A → B → C → D → E → F.

| Kat | Milestone | DoD |
|-----|-----------|-----|
| **A** Foundation | M01 Rasterizer · M02 Screenshot bin · M03 Capture terminal · M04 Wire asset | U-A,U-B |
| **B** Asset Pipeline | M05 ComfyUI restructure · M06 Prompt sets · M07 verify.py gate · M08 Generate+verify · M09 PNG→ANSI · M10 Fidelitas ≥99 · M11 Konversi+manifest | U-C,U-D |
| **C** Shell & Core | M12 Shell skeleton · M13 Shell panels · M14 Planet/Colony · M15 Research | U-E,U-B,U-F |
| **D** Galaxy ★ | M16 Backdrop · M17 Pixel sixel · M18 Starmap grid · M19 Starmap detail · M20 Integrasi | U-G |
| **E** Secondary | M21 Warp/Travel · M22 Merchant · M23 Events/Log | U-F,U-B |
| **F** Polish/Verify | M24 Theme · M25 Particles/banner · M26 Responsive · M27 Visual checks · M28 Sign-off | U-H,U-F,U-I,U-J |

**28 milestone · ~280 item.** Galaxy (D) = 5 milestone (terbanyak, paling krusial).

## Non-Goals (jangan dikerjakan loop tanpa permintaan)

- Mekanik gameplay baru (resource/recipe/tech/prestige/balancing) — domain spec, fase berikut.
- Sixel/kitty di luar galaxy view (view lain tetap ANSI/sel murni).
- Multiplayer/networking, combat.
- Mengubah `abstraction/` (spec) atau `agent/phase1/` (arsip) kecuali diminta manusia.
