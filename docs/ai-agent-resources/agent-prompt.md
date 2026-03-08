# Agent Protocol (SEMMAP-first)

You can browse the repo, but orientation + verification MUST be SEMMAP/Neti-driven. Act without unnecessary permission prompts, but still follow any real tool, sandbox, or approval requirements. Neti and SEMMAP binaries are installed in the system path and can be invoked directly, for example `neti check`, `semmap generate`, and `semmap cat --help`.

## Hard rule: no source before orientation

Before reading implementation source beyond the task-defining docs, you MUST:

1. run `semmap generate` (if not already done for this repo state),
2. read `SEMMAP.md` and cite the specific line(s) you used (Purpose + relevant layer entries/hotspots),
3. run `semmap trace <entry_file>` if flow/ownership is unclear or execution matters,
4. state your **minimal working set** (the 2–5 files you intend to read next, and why).

You may read the task-defining docs first: this prompt, `north-star.md`, `issues-active.md`, `issues-backlog.md` and even `issues-done.md` if you want historical context. You can also peruse /docs/briefs and its contents if you want. After orientation, if you read additional files beyond the working set, justify why SEMMAP/trace was insufficient. Repeat: After orientation, if you read additional files beyond the working set, YOU MUST justify why SEMMAP/trace was insufficient. This a requirement.

## Required evidence per iteration

In any iteration where you plan or change code, include:

* the SEMMAP line(s) you used (Purpose + relevant layer entries/hotspots)
* the exact `semmap trace ...` command(s) you ran (when flow/ownership matters)
* the exact `neti check` result after changes; consult `neti-report.txt` if output is truncated NOTE: `neti check` is a combination command that runs cargo test, my configured clippy lint, and neti scan which is an architectural linter you MUST obey. 

If you cannot provide this evidence, stop and run the missing SEMMAP/Neti steps first.

## Workflow

1. Run `semmap generate` and read `SEMMAP.md`, `north-star.md`, and the active issue file. You can also run semmap deps to see a dependancy graph.
2. Write a short Orientation (Purpose, entrypoint, trace target, hotspots, plan).
3. Use `semmap trace <entry_file>` for flow-dependent work or unclear ownership.
4. Declare a minimal working set, then read only those files (prefer `semmap cat`; use other tools if needed).
5. Make minimal edits that respect SEMMAP boundaries; hotspots = smaller diffs + stronger tests.
6. After every change: `neti check` (view `neti-report.txt` in repo root for full output). Iterate until clean, or until only clearly pre-existing failures remain and are called out explicitly.

## Issue discipline

Work from `docs/issues/issues-active.md` (then backlog). An issue is DONE only when tests prove it (fail-before/pass-after + at least one edge case when relevant). Update the issue record with a concrete Resolution (what, why, how verified, commands run), review the presently used labels and evaluate which of those labels fit best, and apply those labels - if none fit, explain why and ask the user if you can create more labels. Ensure you are logging relevant files to each issue, and that you are specifying what issues it depends on, or is depended on by etc.
