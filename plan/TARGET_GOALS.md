# TARGET_GOALS — Scoring Rubric for the Quality Overhaul

This file defines **exactly how "done" is measured**. Every checklist item in `CHECKLIST.md`
and the final Definition of Done are scored against this rubric — no vibes-based sign-off.

## Overall Pass Condition

**All 4 goals must independently score ≥ 90/100.** This is a MINIMUM-gate, not an average:
the user named 4 distinct goals precisely so that no single strong goal can mask a weak one
(e.g. a rich questline must not be allowed to compensate for a broken save system). An average
is still computed and reported for trend-tracking, but it never substitutes for the per-goal gate.

**Hard prerequisite: `scripts/verify.sh` must be green.** If `scripts/verify.sh` is red (fmt,
clippy -D warnings, conventions, or any test failing), **all 4 goal scores are 0 by definition**,
regardless of how much feature work is "done." Automated tests are therefore not just one line
item among many — they are a gate every other score is conditioned on. Qualitative (screenshot +
code review) judgment only ever adjusts scores *above* this floor.

## Scoring Formula

Each goal has a list of weighted sub-criteria whose weights sum to 100. Each sub-criterion is
scored using one of two methods:

- **Automated** (binary): pass = 100, fail = 0. Backed by a `cargo test`, `clippy`, `grep`/static
  check, or `check_conventions.sh` rule — no room for interpretation.
- **Qualitative** (3-point anchor scale), evaluated via `scripts/screenshot.sh` (Read the PNG) +
  direct code reading of the relevant module:
  - **100** — fully meets the bar described, no notable gaps.
  - **50** — present but flawed/partial (e.g. exists but has an obvious rough edge, missing an
    edge case, inconsistent across breakpoints/themes).
  - **0** — missing, broken, or fake/stub (e.g. a menu entry that goes nowhere, a placeholder).

```
goal_score = Σ (weight_i × item_score_i) / 100
overall_pass = min(PLAYABLE, AAA, RICH_CONTENT, CLEAN_CODE) ≥ 90
overall_average = mean(PLAYABLE, AAA, RICH_CONTENT, CLEAN_CODE)   // reporting only, not gating
```

---

## GOAL 1 — PLAYABLE (weights sum to 100)

| # | Weight | Criterion | Method |
|---|---|---|---|
| P1 | 15 | Every view reachable from real navigation has a real renderer. Zero `draw_placeholder`/"(TODO)" reachable via any keybinding or menu entry. | Automated: test asserts no reachable view falls through `view_main`'s `None` arm. |
| P2 | 10 | Every keybinding advertised anywhere in the UI (footer, shortcuts panel, help screen) is real and does something. No fake bindings like the current `?:Help`. | Automated: test cross-references advertised key strings against `handle_key`'s match arms. |
| P3 | 15 | `run()`'s actual entry point is Splash → Title (New Game / Continue / Settings / Quit) → Game. It no longer drops straight into a hardcoded `App::demo()` state. | Automated (entry view assertion) + qualitative (screenshot). |
| P4 | 15 | Save/load fully wired: autosave fires on interval, manual Save works, quit saves, New Game writes a fresh save via `game::world::load_milky_way`, Continue/Load correctly restores it, version migration path is exercised. | Automated: integration tests for each trigger point. |
| P5 | 10 | No dead menu entries — every `menu.rs` `GROUPS` entry resolves to `Some(view_id)` that is a real, reachable view. | Automated: static check (no `None` left) + navigation test. |
| P6 | 10 | Onboarding: a brand-new player is never stuck with zero guidance from tick 0 (tutorial hint or "next objective" indicator visible). | Qualitative + automated (tutorial_step state test). |
| P7 | 10 | Action feedback: build/research/travel/trade always produce a visible state change or an explicit, visible rejection reason — no more silent `let _ = ...` no-ops with zero player-facing signal. | Qualitative + automated (state assertions post-action). |
| P8 | 10 | An in-game Controls/Help reference exists, is reachable, and accurately lists every real keybinding — kept from drifting via a regression test (not hand-maintained prose only). | Automated + qualitative. |
| P9 | 5 | The game signposts at least one clear "loop closure" moment (e.g. first Warp Jump) — not pure undirected infinite grind with zero explanation. | Qualitative. |

## GOAL 2 — AAA GAME IN TERMINAL (weights sum to 100)

| # | Weight | Criterion | Method |
|---|---|---|---|
| A1 | 20 | Splash/Title presentation quality: real art/banner or animated backdrop, no placeholder look, feels like a "real game boot" not a debug screen. | Qualitative (screenshot). |
| A2 | 15 | Transition/feedback animation pass: visible micro-animation/flourish on confirmed actions (build, research complete, travel arrival, warp jump, trade) — panels are no longer 100% static. | Qualitative. |
| A3 | 15 | Settings screen lets the player actually switch Default/HighContrast/Mono live; all 3 themes render cleanly across all 3 breakpoints. | Qualitative + automated (theme switch test). |
| A4 | 15 | Full/Compact/Minimal breakpoints are screenshot-clean for every view (old + new) — zero overflow/clipping. | Automated (existing snapshot infra extended) + qualitative. |
| A5 | 15 | Sprite/portrait/galaxy_sim assets actually appear in the context they were built for (planet sprite in planet view, merchant portrait in merchant view, galaxy_sim as title/menu backdrop) — no orphaned asset modules. | Qualitative. |
| A6 | 10 | The Help/Controls screen itself is well-presented (organized, themed, matches the visual language of other panels) — not a raw text dump. | Qualitative. |
| A7 | 10 | "Juice" checklist: color-coded status, progress bars, and particle effects are wired into more moments than just ship exhaust (build complete, research complete, warp jump all get *something* visual). | Qualitative. |

## GOAL 3 — RICH CONTENT (weights sum to 100)

| # | Weight | Criterion | Method |
|---|---|---|---|
| R1 | 25 | A branching questline system exists: `data/quests.ron` + `QuestState` in save + a `quest` view. Target: **≥ 10 quests across ≥ 2 narrative arcs, with ≥ 1 quest containing a real branch point with materially different outcomes/rewards.** | Automated (quest count/schema test) + qualitative (read the actual text). |
| R2 | 15 | Travel events (`src/game/events.rs`'s 4 `EventKind`s) get real flavor/character framing text, not bare option labels. | Qualitative. |
| R3 | 15 | Tech tree expanded from 7 to **≥ 20 nodes**, meaningfully deepening all 4 branches, still passing `content::validate`. | Automated (count + validate) + qualitative (progression sense). |
| R4 | 15 | A defined, bounded subset of resources/items/buildings gain a one-line lore/flavor sentence beyond the functional blurb — target: **all "Rare"/"Special" tier resources plus every building**, not an unbounded rewrite of all 88+58+24 entries. | Qualitative. |
| R5 | 10 | At least one additional narrative hook tying a handcrafted location (or a clearly justified, explicitly logged decision to defer this) into the questline. | Qualitative. |
| R6 | 10 | A defined narrative capstone/win-condition-adjacent beat (e.g. quest chain climaxing around the first Warp Jump) layered on top of the existing infinite-prestige loop, not replacing it. | Qualitative. |
| R7 | 10 | All new content is data-driven (`.ron`) and passes `content::validate`/save round-trip — no narrative strings hardcoded in Rust that belong in data. | Automated. |

## GOAL 4 — CLEAN CODE (weights sum to 100)

| # | Weight | Criterion | Method |
|---|---|---|---|
| C1 | 20 | `scripts/verify.sh` green at all times (conventions + fmt + clippy -D warnings + full test suite + snapshots). | Automated. |
| C2 | 20 | Zero dead/unreachable code paths: no `None`-view menu entries, no reachable `draw_placeholder`, stale scaffold `#[allow(dead_code)]` markers removed once the code is actually wired/reachable. | Automated (static checks) + code review. |
| C3 | 15 | All new code honors `CONVENTIONS.md` §1/§2 (≤10 files/folder via thematic subfolders, ≤100-column lines). | Automated (`check_conventions.sh`). |
| C4 | 15 | New modules follow existing architectural conventions: data-driven `Content`/`.ron` pattern for content, `game::actions`-style pure functions returning `Result`, UI-reads-only-state discipline (`07-architecture.md`). | Code review. |
| C5 | 10 | Every new state transition (title→game, save/load wiring, quest state machine, tutorial step advance) has at least one direct test — not just "it compiles." | Automated. |
| C6 | 10 | `agent/phase1/` and `agent/phase2/` deleted; `agent/test/` preserved and still functional (`scripts/screenshot.sh`, `tests/snapshots.rs` still pass). | Automated + manual check. |
| C7 | 10 | No regression in the pre-existing test suite (296+ passing); golden snapshots updated *consciously* (never silently) when layout intentionally changes; any accepted pre-existing long-file debt (`app.rs`, `panels/galaxy_map.rs`, `panels/planet.rs`, `panels/research.rs`) is explicitly logged as out-of-scope rather than ignored. | Automated + review. |

## Standing Guardrails

- Do not modify `abstraction/` (design-spec archive) unless the user explicitly asks.
- Do not delete or restructure `agent/test/` (screenshot/snapshot infra depends on it).
- Only commit when explicitly asked by the user.
- Never mark a `CHECKLIST.md` item done below a 90 score on the sub-criteria it targets.
- No separate iteration log — `CHECKLIST.md` itself, with an inline one-line score note per
  checked item, is the only artifact updated per loop iteration.
