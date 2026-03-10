# ACTIVE Issues

---

***ai agent reading this file - this issue is fucked up and needs you to fix it to make it like the other formats below, please do so before doing any other work***

semmap on  main [!] is 📦 v1.0.5 via 🦀 v1.90.0
❯ ishoo
[Scroll Metrics] Frames: 60 | Avg: 15.3ms | Max: 17.5ms
[Scroll Metrics] Frames: 170 | Avg: 15.6ms | Max: 24.7ms
[Scroll Metrics] Frames: 99 | Avg: 15.3ms | Max: 16.9ms
[Scroll Metrics] Frames: 168 | Avg: 15.5ms | Max: 17.0ms
[Scroll Metrics] Frames: 89 | Avg: 15.4ms | Max: 17.0ms
[Scroll Metrics] Frames: 89 | Avg: 15.6ms | Max: 21.6ms
[Scroll Metrics] Frames: 144 | Avg: 15.5ms | Max: 17.4ms
[Scroll Metrics] Frames: 121 | Avg: 15.4ms | Max: 17.1ms
[Scroll Metrics] Frames: 36 | Avg: 15.4ms | Max: 23.3ms
[Scroll Metrics] Frames: 40 | Avg: 15.2ms | Max: 16.7ms
[Drag Metrics] key=active issues::67 t=7584ms ptr=0 avg_ptr=0.0ms max_ptr=0.0ms frames=4 avg_frame=1896.0ms max_frame=7491.0ms hover=0 logical=45.0 live=45.0 snapped=45.0 hover_y=45.0
[Drag Metrics] key=active issues::66 t=18387ms ptr=0 avg_ptr=0.0ms max_ptr=0.0ms frames=7 avg_frame=2626.7ms max_frame=10740.1ms hover=0 logical=603.0 live=603.0 snapped=603.0 hover_y=603.0
[Scroll Metrics] Frames: 291 | Avg: 15.5ms | Max: 17.2ms
[Drag Metrics] key=active issues::67 t=29010ms ptr=0 avg_ptr=0.0ms max_ptr=0.0ms frames=10 avg_frame=2901.0ms max_frame=10740.1ms hover=0 logical=45.0 live=45.0 snapped=45.0 hover_y=45.0
[Scroll Metrics] Frames: 1 | Avg: 5.6ms | Max: 5.6ms
[Scroll Metrics] Frames: 1 | Avg: 6.6ms | Max: 6.6ms

thread 'main' panicked at C:\Users\SpencerNunamakerTrav\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\dioxus-core-0.7.3\src\diff\mod.rs:44:15:
invalid key
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
semmap on  main [!] is 📦 v1.0.5 via 🦀 v1.90.0 took 7m33s
❯


***that concludes the bullshit im logging super quick, make it an issue, it crashed when I was collapsing sections***

## [140] Feed Link Proximity Lens: sort linked issues nearer each other
**Status:** OPEN
**Files:** `src/ui/app.rs`, `src/ui/feed_lens.rs`, `src/ui/views/feed.rs`, `src/ui/views/feed/card.rs`, `docs/issues/issues-active.md`, `docs/issues/issues-done.md`
**Labels:** feed, polish, links

The feed now exposes issue links and hover brackets, but linked issues can still be far apart in the list. That weakens link discovery because the UI can tell you an issue is related without helping you keep the related work in view. Ishoo should offer a feed lens that sorts linked issues nearer each other without conflating authored links with file-overlap batching.

Current regression note:

- The linked-issue bracket overlay is currently not showing up at all during hover and needs to be restored before this issue can be closed.

Requirements:

- Add a Feed lens that groups authored/reverse-linked issues nearer each other in the feed
- Keep the lens distinct from file-overlap clustering (#60); this is about issue-graph proximity, not shared implementation surface
- Preserve stable ordering for unrelated issues rather than introducing chaotic reshuffles
- Add targeted tests proving linked issues cluster and unlinked issues do not get falsely grouped
- Restore visible linked-issue hover brackets so link discovery still works with the new lens/interactions

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
