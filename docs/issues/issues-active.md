# ACTIVE Issues

---

## [134] CLI Parity: every issue action must be possible from the terminal
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`, `src/model/workspace.rs`, `src/ui/app.rs`
**Labels:** cli, save-load, markdown

The intended working mode is terminal-first. A user or AI should be able to create, inspect, edit, move, validate, and close work without needing to touch the desktop UI. The UI can stay better for browsing, but it must stop being the only place where core issue mutations are possible.

Requirements:

- Every issue mutation that matters in the UI must have a CLI path
- Terminal flows must support both human use and AI/scripted use
- Commands should prefer explicit flags and machine-readable output over interactive-only flows
- Terminal-first workflows like "make that an issue", "edit this", "close this", and "show me what changed" should not require markdown hand-editing
- The CLI should become the trusted automation surface for future agent work

Suggested direction:

- Treat `new`, `show`, `list`, `edit`, `delete`, `set`, `lint`, and section/routing operations as one coherent product surface instead of isolated commands
- Add machine-readable output modes where needed so AI/automation can inspect results safely
- Close command gaps before polishing secondary UI-only affordances

**Resolution:** 

---

## [136] Feed Motion Perfection: make slow drag feel as locked-solid as fast drag
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `src/ui/app.rs`, `src/ui/scroll.rs`, `assets/style.css`
**Labels:** feed, drag, performance, polish
**Depends on:** [132]

Feed drag is much better, but it is still not at the required standard. The target is not "smooth enough" or "good for a desktop webview." The target is that picking up a card and dragging it slowly through the feed feels unnaturally smooth: no visible stepping, no cursor/card drift, no text shimmer that reads as jitter, and no sense that the app is choking under motion.

Current findings from the 2026-03-08 instrumentation pass:

- The remaining problem looks performance-bound, not logic-bound
- Pointer cadence is imperfect but usually workable; frame cadence is the larger problem
- Observed drag/update frame averages often sit around `20ms` to `27ms`, with frequent max spikes in the `32ms` to `52ms` range
- Smooth 60fps behavior requires staying close to the `16.7ms` frame budget
- Intrusive live telemetry made the feel worse, so future instrumentation must be low-overhead and mostly summary-based
- The app’s general scroll/motion path is already under pressure, so drag smoothness cannot be solved only by hover-slot tuning

Requirements:

- Slow drag must feel as stable and premium as fast drag
- The held card must appear locked to the cursor, not merely approximately following it
- Text should remain crisp enough during drag that movement reads as fluid rather than shaky
- Any instrumentation kept in-tree must be low-overhead, Rust-only, and easy to disable
- Do not regress the currently good release/settle behavior
- Do not solve this by dumbing the cards down visually beyond what is necessary for true motion quality

Investigation and implementation direction:

- Remove the intrusive drag HUD and replace it with release-summary metrics only
- Profile the hot path from pointer move to visual update, with special attention to avoid signal churn on every frame
- Reduce or eliminate parent/sibling rerender work during active drag
- Audit the remaining custom scroll/motion ownership for frame-budget waste that bleeds into drag quality
- Tighten low-speed rendering quality without reintroducing the disliked release-motion regressions
- Keep the solution Rust-native; do not extend JavaScript-driven UI logic

Exit criteria:

- Manual testing says slow drags through multiple rows feel "buttery" rather than merely acceptable
- Release motion still reads as one clean settle into slot
- Verification remains clean with `cargo clippy --all-targets --no-deps -- -D warnings` and `cargo test`
- The issue should not close on vague feel claims alone; it should close only after an explicit final feel pass against the “locked-solid slow drag” bar

**Resolution:** 

---

## [133] Machine-Owned Issue Identity: users should not manage IDs by hand
**Status:** OPEN
**Files:** `src/model/mod.rs`, `src/model/parse.rs`, `src/model/workspace.rs`, `src/model/cli.rs`, `src/ui/app.rs`, `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `src/ui/views/board.rs`
**Labels:** cli, architecture, save-load
**Depends on:** [11]

Issue identity should be machine-owned, not user-managed bookkeeping. Users still benefit from visible handles in the UI because they aid navigation and discussion, but the system should generate and maintain them automatically rather than asking either humans or probabilistic agents to think about numbering.

Requirements:

- Stable internal identity must be programmatic and machine-owned
- Human-visible handles may remain visible, but users should not need to author or maintain them
- Renumbering or regenerating visible handles must not break links, dependencies, reorder logic, or persistence
- CLI and UI workflows like "make that an issue" must work without supplying an ID
- Existing markdown should migrate cleanly without destroying references

**Resolution:** 

---

## [126] Board Structure: reduce Board to three feed columns with minimal extra chrome
**Status:** OPEN
**Files:** `src/ui/views/board.rs`, `assets/style.css`
**Labels:** board

Board must be a minimal rearrangement of Feed, not a new visual system. The columns should read as Feed sections laid side by side, with only enough structure to support columns and independent scroll.

Requirements:

- Strip remaining non-feed explanatory or decorative chrome
- Keep separators/rules as faint as Feed
- Maintain independent per-column scroll without introducing heavy lane boxes
- Make the whole surface read as the same product language as Feed

**Resolution:** 

---

## [15] Implement ishoo edit CLI command
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`
**Labels:** cli, markdown

Currently the CLI can `new`, `set` (status only), and `show`. There is no way to edit an issue's title, description, resolution, files, or dependencies from the terminal.
`ishoo edit <id>` with no flags opens `$EDITOR` with the issue rendered as markdown, then parses the result back (like `git commit` without `-m`). The editor approach depends on #8 for robust re-parsing.
Also support field-level updates for scripting: `ishoo edit <id> --title "New title" --files "a.rs,b.rs"`.

**Resolution:** 

---

## [122] Board Displacement: crossed cards must move with Feed-identical local displacement
**Status:** OPEN
**Files:** `src/ui/views/board.rs`, `src/ui/views/feed/card.rs`
**Labels:** board, drag

Board currently displaces cards in a way that is merely similar to Feed. It must use the same local displacement behavior: crossed cards glide one slot, never jump to final order during live drag, and never use bespoke board-only placeholder sockets.

Requirements:

- Replace board-specific drop indicators/sockets with Feed-style local displacement only
- Crossed cards move one slot at a time exactly like Feed
- No instant snap to final order during live drag
- Vertical movement within a lane should be indistinguishable from Feed

**Resolution:** 

---

## [37] Add CI/pre-commit hook integration
**Status:** OPEN
**Files:** `src/main.rs`, `docs/`
**Labels:** cli, docs, test-coverage
**Depends on:** [36]

Provide documentation and a ready-made pre-commit hook config that runs `ishoo lint --strict` before every commit. This catches:

- Duplicate issue IDs introduced by a bad merge
- Dangling dependency references
- Issues left in IN PROGRESS on a branch that's being merged to main
Also consider a GitHub Action / GitLab CI template that runs `ishoo lint` and posts a summary comment on PRs showing which issues were modified.

**Resolution:** ---

---

## [128] Board Verification: prove Feed/Board drag parity with targeted tests and manual checklist
**Status:** OPEN
**Files:** `src/ui/views/feed.rs`, `src/ui/views/board.rs`, `docs/issues/issues-done.md`
**Labels:** board, drag, test-coverage

This work is too easy to hand-wave. Board drag parity must be verified explicitly, not described vaguely as "close" or "similar".

Requirements:

- Add targeted tests for shared drag math where possible
- Add a manual verification checklist covering:
  - drag within lane down
  - drag within lane up
  - release after crossing multiple cards
  - tiny movement click vs drag
  - cross-lane drag into populated lane
  - cross-lane drag into empty lane
- Issue can close only when Board is judged atom-for-atom identical in vertical feel to Feed

**Resolution:** 

---

## [61] Project health pulse & Issue Age
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/components.rs`, `src/model/workspace.rs`
**Labels:** viz, git

Sidebar `.health` pulse and Modal Issue Age. Requires invoking `git log` dynamically to derive sparkline trends and age calculations, which requires a new backend feature.

**Resolution:** 

---
