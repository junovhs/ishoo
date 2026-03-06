# DONE Issues

---

## [47] Fix drag-and-drop state corruption after first reorder
**Status:** DONE
**Files:** `src/ui/views/physics.rs`, `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`

Critical UX bug: drag-and-drop worked on the first attempt but broke progressively on subsequent drags due to stale `item_springs` HashMap entries leaking between drag sessions.

**Resolution:** Four root causes fixed across three files, plus two follow-up fixes identified by manual testing:
1. `physics.rs` — Added `DragState::reset()` that clears `item_springs` and zeroes all spring state atomically. Added `settle_ticks: u32`; `step_settle` hard-caps at 32 ticks (~500 ms) to force-clear springs that hover near the convergence threshold. Raised `done()` threshold 0.5 → 1.0 px.
2. `feed/card.rs` — `onpointerdown` calls `ds.reset()` first, replacing the partial `values_mut()` loop that left stale HashMap keys in place.
3. `feed.rs` — `onpointercancel` calls `ds.reset()` instead of only clearing 2 of ~12 fields.
4. `physics.rs` — `update_rotation`: `rot_spring.target` is now always 0.0 — cards no longer tilt on horizontal movement (matched prototype spec).
5. `feed/card.rs` — `nat_tops` anchored to actual screen coordinates at drag-start: virtual layout positions are shifted so `nat_tops[orig_idx] = start_y`, making slot detection work correctly regardless of CSS height mismatches or section header changes after a reorder.
Tests added in `physics.rs`: `reset_clears_item_springs_completely`, `step_settle_hard_cap_forces_clear_even_when_not_converged`, `second_drag_starts_with_clean_state`.
Verified: `neti check` → clean, clippy PASS, tests PASS. All 3 new tests pass.
Commands: `neti check`

---

## [48] Fix pre-existing neti violations in viz.rs and workspace.rs
**Status:** DONE
**Files:** `src/ui/views/viz.rs`, `src/model/workspace.rs`

7 pre-existing P04 violations (nested loops flagged as quadratic). Rather than suppress with `neti:allow`, the logic was refactored to be genuinely better:
- `viz.rs` — Deleted the `compute_overlaps` / `extract_pairs` pair-enumeration path (O(k²) issue-pairs per file). `GraphView` now calls `shared_file_overlaps` which builds a file→issues map via `flat_map` (O(n)), rendering "file: #A #B #C" instead of "A ⟷ B". More information, less work.
- `workspace.rs` — `file_heatmap` nested for-loops replaced with `flat_map` iterator chain. Removed the stale `neti:allow(P04)` comment that was failing to suppress the violation anyway.

**Resolution:** Refactored to eliminate violations by genuinely improving the algorithms, not suppressing them. `neti check` → clean (0 violations). Clippy ✅ Tests ✅. Commands: `neti check`

---

## [1] Setup initial workspace parsing
**Status:** DONE
**Files:** `src/model/parse.rs`, `src/model/workspace.rs`

Build the core engine to read/write issues from markdown files. Needs to handle custom sections, parse metadata (Status, Files, Depends on), and cleanly separate the Description text from the Resolution text.

**Resolution:** Implemented a robust line-based parser and `Workspace::save` logic that correctly maps Issue structs back to properly formatted Markdown files.

---

## [2] Basic Dioxus desktop UI layout
**Status:** DONE
**Files:** `src/ui/app.rs`, `src/ui/styles.rs`

Create the main shell, sidebar navigation, and a feed view. Ensure it matches a clean, dark-mode aesthetic with DM Sans and JetBrains Mono.

**Resolution:** Built the shell and injected CSS variables for consistent theming across the app.

---

## [3] Implement custom drag-and-drop physics
**Status:** DONE
**Files:** `src/ui/views/physics.rs`, `src/ui/views/feed/card.rs`

Standard HTML5 drag-and-drop feels clunky. Write a custom spring physics engine (stiffness, damping, mass) to make the cards feel tactile, fluid, and fun to reorder.

**Resolution:** Implemented `DragState` with 60fps spring animations for rotation, scaling, and positional snapping. Feels incredibly smooth!

---

## [10] Support SQLite database backend
**Status:** DESCOPED
**Files:** `src/model/mod.rs`

Add support for storing issues in a local `issues.db` SQLite file for faster querying and relationship mapping.

**Resolution:** Descoped. Violates the core philosophy of Ishoo. The whole point is to keep issues as portable, human-readable markdown files that can be version-controlled natively with Git. A database is an anti-goal.

---
