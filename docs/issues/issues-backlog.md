# BACKLOG Issues

---

## [5] Add conflict resolution for concurrent edits
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/model/workspace.rs`
**Depends on:** [4]

If the user modifies an issue in the UI (`dirty = true`) and an external process modifies the markdown simultaneously, "Save All" overwrites the external changes with no warning.

The current poll handler also has an internal race: the `if !dirty()` check and `issues.set()` are not atomic, so a user edit between those two calls is silently dropped even without external interference.

Resolution should include:
- Content hash or generation counter comparison before overwriting
- A warning modal: "The file has changed on disk. Overwrite / Reload / Merge?"
- Optionally, per-issue dirty tracking instead of a single global `dirty` flag

**Resolution:** 

---

## [8] Switch to AST-based markdown parser
**Status:** OPEN
**Files:** `src/model/parse.rs`

The line-based parser breaks on minor formatting variations (e.g., `*Status:**` with a missing asterisk, or extra blank lines inside a field). It also cannot preserve unknown fields through a parse-save round-trip.

Migrate to `pulldown-cmark` or a YAML frontmatter approach. This would:
- Make parsing robust against human typos
- Enable round-tripping of unknown/custom fields
- Simplify the accumulator state machine
- Potentially support richer description content (inline code blocks, lists, etc.)

This is the highest-impact backlog item. Prioritize over [4].

**Resolution:** 

---

## [7] Implement issue deletion via CLI
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`, `src/model/workspace.rs`

Users need `ishoo delete <id>` to permanently remove an issue rather than marking it DESCOPED.

Should prompt for confirmation unless `--force` is passed. After deletion, the issue's ID must never be reused (relevant once [11] lands — the per-category counter must not decrement).

**Resolution:** 

---

## [9] Add global keyboard shortcuts
**Status:** OPEN
**Files:** `src/ui/app.rs`
**Depends on:** [6]

Power-user keyboard shortcuts for the desktop app:
- `Cmd/Ctrl + N` — Open New Issue modal
- `/` — Focus search box
- `Cmd/Ctrl + S` — Save All
- `Esc` — Close modal or collapse active card
- `J/K` — Navigate up/down in feed

Note: Dioxus desktop runs in a webview that swallows some OS-level key combinations. Prototype early to identify which bindings actually work before committing to a full set.

**Resolution:** 

---

## [15] Implement ishoo edit CLI command
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`

Currently the CLI can `new`, `set` (status only), and `show`. There is no way to edit an issue's title, description, resolution, files, or dependencies from the terminal.

Options:
- `ishoo edit <id> --title "New title" --files "a.rs,b.rs"` for field-level updates
- `ishoo edit <id>` with no flags opens `$EDITOR` with the issue rendered as markdown, then parses the result back (like `git commit` without `-m`)

The editor approach is more powerful but depends on [8] for robust re-parsing.

**Resolution:** 

---

## [16] Preserve unknown markdown fields through save
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/model/workspace.rs`
**Depends on:** [8]

If a user manually adds `**Priority:** HIGH` or `**Assignee:** @alice` to an issue, `write_section` silently drops it because it only emits known fields. This is destructive and violates the "your markdown, your rules" philosophy.

After [8] lands (AST parser), the parser should capture unknown `**Key:** Value` pairs into a `HashMap<String, String>` on the Issue struct, and `write_section` should emit them back.

**Resolution:** 

---

## [17] Git branch awareness and per-branch issue views
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/ui/app.rs`

Ishoo lives in the repo but has zero awareness of Git. Since issue files are version-controlled, switching branches already changes the issue state on disk — but the UI doesn't reflect this meaningfully.

Features:
- Detect the current Git branch and display it in the sidebar
- Optionally filter the feed to show only issues whose `Files` overlap with the current branch's diff (i.e., "issues relevant to what I'm working on right now")
- Show a warning if the user tries to edit issues while on a detached HEAD or during a rebase

**Resolution:** 

---

## [18] Implement issue templates
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/main.rs`, `src/ui/app.rs`

Every new issue starts completely blank. Teams develop patterns — bug reports need reproduction steps, feature requests need acceptance criteria, refactors need risk assessments.

Add a `docs/issues/templates/` directory containing markdown templates like `bug.md`, `feature.md`, `refactor.md`. The `ishoo new` command and the UI modal should offer template selection. Templates are just markdown with placeholder tokens like `{{description}}` that get filled in.

**Resolution:** 

---

## [19] Multi-select and bulk status changes
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

There is no way to select multiple issues and change their status at once. If you close out a sprint and need to mark 8 issues as DONE, you click each one individually, expand it, change the dropdown, repeat.

Add:
- Shift+click or checkbox-based multi-select in the Feed view
- A floating action bar that appears when 2+ issues are selected: "Set status → [Open | In Progress | Done | Descoped]"
- `ishoo set 11,12,13,14 done` for bulk CLI updates

**Resolution:** 

---

## [20] Add priority field
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`, `src/ui/views/feed/card.rs`

Issues currently have no concept of priority. Everything is implicitly equal, and ordering is determined only by position in the file.

Add a `**Priority:**` field with values: `Critical`, `High`, `Normal`, `Low`. Display as a colored indicator on the card. Allow sorting by priority in the feed. The CLI `list` command should support `--sort priority`.

This is a "known field" so it can be implemented before [16] (unknown field preservation).

**Resolution:** 

---

## [21] Add labels/tags system
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

Issues need lightweight categorization beyond status and priority. A `**Labels:**` field with comma-separated tags (e.g., `frontend, performance, v2`) would enable:
- Filtering the feed by label
- Color-coded label chips on cards
- CLI filtering: `ishoo list --label performance`
- Label-based grouping in the Board view (columns per label instead of per status)

Labels should be freeform strings, not from a fixed set.

**Resolution:** 

---

## [22] Implement assignee field and filtering
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/app.rs`

For teams using Ishoo in a shared repo, issues need an `**Assignee:**` field. Even for solo developers, it's useful to distinguish "me" from "waiting on someone else."

Format: `**Assignee:** @alice, @bob`

The sidebar should show a "My Issues" filter. The CLI should support `ishoo list --assignee alice`.

**Resolution:** 

---

## [23] Add created/modified timestamps
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`

Issues have no temporal metadata. You can't answer "when was this filed?" or "how long has this been open?" or sort by recency.

Add `**Created:**` and `**Modified:**` fields with ISO 8601 dates. `Created` is set once at creation time. `Modified` updates on every save that touches the issue. The Timeline view should use these for actual chronological ordering instead of the current status-sorted list.

Consider: should these be auto-managed by Ishoo (risk of noisy git diffs), or manually set (risk of being wrong)?

**Resolution:** 

---

## [24] Implement search across descriptions and resolutions
**Status:** OPEN
**Files:** `src/ui/app.rs`

The current search filter only matches on `title` and `id`. It should also search `description`, `resolution`, `files`, and `labels` (once [21] lands). The CLI `list --filter` already searches descriptions, so this is a UI-only gap.

Additionally, the search should support structured queries like `status:open files:parse.rs` for power users.

**Resolution:** 

---

## [25] Export to GitHub/GitLab issues
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/main.rs`

Ishoo is great for local tracking, but at some point a project may outgrow it or need to share issues with non-technical stakeholders. Add an export command:

- `ishoo export github` — generates a JSON payload compatible with GitHub's issue import API
- `ishoo export gitlab` — same for GitLab
- `ishoo export csv` — flat CSV for spreadsheet users

This is a one-way operation. Import from GitHub/GitLab would be a separate issue.

**Resolution:** 

---

## [26] Import from GitHub/GitLab issues
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/main.rs`

The inverse of [25]. Teams migrating from GitHub Issues or GitLab should be able to bootstrap their Ishoo tracker:

- `ishoo import github --repo owner/repo --token <PAT>` — pulls open issues, maps labels to Ishoo labels, creates markdown files
- Should handle pagination for repos with hundreds of issues
- Map GitHub milestones to Ishoo sections or a custom field

**Resolution:** 

---

## [27] Add a comment/activity log per issue
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`, `src/ui/views/feed/card.rs`

Issues currently have a description (immutable context) and a resolution (final outcome), but no way to log intermediate progress, decisions, or blockers over time.

Add a `### Log` subsection under each issue with timestamped entries. The UI should render these as a chronological feed within the expanded card, with an input box to append new entries. Each entry gets an auto-timestamp.

**Resolution:** 

---

## [28] Support arbitrary issue file names
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/workspace.rs`

The three-file structure (`issues-active.md`, `issues-backlog.md`, `issues-done.md`) is mostly hardcoded. But the app already parses the `# HEADING` at the top of each file as the section name, so the file name is nearly irrelevant.

Change `Workspace::load` to scan for all `issues-*.md` files in the directory instead of only the three hardcoded names. On save, write each issue back to whichever file it was loaded from (tracked via a `source_file` field on Issue). The only special-case routing is DONE/DESCOPED issues, which always go to `issues-done.md`.

This means users can create `issues-sprint-42.md`, `issues-frontend.md`, `issues-tech-debt.md` — whatever they want. No config file needed. The file is the config.

If a new issue is created and has no source file, default to `issues-active.md`.

**Resolution:** 

---

## [29] Dark/light theme toggle
**Status:** OPEN
**Files:** `src/ui/styles.rs`, `src/ui/app.rs`
**Depends on:** [6]

The app is dark-mode only. Some users work in bright environments or simply prefer light themes. After CSS moves to asset files ([6]), add a `:root` / `[data-theme="light"]` variable swap and a toggle button in the sidebar.

The theme preference should persist locally. Consider using the OS/desktop theme preference as the default.

**Resolution:** 

---

## [30] Render markdown in description and resolution fields
**Status:** OPEN
**Files:** `src/ui/views/feed/card.rs`
**Depends on:** [8]

Descriptions and resolutions are displayed as raw text via `white-space: pre-wrap`. Any markdown formatting the user writes (bold, code blocks, links, lists) is shown literally rather than rendered.

After [8] provides a proper markdown AST, render these fields as formatted HTML in the card body. The resolution textarea should ideally become a split-pane or toggle between edit and preview modes.

**Resolution:** 

---

## [31] Add drag-and-drop between Board columns
**Status:** OPEN
**Files:** `src/ui/views/board.rs`, `src/ui/views/physics.rs`
**Depends on:** [47]

The Board view is read-only. Cards display in status columns but cannot be dragged between them. The Feed view has a full physics engine for reordering — extend or adapt it so dragging a card from "Open" to "In Progress" in the Board view triggers a status change.

Depends on [47] because the drag system must be stable before extending it.

**Resolution:** 

---

## [32] Implement undo/redo for UI actions
**Status:** OPEN
**Files:** `src/ui/app.rs`

There is no undo. If you accidentally set an issue to DESCOPED, change its resolution, or reorder the feed, the only recovery is to not save (if you catch it) or git-revert (if you don't).

Implement a simple action stack:
- Every mutation (status change, reorder, resolution edit, create, delete) pushes an inverse action onto the undo stack
- `Cmd/Ctrl + Z` pops and applies the inverse
- `Cmd/Ctrl + Shift + Z` redoes
- The undo stack clears on save (since the file is now the source of truth)

**Resolution:** 

---

## [33] Add issue linking and mentions
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/ui/views/feed/card.rs`

The `**Depends on:**` field captures blocking relationships, but there's no way to express softer links: "related to", "duplicates", "superseded by".

More importantly, if a description or resolution mentions `#14` or `[14]`, it should render as a clickable link that navigates to that issue in the UI. The Graph view should pick up these informal references as edges.

**Resolution:** 

---

## [34] Offline-capable auto-update mechanism
**Status:** OPEN
**Files:** `Cargo.toml`, `src/main.rs`

As a standalone binary, Ishoo has no update channel. Users won't know when new versions are available. Add:
- `ishoo --version` (already possible via clap, but not wired up)
- `ishoo update` — checks GitHub releases for a newer version and self-updates the binary
- A subtle "update available" indicator in the sidebar that checks once per day (cached, non-blocking, skipped when offline)

Use `self_update` crate or similar.

**Resolution:** 

---

## [36] Validate and lint issue files
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/parse.rs`

There is no way to check whether the issue markdown files are well-formed without loading the full UI. Add:
- `ishoo lint` — parses all issue files and reports warnings: duplicate IDs, broken dependency references (depends on an ID that doesn't exist), missing required fields, empty titles
- `ishoo lint --strict` — treats warnings as errors (useful for CI)

This enables a pre-commit hook: `ishoo lint --strict || exit 1`

**Resolution:** 

---

## [37] Add CI/pre-commit hook integration
**Status:** OPEN
**Files:** `src/main.rs`, `docs/`
**Depends on:** [36]

Provide documentation and a ready-made pre-commit hook config that runs `ishoo lint --strict` before every commit. This catches:
- Duplicate issue IDs introduced by a bad merge
- Dangling dependency references
- Issues left in IN PROGRESS on a branch that's being merged to main

Also consider a GitHub Action / GitLab CI template that runs `ishoo lint` and posts a summary comment on PRs showing which issues were modified.

**Resolution:** 

---

## [38] Attachment and screenshot support
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/views/feed/card.rs`

Bug reports often need screenshots or log file snippets. Add an `**Attachments:**` field that references files in a `docs/issues/attachments/` directory.

The UI should render image attachments inline and provide a drop zone for adding new files (Dioxus.toml already has `enable_file_drop = true` but it's unused).

**Resolution:** 

---

## [39] Add issue archival beyond DONE
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/main.rs`

Over time, `issues-done.md` will grow unboundedly. A project with hundreds of completed issues will have a massive done file that slows down parsing and clutters the UI.

Add an archive mechanism:
- `ishoo archive` — moves all DONE/DESCOPED issues older than N days (configurable) to `issues-archive-2026-Q1.md`
- Archived issues are excluded from the default UI views but still searchable via `ishoo search --archive`
- The archive files follow the same markdown format so they remain human-readable

**Resolution:** 

---

## [40] Support multiple issue directories per monorepo
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/ui/app.rs`

In a monorepo, different packages may want their own issue trackers. `ishoo` should be able to discover and aggregate issues across multiple subdirectories, with package-scoped views in the UI. `ishoo list --scope auth` would filter to one package. The Board and Heatmap views should support cross-package analysis.

**Resolution:** 

---

## [41] Add a compact/dense display mode
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

The current card layout is spacious and readable for 10-20 issues but wastes vertical space when you have 50+. Add a toggle between:
- **Comfortable** (current) — full card with padding, badges, description preview
- **Compact** — single-line rows with ID, title, status dot, and file count, similar to `ishoo list` CLI output

The toggle should be a small button in the topbar.

**Resolution:** 

---

## [42] Protect against data loss on crash during save
**Status:** OPEN
**Files:** `src/model/workspace.rs`

`write_section` calls `fs::write` directly. If the process crashes or is killed mid-write (e.g., laptop lid close, OOM kill), the file is truncated and all issues in that section are lost.

Fix:
- Write to a temporary file in the same directory (`issues-active.md.tmp`)
- `fsync` the temp file
- Atomically rename the temp file to the target name
- On startup, detect and clean up orphaned `.tmp` files

**Resolution:** 

---

## [43] Add issue description editing in the UI
**Status:** OPEN
**Files:** `src/ui/views/feed/card.rs`

The description field in the expanded card is a read-only `div`. The resolution field is an editable `textarea`. There is no reason the description shouldn't also be editable — users shouldn't have to open their text editor to update an issue's description after creation.

Add a pencil icon or double-click-to-edit interaction that swaps the description `div` for a `textarea`. Consider a markdown preview toggle (depends on [30]).

**Resolution:** 

---

## [44] Add notification/badge for externally changed issues
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/feed/card.rs`
**Depends on:** [4]

When the file watcher detects external changes, the UI silently refreshes. The user has no idea which issues changed or what changed about them.

After reload, diff the old and new issue lists. For any issue that changed, show a subtle "updated" indicator on the card (e.g., a blue dot that fades after 10 seconds). Optionally show a toast: "3 issues updated externally".

**Resolution:** 

---

## [45] Add ishoo stats CLI command
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`

The `list` command shows a one-line summary, but there's no dedicated stats view. Add:
- `ishoo stats` — prints a full dashboard: total issues by status, issues per section, top 5 hotspot files, dependency chain depths, average issues per file
- `ishoo stats --json` — machine-readable output for scripts and CI dashboards

**Resolution:** 

---

## [46] Cross-platform path handling and line ending audit
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/workspace.rs`

Development is on Pop!_OS Linux and Windows 11. PathBuf operations should be cross-platform, but there are untested risks:
- Backslash vs forward slash in `**Files:**` field values parsed on Windows vs displayed on Linux
- Case sensitivity differences (Windows NTFS is case-insensitive, Linux ext4 is not) — could cause duplicate file entries in the heatmap
- Line ending normalization: if a Windows user commits with CRLF and a Linux user opens the same file, does the parser handle `\r\n` correctly? The `lines()` iterator strips `\r` but the `accumulate_text` function may re-introduce inconsistencies
- Long path issues on Windows (>260 chars without the `\\?\` prefix)

Add integration tests that exercise `init → new → save → load` with both forward and backslash paths. Test with CRLF input.

**Resolution:** 

---

## [48] Add issue count badges per section in sidebar
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/components.rs`

The sidebar shows global stats (Backlog, In Flight, Resolved) but doesn't break down counts per section. When using custom file names ([28]), users need to see at a glance how many issues are in each section. Add small count badges next to each section name in the sidebar navigation or in a collapsible section list.

**Resolution:** 

---

## [49] Add due dates and overdue highlighting
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/views/feed/card.rs`
**Depends on:** [23]

Add an optional `**Due:**` field with an ISO 8601 date. In the UI, issues past their due date should show a red "overdue" badge. The Timeline view should plot issues on a calendar-like axis. The CLI should support `ishoo list --overdue`.

This depends on [23] (timestamps) since the infrastructure for date parsing and display is shared.

**Resolution:** 

---

## [50] Add spent/estimated time tracking
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/views/feed/card.rs`

Add optional `**Estimate:**` and `**Spent:**` fields in human-readable format (e.g., `2h`, `3d`, `1w`). Display in the expanded card. The Timeline view should show a burndown chart if enough issues have estimates. The stats command should report total estimated vs spent.

**Resolution:** 

---

## [51] Implement issue pinning
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/model/mod.rs`

Allow pinning issues to the top of the feed regardless of section or status. Useful for "always visible" reminders or the current sprint focus. A `**Pinned:** true` field in the markdown, toggled by a pin icon on the card header.

**Resolution:** 

---

## [52] Add a global activity feed / changelog
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/app.rs`

There is no way to see "what changed recently" across all issues. Add an Activity view (or sidebar panel) that shows a reverse-chronological feed of all mutations: "Issue #14 status changed from OPEN to IN PROGRESS", "Issue #22 created", "Issue #3 resolution updated".

This could be derived from git history on the issue files (`git log --oneline docs/issues/`) rather than maintained as separate state, keeping it zero-config.

**Resolution:** 

---

## [53] Add markdown table of contents generation
**Status:** OPEN
**Files:** `src/model/workspace.rs`

Each issue file should optionally auto-generate a table of contents at the top, listing all issue IDs and titles with anchor links. This makes the raw markdown files much more navigable when viewed on GitHub/GitLab without Ishoo installed.

Format:
```markdown
<!-- TOC -->
- [1] Setup initial workspace parsing
- [2] Basic Dioxus desktop UI layout
<!-- /TOC -->
```

Regenerated on every save. Content between the TOC markers is replaced; if markers don't exist, skip generation.

**Resolution:** 

---

## [54] Add issue dependency blocking visualization
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`

The Graph view shows dependency edges, but doesn't highlight blocked chains. If issue [5] depends on [4], and [4] is still OPEN, then [5] is effectively blocked. Visually distinguish:
- Satisfied dependencies (dependency is DONE) — green edge
- Blocking dependencies (dependency is not DONE) — red edge with a "BLOCKED" badge on the dependent issue

The Feed view should also show a small "blocked" indicator on cards whose dependencies aren't met.

**Resolution:** 

---

## [55] Support issue sub-tasks / checklists
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/views/feed/card.rs`

Large issues often break down into smaller steps. Support markdown checklists within the description:

```markdown
- [x] Write the parser
- [ ] Add tests
- [ ] Update docs
```

Parse these into a structured list. Display as interactive checkboxes in the UI. Show completion progress (e.g., "2/3") on the collapsed card header.

**Resolution:** 

---

## [56] Add right-click context menu on cards
**Status:** OPEN
**Files:** `src/ui/views/feed/card.rs`, `src/ui/views/board.rs`

Currently all actions on an issue require expanding the card. Add a right-click context menu with quick actions:
- Set status (submenu)
- Copy issue ID
- Copy issue as markdown
- Delete issue
- Pin/unpin (if [51] lands)

**Resolution:** 

---
