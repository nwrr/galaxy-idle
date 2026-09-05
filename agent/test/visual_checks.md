# Visual Checks — Assertion Semantik per View

Kriteria objektif "apa yang HARUS tampak". Agent mengecek ini terhadap output snapshot. Berbeda dari
golden snapshot (exact match): ini cek **makna/struktur**, lebih tahan terhadap perubahan kosmetik
kecil. Tiap iterasi UI: snapshot lulus golden **dan** lulus checklist ini.

## Umum (semua view, ukuran Full ≥100×30)

- [ ] Baris atas memuat judul `STELLAR IDLE`.
- [ ] Border luar utuh (karakter box-drawing `┌ ┐ └ ┘ │ ─` di tepi, tak ada baris terpotong).
- [ ] Tidak ada baris melebihi lebar terminal (no overflow / wrap tak sengaja).
- [ ] Footer/sidebar memuat shortcut (mis. `g`, `p`, `r`, `m`, `?`).
- [ ] Tidak ada placeholder `{...}` mentah dari template frame yang belum terisi.

## main_menu (120×40)

- [ ] Panel `RESOURCES` menampilkan ≥3 kelompok tier (BASIC / ADVANCED / RARE) dengan angka.
- [ ] Sidebar `MENU` memuat entri: Galaxy/Map, Planets, Fleet, Research, Merchant, Settings, Save.
- [ ] MAIN VIEW menampilkan nama galaksi aktif (mis. `Milky Way` / `Andromeda`).
- [ ] Penanda fokus (`►`) ada pada tepat satu entri menu.
- [ ] Blok SHORTCUTS memuat ≥4 keybinding.

## planet_view (80×30, Compact)

- [x] Header memuat nama planet + biome + level + jarak (mis. `EARTH - Terran (Lvl 1) - d=1.0`
      — `Sector 001` lama literal HARDCODE dibuang M14.2, ganti data REAL biome+distance).
- [x] Terlihat ringkasan RESOURCE NODES + FACTORIES (selalu, "list tetap") + SHIP/RESEARCH
      (kondisional — disembunyikan di area genuinely sempit, M14.18/19, list inti tak pernah).
- [x] Baris aksi kontekstual memuat opsi sesuai selection (`[j/k]Pilih [1]Node [2]Upgrade`,
      +`[3]Build` bila slot kosong terpilih — M14.7/12, bukan `[1]…[5]` statis lama).
- [x] Status ship terlihat (`Status: Idle` atau `Status: Traveling (Ns left)`).
- [x] Angka richness/level RATA-KOLOM antar baris NODES/FACTORIES (nama di-pad ke max lebar
      seksi, ref Riftborne `fmVy4y.png`/`9weFWl.png` — M14.21).
- [x] Aksi build/upgrade genuinely BERFUNGSI (bukan cuma tampil) — `upgrade_node`/`upgrade_
      factory` (tombol `1`/`2`) dikonfirmasi unit test `upgrade_feedback_visible_immediately_
      in_next_render`+`node_upgrade_feedback_visible_immediately_in_next_render` (level naik
      genuinely terlihat render berikutnya, level lama genuinely hilang). Build (tombol `3`,
      slot kosong) dikonfirmasi `build_picker_confirms_and_builds` (M14.12, factory baru
      genuinely terpasang di slot via `actions::build_picked`).
- [x] **Sign-off M14 planet_view (2026-07-03, iter 123)**: dinilai LENGKAP di 3 breakpoint
      (120×40 Full/80×30 Compact/60×24 Minimal) + 2 boundary tersempit yg pernah py overflow
      finding (70×24 Compact-min, 45×20 Minimal-sempit) — semua kriteria di atas terpenuhi,
      tak ada overflow, sprite+list+aksi semua genuinely berfungsi. 22/22 item M14 selesai.

## research (Full)

- [x] Header memuat laju Data (`Data/sec`) + total.
- [x] Tree tech dikelompokkan per branch (Extraction/Manufacturing/Aerospace/AstroCartography,
      M15.1) — `AVAILABLE TECHS` flat lama diganti (tech locked/done TETAP tampil, bukan cuma
      available).
- [x] 4 status node berbeda warna/marker: `[--]`Locked(dim)/`[OK]`Available(advanced)/
      `[>>]`Active(good)/`[DN]`Done(rare) — M15.2.
- [x] Bila ada riset aktif: progress bar `[####....] NN%` + Data invested/cost + waktu
      elapsed/total MENTAH terpisah (M15.3, bukan cuma pct blended).
- [x] Detail tech terpilih (efek/biaya/depends_on) + Warp Tier saat ini/next unlock — M15.4/7.
- [x] Connector dependency inline (`└{dep}`) di layar LEBAR (M15.6, trade-off disembunyikan
      di layar sempit demi list tetap utuh — DETAIL panel tetap tersedia semua ukuran).
- [x] Navigasi j/k + Enter mulai riset genuinely berfungsi (M15.8/9, dulu 100% fake no-op).
- [x] Highlight node terpilih (`Modifier::REVERSED`, M15.10 — screenshot PNG TAK bisa
      buktikan, diverifikasi `Cell.modifier` langsung, gap sama M13.6/M14.8).
- [x] Scroll bila tree > area (M15.11, infrastruktur utk tech tree bertambah).
- [x] Compact: tree ringkas (M15.12, efek samping connector width-gate M15.6).
- [x] Minimal: flat list tanpa branch header (M15.13).
- [x] Nama tech di-pad rata kolom (M15.15, ref Riftborne `_rTKGm.png` — kolom cost/time
      sejajar, pola sama M14.21's planet_view).
- [x] **Sign-off M15 research view (2026-07-04, iter 140)**: dinilai LENGKAP di 3 breakpoint
      (120×40 Full/80×30 Compact/60×24 Minimal) + 1 boundary tambahan yg py riwayat overflow
      (100×30 Full-minimum) — semua kriteria di atas terpenuhi, tak ada overflow, tree/status/
      progress/detail/navigasi/aksi semua genuinely berfungsi. DoD "navigable" dikonfirmasi via
      unit test `handle_key` ASLI (bukan simulasi), bukan cuma tampil statis. 16/16 item M15
      selesai.

## galaxy_map (Full)

- [ ] Daftar planet dengan status (Active/Locked + syarat) atau daftar ProcGen (nama/biome/jarak).
- [ ] Untuk ProcGen: travel time per planet tampak.
- [ ] Legend simbol map (`■ Planet · Star ═ Route`) ada.

## warp (Full)

- [ ] Target galaksi + requirement (resource & jumlah) tampak.
- [ ] Estimasi Warp Core gained tampak.
- [ ] Peringatan reset + catatan anchor Milky Way tampak.
- [ ] Opsi `[1] INITIATE WARP JUMP` ada.

## Responsiveness

- [ ] 120×40 → layout **Full** (sidebar + detail).
- [ ] 80×30 → layout **Compact** (sidebar tipis, resource ringkas satu baris).
- [ ] 60×24 → layout **Minimal** (tab-based; tidak ada overflow/garis rusak).

## Screenshot PNG berwarna (mulai M02 — `scripts/screenshot.sh` + Read PNG)

Golden teks (`buffer_to_text`) di atas membuang warna; berikut assertion warna/asset/overflow
yang hanya bisa dicek lewat PNG (Phase 2 §`LOOP.md` Verify).

- [x] main_menu 120×40 (default): PNG valid & terbaca jelas — border box-drawing utuh tanpa
      artefak, RESOURCES 3 tier berwarna berbeda (hijau/biru/magenta), MENU kuning dgn `►` fokus,
      tak ada overflow/pixel corrupt. Dicek 2026-07-02 (M02.5).
- [x] Sprite tampil di panel nyata — planet_view: sprite biome `planet_ocean` (Earth/Terran)
      tampil di MAIN, warna biru/hijau/putih jelas. Dicek 2026-07-02 (M04.6, area dummy awal).
      **Update M14 (2026-07-03):** layout final dirapikan penuh — `sprite_width_for()` hitung
      dari breakpoint `SpriteSize` nyata (M14.13), sprite disembunyikan total (0w) bila area
      genuinely tak cukup utk sprite+list floor sekaligus (M14.19) — list (NODES/FACTORIES/
      Status/aksi) SELALU menang ruang, TAK PERNAH lagi kepotong drpd sprite dekoratif. Dicek
      screenshot 120×40/80×30/60×24/70×24/45×20 — tak overflow di semua ukuran.
- [ ] Sprite tampil di MAIN VIEW (main_menu) — belum: MAIN VIEW main_menu masih galaxy spiral
      prosedural placeholder, bukan sprite/starmap. Redesign di M16 (galaxy backdrop).
- [ ] Portrait tampil di konteks nyata (merchant) — `Portraits`/`App::portrait_id` sudah wired
      ke `App` (M04.2/M04.5) tapi belum dipanggil dari panel manapun. Menyusul M22 (Merchant view).
- [ ] Galaxy backdrop mirip `galaxy_2.png` (spiral partikel, core glow) — belum dinilai formal;
      backdrop saat ini pola diamond sederhana, redesign di M16.

> Bila sebuah check gagal: itulah "yang kurang" — perbaiki di iterasi tersebut sebelum centang
> CHECKLIST. Catat check yang gagal di `state.json.failing_checks`.
