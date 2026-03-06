# BACKLOG Issues

---

## [4] Replace polling with OS file system events
**Status:** IN PROGRESS
**Files:** `src/ui/app.rs`, `Cargo.toml`

The dashboard uses a 3-second `tokio::time::sleep` loop to poll for external changes. Replace with the `notify` crate for OS-level file system events (FSEvents/inotify/ReadDirectoryChanges).
Note: switching to `notify` alone does NOT fix the race condition in the current poll handler. The `if !dirty() { issues.set(ws.issues); }` check-then-set is not atomic — a user edit between the check and the set gets silently overwritten. This must be addressed alongside the migration (see issue [5]).

**Resolution:** 

---

## [6] Move CSS to native asset files
**Status:** DONE
**Files:** `assets/style.css`, `src/ui/app.rs`, `src/ui/styles.rs` (deleted)

**Resolution:** Migrated all CSS from Rust string literals into a standard `assets/style.css` file. Used the standard Rust `include_str!` macro to bundle the stylesheet directly into the binary at compile time. This preserves the "single executable" portability and `cargo install` compatibility while allowing for a proper CSS development experience with syntax highlighting and linting.

---

## [7] Implement issue deletion via CLI
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`, `src/model/workspace.rs`

Users need `ishoo delete <id>` to permanently remove an issue rather than marking it DESCOPED.
Should prompt for confirmation unless `--force` is passed. After deletion, the issue's ID must never be reused (relevant once [11] lands — the per-category counter must not decrement).

**Resolution:** 

---

## [31] Status changes move issues between files automatically
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/app.rs`

When an issue's status is changed — via the UI dropdown, the CLI, or the Board view — it should automatically migrate to the appropriate file. DONE and DESCOPED go to `issues-done.md`. Reopening a DONE issue moves it back to `issues-active.md`.
Currently this only happens on explicit "Save All" and only for the DONE→done-file case. Make it consistent and automatic for all status transitions. If arbitrary file names land ([28]), the routing rules should be configurable or at least documented.

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
This is the highest-impact backlog item.

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

## [14] Fix re-render performance in physics loop
**Status:** DONE
**Files:** `src/ui/views/physics.rs` (deleted), `src/ui/views/feed.rs`

**Resolution:** Completely replaced the 60fps manual physics simulation loop with a declarative, slot-based absolute positioning system. By using CSS `transition` for the "sucking into well" effect and index-based offsets for displaced cards, we eliminated the need for high-frequency signal updates. The UI is now significantly more performant and the code is much simpler.

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

## [13] Prevent silent data loss from discover_root ambiguity
**Status:** OPEN
**Files:** `src/model/mod.rs`

`discover_root` checks 6 candidate directories and silently picks the first match. If a project has both `docs/issues/` and `issues/` (e.g., from a migration or misconfiguration), the user gets zero feedback about which was chosen.
Fix:
- If multiple candidates contain issue files, print a warning listing all matches and which was selected
- Default to the first match but make the choice visible
- The `init` command should print the chosen path explicitly

**Resolution:** 

---

## [27] Add comments per issue
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`, `src/ui/views/feed/card.rs`

Issues have a description (immutable context) and a resolution (final outcome), but no way to log intermediate notes, decisions, or blockers.
Add a `### Comments` subsection under each issue. The UI should render comments chronologically within the expanded card, with an input box to append new entries. Each comment gets an auto-timestamp.
Keep it simple — no editing or deleting comments. Append-only log.

**Resolution:** 

---

## [12] Add round-trip save/parse tests
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/model/parse.rs`

There are no tests that verify `parse → mutate → save → parse` produces equivalent results. This is where the real data-loss bugs hide. Specifically:
1. Unknown fields (e.g., a user manually adds `**Priority:** HIGH`) are silently dropped on save because `write_section` only emits known fields
2. Description whitespace and blank lines may not survive a round-trip
3. Section assignment during save is asymmetric — DONE status forces migration to `issues-done.md`, but DESCOPED does not
Write property-based or snapshot tests that:
- Parse sample markdown, save it, parse again, and assert structural equality
- Inject unknown fields and verify they survive (or explicitly document that they won't)
- Mutate status and verify correct file routing

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

## [33] Add issue linking and mentions
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/ui/views/feed/card.rs`

The `**Depends on:**` field captures blocking relationships, but there's no way to express softer links: "related to", "duplicates", "superseded by".
More importantly, if a description or resolution mentions `#14` or `[14]`, it should render as a clickable link that navigates to that issue in the UI. The Graph view should pick up these informal references as edges.

**Resolution:** 

---

## [21] Add labels/tags system
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`

Issues need lightweight categorization beyond status. A `**Labels:**` field with comma-separated tags (e.g., `frontend, performance, v2`) would enable:
- Filtering the feed by label
- Color-coded label chips on cards
- CLI filtering: `ishoo list --label performance`
Labels should be freeform strings, not from a fixed set.

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

## [16] Preserve unknown markdown fields through save
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/model/workspace.rs`
**Depends on:** [8]

If a user manually adds `**Priority:** HIGH` or `**Assignee:** @alice` to an issue, `write_section` silently drops it because it only emits known fields. This is destructive and violates the "your markdown, your rules" philosophy.
After [8] lands (AST parser), the parser should capture unknown `**Key:** Value` pairs into a `HashMap<String, String>` on the Issue struct, and `write_section` should emit them back.

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

## [36] Validate and lint issue files
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/parse.rs`

There is no way to check whether the issue markdown files are well-formed without loading the full UI. Add:
- `ishoo lint` — parses all issue files and reports warnings: duplicate IDs, broken dependency references (depends on an ID that doesn't exist), missing required fields, empty titles
- `ishoo lint --strict` — treats warnings as errors (useful for CI)
This enables a pre-commit hook: `ishoo lint --strict || exit 1`

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

## [54] Add issue dependency blocking visualization
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`

The Graph view shows dependency edges, but doesn't highlight blocked chains. If issue [5] depends on [4], and [4] is still OPEN, then [5] is effectively blocked. Visually distinguish:
- Satisfied dependencies (dependency is DONE) — green edge
- Blocking dependencies (dependency is not DONE) — red edge with a "BLOCKED" badge on the dependent issue
The Feed view should also show a small "blocked" indicator on cards whose dependencies aren't met.

**Resolution:** 

---

## [57] Feed view lenses: Next Up, Hot Path, Quick Wins
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/app.rs`, `src/model/workspace.rs`

The Feed view currently shows issues in file order (manual position). Add toggle pills at the top of the feed for alternative lenses:

- **My Order** — current manual position (default)
- **Next Up** — sorted by transitive unblock count. The issue whose completion enables the most other issues to start ranks first. Computed from `**Depends on:**` data.
- **Hot Path** — sorted by weighted sum of heatmap scores for the issue's `**Files:**` entries. Issues touching contested, high-heat files rank first.
- **Quick Wins** — sorted by fewest and coldest file touches. Issues that can be knocked out in isolation without touching hotspots rank first.

The data for all three is already available — heatmap scores from `file_heatmap()`, dependency edges from `dependency_edges()`. This is just sort functions on existing data, surfaced as pills.

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

## [61] Project health pulse in sidebar
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/components.rs`, `src/model/workspace.rs`

Evolve the existing sidebar metrics (Backlog 28, In Flight 1, Resolved 3) into a simple health pulse:
- Resolved / Total ratio as a percentage or progress bar
- A small sparkline or trend arrow showing whether the ratio is improving or declining (derived from git history, same approach as [59])
- Keep it to one glanceable element, not a dashboard

This answers "am I making progress or am I treading water?"

**Resolution:** 

---

## [62] "Cost" lens for ruthless scoping
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/model/workspace.rs`

Add a Feed lens or a sortable column that shows the "cost" of each issue — computed as the number of files touched weighted by their heatmap scores, plus the number of dependencies.

High-cost issues that are still OPEN and have no dependents (nothing else is waiting on them) are candidates for cutting. Surface them visually — maybe a "heavy" badge or a sort that puts the most expensive, least-blocking issues at the top.

This answers "what should I seriously consider descoping?"

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
