# RUNBOOK — Menjalankan Phase 2 (UI/UX) via Looping Agent

Cara menyuruh Claude Code `/loop` meningkatkan **UI/UX** galaxy-idle di atas hasil Phase 1,
mengikuti [`LOOP.md`](LOOP.md) + [`CHECKLIST.md`](CHECKLIST.md).

## Prasyarat (sekali)

- Phase 1 selesai & repo bersih (lihat [`../phase1/PROGRESS.md`](../phase1/PROGRESS.md)).
- Toolchain Rust (`cargo`, `rustfmt`, `clippy`) + Python 3 (untuk `gen_assets.py`/`check_assets.py`).
- Untuk regen portrait: ComfyUI lokal atau key cloud — lihat [`../../comfyui/README.md`](../../comfyui/README.md).
- Skrip executable: `chmod +x scripts/snapshot.sh scripts/verify.sh`.

## Memulai Loop (self-paced)

Tempel prompt berikut ke Claude Code (`/loop` tanpa interval → model atur tempo sendiri):

```
/loop Tingkatkan UI/UX galaxy-idle (Phase 2) mengikuti agent/phase2/LOOP.md. Tiap iterasi:
1) baca agent/phase2/state.json (milestone aktif M01-M28) + file milestone di
   agent/phase2/milestones/<kat>/M<NN>-*.md, pilih SATU item belum selesai (prioritaskan
   failing_checks; hormati depends_on, urut A->F);
2) implementasi sesuai spec tampilan di abstraction/design/ (06,07,14,15); jangan ubah mekanik;
   default aman bila ambigu, catat asumsi. Asset baru: comfyui/generate.py --set <x> →
   comfyui/verify.py (skor >=90 vs referensi, regen seed++ bila kurang) → Read PNG konfirmasi →
   scripts/gen_assets.py convert (ANSI >=99, 9 varian) → daftar di assets/manifest.ron;
3) GERBANG: scripts/verify.sh wajib hijau. Item UI: scripts/screenshot.sh <view> <w> <h> lalu
   READ PNG hasilnya (agent/test/screens/) dan NILAI vs agent/phase2/VISUAL_TARGETS.md —
   perbaiki & ulangi sampai layak (bukan sekali jadi). Galaxy pixel sixel: audit scripts/capture_term.sh.
   Item asset: scripts/check_assets.py. Update agent/test/visual_checks.md;
4) centang CHECKLIST, update PROGRESS.md + state.json, append ITERATION_LOG.md (semua di agent/phase2/);
5) berhenti bila semua DoD di agent/phase2/GOALS.md tercapai, atau bila blocker / butuh keputusan
   desain besar dari user (Stop Conditions).
Hormati guardrails: hanya edit src/, assets/, comfyui/, agent/phase2/, agent/test/, scripts/, data/,
Cargo.toml. Jangan sentuh abstraction/ atau agent/phase1/. Jangan commit kecuali aku minta.
```

> Loop dinamis pakai `ScheduleWakeup` untuk iterasi berikut. Delay panjang (≥1200s) sebagai fallback
> bila build/asset-gen lama; lanjut segera saat verify hijau.

## Guardrails (WAJIB)

| Aturan | Detail |
|--------|--------|
| Scope edit | `src/`, `assets/`, `comfyui/`, `agent/phase2/`, `agent/test/`, `scripts/`, `data/`, `Cargo.toml`. **Tidak** `abstraction/` (spec) maupun `agent/phase1/` (arsip). |
| Verify gate | Tiap iterasi lulus `scripts/verify.sh` sebelum centang. Asset → `scripts/check_assets.py` juga. |
| Satu langkah | Satu iterasi = satu item; jangan ubah banyak view sekaligus. |
| Visual | Item UI wajib dilihat via `scripts/snapshot.sh` + dicek `agent/test/visual_checks.md`; warna via `.ans`. |
| Commit | Hanya bila manusia minta; branch dulu (jangan ke `main` langsung). |
| Golden | Regen (`UPDATE_SNAPSHOTS=1`) hanya bila layout sengaja berubah; selalu tetap lulus visual_checks. |

## Memantau

- **Cepat:** `agent/phase2/PROGRESS.md`. **Detail:** `agent/phase2/ITERATION_LOG.md`.
- **Mesin:** `agent/phase2/state.json`. **Visual:** `scripts/snapshot.sh planet_view 120 40` kapan saja.

## Menghentikan / Melanjutkan

- Stop manual: hentikan sesi `/loop`; state tersimpan di `state.json` + `PROGRESS.md`.
- Lanjut: `/loop` lagi dengan prompt sama; loop membaca `agent/phase2/state.json` dan menyambung.
