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

- [ ] Header memuat nama planet + sektor (mis. `EARTH - Sector 001`).
- [ ] Terlihat ringkasan RESOURCES (node), FACTORIES, SHIP, RESEARCH.
- [ ] Baris aksi memuat opsi bernomor `[1]…[5]` (Node/Upgrade/Factory/Lab/Ship).
- [ ] Status ship terlihat (`Idle` atau `Next: <planet> (<eta>)`).

## research (Full)

- [ ] Header memuat laju Data (`Data/sec`) + total.
- [ ] Daftar AVAILABLE TECHS (≥1) dengan cost + waktu.
- [ ] Bila ada riset aktif: progress bar `[####....] NN%` + sisa waktu.

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

> Bila sebuah check gagal: itulah "yang kurang" — perbaiki di iterasi tersebut sebelum centang
> CHECKLIST. Catat check yang gagal di `state.json.failing_checks`.
