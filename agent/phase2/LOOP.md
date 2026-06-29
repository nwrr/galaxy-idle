# LOOP — Protokol Satu Iterasi (Phase 2: UI/UX)

Protokol looping agent Phase 2. Tujuan: maju **satu langkah UI/UX kecil yang terverifikasi** menuju
[`GOALS.md`](GOALS.md), serahkan state bersih ke iterasi berikut. Turunan dari
[`../phase1/LOOP.md`](../phase1/LOOP.md) dengan penekanan **verifikasi visual + asset**.

## Invariant

- **Spec visual = `../../abstraction/design/`** (`06-ui.md`, `07`, `14`, `15`). Jangan ubah mekanik;
  Phase 2 hanya tampilan & pengalaman. Spec ambigu → opsi paling sederhana, catat asumsi di
  `ITERATION_LOG.md`.
- **Satu iterasi = satu item checklist.** Hindari mengubah banyak view sekaligus → susah verify.
- **Tidak ada iterasi tanpa verify hijau.** Tak bisa hijau → kembalikan ke keadaan compile + catat.
- **Edit hanya** `src/`, `assets/`, `comfyui/`, `agent/phase2/`, `agent/test/`, `scripts/`, `data/`,
  `Cargo.toml`. Jangan sentuh `abstraction/` (spec) atau `agent/phase1/` (arsip) kecuali diminta.

## Langkah Iterasi

### 1. Orient
- Baca [`state.json`](state.json) → `milestone` aktif (M01–M28), `current_item`, `failing_checks`.
- Buka file milestone aktif di [`milestones/<kat>/M<NN>-*.md`](milestones/README.md) → ambil item
  belum `[x]` paling atas. (Indeks status: [`CHECKLIST.md`](CHECKLIST.md).)
- `failing_checks` dari iterasi lalu → prioritaskan. Hormati `depends_on` milestone (urut A→F).

### 2. Plan (ringan)
- Tentukan perubahan minimal. Identifikasi view/panel/asset yang disentuh.
- Pencarian "di mana X" → subagent (lihat [`PROMPTS.md`](PROMPTS.md)).

### 3. Build
- Implementasi sesuai `06-ui.md`/`07`. UI hanya **membaca** snapshot `App`/`GameState` (tak memutasi).
- **Asset baru:** pipeline ComfyUI → cek → ANSI:
  1. `comfyui/generate.py --set <set>` → PNG di `assets/source/<set>/` (prompt ⟂ workflow terpisah).
  2. `comfyui/verify.py` → skor SSIM+phash vs referensi; **<90 → regen seed++** (≤N) sampai lulus.
     Lalu **Read PNG** + Read referensi → konfirmasi vision (selera desain).
  3. `scripts/gen_assets.py convert <src.png> <base>` → 9 varian ANSI; skor fidelitas **≥99**.
  4. Daftarkan di `assets/manifest.ron`.

### 4. Verify (WAJIB)
```
scripts/verify.sh                      # conventions, fmt --check, clippy -D, cargo test, snapshot
scripts/check_assets.py                # item asset — varian lengkap, dalam box, fidelitas
scripts/screenshot.sh <view> <w> <h>   # item UI — render view → PNG BERWARNA (Buffer→PNG rasterizer)
```
- Patuhi [`../../CONVENTIONS.md`](../../CONVENTIONS.md): ≤10 file/folder, baris ≤100 kolom, modul >~300
  baris dipecah.
- **Verifikasi visual = SCREENSHOT LANGSUNG (inti Phase 2):**
  - `scripts/screenshot.sh <view> <w> <h> [theme]` → `agent/test/screens/<view>_<w>x<h>.png`.
    **Read PNG itu** (Claude vision) → nilai vs [`VISUAL_TARGETS.md`](VISUAL_TARGETS.md)
    (Layout/Density/Color/Asset/Polish) → **perbaiki & ulangi sampai layak** (bukan sekali jadi).
  - Golden teks (`snapshot.sh`/`UPDATE_SNAPSHOTS=1`) tetap dipakai utk **diff layout/CI**, bukan
    penilaian desain. Regen golden hanya bila layout sengaja berubah.
  - **Galaxy pixel (sixel/kitty)** tak muncul di rasterizer sel → audit `scripts/capture_term.sh
    <view> <w> <h>` (terminal nyata). Skip-aman bila tak ada terminal grafis.
  - Update assertion di [`../test/visual_checks.md`](../test/visual_checks.md) (warna/asset/overflow).

### 5. Record
- Centang item di **file milestone** (`milestones/<kat>/M<NN>-*.md`). Bila semua item milestone `[x]`,
  set `status: done` di header file + centang milestone di `CHECKLIST.md` + `state.json.milestones`.
- Update `PROGRESS.md` + `state.json` (`iter++`, `milestone`, `current_item`, `last_screenshot`,
  `failing_checks:[]`, `done_criteria`).
- Append satu blok ke `ITERATION_LOG.md`.

### 6. Next
- Milestone aktif semua `[x]` → naik milestone, update `state.json`.
- Semua DoD `GOALS.md` ✅ → **STOP** (Stop Conditions), tutup `PROGRESS.md`.
- Jadwalkan iterasi berikut via `ScheduleWakeup` (self-paced; lihat `RUNBOOK.md`).

## Stop Conditions

| Kondisi | Aksi |
|---------|------|
| Semua DoD `GOALS.md` ✅ | STOP, ringkasan akhir di `PROGRESS.md` |
| Verify merah 2× berturut akar sama | STOP, BLOCKER di `PROGRESS.md` + `state.json.blocker` |
| Keputusan desain tak ada di spec & tak bisa default aman | STOP, ajukan ke manusia di `PROGRESS.md` |
| Build tak terpulihkan dalam 1 iterasi | STOP, kembalikan ke keadaan terkompilasi terakhir, catat |

## Peran Subagent (opsional)

| Subagent | Untuk |
|----------|-------|
| `cavecrew-investigator` | "di mana panel X dirender / apa yang panggil draw_view" (read-only) |
| `cavecrew-builder` | edit terfokus 1–2 file UI dengan scope jelas |
| `cavecrew-reviewer` | review diff sebelum centang |
| `Explore` | sweep luas bila scope tak pasti |

Gunakan hanya bila benar membantu; tugas kecil cukup inline.
