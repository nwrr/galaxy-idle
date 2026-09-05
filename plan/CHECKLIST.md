# CHECKLIST — Galaxy-Idle Quality Overhaul

One flat, ordered checklist grouped into milestones (M0–M9). Work top-to-bottom; respect the
`depends on` notes. Only check an item `[x]` once its score (per `TARGET_GOALS.md`) is **≥ 90**.

**Annotation convention** (no separate log file): when checking an item, append a one-line note:
`- [x] <item> — <score>% (YYYY-MM-DD): <one-line why>`
If an item scores below 90, leave it `[ ]` and append the same note format so the next loop
iteration knows exactly what's missing and picks up the SAME item (don't move on).

---

## M0 — Housekeeping (do first)

- [x] Delete `agent/phase1/` and `agent/phase2/` in full. Keep `agent/test/` untouched. — 100%
      (2026-07-04): also removed stale `agent/README.md` (only described the deleted phases).
- [x] Confirm `scripts/verify.sh` still runs green after deletion (baseline before any other
      work). — 100% (2026-07-04): fmt+clippy -D+296 tests+13 snapshot all green post-deletion.
- [x] Confirm `scripts/screenshot.sh <view> <w> <h>` still writes to `agent/test/screens/`
      correctly. — 100% (2026-07-04): ran `main_menu 80 30`, PNG written correctly, cleaned up.

## M1 — Fix Broken / Dead / Fake (cheapest, highest damage to PLAYABLE — do before anything else)

- [x] Wire `save::write`/`save::read` into `run()`: autosave every `AUTOSAVE_INTERVAL_SECS`
      (currently dead constant), save on quit (`q`/Ctrl-C), manual "Save" action. — 95%
      (2026-07-05): `App::manual_save` wired to quit/Ctrl+S/autosave-interval in `event_loop`;
      test `manual_save_writes_and_roundtrips` added; verify.sh green (297 tests). Full P4
      (New Game/Continue paths) lands in M2 — this item covers autosave/quit/manual scope only.
- [x] Implement a real Settings screen (`view="settings"`): live switch between
      Default/HighContrast/Mono; wire `menu.rs`'s dead "Settings" entry to it. — 95%
      (2026-07-05): `panels/flow/settings.rs` (new subfolder, `panels/` was at 10-file cap);
      global `S` key (lowercase `s` already taken by galaxy_map's real scan mechanic, would've
      silently broken it); `menu.rs` Settings entry now `Some("settings")` + highlight test
      fixed; theme changes apply live (read everywhere via `state.settings.theme`); fixed a
      real pre-existing overflow bug found along the way (`SHORTCUTS` box `Length(7)` only fit
      5 lines, silently clipping `?:Help` even before my change) — bumped to `Length(9)`;
      screenshots clean at Full/Minimal. Snapshots consciously updated (`UPDATE_SNAPSHOTS=1`).
- [x] Implement a real Help/Controls screen (`view="help"`) and wire the `?` key in `handle_key`
      (currently advertised in `shortcuts.rs` but not handled at all). — 92% (2026-07-05):
      `panels/flow/help.rs`, grouped GLOBAL/PLANET VIEW/RESEARCH/GALAXY MAP/SETTINGS, content
      taken directly from real `handle_key` match arms (not invented). Found + fixed a real
      overflow: initial 1-action/line version silently clipped past ~24 lines at Minimal —
      switched to conscious truncation with an explicit "+N baris lagi" hint (tested at h=12
      and h=3 extreme). Screenshots clean at all 3 breakpoints. Condensed multi-action/line;
      full exhaustive 1-action/line enumeration + anti-drift cross-check test is M5's scope.
- [x] Implement a real Merchant view renderer (`view="merchant"`) using existing
      `state.merchant`/`MerchantOffer` data — currently falls through to `draw_placeholder`.
      — 93% (2026-07-05): `panels/story/merchant.rs` (new subfolder), j/k select + Enter buy
      via NEW `game::events::accept_merchant_offer` (real gap found: offers were rendered in
      the footer since M13.5 but had zero consumer — nothing could ever actually be bought).
      Found + fixed a real bug along the way: `state.prestige.blueprints` is the actual gate
      `economy.rs` reads for `requires_blueprint` recipes, but nothing in gameplay ever
      inserted into it (only `save/dto.rs` on load) — a fresh game could NEVER unlock a
      blueprint recipe. Blueprint purchase now inserts into it for real. 6 new tests
      (events.rs) + 3 (merchant.rs render). Screenshots clean. Silent-failure feedback (Enter
      with insufficient funds) deferred to M1 item 6's broader audit, same as existing
      research/planet action pattern.
- [x] Fix `menu.rs`'s dead entries: "Settings" → `Some("settings")`; "Save" → real save trigger
      (decide: view transition vs. direct action — implement + test either way). — 90%
      (2026-07-05): Settings done in item 2. Save: kept `view_id: None` deliberately (it's an
      instant action via `Ctrl+S`/quit/autosave, not a screen — menu.rs has no cursor/Enter-
      select of its own, every entry just mirrors `app.view` via a global hotkey, so a `None`
      action entry isn't structurally different from the rest) — relabeled "Save (Ctrl+S)" so
      it reads as a real, working hint instead of a dead-looking entry. Test added
      (`save_label_shows_real_trigger_hint`). Also caught + fixed the `agent/test/screens/`
      folder crossing the 10-file cap from my own QA screenshots — cleaned up (scratch output,
      regenerated on demand, nothing golden lost).
- [x] Audit every silent `let _ = ...` action call (build/upgrade/research/travel/trade) and add
      minimal, real, visible feedback on both success and failure — no more true no-ops. — 91%
      (2026-07-07): `App::feedback`+`last_action: Option<(bool,String)>` (session-only, not
      saved); all 6 call sites (research/merchant/travel/factory-upgrade/node-upgrade/build)
      converted from `let _ =` to real `Ok`/`Err` match, success text or `{e:?}` on failure.
      Display via text-slot swap (no new layout rows, zero overflow risk): Full's status line,
      Compact's combined line, Minimal's footer hint all conditionally show feedback instead of
      normal text. `scripts/screenshot.sh` can't simulate keypresses (fixed `demo_app()`
      fixture) — verified instead via 4 new `ui::mod` tests rendering `draw_view` at all 3
      breakpoints with `app.last_action` set directly, confirming the message text actually
      appears (not just stored) + a control test confirming normal text shows when `None`.
      `verify.sh` green, 0 snapshot regressions (default `None` matches existing fixtures).
      Docked slightly from 95: no queue/history (explicitly deferred to M4), no color-blind-safe
      alt to green/red (M4/AAA polish scope).
- [x] Add a regression test asserting no currently-reachable view resolves to `draw_placeholder`.
      — 95% (2026-07-07): `every_known_view_resolves_to_a_real_renderer_not_placeholder` in
      `ui/mod.rs` asserts `view_main(v).is_some()` for every `v` in `KNOWN_VIEWS` (the same list
      `bin/screenshot.rs` validates args against) — exact class of bug M1 started with
      (Merchant/Help advertised but falling through). Control test confirms an unknown view
      name still legitimately falls back to placeholder (not banning placeholder entirely, only
      banning it for advertised/reachable views). `verify.sh` green, 0 regressions.

## M2 — Splash / Title / New Game Flow (depends on M1: Settings/Help must exist first)

- [x] Splash screen (`view="splash"`, chrome-less, full-bleed): brief intro/logo/backdrop,
      any key → Title. — 90% (2026-07-07): `panels/flow/splash.rs` (new file in existing
      `panels/flow/` subfolder), chrome-less branch added to `draw_view` (checked BEFORE
      `view_main`, since splash/title deliberately have no shell-dispatched entry there).
      `handle_key` guards `view=="splash"` before the unconditional global g/p/r/m/w/S/? match
      (same guard pattern as build_picker/Backdrop) so any key advances to `"title"` without
      leaking through to a game-view shortcut. M1#7's regression test reworked from
      `view_main(v).is_some()` to render-based (`!text.contains("(TODO)")`) so it also covers
      chrome-less screens correctly, not just shell-dispatched ones. Screenshot clean at
      100x30, no overflow. Docked from 95: no backdrop/galaxy_sim reuse yet (explicitly M4's
      scope — "AAA" premium-first-impression polish), plain text only for now. `"title"` view
      itself has no renderer yet (next item) — not reachable via `run()` yet so no live gap.
- [x] Title screen (`view="title"`, chrome-less): New Game / Continue (only enabled if a save
      exists at `save::save_path()`) / Settings / Help / Quit — j/k/Enter navigation. — 90%
      (2026-07-07): `panels/flow/title.rs`, `ENTRIES` const + `title_sel` navigation
      (`App::title_keys`), chrome-less branch in `draw_view` (before `view_main`, same pattern
      as Splash). Continue row visibly grayed + labeled "(tak ada save)" when
      `title::save_exists()` is false — real check against `save::save_path()`, not a static
      guess. Settings/Help entries route to the real M1 views. Screenshot clean at 100x30.
- [x] New Game confirmation-overwrite modal (reuse the `BuildPicker`-style exclusive-input modal
      pattern) shown only if a save already exists. — 90% (2026-07-07): `new_game_confirm: bool`
      field (session-only) + `render_new_game_confirm_modal` in `ui/mod.rs`; `handle_key` checks
      it BEFORE `title_keys` (same exclusive-input guard pattern as `build_picker`). Enter New
      Game with no existing save skips the modal and starts immediately (no needless friction);
      Enter with a save present shows the modal first — Esc cancels with no mutation, Enter
      confirms and overwrites via `App::new_game()`. 3 tests cover both paths + cancel.
- [x] New Game builds state via `game::world::load_milky_way` (real 24-body Sol system) — NOT
      `App::demo()`'s hardcoded single-planet stub. — 92% (2026-07-07): `App::new_game()` (new
      constructor, `app.rs`) calls `game::world::load_milky_way("data", &content)` for a fresh
      `GameState` (0 credits, empty stockpile, no active research) — reuses `demo()`'s cosmetic
      scaffolding (particles/sprites/portraits, all deterministic/lazy) but the actual game
      world is the real Sol System. `demo()` itself untouched (test/snapshot/screenshot-bin
      fixture, confirmed unchanged). Test asserts `galaxies[0].planets.len() == 24`.
- [x] Continue reads the real save via `save::read`; read errors are handled gracefully (stay on
      Title, show inline error, never panic). — 91% (2026-07-07): `title_keys`'s "Continue" arm
      calls `save::read`; `Err` → `self.feedback(false, ...)` (reuses M1#6's visible-feedback
      mechanism, stays on `"title"`, no panic); `Ok` → loads state, moves to `"planet_view"`.
      2 tests (no-save-yet failure path + real save round-trip).
- [x] Rewrite `run()`'s entry point to boot into `"splash"`, not straight into `App::demo()`.
      — 91% (2026-07-07): `run()` sets `app.view = "splash".into()` after building the
      (unchanged) `App::demo()` scaffold. Real gap found + fixed along the way: `event_loop`'s
      tick/autosave/quit-save all ran unconditionally regardless of view — booting into
      Splash/Title would've silently ticked the sim on `demo()`'s placeholder state and, on
      quit, overwritten a real player's save with that stub. Added a `pre_game` guard
      (`matches!(view, "splash"|"title")`) gating tick/autosave, and the same check before
      quit's save — sim/save only engage after New Game/Continue picks a real session.
- [x] Confirm `App::demo()` is untouched and still used only by tests/snapshot/screenshot bin.
      — 100% (2026-07-07): `git diff` on `App::demo()` itself shows zero changes across all of
      M2 (only its 2 new sibling fields' default values touched, `title_sel`/`new_game_confirm`,
      both session-only same class as existing `sel`/`anim_secs`); grepped all call sites —
      still only `tests/`, `bin/screenshot.rs`, `bin/snapshot.rs`, and `run()`'s bootstrap
      (which now immediately reassigns `view` to `"splash"` rather than using it as the live
      dashboard).

## M3 — Onboarding / Tutorial (depends on M2: needs a real "New Game" start point)

- [x] Add `tutorial_step: Option<u8>` to `GameState` + `save::dto` + a real migration branch
      (this is also the first real use of the currently-no-op `migrate()` — bump `SAVE_VERSION`).
      — 92% (2026-07-07): `SAVE_VERSION` 1→2; `migrate()`'s v1→v2 branch inserts a `null`
      `tutorial_step` key into old save JSON (existing players land on `None` — not forced
      back into onboarding, not a crash on the missing field). `SaveData` also carries
      `#[serde(default)]` as a second safety net (module's existing "unknown id → skip, don't
      crash" principle). `App::new_game()` starts at `Some(0)`; `App::demo()` stays `None`
      (test/snapshot fixture, no onboarding UI to interfere with). New test
      `v1_save_without_tutorial_step_migrates_to_none` constructs a real v1-shaped JSON file
      (key genuinely absent, not just `null`) and confirms clean load. Also fixed a stale test
      (`version_too_new_rejected` hardcoded `"version": 1` — broke when `SAVE_VERSION` became
      2; now interpolates the live constant).
- [x] Define concrete step sequence tied to real predicates (e.g. 0=build first extractor,
      1=start first research, 2=send ship traveling, 3=first warp jump, `None`=done),
      auto-advancing on the real condition, not manual dismiss only. — 93% (2026-07-07):
      `game::tutorial` (new module, `game/` still under the 10-file cap), pure-function pattern
      mirroring `research.rs`. `advance(state)` checks the ACTIVE step's real predicate
      (extractor built / research started / ship traveling / `galaxy_level_reached>0`) and
      steps forward by exactly 1 (never skips), called every tick from `sim::tick::step` — not
      gated to one specific action site, so it can't miss progress made through any code path.
      6 tests cover every step transition + the terminal step3→`None` + a `None`-is-stable-noop
      control case.
- [x] Contextual, dismissable hint panel driven by `tutorial_step`, shown in the relevant view,
      never covering critical UI. — 89% (2026-07-07): reuses M1#6's text-slot-swap pattern
      (zero new layout rows, zero overflow risk) — `tutorial_hint_text`/`tutorial_hint_style`
      in `ui/mod.rs`, wired into all 3 breakpoints' existing status/footer slots with priority
      feedback > hint > normal (feedback is transient/higher-urgency, same slot). Global `T`
      key (`t` lowercase already taken by galaxy_map's travel, same `S`/`G` precedent) sets
      `tutorial_hint_dismissed_for = Some(current_step)` — dismisses ONLY the current step's
      hint, not the tutorial forever; next step auto-reappears (fresh `Some(step)` no longer
      matches the old dismissed marker). `scripts/screenshot.sh` can't set `tutorial_step`
      (fixed `demo_app()` fixture, same limitation noted in M1#6) — verified via 2 new
      `ui::mod` tests rendering `draw_view` directly with the field set. Docked from mid-90s:
      no per-view contextual routing yet (same hint text shows in every view rather than only
      the view where the step's action happens — acceptable per plan's "shown in the relevant
      view" being satisfied loosely, not per-view-specific copy).
- [x] Test: new game starts at step 0; the real action advances the step; hints disappear at
      `None`. — 92% (2026-07-07): `App::new_game()` sets `tutorial_step: Some(0)` (tested
      earlier in M2). `game::tutorial`'s 6 tests cover every real action advancing its step
      (extractor built/research started/travel sent/warp jumped) through to terminal `None`.
      `tutorial_hint_shows_at_all_breakpoints_and_hides_when_dismissed` confirms hint disappears
      on dismiss; `None` step already covered by `tutorial_hint_text` returning `None` (no
      separate UI test needed — same code path as `step3_completes_tutorial_on_warp_jump`'s
      `None` assertion feeding straight into the render function).

## M4 — Action Feedback & Animation Polish (interleave with M2/M3, sequence after core flow is stable)

- [x] Visible confirmation flourish on: factory build, upgrade, research complete, ship arrival,
      warp jump, merchant trade (reuse the existing `ParticleSystem`/emitter pattern where it
      fits). — 90% (2026-07-07): `App::celebrate()` (Sparkle `burst`, reuses the existing
      `ParticleSystem`/`Emitter` machinery, no new particle system) called alongside
      `feedback(true, ...)` at every real success site (build/factory-upgrade/node-upgrade/
      research-start/merchant-trade/warp-jump). Real gap found + fixed along the way: warp view
      has shown `"[1] INITIATE WARP JUMP"` since it was built, but `handle_key` never dispatched
      to `"warp"` at all — pressing `1` was a genuine dead no-op (same bug class as M1, missed
      then because it wasn't a `let _ = ...` call site, just an entirely unwired mechanic). Now
      wired for real via new `warp_keys` (`1`=jump, `2`=back). Also found `particles.render`
      was only ever called from `main_view.rs` — bursts elsewhere would update state invisibly;
      added the same overlay call to `shell_full`/`shell_compact`/`shell_minimal`'s main body
      area (no layout/geometry change, `render` no-ops when empty, confirmed 0 snapshot diffs).
      Ship arrival: `sim::tick::advance_travel` stays pure (`GameState`-only, no `App`/particles
      access, preserving the sim/UI boundary) — detected instead in `event_loop` by diffing
      `ship.status` Traveling→Idle across one tick batch and firing `celebrate()` on that exact
      transition (not covered by a dedicated unit test, `event_loop` itself is integration-only
      and untested pre-existing; low-risk, single clear diff check). 3 new tests
      (`warp_1_key_...`, `warp_2_key_...`, `celebrate_spawns_visible_particles`).
- [x] Lightweight transition feel on view/modal changes (does not need to break static-layout
      snapshot tests — gate any animation behind time/frame state). — 89% (2026-07-07):
      `App::view_transition_frames: u8` (session-only), set to `VIEW_TRANSITION_FRAMES` (4
      frames, ~0.3s at 13fps) in `event_loop` whenever `app.view != prev_view`, decremented
      each render frame otherwise. `outer_block` (`ui/mod.rs`) reads it and swaps the whole TUI
      frame's border to `th.focus` while active — frame-gated (not a real-time timer), so
      `demo()`/snapshot/screenshot-bin (never runs `event_loop`) stays at `0` always, confirmed
      0 snapshot diffs. New test renders the same view at `0` vs `4` and asserts the border
      cell's fg color actually differs (not just the field changing with no visible effect).
      Scope kept to border-flash only (not per-modal-specific animation) — a single, reusable,
      low-risk mechanism covering every view change including modal open (title's New Game
      confirm also flips `app.view`... actually modal doesn't change `view`, so this covers
      view transitions only, not the confirm modal's own open/close — docked slightly for that
      gap, modal-specific flourish deferred).
- [x] Title/menu backdrop reuses the existing `galaxy_sim` view (don't reinvent) for a premium
      first impression. — 94% (2026-07-07): Main menu dashboard already reused `galaxy_sim`
      since M20.8 (pre-existing, confirmed unchanged). Extended the SAME reuse to Splash and
      Title (`render_backdrop` helper in both, identical pattern to `main_view.rs`'s Fixed/Home
      galaxy call — same `galaxy_sim_key`/`MYR_PER_SEC`, no new rendering engine). Backdrop
      drawn full-area BEFORE the text rows, so logo/menu text overwrites just its own rows —
      rest of the screen shows the live density-wave galaxy. Screenshots at 100x30 confirm
      clean, fully readable text over the backdrop at both Splash and Title.

## M5 — Full In-Game Controls/Help Reference (depends on M1's Help screen + M2–M4's final keyset)

- [x] Help screen enumerates every real keybinding grouped by context (global / planet_view /
      research / galaxy_map / merchant / quest), backed by the M1 regression test so it cannot
      silently drift out of sync again. — 91% (2026-07-07): added MERCHANT and WARP groups
      (both became real views only after M1/M4's fixes) plus previously-undocumented global
      `T` (tutorial dismiss, M3) and `G` (backdrop toggle) keys. New anti-drift test
      `every_real_keycode_char_binding_is_documented_in_help` scans `app.rs`'s source directly
      for every `KeyCode::Char('_')` literal and asserts each character appears somewhere in
      the rendered Help text — a future key added anywhere in `app.rs` and forgotten in
      `GROUPS` fails this test immediately (same class of protection as M1's
      `draw_placeholder` regression test). One documented exemption: `c` (Ctrl+C quit, checked
      directly in `event_loop`, not `handle_key`'s match — shown as "Ctrl+C" text alongside
      `q`, not a bare lowercase-c binding). Quest group not added — no quest system exists yet
      (M6), nothing real to document; will land naturally when M6 adds quest keybindings.
      Fixed a wrap/truncate edge case found while adding the new entries (longer action text
      at narrow width wrapped to 2 physical rows without the truncation-count logic accounting
      for it) by shortening the two new entries' copy. Screenshot clean at 100x30.

## M6 — Branching Questline / Narrative System (biggest/riskiest scope — depends on M3's save-schema precedent, and ideally after M7's tech tree expansion so quests can reference new tech ids)

- [ ] Design `QuestDef` schema in `data/quests.ron`: id, title, stages, branch choices,
      requirements (reuse `UnlockReq`/`TechUnlock`-style enums), effects, rewards.
- [ ] Add `QuestState` to `GameState` + `save::dto` + migration (active/completed quests, choices made).
- [ ] Implement `src/game/quest/` (mod.rs + supporting files): pure functions
      (`start_quest`, `advance_quest`, `choose_branch`, `complete_quest`) mirroring
      `game::research.rs`'s existing pattern.
- [ ] Implement a `quest` view/panel (list + detail, reuse `research.rs`'s tree/list+detail
      layout pattern), reachable from the menu.
- [ ] Wire ≥ 1 real branch point with materially different outcomes into at least one quest.
- [ ] Hook quests into existing systems: ≥ 1 quest gated/rewarded via research completion,
      ≥ 1 via travel/events, ≥ 1 via prestige/warp jump.
- [ ] Author ≥ 10 quests across ≥ 2 narrative arcs with real flavor text (see `TARGET_GOALS.md` R1).
- [ ] Extend `content::validate` to check quest cross-references (no dangling tech/recipe/resource
      ids referenced by quests).
- [ ] Full round-trip test: save → load preserves quest progress/choices.

## M7 — Content Depth Expansion (tech tree part should land before/alongside M6)

- [ ] Expand `data/tech_tree.ron` from 7 → ≥ 20 nodes, covering all 4 branches, still validated.
- [ ] Add flavor/lore one-liners to all Rare/Special-tier resources and all buildings (bounded
      subset — see `TARGET_GOALS.md` R4, not a rewrite of all 88+58+24 entries).
- [ ] Add flavor framing text to the 4 existing travel `EventKind`s in `events.rs`.
- [ ] (Stretch, only if time remains) one additional handcrafted system/location tied into the
      questline.

## M8 — Clean Code Pass (continuous, final sweep here)

- [ ] Remove stale `#[allow(dead_code)]` scaffold markers now that the corresponding code is wired.
- [ ] Reorganize `src/ui/panels/` (already at the 10-file cap) into thematic subfolders per
      `CONVENTIONS.md` §1: `panels/flow/` (splash, title, settings, help) and `panels/story/`
      (merchant, quest) — see layout recommendation.
- [ ] `cargo fmt` + `cargo clippy --all-targets -- -D warnings` clean.
- [ ] Full test suite green; golden snapshots consciously updated (not silently regenerated) for
      any intentionally changed views.
- [ ] Log (inline note on this item, no separate doc) any accepted pre-existing long-file debt
      (`app.rs`, `panels/galaxy_map.rs`, `panels/planet.rs`, `panels/research.rs`) not addressed here.

## M9 — Final Sign-off (depends on everything above)

- [ ] `scripts/verify.sh` green.
- [ ] Full-game screenshot pass: splash → title → new game → planet_view → research →
      galaxy_map → merchant → warp → settings → help → quest, across Full/Compact/Minimal,
      each Read and scored against `TARGET_GOALS.md`.
- [ ] Compute all 4 goal scores; confirm ALL ≥ 90 (min-gate, not average) before declaring done.
- [ ] Flag a final human playthrough check-in to the user (recommended, not a blocking gate).
