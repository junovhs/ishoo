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

## [15] Implement ishoo edit CLI command
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`
**Labels:** cli, markdown

Currently the CLI can `new`, `set` (status only), and `show`. There is no way to edit an issue's title, description, resolution, files, or dependencies from the terminal.
`ishoo edit <id>` with no flags opens `$EDITOR` with the issue rendered as markdown, then parses the result back (like `git commit` without `-m`). The editor approach depends on #8 for robust re-parsing.
Also support field-level updates for scripting: `ishoo edit <id> --title "New title" --files "a.rs,b.rs"`.

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

## [7] Implement issue deletion via CLI
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/cli.rs`, `src/model/workspace.rs`
**Labels:** cli, save-load

Users need `ishoo delete <id>` to permanently remove an issue rather than marking it DESCOPED.
Should prompt for confirmation unless `--force` is passed. After deletion, the issue's ID must never be reused (relevant once #11 lands — the per-category counter must not decrement).

**Resolution:** 

---

## [36] Validate and lint issue files
**Status:** OPEN
**Files:** `src/main.rs`, `src/model/parse.rs`
**Labels:** cli, markdown, test-coverage

There is no way to check whether the issue markdown files are well-formed without loading the full UI. Add:

- `ishoo lint` — parses all issue files and reports warnings: duplicate IDs, broken dependency references (depends on an ID that doesn't exist), missing required fields, empty titles
- `ishoo lint --strict` — treats warnings as errors (useful for CI)
This enables a pre-commit hook: `ishoo lint --strict || exit 1`

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

## [123] Board Release: settle, commit delay, and post-drop state must match Feed exactly
**Status:** OPEN
**Files:** `src/ui/views/board.rs`, `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`
**Labels:** board, drag

Board drop/release still has its own sequencing. That creates risk of pop, dip, snap, or timing mismatch. Feed already solved these edge cases and Board must reuse that exact sequencing.

Requirements:

- Match Feed release timing and delayed reorder commit exactly
- No dip/pop/rebound after release
- No alternate board-only settle animation
- Post-drop hover suppression/re-arm should match Feed behavior where applicable

**Resolution:** 

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
