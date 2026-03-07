# ACTIVE Issues

---

## [21] Add labels/tags system
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

Freeform tags for categorization. Requires updating the parser to extract `**Labels:**` from markdown, storing in `Issue`, and rendering `.label` chips on the UI cards and modal.

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

## [27] Add comments per issue
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`, `src/ui/views/feed/card.rs`

Comments/Notes section in the modal (`.m-comments`). Requires backend parsing to read the `### Comments` markdown blocks into the Issue model first.

**Resolution:** 

---

## [105] UI: Modal Accent Bar & Next/Prev Navigation
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`

The issue modal is missing the top colored accent bar (`.m-accent`), properly styled header layout, and keyboard navigation hints.
Note: The UI HTML layout has been completed. Keyboard `ArrowUp`/`ArrowDown` navigation logic still needs to be implemented.

**Resolution:** 

---

## [57] Feed view lenses: Next Up, Hot Path, Quick Wins
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/app.rs`, `src/model/workspace.rs`

Add toggle pills at the top of the feed (`.lens-row`) for alternative lenses.
Note: The HTML UI buttons have been added to the Topbar. Still requires wiring up sorting functions using existing dependency and heatmap data before rendering the feed.

**Resolution:** 

---
