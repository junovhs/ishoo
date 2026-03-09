# Agent Protocol (SEMMAP-first)

You can browse the repo, but orientation and verification MUST be SEMMAP/Neti-driven. Act without unnecessary permission prompts, but still follow any real tool, sandbox, or approval requirements. Neti and SEMMAP binaries are installed in the system path and can be invoked directly, for example `neti check`, `semmap generate`, `semmap trace`, and `semmap cat`.

## Hard rule: no source before orientation

Before reading implementation source beyond the task-defining docs, you MUST:

1. run `semmap generate` (if not already done for this repo state),
2. read `SEMMAP.md` and cite the specific line(s) you used (Purpose plus the relevant layer entries and hotspots),
3. run `semmap trace <entry_file>` when flow, ownership, or execution path matters,
4. state your **minimal working set** (the 2-5 files you intend to read next, and why).

You may read the task-defining docs first: this prompt, `north-star.md`, `issues-active.md`, `issues-backlog.md`, and `issues-done.md` if you want historical context. You may also read `docs/briefs/` when relevant. After orientation, if you read additional files beyond the working set, you MUST justify why SEMMAP and the trace output were insufficient.

## Required evidence per iteration

In any iteration where you plan or change code, include:

* the SEMMAP line(s) you used (Purpose plus relevant layer entries and hotspots)
* the exact `semmap trace ...` command(s) you ran, when applicable
* the exact `neti check` result after changes; consult `neti-report.txt` if output is truncated

`neti check` is the canonical verification command. It already runs the configured verification suite, including `cargo test`, the configured Clippy gate, and Neti scan. Do not treat ad hoc `cargo test` or `cargo clippy` runs as a substitute for `neti check`.

If you cannot provide this evidence, stop and run the missing SEMMAP/Neti steps first.

## Workflow

1. Run `semmap generate` and read `SEMMAP.md`, `north-star.md`, and the active issue file. You can also run `semmap deps` if you need a dependency graph.
2. Write a short Orientation (Purpose, entrypoint, trace target, hotspots, plan).
3. Use `semmap trace <entry_file>` for flow-dependent work or unclear ownership.
4. Declare a minimal working set, then read only those files (prefer `semmap cat`; use other tools if needed).
5. Make minimal edits that respect SEMMAP boundaries; hotspots = smaller diffs + stronger tests.
6. After the change set is in place, run `neti check` (view `neti-report.txt` in repo root for full output). Iterate until clean, or until only clearly pre-existing failures remain and are called out explicitly.
7. If you manually exercise a CLI or user-facing flow, report the exact command, the important output, and the exit code when relevant.

## Issue discipline

Work from `docs/issues/issues-active.md` first, then `docs/issues/issues-backlog.md`.

An issue is DONE only when verification proves it. When relevant, that means fail-before/pass-after and at least one edge case.

Every issue you touch must stay hygienic:

* keep `**Status:**`, `**Labels:**`, and `**Resolution:**` present
* update `**Files:**` so it matches the real implementation surface
* add or fix dependency references when they matter
* move finished work into the correct issue file

When you complete or materially refine an issue, update its Resolution concretely:

* what changed
* why the approach was chosen
* how it was verified
* which commands were run
* whether any remaining failures are pre-existing

If the existing labels are insufficient, say so explicitly and propose the smallest useful addition.

## Minimal close-out

A compliant final report for code work should usually contain:

1. the issue handled
2. the SEMMAP evidence used
3. the key files changed
4. the exact `neti check` outcome
5. any manual CLI or UX verification that was performed
6. whether issue records were updated
