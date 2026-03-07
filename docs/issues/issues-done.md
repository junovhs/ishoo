# DONE Issues

---

## [30] Render markdown in description and resolution fields
**Status:** DONE
**Files:** `src/ui/views/feed.rs`
**Depends on:** [8]

Relies on the AST parser to generate HTML for the modal descriptions, replacing raw text.

**Resolution:** Injected `pulldown_cmark::html::push_html` into a new `render_markdown` helper inside `feed.rs`. Modified `IssueModal` to use Dioxus's `dangerous_inner_html` to emit the parsed HTML string directly into the `.m-body` DOM element. Added rigor tests inside a new `mod tests` block in `feed.rs` checking output tags exactly. Verified via `neti check`.

---

## [8] Switch to AST-based markdown parser
**Status:** DONE
**Files:** `src/model/parse.rs`, `Cargo.toml`

Prerequisite for rendering beautiful markdown (`.m-body`). Moving to `pulldown-cmark`.

**Resolution:** Replaced manual array splits with `pulldown_cmark` event iteration inside `parse_markdown`. Implemented token extraction that consumes `Text[range]` cleanly across paragraph blocks. Tested via `cargo test parse` and passed `neti check`.

---

## [103] UI: Feed Collapsible Sections
**Status:** DONE
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`

The spike groups issues into sections (Active, Backlog, Done) with collapsible headers (`.section`, `.section-head`).

**Resolution:** Implemented visually *without* breaking the absolute positioning math of the physics engine. Injected `.section-head` elements into the Dioxus virtual list, computing index offsets dynamically so they occupy slot space. Added a `collapsed` state signal to filter out cards that are grouped beneath a collapsed `section-head`. Verified the physics math works perfectly by running the app and manually moving items across sections. Verified unit tests via `cargo test` and `neti check`.

---

## [104] UI: Category Color Dots & Keyboard Guide
**Status:** DONE
**Files:** `src/ui/app.rs`, `src/ui/views/feed/card.rs`

1. The issue rows are missing the `.s-dot` color indicators based on status (orange/blue/green).
2. The sidebar is missing the `.kb-ref` keyboard shortcut guide.

**Resolution:** Added the keyboard reference HTML structure to `app.rs`. Added the `.s-dot` and `.xlink` Mock indicators to `card.rs` based on the status styling rules and dependencies list length. Evaluated via `neti check` that UI atomic boundaries are pristine. Tests run: None added as this is pure HTML layout changes. Verified via `cargo test` and manual review.

---

## [41] Add a compact/dense display mode
**Status:** DONE
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

The current card layout is spacious and readable for 10-20 issues but wastes vertical space when you have 50+. Added a toggle between Comfortable and Compact.

**Resolution:** Added an `is_compact` boolean signal to `AppState` and passed it down to `FeedViewProps` so the class could be injected into the root `.feed` container. Connected the button toggles to the signal state in `app.rs`. Verified visually using standard testing to ensure transitions trigger with zero delays. Verified via `cargo test` and `neti check`.

---

## [100] Upgrade UI styling to V2 Spike Layout
**Status:** DONE
**Files:** `assets/style.css`, `src/ui/app.rs`, `src/ui/components.rs`, `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`

Replaces the base UI application with the improved styling logic from the docs/UI Concepts/ui-v2-spike.html spike.

**Resolution:** The task to migrate styling and layout components was completed successfully.
1. `style.css` was fully replaced with the `<style>` block content inside the provided spike file.
2. The core structures inside `app.rs` and `components.rs` were updated to reflect the spike tag layout (`app`, `sb`, `mn` shells, with `.vb` navigation styles).
3. The `.issue-title` and `.issue-sub` tags were assigned standard ellipsis truncation CSS (`white-space: nowrap; overflow: hidden; text-overflow: ellipsis;`) as requested by the user, specifically to preserve a static `54px` card height during window resizing, protecting the stability of the custom physics loop math.
4. `feed.rs` and `feed/card.rs` were safely restructured to embed their content in the new `issue-row` wrapper, leaving the original `onpointerdown` handlers and positioning mechanics identical to what existed before the task.
Verified via `neti check` (no Atomic layers broken), and `cargo check`/`cargo test` generated fully passing returns.

---

## [6] Move CSS to native asset files
**Status:** DONE
**Files:** `assets/style.css`, `src/ui/app.rs`, `src/ui/styles.rs (deleted)`

**Resolution:** Migrated all CSS from Rust string literals into a standard `assets/style.css` file. Used the standard Rust `include_str!` macro to bundle the stylesheet directly into the binary at compile time. This preserves the "single executable" portability and `cargo install` compatibility while allowing for a proper CSS development experience with syntax highlighting and linting.

---

## [14] Fix re-render performance in physics loop
**Status:** DONE
**Files:** `src/ui/views/physics.rs (deleted)`, `src/ui/views/feed.rs`

**Resolution:** Completely replaced the 60fps manual physics simulation loop with a declarative, slot-based absolute positioning system. By using CSS `transition` for the "sucking into well" effect and index-based offsets for displaced cards, we eliminated the need for high-frequency signal updates. The UI is now significantly more performant and the code is much simpler.

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
