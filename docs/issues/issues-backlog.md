# BACKLOG Issues

---

## [27] Add comments per issue
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`, `src/ui/views/feed/card.rs`
**Labels:** markdown, modal

Comments/Notes section in the modal (`.m-comments`). Requires backend parsing to read the `### Comments` markdown blocks into the Issue model first.

**Resolution:** 

---

## [121] Board Drag Feel: cursor anchoring must match Feed exactly
**Status:** OPEN
**Files:** `src/ui/views/board.rs`, `src/ui/views/feed/card.rs`
**Labels:** board, drag

While dragging in Board, the held card must stay under the cursor with the same deadzone break, live follow, and no-drift behavior as Feed. No shrink, no offset drift, no alternate ghost logic that changes the feel.

Requirements:

- Use the same deadzone behavior as Feed
- Keep the held card anchored identically under the cursor
- Remove any visual shrink/compression behavior not present in Feed
- Match Feed lift/shadow/scale treatment while dragging

**Resolution:** 

---

## [61] Project health pulse & Issue Age
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/components.rs`, `src/model/workspace.rs`
**Labels:** viz, git

Sidebar `.health` pulse and Modal Issue Age. Requires invoking `git log` dynamically to derive sparkline trends and age calculations, which requires a new backend feature.

**Resolution:** 

---

## [43] Add issue description editing in the UI
**Status:** OPEN
**Files:** `src/ui/views/feed/card.rs`
**Labels:** modal, feed

The description field in the expanded card is a read-only `div`. The resolution field is an editable `textarea`. There is no reason the description shouldn't also be editable — users shouldn't have to open their text editor to update an issue's description after creation.
Add a pencil icon or double-click-to-edit interaction that swaps the description `div` for a `textarea`. Consider a markdown preview toggle (depends on #30).

**Resolution:** 

---

## [131] Feed/Text Crispness: eliminate transform-induced fuzz during scroll settle and hover
**Status:** OPEN
**Files:** `src/ui/scroll.rs`, `src/ui/app.rs`, `src/ui/views/feed/card.rs`, `assets/style.css`
**Labels:** feed, polish

The feed text is not consistently crisp. In particular:

- Scroll all the way to the top, let the motion settle, and issue text can land in a slightly fuzzy state
- Hovering an issue can also soften the text briefly when the row lifts/expands
- This is subtle, but once noticed it makes the UI feel less exact than it should

This is likely a transform/rendering problem, not a typography problem. The most suspicious causes already in the code are:

- Fractional `translate3d(...)` values applied during custom scrolling in `src/ui/scroll.rs`
- Feed card positioning with transform-based movement in `src/ui/views/feed/card.rs`
- Hover/press scaling on text-bearing issue rows in `assets/style.css`
- App-level `zoom` usage in `src/ui/app.rs`, which may amplify subpixel softness in combination with transforms

Requirements:

- Keep issue text visually crisp at rest after scrolling settles
- Keep issue text crisp during normal hover and press interactions
- Preserve the existing feel as much as possible; this should be a rendering-quality fix, not a behavioral redesign
- Prefer pixel-snapped movement and non-scaling hover treatments over text-bearing scale transforms
- If there is still an unavoidable tradeoff, prioritize text crispness over tiny amounts of motion flourish

Suggested direction:

- Snap scroll and sticky-header transforms to whole pixels before writing them to the DOM
- Remove or reduce scale transforms on `.issue-row` and preserve the feel through shadow, color, border, or slight translate-only motion
- Audit whether app zoom plus transformed children is producing compounded softness, and tighten that path if needed

**Resolution:** Snapped feed scroll/content and sticky-header transforms to whole pixels in `src/ui/scroll.rs`, snapped feed card Y positioning to whole pixels in `src/ui/views/feed/card.rs`, and removed text-bearing scale transforms from issue-row drag/hover/press states in `assets/style.css` in favor of translate-only motion. Verified with `neti check` on 2026-03-08: `cargo clippy --all-targets --no-deps -- -D warnings` PASS, `cargo test` PASS, Neti scan reported only the pre-existing `Workspace` CBO/SFOUT warnings in `src/model/workspace.rs`. Relevant labels remain `feed, polish`.

---

## [125] Board Cards: reuse Feed card interaction language at the atomic level
**Status:** OPEN
**Files:** `src/ui/views/board.rs`, `src/ui/views/feed/card.rs`, `assets/style.css`
**Labels:** board, modal

Board cards still differ too much from Feed in interaction language. The board should feel like Feed cards rearranged into columns, not a second card system.

Requirements:

- Reuse Feed hover, press, drag, and shadow language as closely as possible
- Keep the open layout; avoid boxed sockets or custom board chrome that Feed does not use
- Preserve Board-specific hierarchy improvement where IDs lead scanning
- Clicking/tapping a non-dragged board card must open the issue modal reliably

**Resolution:** 

---

## [127] Board Modal Parity: board-opened issue modal must match Feed modal behavior
**Status:** OPEN
**Files:** `src/ui/views/board.rs`, `src/ui/views/feed.rs`
**Labels:** board, modal

Board can open issues now, but modal behavior is not yet guaranteed to be on par with Feed. Board-opened issues should have the same editing confidence and interaction quality.

Requirements:

- Opening an issue from Board must feel identical in quality to opening from Feed
- Status, labels, and resolution editing must persist the same way
- Any modal layout/content drift from Feed should be eliminated unless explicitly intentional
- Do not fork modal behavior further while drag work is ongoing

**Resolution:** 

---

## [120] Board Drag Engine: extract feed drag/release physics into a shared engine
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `src/ui/views/board.rs`, `src/ui/views/mod.rs`
**Labels:** board, drag

Board drag behavior is still a parallel implementation that only approximates Feed. That is the wrong architecture. Feed and Board must share the same drag state model, release timing, displacement math, and cursor anchoring. Only board lane targeting should differ.

Requirements:

- Extract the feed drag/release model into shared reusable code
- Preserve Feed behavior exactly while moving logic out
- Make Board consume the same engine instead of a separate approximation
- Do not change Feed feel while doing this refactor

**Resolution:** 

---

## [9] Add global keyboard shortcuts
**Status:** OPEN
**Files:** `src/ui/app.rs`
**Labels:** modal
**Depends on:** [6]

Essential keyboard shortcuts for the desktop app:

- `Cmd/Ctrl + S` — Save All
- `Esc` — Close modal or collapse active card
Note: Dioxus desktop runs in a webview that swallows some OS-level key combinations. Prototype early to identify which bindings actually work before committing to a full set. Expand later based on what's possible.

**Resolution:** 

---

## [124] Board Cross-Lane Drag: add left/right lane targeting without altering Feed vertical physics
**Status:** OPEN
**Files:** `src/ui/views/board.rs`, `src/ui/views/feed.rs`
**Labels:** board, drag

The only behavior Board should add on top of Feed drag is lane selection. Left/right movement should choose a target lane, but vertical drag behavior inside the chosen lane must remain Feed-identical.

Requirements:

- Lane targeting is the only board-specific drag extension
- Switching lanes must not change drag feel, deadzone, or release behavior
- Empty lanes must accept drops cleanly
- Cross-lane insertion should still preserve Feed-style displacement inside the destination lane

**Resolution:** 

---

## [5] Add conflict resolution for concurrent edits
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/model/workspace.rs`
**Labels:** save-load, bugs
**Depends on:** [4]

If the user modifies an issue in the UI (`dirty = true`) and an external process modifies the markdown simultaneously, "Save All" overwrites the external changes with no warning.
The current poll handler also has an internal race: the `if !dirty()` check and `issues.set()` are not atomic, so a user edit between those two calls is silently dropped even without external interference.
Resolution should include:

- Content hash or generation counter comparison before overwriting
- A warning modal: "The file has changed on disk. Overwrite / Reload / Merge?"
- Optionally, per-issue dirty tracking instead of a single global `dirty` flag

**Resolution:** 

---

## [58] Bottleneck highlighting in Graph view
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`, `src/model/workspace.rs`
**Labels:** viz
**Depends on:** [54]

After #54 adds blocking visualization (green/red edges), add a "Bottlenecks" mode to the Graph view that highlights the issues with the highest transitive dependent count. The issue whose completion unblocks the most downstream work should visually glow or scale larger.

This answers "what's the single most important thing to do right now" from a pure dependency perspective. Compute transitive dependents by walking the dependency graph — no new data source needed.

**Resolution:** 

---

## [12] Add round-trip save/parse tests
**Status:** DESCOPED
**Files:** `src/model/workspace.rs`, `src/model/parse.rs`
**Labels:** test-coverage, markdown, save-load

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

## [42] Protect against data loss on crash during save
**Status:** OPEN
**Files:** `src/model/workspace.rs`
**Labels:** save-load, bugs, test-coverage

`write_section` calls `fs::write` directly. If the process crashes or is killed mid-write (e.g., laptop lid close, OOM kill), the file is truncated and all issues in that section are lost.
Fix:

- Write to a temporary file in the same directory (`issues-active.md.tmp`)
- `fsync` the temp file
- Atomically rename the temp file to the target name
- On startup, detect and clean up orphaned `.tmp` files

**Resolution:** 

---

## [16] Preserve unknown markdown fields through save
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/model/workspace.rs`
**Labels:** markdown, save-load
**Depends on:** [8]

If a user manually adds `**Priority:** HIGH` or `**Assignee:** @alice` to an issue, `write_section` silently drops it because it only emits known fields. This is destructive and violates the "your markdown, your rules" philosophy.
After #8 lands (AST parser), the parser should capture unknown `**Key:** Value` pairs into a `HashMap<String, String>` on the Issue struct, and `write_section` should emit them back.

**Resolution:** 

---

## [28] Support arbitrary issue file names
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/workspace.rs`
**Labels:** save-load

The three-file structure (`issues-active.md`, `issues-backlog.md`, `issues-done.md`) is mostly hardcoded. But the app already parses the `# HEADING` at the top of each file as the section name, so the file name is nearly irrelevant.
Change `Workspace::load` to scan for all `issues-*.md` files in the directory instead of only the three hardcoded names. On save, write each issue back to whichever file it was loaded from (tracked via a `source_file` field on Issue). The only special-case routing is DONE/DESCOPED issues, which always go to `issues-done.md`.
This means users can create `issues-sprint-42.md`, `issues-frontend.md`, `issues-tech-debt.md` — whatever they want. No config file needed. The file is the config.
If a new issue is created and has no source file, default to `issues-active.md`.

**Resolution:** 

---

## [44] Add notification/badge for externally changed issues
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/feed/card.rs`
**Labels:** feed
**Depends on:** [4]

When the file watcher detects external changes, the UI silently refreshes. The user has no idea which issues changed or what changed about them.
After reload, diff the old and new issue lists. For any issue that changed, show a subtle "updated" indicator on the card (e.g., a blue dot that fades after 10 seconds). Optionally show a toast: "3 issues updated externally".

**Resolution:** 

---

## [46] Cross-platform path handling and line ending audit
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/workspace.rs`
**Labels:** save-load, test-coverage

Development is on Pop!_OS Linux and Windows 11. PathBuf operations should be cross-platform, but there are untested risks:

- Backslash vs forward slash in `**Files:**` field values parsed on Windows vs displayed on Linux
- Case sensitivity differences (Windows NTFS is case-insensitive, Linux ext4 is not) — could cause duplicate file entries in the heatmap
- Line ending normalization: if a Windows user commits with CRLF and a Linux user opens the same file, does the parser handle `\r\n` correctly? The `lines()` iterator strips `\r` but the `accumulate_text` function may re-introduce inconsistencies
- Long path issues on Windows (>260 chars without the `\\?\` prefix)
Add integration tests that exercise `init → new → save → load` with both forward and backslash paths. Test with CRLF input.

**Resolution:** 

---

## [59] Stale issue detection
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`
**Labels:** git, feed

Surface issues that haven't changed status in a long time. Derive staleness from git history on the issue markdown files (`git log --format=%aI -- <file>`) — no new metadata fields needed.

Show a subtle "stale" indicator on cards that haven't been touched in N days (configurable, default 14). The Timeline view could also dim or desaturate stale issues.

This answers "what's stalled and why am I pretending it isn't?"

**Resolution:** 

---

## [31] Status changes move issues between files automatically
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/app.rs`
**Labels:** save-load

When an issue's status is changed — via the UI dropdown, the CLI, or the Board view — it should automatically migrate to the appropriate file. DONE and DESCOPED go to `issues-done.md`. Reopening a DONE issue moves it back to `issues-active.md`.
Currently this only happens on explicit "Save All" and only for the DONE→done-file case. Make it consistent and automatic for all status transitions. If arbitrary file names land (#28), the routing rules should be configurable or at least documented.

**Resolution:** 

---

## [63] Velocity visualization in Timeline view
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`, `src/model/workspace.rs`
**Labels:** viz, git

The Timeline view should show completed issues plotted over time (derived from git history of when issues moved to DONE status). A simple cumulative line or bar chart showing resolved issues per week/month.

Doesn't need to be fancy — even a stepped line that goes up by one each time an issue is resolved. The point is to see momentum. When you're grinding and it feels like nothing's moving, the upward slope is motivating.

**Resolution:** 

---

## [60] Issue clustering by shared files
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`, `src/model/workspace.rs`
**Labels:** viz

Add a view mode (or a tab within Heatmap) that groups issues by file overlap rather than by status or section. Issues that share 2+ files get clustered together.

Output something like:

```
Cluster: parse.rs + workspace.rs
  → #8 AST parser, #12 round-trip tests, #16 preserve unknown fields, #27 comments
Cluster: card.rs + feed.rs
  → #47 drag fix, #14 physics performance, #41 compact mode, #43 description editing
```

This answers "if I'm already in these files, what else can I batch?" Reduces context switching and merge conflict risk.

**Resolution:** 

---

## [54] Add issue dependency blocking visualization
**Status:** OPEN
**Files:** `src/ui/views/viz.rs`
**Labels:** viz, feed

The Graph view shows dependency edges, but doesn't highlight blocked chains. If issue #5 depends on #4, and #4 is still OPEN, then #5 is effectively blocked. Visually distinguish:

- Satisfied dependencies (dependency is DONE) — green edge
- Blocking dependencies (dependency is not DONE) — red edge with a "BLOCKED" badge on the dependent issue
The Feed view should also show a small "blocked" indicator on cards whose dependencies aren't met.

**Resolution:** 

---

## [64] Focus mode
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`
**Labels:** focus, feed, modal

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
**Labels:** feed

Add a Feed lens or a sortable column that shows the "cost" of each issue — computed as the number of files touched weighted by their heatmap scores, plus the number of dependencies.

High-cost issues that are still OPEN and have no dependents (nothing else is waiting on them) are candidates for cutting. Surface them visually — maybe a "heavy" badge or a sort that puts the most expensive, least-blocking issues at the top.

This answers "what should I seriously consider descoping?"

**Resolution:** 

---

## [65] Precompute viz data in model layer
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/views/viz.rs`
**Labels:** viz, performance

The graph and heatmap views currently compute overlap indices, pair intersections, and layout data inline during rendering. This causes the P02/P04 violations in `viz.rs` and will get worse as the issue count grows.

Move all graph computation into `Workspace`:

- `file_overlap_index: HashMap<(IssueId, IssueId), Vec<String>>` — precomputed at load time
- `transitive_dependents: HashMap<IssueId, usize>` — for bottleneck highlighting (#58)
- `issue_heat_score: HashMap<IssueId, usize>` — weighted file touch count for feed lenses (#57)

The viz views become pure consumers of precomputed data. This fixes the current Neti violations and makes all the new lens/sort features cheap to render.

**Resolution:** 

---

## [66] Add Brief model and brief kind enum
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`
**Labels:** brief, markdown

Ishoo currently only has one first-class artifact: `Issue`. To support the new Briefs layer cleanly, add a separate `Brief` model rather than shoving extra fields into `Issue`.

Needs:

* `BriefId` type
* `BriefKind` enum: `DECISION`, `INSIGHT`, `RESEARCH`, `PRINCIPLE`, `QUESTION`, `GUIDE`
* Core shared fields: title, status, summary/body, related issues, related briefs, created/updated metadata if applicable
* Separate parse/save pipeline from issues so Briefs can evolve independently

This is the foundation. If this is hacked into `Issue`, the concept gets watered down immediately.

**Resolution:** 

---

## [67] Define canonical markdown schema for Briefs
**Status:** OPEN
**Files:** `src/model/parse.rs`, `docs/`
**Labels:** brief, markdown, docs

Before implementing UI or CLI, define the exact markdown contract for Briefs. Ishoo issues became powerful once the format was explicit; Briefs need the same treatment.

Define:

* required frontmatter/fields
* required sections per brief kind
* which fields are shared vs kind-specific
* allowed statuses (`CAPTURE`, `REFINED`, `ADOPTED`, `UNCERTAIN`, `SUPERSEDED`, `ARCHIVED`)
* relationship syntax (`**Related Issues:**`, `**Related Briefs:**`, etc.)

This should be treated like product shape, not left implicit in parser code.

**Resolution:** 

---

## [68] Parse and load Brief markdown files from disk
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/model/workspace.rs`
**Labels:** brief, markdown, save-load

Add workspace loading for Briefs in addition to Issues. Briefs should live under `docs/briefs/`, not be mixed into issue files.

Requirements:

* scan brief directories on load
* parse all supported brief kinds
* route malformed briefs into lint/warning flow instead of crashing the whole app
* preserve source path on each Brief for round-trip save

This is the equivalent of `Workspace::load` for the new artifact family.

**Resolution:** 

---

## [69] Save Briefs with round-trip-safe serialization
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/model/parse.rs`
**Labels:** brief, markdown, save-load
**Depends on:** [66], [67], [68]

Once Briefs are parsed, saving them must preserve structure and avoid destructive rewrites. Same philosophy as issues: the markdown is the source of truth.

Needs:

* deterministic field ordering
* preservation of blank lines and body sections where reasonable
* stable file naming / file path routing
* atomic write path like issue save should have (#42)
* no silent dropping of unknown fields

If Briefs are lossy, users will stop trusting them immediately.

**Resolution:** 

---

## [70] Add ishoo briefs lint validation command
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/parse.rs`, `src/model/workspace.rs`
**Labels:** brief, cli, test-coverage

Issues already need linting (#36). Briefs need equivalent validation from day one or they will become a graveyard of malformed pseudo-docs.

Add:

* `ishoo briefs lint`
* `ishoo briefs lint --strict`

Checks should include:

* duplicate Brief IDs
* invalid brief kinds
* invalid status values
* missing required sections for the brief kind
* broken `Related Issues` / `Related Briefs` references
* empty titles or summaries
* guide/principle/research briefs exceeding configured size caps

This should eventually plug into the same CI/pre-commit story as issues.

**Resolution:** 

---

## [72] Add Brief editing via CLI
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`, `src/model/workspace.rs`
**Labels:** brief, cli
**Depends on:** [71]

Like issues (#15), Briefs need a terminal editing flow. `ishoo brief edit <id>` should open `$EDITOR` with the rendered markdown artifact and parse it back on save.

Also support field-level edits for scripting:

* `--title`
* `--status`
* `--related-issues`
* `--related-briefs`

This should not be a separate bespoke system if issue edit infra can be reused.

**Resolution:** 

---

## [71] Add Brief creation via CLI
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`, `src/model/workspace.rs`
**Labels:** brief, cli

Need a clean CLI path to create Briefs without the UI. This is especially important because AI/terminal workflows are likely how these get generated first.

Add:

* `ishoo brief new decision`
* `ishoo brief new insight`
* `ishoo brief new research`
* etc.

Should support:

* title flag
* optional related issue IDs
* optional related brief IDs
* generated starter template per brief kind

The command should scaffold the correct markdown shape instead of creating blank freeform docs.

**Resolution:** 

---

## [73] Add Brief list/show CLI commands
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`
**Labels:** brief, cli

Once Briefs exist, users need terminal-native retrieval and filtering.

Add:

* `ishoo brief list`
* `ishoo brief show <id>`

Filters:

* by kind
* by status
* by related issue
* by text search in title/body
* maybe by “adopted only”

This is the minimal retrieval surface before the UI catches up.

**Resolution:** 

---

## [74] Support typed relationships between Issues and Briefs
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`
**Labels:** brief, issue-brief-bridge

The whole point of Briefs is not just storing documents — it’s linking project understanding to actionable work.

Add explicit relationship support:

* Brief ↔ Issue
* Brief ↔ Brief

Initial relationship types can stay simple, but at minimum Ishoo needs to know:

* issue is informed by brief
* issue implements decision
* issue resolves question
* brief spawns issue
* brief supersedes brief

Even if the markdown stores these as plain ID lists at first, the model layer should not treat them as meaningless strings forever.

**Resolution:** 

---

## [75] Add dedicated Briefs section to workspace layout
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `docs/`
**Labels:** brief, docs

Need a concrete filesystem convention for where Briefs live.

Default layout:

* `docs/issues/`
* `docs/briefs/decisions/`
* `docs/briefs/insights/`
* `docs/briefs/research/`
* `docs/briefs/principles/`
* `docs/briefs/questions/`
* `docs/briefs/guides/`

Should also document:

* whether arbitrary subfolders are supported
* how file names are derived
* what happens when a Brief changes kind/status
* whether archived/superseded briefs move directories automatically

**Resolution:** 

---

## [76] Add Briefs browser view in UI
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/`, `src/model/workspace.rs`
**Labels:** brief

Briefs should not be jammed into the existing issue feed. Add a dedicated Briefs view/tab so Ishoo stays clean:

* filter by kind
* filter by status
* sort by updated / title / linkage count
* search
* compact and expanded cards

This is the core UI surface for the new artifact family. Without it, Briefs will feel second-class and invisible.

**Resolution:** 

---

## [77] Add Brief detail view with linked issues and linked briefs
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/feed/card.rs`, `src/ui/views/`
**Labels:** brief, modal

Opening a Brief should show:

* the full brief content
* its kind/status
* linked issues
* linked briefs
* spawned/follow-on work
* superseded-by / supersedes chain if applicable

This should feel like project memory with context, not just a markdown preview.

**Resolution:** 

---

## [79] Add “Spawn Issue from Brief” action
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/`, `src/model/workspace.rs`
**Labels:** brief, issue-brief-bridge

The inverse of #78. If a Brief implies work, creating the Issue should be one click, not manual copy/paste.

Examples:

* research brief spawns implementation issue
* question brief spawns investigation spike
* guide brief spawns cleanup/refactor work
* decision brief spawns follow-through tasks

Should create an Issue pre-linked back to the Brief.

This is a core part of making Briefs useful rather than archival.

**Resolution:** 

---

## [80] Add Brief status lifecycle enforcement
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/workspace.rs`, `src/model/parse.rs`
**Labels:** brief, cli

Brief statuses should not just be arbitrary strings. Enforce the new lifecycle cleanly:

* `CAPTURE`
* `REFINED`
* `ADOPTED`
* `UNCERTAIN`
* `SUPERSEDED`
* `ARCHIVED`

Need:

* parse validation
* save normalization
* UI badge styling
* CLI validation

If lifecycle is fuzzy, the app will not solve the “what’s actually current?” problem.

**Resolution:** 

---

## [83] Add integrated search across Issues and Briefs
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/model/workspace.rs`, `src/ui/views/`
**Labels:** brief

Once Briefs exist, users will need a single way to answer:

* where did we decide this?
* what issue is tied to this guide?
* what research informed this feature?
* did we already learn something about scrolling?

Add global search that spans:

* issue titles/descriptions
* brief titles/body
* linked IDs
* maybe files for issues

Results should clearly distinguish artifact type.

**Resolution:** 

---

## [81] Add size-cap validation for Brief kinds
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/main.rs`, `docs/`
**Labels:** brief, test-coverage

Part of the concept is that Briefs stay distilled. Add lint warnings/errors when Briefs exceed configured size caps by kind.

Examples:

* principle too long
* decision bloated into essay
* research brief becoming raw dump
* guide turning into amorphous wiki page

This can start as a soft lint warning and become stricter later. Without pressure, the whole Briefs layer will sludge up.

**Resolution:** 

---

## [78] Add “Create Brief from Issue” action
**Status:** OPEN
**Files:** `src/ui/views/feed/card.rs`, `src/ui/app.rs`, `src/model/workspace.rs`
**Labels:** brief, issue-brief-bridge

A lot of decisions/insights/research conclusions will emerge while working an issue. There should be a fast path to promote that learning into a Brief directly from the issue card/modal.

Add an action like:

* “Promote to Decision”
* “Extract Insight”
* “Create Research Brief”
* “Mark as Open Question”

Should prefill:

* title
* related issue link
* maybe files/context from the source issue

This is one of the most important bridges between execution and understanding.

**Resolution:** 

---

## [84] Add related context panel on issue cards/modals
**Status:** OPEN
**Files:** `src/ui/views/feed/card.rs`, `src/ui/app.rs`, `src/model/workspace.rs`
**Labels:** brief, issue-brief-bridge, modal
**Depends on:** [74]

If an Issue is linked to Briefs, that context should show up directly where the user is deciding what to do.

Add a contextual panel on issue cards/modal:

* linked decisions
* linked research
* linked principles/guides
* unresolved questions related to this issue

This is where the Briefs feature becomes operational instead of academic.

**Resolution:** 

---

## [85] Add related context panel on Brief detail views
**Status:** OPEN
**Files:** `src/ui/views/`, `src/ui/app.rs`, `src/model/workspace.rs`
**Labels:** brief, modal
**Depends on:** [74], [77]

Likewise, opening a Brief should expose:

* linked issues
* downstream issues spawned by it
* related/superseding briefs
* open questions nearby in the same topic area

This should make it easy to traverse understanding → work → understanding without losing the thread.

**Resolution:** 

---

## [86] Add Briefs-aware workspace stats and lenses
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/views/feed.rs`, `src/ui/views/`
**Labels:** brief, viz

Once Briefs exist, Ishoo should surface lightweight aggregate signals:

* count of open questions
* count of adopted decisions
* count of stale CAPTURE briefs never refined
* issues with no linked context
* briefs with no downstream impact

These should be optional lenses, not dashboard sludge. The point is to expose health, not create managerial theater.

**Resolution:** 

---

## [82] Preserve unknown markdown fields through Brief save
**Status:** OPEN
**Files:** `src/model/parse.rs`, `src/model/workspace.rs`
**Labels:** brief, markdown, save-load

Same issue as #16, but for Briefs. If a user adds custom metadata or manually extends a brief, Ishoo should not destroy it on save.

After Brief parsing is implemented:

* capture unknown `**Key:** Value` fields
* preserve them in memory
* emit them back on save in stable order

This is necessary to preserve the “your markdown, your rules” philosophy for Briefs too.

**Resolution:** 

---

## [88] Add docs explaining boundary between Issues and Briefs
**Status:** OPEN
**Files:** `docs/`
**Labels:** brief, docs

This feature will fail if users don’t know where things belong. Need explicit docs that answer:

* when is something an Issue vs a Brief?
* when should research become project-facing?
* what kinds of Briefs exist?
* when should a Brief spawn an Issue?
* when should a Question be archived vs resolved?

This should be framed as a routing guide, not a giant theoretical essay.

**Resolution:** 

---

## [87] Add stale Brief detection
**Status:** OPEN
**Files:** `src/model/workspace.rs`, `src/ui/views/`, `src/ui/app.rs`
**Labels:** brief, git

Questions, research, and guides can go stale even if issue workflow is healthy. Add the equivalent of #59 for Briefs.

Examples:

* `UNCERTAIN` question untouched for 90 days
* `RESEARCH` brief old enough that freshness is suspect
* `CAPTURE` briefs never promoted or archived
* `ADOPTED` guide that has not been revisited in a long time

This should help prevent the Briefs layer from quietly turning into dead philosophy.

**Resolution:** 

---

## [89] Add migration / bootstrap command for existing project docs into Briefs
**Status:** OPEN
**Files:** `src/main.rs`, `docs/`
**Labels:** brief, cli, docs

For real-world repos, there may already be lots of markdown docs, archives, and north-star-style documents. Provide a pragmatic migration path rather than assuming greenfield adoption.

Possible command:

* `ishoo brief import <path> --kind guide`
* `ishoo brief import docs/archive/*.md --kind research`

Even if the import is rough, it should:

* assign IDs
* normalize filenames
* attach starter metadata
* mark imported briefs as `CAPTURE` or `REFINED`

This lowers activation energy for people who already have a pile of markdown thinking.

**Resolution:** 

---

## [90] Add pre-commit / CI support for Brief linting
**Status:** OPEN
**Files:** `src/main.rs`, `docs/`
**Labels:** brief, cli, test-coverage
**Depends on:** [70], [88]

Once `ishoo briefs lint` exists, extend CI/pre-commit support so malformed or broken project context doesn’t silently land in the repo.

Checks should include:

* broken links between issues/briefs
* invalid lifecycle status
* missing required sections
* size-cap violations in strict mode

This should integrate with the existing lint/pre-commit story, not create a second unrelated one.

**Resolution:** 

---

## [91] Add Briefs focus mode for open questions and active decisions
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/`, `src/model/workspace.rs`
**Labels:** brief, focus

Once the artifact family exists, Ishoo should support a thinking mode equivalent to issue Focus mode (#64), but for project understanding.

Examples:

* show one open Question with related issues and nearby research
* show one Decision with its consequences and implementation work
* show one Guide with all related issues

This should be a calm, low-noise mode for project steering rather than task execution.

**Resolution:** 

---

## [92] Ensure Briefs do not pollute issue-first workflows
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/views/feed.rs`, `docs/`
**Labels:** brief, docs

The whole point of folding Briefs into Ishoo rather than making a separate app is lower cognitive load. That only works if basic issue-tracking workflows stay simple for users who don’t care about Briefs yet.

Requirements:

* issue views should still work cleanly with Briefs hidden/minimized
* Briefs should not add noisy required fields to issue creation
* Brief UI affordances should feel progressive, not mandatory
* docs/onboarding should preserve issue-first happy path

This is a product-boundary issue, not just implementation polish.

**Resolution:** 

---
