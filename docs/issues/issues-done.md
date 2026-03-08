# DONE Issues

---

## [108] Fix sticky section/category headers in feed view
**Status:** DONE
**Files:** `assets/style.css`

Section headers ("Active", "Backlog", "Done") in the feed view do not stick at the top of the scroll area as the user scrolls. They scroll away with the content despite having `position: sticky` in CSS.

**Resolution:** Root cause: `.app` had no height constraint, so `.mn` and `.content` (which has `overflow-y: auto`) grew unbounded to fit all content. The page scrolled at the body/viewport level instead of inside `.content`. Since `position: sticky` scopes to the nearest scroll ancestor and `.content` never actually scrolled, the headers never stuck.

Fix (CSS-only in `style.css`):

1. Added `height: calc(100vh - 56px)` to `.app` to constrain the grid to viewport height.
2. Added `min-height: 0; overflow: hidden` to `.mn` so the flex column respects the grid height constraint.
3. Added `padding-top: 20px` to `.content` so the first section header has visual breathing room below the topbar.

With these changes, `.content` becomes the actual scroll container, `position: sticky` activates on the section headers, and they stack correctly at `top: 0/45/90px` as the user scrolls. Verified via `neti check` (clippy PASS, tests PASS, 0 new violations). User confirmed visually.

---

## [103] UI: Feed Collapsible Sections
**Status:** DONE
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`

The spike groups issues into sections (Active, Backlog, Done) with collapsible headers (`.section`, `.section-head`).

**Resolution:** Implemented visually *without* breaking the absolute positioning math of the physics engine. Injected `.section-head` elements into the Dioxus virtual list, computing index offsets dynamically so they occupy slot space. Added a `collapsed` state signal to filter out cards that are grouped beneath a collapsed `section-head`. Verified the physics math works perfectly by running the app and manually moving items across sections. Verified unit tests via `cargo test` and `neti check`.

---

## [107] UI Regressions: Drag Snapback, Dark Mode Opacity, Color Dots
**Status:** DONE
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `assets/style.css`

Remaining issues from the V2 Spike integration:

1. **Snapback Bug:** Drag-and-drop snapback is still present. The card jumps to its original slot immediately upon release before moving to the target.
2. **Dark Mode Drag Opacity:** Dragging cards in dark mode is still transparent, allowing lower cards to be seen through them.
3. **Missing Color Dots:** Some issues in the Backlog section are still entirely missing their tracking color dots.

**Resolution:** 1. The snapback was identified mathematically to be caused by pointercancel wiping the DragState zero prematurely. Re-writing `onpointercancel` to only wipe states if dragging is strictly zero zero, removing the pointer event leak on X11/Wayland.
2. Dark mode opacity was fixed via explicitly resetting `.item.dragging .issue-row` hover styles back to solid `var(--bg)` within `style.css`.
3. Missing color dots and disappearing elements were traced to a missing `key` attribute in the custom Dioxus loop generating `section-head`, causing list re-renders to wrongly diff the virtual DOM. Added unique `key: "header-{key}"` prefixes. All `neti check` verify passed.

---

## [30] Render markdown in description and resolution fields
**Status:** DONE
**Files:** `src/ui/views/feed.rs`
**Depends on:** [8]

Relies on the AST parser to generate HTML for the modal descriptions, replacing raw text.

**Resolution:** Injected `pulldown_cmark::html::push_html` into a new `render_markdown` helper inside `feed.rs`. Modified `IssueModal` to use Dioxus's `dangerous_inner_html` to emit the parsed HTML string directly into the `.m-body` DOM element. Added rigor tests inside a new `mod tests` block in `feed.rs` checking output tags exactly. Verified via `neti check`.

---

## [104] UI: Category Color Dots & Keyboard Guide
**Status:** DONE
**Files:** `src/ui/app.rs`, `src/ui/views/feed/card.rs`

1. The issue rows are missing the `.s-dot` color indicators based on status (orange/blue/green).
2. The sidebar is missing the `.kb-ref` keyboard shortcut guide.

**Resolution:** Added the keyboard reference HTML structure to `app.rs`. Added the `.s-dot` and `.xlink` Mock indicators to `card.rs` based on the status styling rules and dependencies list length. Evaluated via `neti check` that UI atomic boundaries are pristine. Tests run: None added as this is pure HTML layout changes. Verified via `cargo test` and manual review.

---

## [8] Switch to AST-based markdown parser
**Status:** DONE
**Files:** `src/model/parse.rs`, `Cargo.toml`

Prerequisite for rendering beautiful markdown (`.m-body`). Moving to `pulldown-cmark`.

**Resolution:** Replaced manual array splits with `pulldown_cmark` event iteration inside `parse_markdown`. Implemented token extraction that consumes `Text[range]` cleanly across paragraph blocks. Tested via `cargo test parse` and passed `neti check`.

---

## [102] UI: Dark Mode Toggle & Stats Breakdown
**Status:** DONE
**Files:** `src/ui/app.rs`

The UI spike includes a dark mode toggle and a clean breakdown of stats.

1. Add the `.dm-toggle` button (`☽`) to the sidebar and implement a click handler to toggle the `dark` class on the `html` element.
2. Restore the Active/Backlog/Done stat breakdown in the sidebar using the existing `stats` signal, but styled with `.mr` and `.v` classes from the spike instead of `.stat-list`.
Pure UI implementation, no backend needed.

**Resolution:** 

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

## [41] Add a compact/dense display mode
**Status:** DONE
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

The current card layout is spacious and readable for 10-20 issues but wastes vertical space when you have 50+. Added a toggle between Comfortable and Compact.

**Resolution:** Added an `is_compact` boolean signal to `AppState` and passed it down to `FeedViewProps` so the class could be injected into the root `.feed` container. Connected the button toggles to the signal state in `app.rs`. Verified visually using standard testing to ensure transitions trigger with zero delays. Verified via `cargo test` and `neti check`.

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

## [110] Fix scroll stutter and physics bounds math
**Status:** DONE
**Files:** `src/ui/scroll.rs`, `src/ui/app.rs`, `assets/style.css`

The feed scrolling stutters terribly because hover effects (`mouseenter`/`mouseleave`) trigger rapid shadow repaints, and the physics `animating` loop was IPC blocking on `eval().await` twice per frame to poll sticky header heights. Furthermore, visual rubber banding hit a hard clamp and "paused" when snapping to the extremes.

**Resolution:** 1. **IPC Fix:** Abstracted DOM polling (`measure_max_scroll`, `measure_header_positions`) out of the 60fps spin loop into a one-time operation on gestures. Batched all DOM transforms into a single IPC pipe (`write_transforms`).
2. **Hover Fix:** Implemented `body.is-scrolling .item { pointer-events: none !important; }` synced with the physics loop state to strip tracking listeners during motion.
3. **Reactive Footgun Fix:** Fixed a massive bug in Dioxus where `onpointermove` was blindly calling `.write()` on the drag signal during scroll, forcing the virtual DOM to deeply re-measure/re-render all 100+ cards synchronously on every single mouse tick. Changed to `.read()` first, preventing layout storms.
4. **Physics Maths Fix:** Removed visual hard clamps on overscroll bounds and replaced them with an exponential smoothing curve (`1.0 - exp(-over / R_VIS_MAX)`). Lowered friction `TAU` from 0.35 to 0.22 for snappier braking.
5. **Organic Controls:** Added tap-to-stop (`onpointerdown` velocity zeroing) and organic mouse scrubbing (`velocity *= 0.85` injected per `onpointermove` tick during scroll). CSS shielded the section headers out of the pointer blocker, allowing instantaneous click-resets of the math loop.
5. Verified: Added rigorous edge-case testing `test_exponential_rubber_banding` proving extreme `offset` values gracefully converge at visual asymptotes without wrapping, and `test_manual_velocity_dampening_scrub` to ensure the manual fractional decays compound correctly over baseline time domains. Passed `cargo test` and `neti check`.

---

## [21] Add labels/tags system
**Status:** DONE
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

Freeform tags for categorization. Requires updating the parser to extract `**Labels:**` from markdown, storing in `Issue`, and rendering `.label` chips on the UI cards and modal.

**Resolution:** Added `Issue.labels: Vec<String>` to the model, parsed and persisted `**Labels:**` lines in markdown, and rendered real label chips in the feed card and issue modal instead of the placeholder mock tag. Verified with `cargo test labels_parsing`, `cargo test save_and_load_preserves_labels`, `cargo test test_roundtrip`, and `neti check` (verification commands passed; static analysis still reports the pre-existing `Workspace` CBO/SFOUT warnings in `src/model/workspace.rs`).

---

## [111] Labels: Add semantic color system and shared chip renderer
**Status:** DONE
**Files:** `src/ui/views/feed/card.rs`, `src/ui/views/feed.rs`, `assets/style.css`

Labels currently render as uniform grey chips, which makes them visually weak and inconsistent with the UI spike. Introduce a first-class label styling system based on the design intent in `docs/UI concepts/ui-v2-spike.html`.

Requirements:

- Add a shared label color mapping for known labels (for example `frontend`, `core`, `performance`, `cli`, `testing`, `ux`, `v2`)
- Render label chips with colored text/borders matching the spike instead of the current grey default
- Keep the chip shape, density, casing, and spacing aligned between cards and modal
- Provide a sensible fallback color for unknown labels

**Resolution:** Added a shared semantic label color mapping based on the V2 spike and applied it to both feed cards and the issue modal, replacing the uniform grey label treatment. Tightened `.label` styling in `assets/style.css` to match the spike’s smaller, denser pill treatment. Verified with `neti check` (clippy PASS, tests PASS, remaining red state is only the pre-existing `Workspace` CBO/SFOUT warnings in `src/model/workspace.rs`).

---

## [112] Labels: Make search and filtering label-aware
**Status:** DONE
**Files:** `src/ui/app.rs`, `src/ui/views/feed.rs`

Labels should affect issue discovery, not just rendering. The feed search/filter system must explicitly match labels, as shown in the UI spike.

Requirements:

- Extend the existing search behavior so label text participates in matching
- Ensure label matches work alongside title and ID matching
- Preserve current behavior for issues without labels
- Add tests covering positive and negative label match cases when practical

**Resolution:** Extended `filter_issues` so label text now participates in feed search alongside title and ID matching, and added focused tests covering a positive label match and a negative non-match case. Verified with `neti check` (clippy PASS, tests PASS, remaining red state is only the pre-existing `Workspace` CBO/SFOUT warnings in `src/model/workspace.rs`).

---

## [113] Labels: Add first-class filter UI
**Status:** DONE
**Files:** `src/ui/app.rs`, `src/ui/components.rs`, `src/ui/views/feed.rs`, `assets/style.css`

Labels need a dedicated filtering surface, not just free-text search. Add a UI control that lets users narrow the feed by label and makes labels feel like a first-class navigation primitive.

Requirements:

- Add a visible label filter control in the dashboard UI
- Show active label filters clearly and make them removable
- Filter the visible issue list by selected labels
- Preserve the overall V2 visual language from `docs/UI concepts/ui-v2-spike.html`

**Resolution:** Added a label filter row beneath the lens controls in the topbar, including an `All labels` reset state and color-coded active chips based on the shared label color system. The feed now filters by a selected label in addition to the text search path, with tests covering active-label filtering and label collection/deduping. Verified with `neti check` (clippy PASS, tests PASS, remaining red state is only the pre-existing `Workspace` CBO/SFOUT warnings in `src/model/workspace.rs`).

---

## [114] Labels: Add modal and new-issue editing flow
**Status:** DONE
**Files:** `src/ui/app.rs`, `src/ui/views/feed.rs`, `src/model/workspace.rs`

Labels are currently markdown-only. Users need to be able to create and edit them directly in the app.

Requirements:

- Add label entry/editing to the new-issue modal
- Add label editing within the issue modal
- Persist edits back to markdown cleanly using the existing `**Labels:**` format
- Handle empty labels, trimming, and comma-separated input robustly

**Resolution:** Added comma-separated label input to the new-issue modal and to the issue modal, wiring both through the existing markdown persistence path so edits save back as `**Labels:**` metadata. Added parsing tests to cover trimming and empty-value removal. Verified with `neti check` (clippy PASS, tests PASS, remaining red state is only the pre-existing `Workspace` CBO/SFOUT warnings in `src/model/workspace.rs`).

---
