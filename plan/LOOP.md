# LOOP — Protocol for One Self-Paced Iteration

Invoked via `/loop` against `plan/`. Goal: advance **one checklist item** with a verified,
scored, honest result — no running iteration log this time (see Guardrails).

## Invariant

- **One iteration = one `CHECKLIST.md` item.** Don't touch multiple milestones' items in one pass.
- **Never check off an item below a 90 score.** If it scores lower, leave it `[ ]`, append an
  inline note of the measured score and what's missing, and the NEXT iteration re-attempts the
  SAME item — do not move on to a fresh item while a started one is incomplete.
- **No separate log file.** The only artifact updated per iteration is `plan/CHECKLIST.md` itself
  (checkbox + one-line inline note). Do not create `ITERATION_LOG.md` / `log/NNN-NNN.md` files.
- Spec for scoring = [`TARGET_GOALS.md`](TARGET_GOALS.md). Spec for visual/mechanical conventions
  = [`../CONVENTIONS.md`](../CONVENTIONS.md) and [`../abstraction/design/`](../abstraction/design/)
  (read for context, do not edit).

## Steps

### 1. Orient
- Open [`CHECKLIST.md`](CHECKLIST.md). Find the first `[ ]` item, top to bottom, respecting the
  `depends on` notes in each milestone header (M0→M9 order). Read any inline note left on it from
  a prior failed attempt — that tells you exactly what's still missing.

### 2. Plan (lightweight)
- Identify the minimal real change needed to satisfy that item's mapped sub-criteria in
  `TARGET_GOALS.md` (each `CHECKLIST.md` item maps to one or more lettered criteria — e.g. a
  Settings-screen item maps to P5/A3 and part of C2).
- Prefer reusing existing patterns over inventing new infrastructure:
  - `Content`/`data/*.ron` loader pattern (`src/content.rs`) for any new data.
  - `save::read`/`save::write` + `save::dto` for anything persisted.
  - `KNOWN_VIEWS`/`view_main`/`draw_view` dispatch (`src/ui/mod.rs`) for any new screen.
  - `BuildPicker`-style exclusive-input modal pattern (`src/app.rs`) for any new modal/confirm flow.
  - `game::research.rs`'s pure-function + `Result` pattern for any new game-logic module.

### 3. Build
- Implement the change. UI code only **reads** `App`/`GameState` snapshots, never mutates
  (existing invariant, `07-architecture.md`).
- Respect `CONVENTIONS.md`: ≤10 files/folder (use thematic subfolders once at the cap — see
  `panels/flow/` and `panels/story/` in the layout recommendation), ≤100-column lines.

### 4. Verify (mandatory, in this order)
```
scripts/check_conventions.sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all
scripts/screenshot.sh <affected-view> <w> <h> [theme]   # for every affected view × breakpoint
```
- If `scripts/verify.sh` (wraps the first four) is red: fix before doing anything else. A red
  `verify.sh` means **all 4 `TARGET_GOALS.md` scores are 0** — there is nothing to score yet.
- Read the resulting PNG(s) directly (Claude vision) — don't assume from code alone.

### 5. Score
- Score the item using `TARGET_GOALS.md`'s anchor scale (100/50/0 for qualitative criteria,
  100/0 for automated ones) against the specific sub-criteria it targets.
- **≥ 90** → check the item `[x]` in `CHECKLIST.md`, append the one-line note:
  `— <score>% (<date>): <why>`.
- **< 90** → leave `[ ]`, append the same note format describing the gap, stop this iteration
  (do not silently start a different item instead).

### 6. Next
- If this was the last item in `M9`, and all four `TARGET_GOALS.md` goal scores are ≥ 90:
  **STOP** — the overhaul's Definition of Done is met. Recommend a final human playthrough.
- Otherwise, end the iteration; the next `/loop` invocation re-reads `CHECKLIST.md` from step 1.

## Stop Conditions

| Condition | Action |
|---|---|
| All 4 `TARGET_GOALS.md` scores ≥ 90 | STOP, report final scores, recommend human sign-off |
| `scripts/verify.sh` red twice in a row on the same root cause | STOP, revert to last green state, describe the blocker in the checklist item's note |
| A design decision has no clear answer in `TARGET_GOALS.md`/`CHECKLIST.md`/`abstraction/design/` | STOP, pick the simplest safe default OR ask the user — do not silently invent large new scope (e.g. a multi-slot save system was NOT asked for; don't add one) |

## Guardrails

- **Editable paths:** `src/`, `data/`, `assets/`, `scripts/`, `tests/`, `comfyui/` (only if a new
  splash/title asset is genuinely needed), `Cargo.toml`, `plan/CHECKLIST.md` (checkbox+note only).
- **Do not touch:** `abstraction/` (design-spec archive) unless the user explicitly asks;
  `agent/test/` structure (screenshot/snapshot infra depends on it — its *contents* under
  `screens/`/`snapshots/` are written to automatically by the scripts, that's fine).
- **No auto-commit.** Only commit when the user explicitly asks.
- **No new iteration-log files.** `CHECKLIST.md` is the only per-iteration artifact.
- **Don't invent scope the user didn't ask for** (e.g. multi-slot saves, combat, multiplayer) —
  if a checklist item seems to imply something bigger than what's written, default to the
  smallest interpretation consistent with the existing save/content architecture.
