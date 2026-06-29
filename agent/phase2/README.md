# agent/phase2/ — Looping-Agent System (Phase 2: UI/UX Design)

Loop **improvement** di atas hasil Phase 1 (✅ build selesai). Fokus: **tampilan & pengalaman** —
sprite/portrait berwarna tampil di view, layout rapi & responsif, theme konsisten, navigasi jelas.
Dibantu **ComfyUI asset generator** + **ANSI converter**.

## Sumber Kebenaran

- **Spec tampilan:** [`../../abstraction/design/`](../../abstraction/design/) — `06-ui.md`, `07`, `14`, `15`.
- **Proses:** folder ini (LOOP/RUNBOOK/CHECKLIST/GOALS).
- **Kode UI:** [`../../src/ui/`](../../src/ui/). **Asset:** [`../../assets/`](../../assets/).
- **Generator:** [`../../comfyui/`](../../comfyui/) (portrait) + [`../../scripts/gen_assets.py`](../../scripts/gen_assets.py)
  + [`../../scripts/genassets/ansi.py`](../../scripts/genassets/ansi.py) (ANSI converter).
- **Infra test bersama:** [`../test/`](../test/) (snapshot + `visual_checks.md`).

## File & Perannya

| File | Peran | Siapa update |
|------|-------|--------------|
| [`GOALS.md`](GOALS.md) | DoD UI/UX (U-A..U-J) + milestone U0–U11 + keputusan arsitektur | manusia (jarang) |
| [`LOOP.md`](LOOP.md) | Protokol satu iterasi (gerbang screenshot + comfyui + ansi) | manusia (jarang) |
| [`CHECKLIST.md`](CHECKLIST.md) | ~250 item berfase (U0–U11), checkbox | **agent tiap iterasi** |
| [`VISUAL_TARGETS.md`](VISUAL_TARGETS.md) | Peta view→referensi + kriteria "layak" (gerbang screenshot) | manusia/agent |
| [`PROGRESS.md`](PROGRESS.md) | Ringkasan state terbaca-manusia | **agent tiap iterasi** |
| [`state.json`](state.json) | State mesin (iter, milestone, done_criteria) | **agent tiap iterasi** |
| [`ITERATION_LOG.md`](ITERATION_LOG.md) | Log append-only | **agent tiap iterasi** |
| [`PROMPTS.md`](PROMPTS.md) | Prompt subagent | manusia/agent |
| [`RUNBOOK.md`](RUNBOOK.md) | Cara menjalankan loop + guardrails + prompt `/loop` | manusia |
| `UX_REVIEW.md` | (dibuat di U0) audit defisiensi + target | agent |

## Alur Singkat (1 iterasi)

```
baca state.json + CHECKLIST.md
   └─▶ pilih item UI/UX belum selesai (urut milestone)
        └─▶ build (edit src/ui, assets) sesuai spec abstraction/design
             └─▶ verify: scripts/verify.sh (+ check_assets.py bila asset)
                  └─▶ VISUAL: scripts/snapshot.sh + visual_checks.md (+ cat .ans utk warna)
                       ├─ hijau & visual OK → centang, update PROGRESS+state, log, lanjut
                       └─ merah → perbaiki; 2× gagal akar sama → stop, tulis blocker
```

Mulai → baca [`RUNBOOK.md`](RUNBOOK.md). Phase sebelumnya → [`../phase1/`](../phase1/).
