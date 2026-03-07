# ACTIVE Issues

---

## [107] UI Regressions: Drag Snapback, Dark Mode Opacity, Color Dots
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `assets/style.css`

Remaining issues from the V2 Spike integration:
1. **Snapback Bug:** Drag-and-drop snapback is still present. The card jumps to its original slot immediately upon release before moving to the target.
2. **Dark Mode Drag Opacity:** Dragging cards in dark mode is still transparent, allowing lower cards to be seen through them.
3. **Missing Color Dots:** Some issues in the Backlog section are still entirely missing their tracking color dots. These skip rendering somewhat arbitrarily and may be linked to the collapsible section loop logic.

**Resolution:** 

---

## [21] Add labels/tags system
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

Freeform tags for categorization. Requires updating the parser to extract `**Labels:**` from markdown, storing in `Issue`, and rendering `.label` chips on the UI cards and modal.

**Resolution:** 

---

## [27] Add comments per issue
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`, `src/ui/views/feed/card.rs`

Comments/Notes section in the modal (`.m-comments`). Requires backend parsing to read the `### Comments` markdown blocks into the Issue model first.

**Resolution:** 

---

## [57] Feed view lenses: Next Up, Hot Path, Quick Wins
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/app.rs`, `src/model/workspace.rs`

Add toggle pills at the top of the feed (`.lens-row`) for alternative lenses.
Note: The HTML UI buttons have been added to the Topbar. Still requires wiring up sorting functions using existing dependency and heatmap data before rendering the feed.

**Resolution:** 

---

## [102] UI: Dark Mode Toggle & Stats Breakdown
**Status:** OPEN
**Files:** `src/ui/app.rs`

The UI spike includes a dark mode toggle and a clean breakdown of stats.
1. Add the `.dm-toggle` button (`☽`) to the sidebar and implement a click handler to toggle the `dark` class on the `html` element.
2. Restore the Active/Backlog/Done stat breakdown in the sidebar using the existing `stats` signal, but styled with `.mr` and `.v` classes from the spike instead of `.stat-list`.
Pure UI implementation, no backend needed.

**Resolution:** 

---

## [105] UI: Modal Accent Bar & Next/Prev Navigation
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`

The issue modal is missing the top colored accent bar (`.m-accent`), properly styled header layout, and keyboard navigation hints.
Note: The UI HTML layout has been completed. Keyboard `ArrowUp`/`ArrowDown` navigation logic still needs to be implemented.

**Resolution:** 

---

## [33] Add issue linking, mentions, and hover brackets
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/ui/views/feed/card.rs`

Requires parsing `#ID` mentions from markdown text to build a list of `issue.links`.
Once parsed, the UI must implement the `.bracket-svg` hover effect bridging linked issues in the feed, as well as the `.m-links` section in the modal.

**Resolution:** 

---

## [61] Project health pulse & Issue Age
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/components.rs`, `src/model/workspace.rs`

Sidebar `.health` pulse and Modal Issue Age. Requires invoking `git log` dynamically to derive sparkline trends and age calculations, which requires a new backend feature.

**Resolution:** 

---
