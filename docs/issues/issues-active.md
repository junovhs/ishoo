# ACTIVE Issues

---

## [6] Move CSS to native asset files
**Status:** OPEN
**Files:** `src/ui/styles.rs`, `src/ui/styles_viz.rs`, `Dioxus.toml`, `assets/`

Dioxus supports standard CSS files served from an `assets/` directory. There is zero reason to embed 4KB+ of minified CSS inside Rust string literals — it kills syntax highlighting, linting, and auto-completion.

Additionally, the current `@import url()` for Google Fonts (DM Sans, JetBrains Mono) fetches from the network at runtime, which silently degrades to system fonts when offline. This contradicts the local-first philosophy. The font files should be bundled in `assets/fonts/` and loaded via `@font-face`.

Steps:
1. Create `assets/base.css`, `assets/card.css`, `assets/drag.css`, `assets/modal.css`, `assets/viz.css`
2. Move each `pub const` CSS block from `styles.rs` / `styles_viz.rs` into the corresponding file
3. Download DM Sans and JetBrains Mono `.woff2` files into `assets/fonts/`
4. Replace the `@import url()` with local `@font-face` declarations
5. Update `Dioxus.toml` to bundle the `assets/` directory
6. Delete `styles.rs` and `styles_viz.rs`

**Resolution:** 

---

## [4] Replace polling with OS file system events
**Status:** IN PROGRESS
**Files:** `src/ui/app.rs`, `Cargo.toml`

The dashboard uses a 3-second `tokio::time::sleep` loop to poll for external changes. Replace with the `notify` crate for OS-level file system events (FSEvents/inotify/ReadDirectoryChanges).

Note: switching to `notify` alone does NOT fix the race condition in the current poll handler. The `if !dirty() { issues.set(ws.issues); }` check-then-set is not atomic — a user edit between the check and the set gets silently overwritten. This must be addressed alongside the migration (see issue [5]).

**Resolution:** 

---

## [11] Implement categorical issue IDs
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`, `src/ui/views/feed/card.rs`

Replace numeric-only issue IDs with categorical alphanumeric IDs (e.g., `BUG-01`, `FT-12`, `UI-03`, `DX-07`). The current system uses sequential integers which are fragile — deleting the highest-numbered issue causes ID reuse on the next create.

New ID format: `<CATEGORY>-<NUMBER>` where:
- Category is a 1-4 letter uppercase prefix chosen at creation (e.g., BUG, FT, UI, DX, ARCH, PERF)
- Number is zero-padded, monotonically increasing per category, never reused
- A `.ishoo` metadata file (or comment header in each markdown file) tracks the next number per category

This requires updating:
- The `Issue` struct (`id: u32` → `id: String`)
- The parser heading regex (`## [47]` → `## [BUG-47]`)
- All ID comparisons, sorting, and display logic
- The CLI `show`, `set`, and `new` commands to accept string IDs
- The `new` command to accept `--category` or infer from a default

**Resolution:** 

---

## [12] Add round-trip save/parse tests
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/model/parse.rs`

There are no tests that verify `parse → mutate → save → parse` produces equivalent results. This is where the real data-loss bugs hide. Specifically:

1. Unknown fields (e.g., a user manually adds `**Priority:** HIGH`) are silently dropped on save because `write_section` only emits known fields
2. Description whitespace and blank lines may not survive a round-trip
3. Section assignment during save is asymmetric — DONE status forces migration to `issues-done.md`, but DESCOPED does not

Write property-based or snapshot tests that:
- Parse sample markdown, save it, parse again, and assert structural equality
- Inject unknown fields and verify they survive (or explicitly document that they won't)
- Mutate status and verify correct file routing

**Resolution:** 

---

## [13] Prevent silent data loss from discover_root ambiguity
**Status:** OPEN
**Files:** `src/model/mod.rs`

`discover_root` checks 6 candidate directories and silently picks the first match. If a project has both `docs/issues/` and `issues/` (e.g., from a migration or misconfiguration), the user gets zero feedback about which was chosen.

Fix:
- If multiple candidates contain issue files, print a warning listing all matches and which was selected
- Default to the first match but make the choice visible
- The `init` command should print the chosen path explicitly

**Resolution:** 

---

## [14] Fix re-render performance in physics loop
**Status:** OPEN
**Files:** `src/ui/views/physics.rs`, `src/ui/views/feed/card.rs`

`Signal<DragState>` compares by pointer, not by value (DragState doesn't implement PartialEq). This means every physics tick (60fps) triggers a re-render of every `IssueCard`, even cards that aren't moving. With 50+ issues this will be visibly slow.

Options:
- Derive or implement `PartialEq` on `DragState` (complex due to `HashMap<u32, Spring>`)
- Split drag state into per-card signals so only affected cards re-render
- Use CSS transforms driven by a single DOM manipulation pass instead of per-component state

**Resolution:** 

---

## [47] Fix drag-and-drop state corruption after first reorder
**Status:** OPEN
**Files:** `src/ui/views/physics.rs`, `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`

Critical UX bug: drag-and-drop works correctly on the first attempt but breaks progressively on subsequent drags. Cards can be picked up but won't reorder. Each additional attempt makes the behavior worse.

Root cause analysis:
1. After a successful drag, `finalize_drag` sets `settling_id` and the settle animation runs. `step_settle` only clears `item_springs` when ALL springs report `done(0.5)`. Floating point precision means some springs hover near the threshold (`pos = 0.49, vel = 0.002`) — technically "done" but the HashMap entry persists with a non-zero position.

2. On the next drag, `onpointerdown` calls `build_virtual_layout` to compute fresh `nat_tops`, but stale `item_springs` from the previous drag still contain displacement values for card IDs that now have different indices. `compute_styles` reads these stale springs and applies incorrect `translateY` offsets to cards that shouldn't be moving.

3. `update_item_springs` in `step_drag` creates new spring entries but never removes springs for IDs that are no longer in the layout. The HashMap grows monotonically.

4. The `on_reorder` callback in `app.rs` calls `issues.set(all)` which triggers a re-render and rebuilds the DOM order, but the physics state still references the OLD positions. The new DOM positions and the stale spring offsets compound, producing visible jitter.

Fix:
- Clear `item_springs` completely in `onpointerdown` before starting a new drag, not just set targets to 0
- Add an explicit `reset()` method to `DragState` that zeroes ALL transient state
- In `step_settle`, use a more aggressive completion threshold OR add a max settle duration (e.g., 500ms) after which all springs are force-cleared
- After `on_reorder` fires and the issue list is reordered, the drag state must be fully reset before the next frame renders
- Consider whether `settling_id` should block new drags entirely — currently nothing prevents starting a new drag while a settle animation is running, which produces two concurrent animations fighting each other

Secondary concern: the `use_effect` that syncs `issues_for_layout` may lag one frame behind after a reorder, causing `build_virtual_layout` to compute positions from stale data on the first frame of the next drag.

**Resolution:** 

---
