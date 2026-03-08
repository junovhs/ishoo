# ACTIVE Issues

---

## [116] Gotta take a shit
**Status:** OPEN
**Labels:** ux, frontend, compiler, ux, backend, business logic

**Resolution:** 

---

## [61] Project health pulse & Issue Age
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/components.rs`, `src/model/workspace.rs`

Sidebar `.health` pulse and Modal Issue Age. Requires invoking `git log` dynamically to derive sparkline trends and age calculations, which requires a new backend feature.

**Resolution:** 

---

## [115] Labels: Reuse the system across views
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `src/ui/views/viz.rs`

Once labels are promoted to first-class UI data, they need consistent rendering across the product rather than being feed-only details.

Requirements:

- Centralize label presentation so multiple views do not drift stylistically
- Reuse the label color system anywhere issue metadata appears
- Ensure future views can consume the same label rendering/model helpers
- Avoid duplicating label formatting logic in each component

**Resolution:** 

---

## [4] Replace polling with OS file system events
**Status:** IN PROGRESS
**Files:** `src/ui/app.rs`, `Cargo.toml`

The dashboard uses a 3-second `tokio::time::sleep` loop to poll for external changes. Replace with the `notify` crate for OS-level file system events (FSEvents/inotify/ReadDirectoryChanges).
Note: switching to `notify` alone does NOT fix the race condition in the current poll handler. The `if !dirty() { issues.set(ws.issues); }` check-then-set is not atomic — a user edit between the check and the set gets silently overwritten. This must be addressed alongside the migration (see issue [5]).

**Resolution:** 

---

## [33] Add issue linking, mentions, and hover brackets
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/ui/views/feed/card.rs`
**Labels:** core, frontend, ux, cli

Requires parsing `#ID` mentions from markdown text to build a list of `issue.links`.
Once parsed, the UI must implement the `.bracket-svg` hover effect bridging linked issues in the feed, as well as the `.m-links` section in the modal.

**Resolution:** 

---

## [105] UI: Modal Accent Bar & Next/Prev Navigation
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`
**Labels:** polish

The issue modal is missing the top colored accent bar (`.m-accent`), properly styled header layout, and keyboard navigation hints.
Note: The UI HTML layout has been completed. Keyboard `ArrowUp`/`ArrowDown` navigation logic still needs to be implemented.

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
