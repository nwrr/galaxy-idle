# agent/ — Looping-Agent System

Infrastruktur agar `galaxy-idle` bisa dibangun & diiterasi oleh **looping agent** (Claude Code
`/loop`) dari ide → goal, dengan verifikasi tampilan terminal otomatis.

## Sumber Kebenaran

- **Spec game:** [`../abstraction/`](../abstraction/) (00–16) — apa yang dibangun.
- **Proses build:** folder ini — bagaimana agent membangunnya secara iteratif.
- **Kode:** [`../src/`](../src/), asset [`../assets/`](../assets/).

## File & Perannya

| File | Peran | Siapa update |
|------|-------|--------------|
| [`GOALS.md`](GOALS.md) | Milestone M0–M9 + Definition of Done terukur | manusia (jarang) |
| [`LOOP.md`](LOOP.md) | Protokol satu iterasi (plan→build→verify→log→next) + stop conditions | manusia (jarang) |
| [`CHECKLIST.md`](CHECKLIST.md) | Daftar tugas berfase, checkbox `[ ]/[x]` | **agent tiap iterasi** |
| [`PROGRESS.md`](PROGRESS.md) | Ringkasan state terbaca-manusia: milestone aktif, blocker, next | **agent tiap iterasi** |
| [`state.json`](state.json) | State machine-readable (iter, milestone, failing_checks) | **agent tiap iterasi** |
| [`ITERATION_LOG.md`](ITERATION_LOG.md) | Log append-only tiap iterasi | **agent tiap iterasi** |
| [`PROMPTS.md`](PROMPTS.md) | Prompt subagent siap-pakai | manusia/agent |
| [`RUNBOOK.md`](RUNBOOK.md) | Cara menjalankan loop otomatis + guardrails | manusia |
| [`test/`](test/) | Verifikasi visual terminal (snapshot TestBackend) | agent + manusia |

## Alur Singkat (1 iterasi)

```
baca state.json + CHECKLIST.md
   └─▶ pilih item belum selesai (urut fase)
        └─▶ build (edit src/assets) sesuai spec abstraction/
             └─▶ verify: scripts/verify.sh (fmt+clippy+test+snapshot)
                  ├─ hijau → centang CHECKLIST, update PROGRESS+state, log, lanjut
                  └─ merah → perbaiki; 2× gagal akar sama → eskalasi (stop, tulis blocker)
```

Mulai → baca [`RUNBOOK.md`](RUNBOOK.md).
