# BACKLOG Issues

---

## [7] Implement issue deletion via CLI
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`, `src/model/workspace.rs`

Users need `ishoo delete <id>` to permanently remove an issue rather than marking it DESCOPED.
Should prompt for confirmation unless `--force` is passed. After deletion, the issue's ID must never be reused (relevant once [11] lands — the per-category counter must not decrement).

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

## [27] Add comments per issue
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`, `src/ui/views/feed/card.rs`

Comments/Notes section in the modal (`.m-comments`). Requires backend parsing to read the `### Comments` markdown blocks into the Issue model first.

**Resolution:** 

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

## [15] Implement ishoo edit CLI command
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`

Currently the CLI can `new`, `set` (status only), and `show`. There is no way to edit an issue's title, description, resolution, files, or dependencies from the terminal.
`ishoo edit <id>` with no flags opens `$EDITOR` with the issue rendered as markdown, then parses the result back (like `git commit` without `-m`). The editor approach depends on [8] for robust re-parsing.
Also support field-level updates for scripting: `ishoo edit <id> --title "New title" --files "a.rs,b.rs"`.

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

## [12] Add round-trip save/parse tests
**Status:** DESCOPED
**Files:** `src/model/workspace.rs`, `src/model/parse.rs`

There are no tests that verify `parse → mutate → save → parse` produces equivalent results. This is where the real data-loss bugs hide. Specifically:

1. Unknown fields (e.g., a user manually adds `**Priority:** HIGH`) are silently dropped on save because `write_section` only emits known fields
2. Description whitespace and blank lines may not survive a round-trip
3. Section assignment during save is asymmetric — DONE status forces migration to `issues-done.md`, but DESCOPED does not
Write property-based or snapshot tests that:

- Parse sample markdown, save it, parse again, and assert structural equality
- Inject unknown fields and verify they survive (or explicitly document that they won't)
- Mutate status and verify correct file routing

**Resolution:** Descoped. Dumb to make this an issue, lets just do actual mutation testing.

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

## [9] Add global keyboard shortcuts
**Status:** OPEN
**Files:** `src/ui/app.rs`
**Depends on:** [6]

Essential keyboard shortcuts for the desktop app:

- `Cmd/Ctrl + S` — Save All
- `Esc` — Close modal or collapse active card
Note: Dioxus desktop runs in a webview that swallows some OS-level key combinations. Prototype early to identify which bindings actually work before committing to a full set. Expand later based on what's possible.

**Resolution:** 

---

## [31] Status changes move issues between files automatically
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/app.rs`

When an issue's status is changed — via the UI dropdown, the CLI, or the Board view — it should automatically migrate to the appropriate file. DONE and DESCOPED go to `issues-done.md`. Reopening a DONE issue moves it back to `issues-active.md`.
Currently this only happens on explicit "Save All" and only for the DONE→done-file case. Make it consistent and automatic for all status transitions. If arbitrary file names land ([28]), the routing rules should be configurable or at least documented.

**Resolution:** 

---

## [43] Add issue description editing in the UI
**Status:** OPEN
**Files:** `src/ui/views/feed/card.rs`

The description field in the expanded card is a read-only `div`. The resolution field is an editable `textarea`. There is no reason the description shouldn't also be editable — users shouldn't have to open their text editor to update an issue's description after creation.
Add a pencil icon or double-click-to-edit interaction that swaps the description `div` for a `textarea`. Consider a markdown preview toggle (depends on [30]).

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

## [44] Add notification/badge for externally changed issues
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/feed/card.rs`
**Depends on:** [4]

When the file watcher detects external changes, the UI silently refreshes. The user has no idea which issues changed or what changed about them.
After reload, diff the old and new issue lists. For any issue that changed, show a subtle "updated" indicator on the card (e.g., a blue dot that fades after 10 seconds). Optionally show a toast: "3 issues updated externally".

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

## [54] Add issue dependency blocking visualization
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`

The Graph view shows dependency edges, but doesn't highlight blocked chains. If issue [5] depends on [4], and [4] is still OPEN, then [5] is effectively blocked. Visually distinguish:

- Satisfied dependencies (dependency is DONE) — green edge
- Blocking dependencies (dependency is not DONE) — red edge with a "BLOCKED" badge on the dependent issue
The Feed view should also show a small "blocked" indicator on cards whose dependencies aren't met.

**Resolution:** 

---

## [60] Issue clustering by shared files
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`, `src/model/workspace.rs`

Add a view mode (or a tab within Heatmap) that groups issues by file overlap rather than by status or section. Issues that share 2+ files get clustered together.

Output something like:

```
Cluster: parse.rs + workspace.rs
  → [8] AST parser, [12] round-trip tests, [16] preserve unknown fields, [27] comments
Cluster: card.rs + feed.rs
  → [47] drag fix, [14] physics performance, [41] compact mode, [43] description editing
```

This answers "if I'm already in these files, what else can I batch?" Reduces context switching and merge conflict risk.

**Resolution:** 

---

## [58] Bottleneck highlighting in Graph view
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`, `src/model/workspace.rs`
**Depends on:** [54]

After [54] adds blocking visualization (green/red edges), add a "Bottlenecks" mode to the Graph view that highlights the issues with the highest transitive dependent count. The issue whose completion unblocks the most downstream work should visually glow or scale larger.

This answers "what's the single most important thing to do right now" from a pure dependency perspective. Compute transitive dependents by walking the dependency graph — no new data source needed.

**Resolution:** 

---

## [59] Stale issue detection
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

Surface issues that haven't changed status in a long time. Derive staleness from git history on the issue markdown files (`git log --format=%aI -- <file>`) — no new metadata fields needed.

Show a subtle "stale" indicator on cards that haven't been touched in N days (configurable, default 14). The Timeline view could also dim or desaturate stale issues.

This answers "what's stalled and why am I pretending it isn't?"

**Resolution:** 

---

## [63] Velocity visualization in Timeline view
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`, `src/model/workspace.rs`

The Timeline view should show completed issues plotted over time (derived from git history of when issues moved to DONE status). A simple cumulative line or bar chart showing resolved issues per week/month.

Doesn't need to be fancy — even a stepped line that goes up by one each time an issue is resolved. The point is to see momentum. When you're grinding and it feels like nothing's moving, the upward slope is motivating.

**Resolution:** 

---

## [64] Focus mode
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`

Add a "Focus" action on any issue card (click a target icon, or double-click the card). The UI strips away everything except:

- The focused issue, fully expanded
- Its `**Files:**` list with heatmap scores for each
- Its direct dependencies and their status (blocked? done?)
- Issues it unblocks (what opens up when this is done?)

A single-issue view with full context and zero noise. Press Esc or click "Back to Feed" to return.

This answers "I've decided what to work on, now show me everything I need to know about just this one thing."

**Resolution:** 

---

## [62] “Cost” lens for ruthless scoping
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/model/workspace.rs`

Add a Feed lens or a sortable column that shows the "cost" of each issue — computed as the number of files touched weighted by their heatmap scores, plus the number of dependencies.

High-cost issues that are still OPEN and have no dependents (nothing else is waiting on them) are candidates for cutting. Surface them visually — maybe a "heavy" badge or a sort that puts the most expensive, least-blocking issues at the top.

This answers "what should I seriously consider descoping?"

**Resolution:** 

---

## [65] Precompute viz data in model layer
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/views/viz.rs`

The graph and heatmap views currently compute overlap indices, pair intersections, and layout data inline during rendering. This causes the P02/P04 violations in `viz.rs` and will get worse as the issue count grows.

Move all graph computation into `Workspace`:

- `file_overlap_index: HashMap<(IssueId, IssueId), Vec<String>>` — precomputed at load time
- `transitive_dependents: HashMap<IssueId, usize>` — for bottleneck highlighting ([58])
- `issue_heat_score: HashMap<IssueId, usize>` — weighted file touch count for feed lenses ([57])

The viz views become pure consumers of precomputed data. This fixes the current Neti violations and makes all the new lens/sort features cheap to render.

**Resolution:** 

---
