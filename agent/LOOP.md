# LOOP — Protokol Satu Iterasi

Protokol yang diikuti looping agent tiap kali bangun (`/loop`). Tujuan: maju **satu langkah kecil
yang terverifikasi** menuju [`GOALS.md`](GOALS.md), lalu serahkan state bersih ke iterasi berikut.

## Invariant

- **Spec = `abstraction/`**. Jangan mengarang mekanik; bila spec ambigu, pilih opsi paling sederhana
  & catat asumsi di `ITERATION_LOG.md` (jangan blokir).
- **Satu iterasi = satu item checklist** (atau satu sub-langkah bila item besar). Hindari mengubah
  banyak area sekaligus → susah verify.
- **Tidak ada iterasi tanpa verify hijau.** Bila tak bisa hijau, kembalikan kode ke keadaan compile
  + catat blocker.
- **Edit hanya** `src/`, `assets/`, `agent/`, `scripts/`, `data/`, `Cargo.toml`. Jangan sentuh
  `abstraction/` (spec) kecuali diminta manusia.

## Langkah Iterasi

### 1. Orient
- Baca [`state.json`](state.json) → `milestone`, `current_item`, `failing_checks`.
- Baca [`CHECKLIST.md`](CHECKLIST.md) → item belum `[x]` paling atas pada milestone aktif.
- Bila ada `failing_checks` dari iterasi lalu → prioritaskan perbaiki itu dulu.

### 2. Plan (ringan)
- Tentukan perubahan minimal untuk menuntaskan item. Identifikasi file `src/` yang disentuh.
- Tugas pencarian "di mana X" → subagent **cavecrew-investigator** (lihat [`PROMPTS.md`](PROMPTS.md)).

### 3. Build
- Implementasi sesuai spec `abstraction/`. Reuse util yang ada; ikuti layout modul `07-architecture.md`.
- Edit 1–2 file terfokus → boleh subagent **cavecrew-builder**. Perubahan lintas file → kerjakan inline.

### 4. Verify (WAJIB)
```
scripts/verify.sh          # conventions, fmt --check, clippy -D warnings, cargo test, snapshot diff
```
- Patuhi [`../CONVENTIONS.md`](../CONVENTIONS.md): ≤10 file/folder (pakai sub-folder), baris kode
  ≤100 kolom, pecah modul >~300 baris. `verify.sh` memanggil `scripts/check_conventions.sh`.
- Bila item menyentuh UI: render snapshot (`scripts/snapshot.sh <view> <w> <h>`) → cek tiap assertion
  di [`test/visual_checks.md`](test/visual_checks.md). **Lihat outputnya secara langsung** dan
  temukan yang kurang (panel hilang, overflow, teks salah) → perbaiki sebelum lanjut.
- Review diff terfokus → boleh subagent **cavecrew-reviewer**.

### 5. Record
- Centang item di `CHECKLIST.md`.
- Update `PROGRESS.md` (milestone, apa selesai, next) & `state.json` (`iter++`, `current_item`,
  `last_snapshot`, `failing_checks: []`).
- Append satu blok ke `ITERATION_LOG.md` (lihat format di file itu).

### 6. Next
- Bila milestone aktif semua `[x]` → naik milestone berikut, update `state.json`.
- Bila DoD `GOALS.md` semua terpenuhi → **STOP** (lihat Stop Conditions).
- Jadwalkan iterasi berikut via `ScheduleWakeup` (self-paced; lihat `RUNBOOK.md`).

## Stop Conditions

| Kondisi | Aksi |
|---------|------|
| Semua DoD `GOALS.md` ✅ | STOP, tulis ringkasan akhir di `PROGRESS.md` |
| Verify merah 2× iterasi berturut dengan **akar sama** | STOP, tulis **BLOCKER** di `PROGRESS.md` + `state.json.blocker` |
| Butuh keputusan desain yang tak ada di spec & tak bisa di-default aman | STOP, ajukan pertanyaan ke manusia di `PROGRESS.md` |
| `cargo build` tak bisa dipulihkan dalam 1 iterasi | STOP, kembalikan ke commit/keadaan terkompilasi terakhir, catat |

## Peran Subagent (opsional, hemat konteks)

| Subagent | Untuk |
|----------|-------|
| `cavecrew-investigator` | "di mana X didefinisikan / apa yang memanggil Y" (read-only) |
| `cavecrew-builder` | edit terfokus 1–2 file dengan scope jelas |
| `cavecrew-reviewer` | review diff/branch sebelum centang checklist |
| `Explore` | sweep luas banyak file bila scope tak pasti |

Gunakan hanya bila benar membantu; tugas kecil cukup inline.
