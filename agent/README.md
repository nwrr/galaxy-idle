# agent/ — Looping-Agent System (multi-fase)

Infrastruktur agar `galaxy-idle` dibangun & diiterasi oleh **looping agent** (Claude Code `/loop`)
dengan verifikasi terminal otomatis. Pekerjaan dibagi per **fase**; tiap fase = satu workspace loop
mandiri (GOALS, CHECKLIST, PROGRESS, state.json, ITERATION_LOG, LOOP/RUNBOOK/PROMPTS sendiri).

## Fase

| Fase | Fokus | Status | Masuk |
|------|-------|--------|-------|
| **Phase 1** | Build game scaffold → DoD (M0–M9, ekonomi/sim/save/prestige/procgen) | ✅ SELESAI (iter 37) | [`phase1/`](phase1/) |
| **Phase 2** | **UI/UX design** — sprite/portrait berwarna, layout, theme; pakai ComfyUI asset generator + ANSI converter | 🟦 AKTIF | [`phase2/`](phase2/) |

## Bersama Lintas-Fase

- [`test/`](test/) — verifikasi visual terminal (snapshot `TestBackend`); dipakai `tests/snapshots.rs`
  & `scripts/verify.sh`. **Tidak** dipindah ke fase manapun (kode mereferensikan `agent/test/`).
- Spec game: [`../abstraction/`](../abstraction/) (00–16) — sumber kebenaran mekanik (jangan diubah loop).
- Kode: [`../src/`](../src/) · asset: [`../assets/`](../assets/) · generator: [`../comfyui/`](../comfyui/),
  [`../scripts/gen_assets.py`](../scripts/gen_assets.py), [`../scripts/genassets/`](../scripts/genassets/).

## Memulai / Melanjutkan Loop

Fase aktif → buka README fase itu untuk prompt `/loop` & guardrails. Saat ini: [`phase2/RUNBOOK.md`](phase2/RUNBOOK.md).
