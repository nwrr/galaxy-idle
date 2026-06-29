# assets/source/ — Referensi Desain (NON-runtime)

Folder ini menampung **sumber desain** (SVG, PNG, sketsa) yang dipakai sebagai acuan saat membuat
asset teks. **Tidak pernah di-load oleh program** — runtime hanya membaca `.txt`/`.ans` (lihat
[`../README.md`](../README.md)).

## Aturan

- Boleh: `.svg`, `.png`, `.jpg`, `.excalidraw`, catatan desain.
- Workflow: gambar/desain di sini → terjemahkan manual jadi ASCII art di `../sprites|banner|ui/` →
  daftarkan di `../manifest.ron`.
- Jangan `include_str!`/baca file di folder ini dari kode.
- Bila ingin repo ramping, folder ini boleh ditambahkan ke `.gitignore` (opsional).

## Kredit / Sumber

Bila mengimpor aset pihak ketiga, catat di sini: `file — sumber — lisensi`.

_(kosong untuk saat ini — semua asset dibuat sendiri)_
