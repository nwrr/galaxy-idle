# CHECKLIST — Phase 2 (INDEKS)

> Checklist penuh dipecah per milestone di [`milestones/`](milestones/) (1 milestone = 1 file,
> dikategorikan A–F). File ini = **indeks tipis**: status 28 milestone + link. Detail item granular
> ada di tiap file milestone. Protokol: [`LOOP.md`](LOOP.md). Indeks lengkap + gerbang:
> [`milestones/README.md`](milestones/README.md).

Satu iterasi = satu **item** di dalam satu milestone. Centang item di file milestone-nya, lalu
update status milestone di sini + [`state.json`](state.json). Gerbang: **V** verify.sh · **S**
screenshot+nilai · **T** capture_term · **A** verify.py/check_assets · **U** unit test.

## Progres Milestone

### A — Foundation / Harness
- [ ] **M01** [Rasterizer Buffer→PNG](milestones/A-foundation/M01-rasterizer.md) — V,U
- [ ] **M02** [Screenshot bin + script](milestones/A-foundation/M02-screenshot-bin.md) — V,S
- [ ] **M03** [Capture terminal nyata](milestones/A-foundation/M03-capture-terminal.md) — T
- [ ] **M04** [Wire asset ke App](milestones/A-foundation/M04-wire-assets.md) — V,U,S

### B — Asset Pipeline
- [ ] **M05** [ComfyUI restructure](milestones/B-asset-pipeline/M05-comfyui-restructure.md) — A
- [ ] **M06** [ComfyUI prompt sets](milestones/B-asset-pipeline/M06-comfyui-prompts.md) — A
- [ ] **M07** [verify.py similarity gate](milestones/B-asset-pipeline/M07-verify-similarity.md) — A
- [ ] **M08** [Generate+verify semua aset](milestones/B-asset-pipeline/M08-generate-verify-all.md) — S,A
- [ ] **M09** [PNG→ANSI converter](milestones/B-asset-pipeline/M09-png-to-ansi.md) — U,A
- [ ] **M10** [ANSI fidelitas ≥99 + varian](milestones/B-asset-pipeline/M10-ansi-fidelity.md) — A,S
- [ ] **M11** [Konversi semua + manifest](milestones/B-asset-pipeline/M11-convert-all.md) — A,S

### C — Shell & Core Views
- [ ] **M12** [Shell skeleton 3-kolom](milestones/C-shell-views/M12-shell-skeleton.md) — V,S
- [ ] **M13** [Shell panels + responsif](milestones/C-shell-views/M13-shell-panels.md) — V,S
- [ ] **M14** [Planet/Colony view](milestones/C-shell-views/M14-planet-colony.md) — V,S,U
- [ ] **M15** [Research view](milestones/C-shell-views/M15-research.md) — V,S

### D — Galaxy (krusial)
- [ ] **M16** [Backdrop procedural](milestones/D-galaxy/M16-backdrop-procedural.md) — S,U
- [ ] **M17** [Galaxy pixel sixel/kitty](milestones/D-galaxy/M17-galaxy-pixel.md) — T,U
- [ ] **M18** [Starmap grid + navigasi](milestones/D-galaxy/M18-starmap-grid.md) — S,U
- [ ] **M19** [Starmap detail + aksi](milestones/D-galaxy/M19-starmap-detail.md) — S,U
- [ ] **M20** [Galaxy integrasi + responsif](milestones/D-galaxy/M20-galaxy-integration.md) — V,S,T

### E — Secondary Views
- [ ] **M21** [Warp/Travel view](milestones/E-secondary-views/M21-warp-travel.md) — V,S,U
- [ ] **M22** [Merchant view](milestones/E-secondary-views/M22-merchant.md) — V,S,U
- [ ] **M23** [Events/Log view](milestones/E-secondary-views/M23-events-log.md) — V,S,U

### F — Polish & Verification
- [ ] **M24** [Theme & depth](milestones/F-polish-verify/M24-theme-depth.md) — S,U
- [ ] **M25** [Particles & banner](milestones/F-polish-verify/M25-particles-banner.md) — S
- [ ] **M26** [Responsive pass](milestones/F-polish-verify/M26-responsive.md) — S,T
- [ ] **M27** [Visual checks + golden](milestones/F-polish-verify/M27-visual-checks-golden.md) — V,S
- [ ] **M28** [Final sign-off](milestones/F-polish-verify/M28-signoff.md) — S,T,A

## Goal Gate
- [ ] Semua DoD di [`GOALS.md`](GOALS.md) (U-A..U-J) ✅
- [ ] 28 milestone ✅; verify.sh hijau + check_assets PASS + visual_checks lulus
- [ ] Screenshot tiap view × 3 breakpoint disetujui (`agent/test/screens/approved/`)
