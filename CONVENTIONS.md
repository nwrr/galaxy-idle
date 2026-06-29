# Konvensi Proyek — `galaxy-idle`

Aturan struktur & gaya. **Wajib** dipatuhi (manusia & looping agent). Pelanggaran = blok merge.

## 1. Maksimal 10 file per folder

- Satu folder **maksimal 10 file** (sub-folder tidak dihitung sebagai file).
- Bila menambah file ke-11, **buat sub-folder** dan kelompokkan secara tematik.
- Contoh penerapan:
  - `assets/sprites/<base>/<size>.<depth>.ans` — sprite celestial dikelompokkan per base
    (11 sub-folder × 9 file), bukan 99 file datar.
  - `abstraction/design/` (spec inti 00–08) + `abstraction/content/` (katalog dunia 09–16).
  - `scripts/genassets/` — paket modul generator (bukan 1 skrip raksasa).

## 2. Panjang baris (best practice Rust/rustfmt)

- **Maksimal 100 kolom per baris** untuk kode (default `rustfmt` `max_width = 100`). Lihat
  [`rustfmt.toml`](rustfmt.toml). Berlaku juga untuk Python (`scripts/`).
- Jaga baris pendek & jelas; pecah ekspresi panjang, bukan menulis 1 baris super-lebar.

## 3. Panjang file → pecah jadi beberapa file

- Modul kode sebaiknya **fokus & < ~300 baris**. Bila > 300 baris (atau menggabung > 1 tanggung
  jawab), **pecah jadi beberapa file** dalam sub-folder modul.
  - Rust: pisah ke `mod.rs` + sub-modul (`ui/`, `game/`, …) sesuai
    [`abstraction/design/07-architecture.md`](abstraction/design/07-architecture.md).
  - Python: pisah ke paket (`scripts/genassets/`: `numfx.py`, `celestial.py`, `ansi.py`, …).
- Dokumen markdown naratif (`abstraction/`) **dikecualikan** dari batas baris-file (prosa, bukan
  kode) — tetap tunduk aturan #1 (≤ 10 file/folder).

## 4. Penegakan

- Manual/agent: `scripts/check_conventions.sh` (cek ≤10 file/folder + baris ≤100 kolom) sebelum
  commit. Sudah dirangkai ke `scripts/verify.sh` (gerbang loop).
- Rust: `cargo fmt --check` + `cargo clippy -D warnings` (lihat `scripts/verify.sh`).
- Saat memindah file, **perbarui semua referensi** (link markdown, `manifest.ron`, `mod` Rust,
  `import` Python) di repo agar tidak ada tautan rusak.
