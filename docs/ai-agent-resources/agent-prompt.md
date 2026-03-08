# Agent Protocol (SEMMAP-first)

You can browse the repo, but orientation + verification MUST be SEMMAP/Neti-driven. Do not ask for permission to proceed; act, then verify. Note - Neti and SEMMAP binaries are installed in the system path and can be invoked with name only (e.g. `neti check` not `cargo run --bin neti check`, and 'semmap generate' and 'semmap cat --help' to learn how to request files efficiently, semmap is a super useful tool for you orienting yourself without greping and seding around - the trace command is especially useful, I have found progressive disclosure is the way to go)

## Hard rule: no source before orientation

Before reading any source file contents (via `view_file`, `cat`, IDE open, etc.), you MUST:

1. run `semmap generate` (if not already done for this repo state),
2. read `@SEMMAP.md` and cite the specific line(s) you used (Purpose + relevant layer entries/hotspots),
3. run `semmap trace <entry_file>` if flow/ownership is unclear or execution matters,
4. state your **minimal working set** (the 2–5 files you intend to read next, and why).

Only then may you read file contents. If you read additional files beyond the working set, you must justify why SEMMAP/trace was insufficient.

## Required evidence per iteration

In any iteration where you plan or change code, include:

* the SEMMAP line(s) you used (Purpose + relevant layer entries/hotspots)
* the exact `semmap trace ...` command(s) you ran (when flow/ownership matters)
* the exact `neti check` result after changes (green/red); consult `@neti-report.txt` if output is truncated

If you cannot provide this evidence, stop and run the missing SEMMAP/Neti steps first.

## Workflow

1. Run `semmap generate` and read `@SEMMAP.md` + `@model-driven-code-intelligence.md`.
2. Write a short Orientation (Purpose, entrypoint, trace target, hotspots, plan).
3. Use `semmap trace <entry_file>` for flow-dependent work or unclear ownership.
4. Declare a minimal working set, then read only those files (prefer `semmap cat`; use other tools if needed).
5. Make minimal edits that respect SEMMAP boundaries; hotspots = smaller diffs + stronger tests.
6. After every change: `neti check` (view `@neti-report.txt` in repo root for full output). Iterate until green.

## Issue discipline

Work from `/docs/issues/issues-active.md` (then backlog). An issue is DONE only when tests prove it (fail-before/pass-after + at least one edge case when relevant). Update the issue record with a concrete Resolution (what, why, how verified, commands run).