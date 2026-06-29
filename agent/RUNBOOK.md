# RUNBOOK — Menjalankan Project Otomatis via Looping Agent

Cara menyuruh Claude Code `/loop` membangun `galaxy-idle` dari scaffold → goal secara otomatis,
mengikuti [`LOOP.md`](LOOP.md) dan [`CHECKLIST.md`](CHECKLIST.md).

## Prasyarat (sekali)

- Toolchain Rust terpasang (`cargo`, `rustc`), komponen `rustfmt` + `clippy`.
- Jadikan skrip executable:
  ```
  chmod +x scripts/snapshot.sh scripts/verify.sh
  ```
- Repo dalam keadaan bersih (commit/stash perubahan WIP) agar diff tiap iterasi terbaca.

## Memulai Loop (self-paced)

Tempel prompt berikut ke Claude Code (skill `/loop` tanpa interval → model mengatur tempo sendiri):

```
/loop Bangun galaxy-idle mengikuti agent/LOOP.md. Tiap iterasi:
1) baca agent/state.json + agent/CHECKLIST.md, pilih item belum selesai pada milestone aktif
   (prioritaskan agent/state.json.failing_checks lebih dulu);
2) implementasi sesuai spec di abstraction/ (jangan mengarang; default aman bila ambigu, catat asumsi);
3) jalankan scripts/verify.sh — wajib hijau; untuk item UI, scripts/snapshot.sh <view> <w> <h> lalu
   cek agent/test/visual_checks.md dan perbaiki yang kurang;
4) centang CHECKLIST, update PROGRESS.md + state.json, append ITERATION_LOG.md;
5) berhenti bila semua DoD di agent/GOALS.md tercapai, atau bila blocker (lihat Stop Conditions).
Hormati guardrails: hanya edit src/, assets/, agent/, scripts/, data/, Cargo.toml. Jangan commit
kecuali aku minta.
```

> Loop dinamis memakai `ScheduleWakeup` untuk menjadwalkan iterasi berikut. Pakai delay panjang
> (≥1200s) sebagai fallback bila menunggu build lama; lanjut segera saat verify hijau.

## Guardrails (WAJIB dipatuhi loop)

| Aturan | Detail |
|--------|--------|
| Scope edit | Hanya `src/`, `assets/`, `agent/`, `scripts/`, `data/`, `Cargo.toml`. **Tidak** menyentuh `abstraction/` (spec) kecuali diminta. |
| Verify gate | Tiap iterasi harus lulus `scripts/verify.sh` sebelum centang checklist. |
| Satu langkah | Satu iterasi = satu item checklist; jangan ubah banyak area sekaligus. |
| Commit | Hanya bila manusia minta. Bila perlu, branch dulu (jangan commit ke `main`/`master` langsung). |
| Destructive | Jangan hapus file/data di luar yang dibuat loop tanpa konfirmasi. |
| Snapshot golden | Regenerasi (`UPDATE_SNAPSHOTS=1`) hanya bila perubahan layout disengaja; selalu tetap lulus `visual_checks.md`. |

## Stop Conditions (lihat `LOOP.md` untuk detail)

1. **Selesai:** semua DoD `GOALS.md` ✅ → tutup `PROGRESS.md` dengan ringkasan.
2. **Blocker:** verify merah 2× iterasi akar sama → tulis blocker di `PROGRESS.md` + `state.json.blocker`, stop.
3. **Butuh keputusan manusia:** spec ambigu tanpa default aman → tulis pertanyaan di `PROGRESS.md`, stop.

## Memantau

- **Cepat:** `agent/PROGRESS.md` (milestone aktif, blocker).
- **Detail:** `agent/ITERATION_LOG.md` (apa berubah tiap iterasi).
- **Mesin:** `agent/state.json` (iter, failing_checks, done_criteria).
- **Visual:** `scripts/snapshot.sh main_menu 120 40` kapan saja untuk melihat UI terkini.

## Menghentikan / Melanjutkan

- **Stop manual:** hentikan sesi `/loop`. State tersimpan di `state.json` + `PROGRESS.md`.
- **Lanjut:** mulai `/loop` lagi dengan prompt sama; loop membaca `state.json` dan menyambung.
- **Reset milestone:** edit `state.json` (`milestone`, `current_item`) + uncheck item di `CHECKLIST.md`.

## Recovery

- `cargo build` rusak tak terpulihkan dalam 1 iterasi → kembalikan ke keadaan terkompilasi terakhir
  (git stash/checkout file terkait), catat di `ITERATION_LOG.md`, coba pendekatan lain.
- Snapshot mismatch tak terduga → bandingkan dengan `visual_checks.md`; bila layout benar tapi golden
  usang, regenerasi; bila layout salah, perbaiki kode.
