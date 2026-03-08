# ACTIVE Issues

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

## [116] Gotta take a shit
**Status:** OPEN
**Labels:** ux, frontend, compiler, ux, backend, business logic

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

## [61] Project health pulse & Issue Age
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/components.rs`, `src/model/workspace.rs`

Sidebar `.health` pulse and Modal Issue Age. Requires invoking `git log` dynamically to derive sparkline trends and age calculations, which requires a new backend feature.

**Resolution:** 

---
