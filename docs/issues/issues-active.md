# ACTIVE Issues

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

## [43] Add issue description editing in the UI
**Status:** OPEN
**Files:** `src/ui/views/feed/card.rs`
**Labels:** modal, feed

The description field in the expanded card is a read-only `div`. The resolution field is an editable `textarea`. There is no reason the description shouldn't also be editable — users shouldn't have to open their text editor to update an issue's description after creation.
Add a pencil icon or double-click-to-edit interaction that swaps the description `div` for a `textarea`. Consider a markdown preview toggle (depends on #30).

**Resolution:** 

---

## [61] Project health pulse & Issue Age
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/components.rs`, `src/model/workspace.rs`
**Labels:** viz, git

Sidebar `.health` pulse and Modal Issue Age. Requires invoking `git log` dynamically to derive sparkline trends and age calculations, which requires a new backend feature.

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
