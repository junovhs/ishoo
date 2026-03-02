# BACKLOG Issues

---

## [7] Implement `ishoo delete` CLI command
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`

Users need a way to completely remove an issue from the tracker via the CLI, rather than just marking it as DESCOPED or DONE.
Usage: `ishoo delete <id>`

**Resolution:** 

---

## [5] Add conflict resolution for concurrent edits
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/model/workspace.rs`
**Depends on:** [4]

If the user modifies an issue in the UI (setting `dirty = true`), and an external script modifies the markdown file at the same time, hitting "Save All" currently overwrites the external changes.
We need a basic merge strategy or a warning modal: "The underlying file has changed! Overwrite?"

**Resolution:** 

---

## [8] Switch to AST-based markdown parser
**Status:** OPEN
**Files:** `src/model/parse.rs`

Our current parser (`parse.rs`) is line-based. If a user manually edits the markdown and makes a typo like `*Status:**` (missing an asterisk), the parser completely misses the field.
Refactor `parse_markdown` to use `pulldown-cmark` or YAML Frontmatter to make it much more forgiving for humans to edit manually.

**Resolution:** 

---

## [9] Add global keyboard shortcuts
**Status:** OPEN
**Files:** `src/ui/app.rs`

Power users need keyboard shortcuts for the desktop app to navigate without the mouse:
- `Cmd/Ctrl + N`: Open New Issue Modal
- `/`: Focus topbar search box
- `Cmd/Ctrl + S`: Force "Save All"
- `Esc`: Close modal / collapse active issue card

**Resolution:** 

---
