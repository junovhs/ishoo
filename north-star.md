# North Star

This repo is a portable issues tracker with two equal priorities:

1. Preserve correctness of issue storage and parsing.
2. Preserve the feel of the UI, especially Feed and Board interactions.

## Operating Principles

- Orient before editing. Use `semmap generate`, read `SEMMAP.md`, and use `semmap trace` when flow or ownership is unclear.
- Read the minimum set of files needed for the current issue. Expand only when the current evidence is insufficient.
- Treat `src/model/*` as persistence and domain truth. Do not break issue loading, saving, IDs, sections, labels, links, or status transitions while working on UI behavior.
- Treat Feed drag behavior as the reference interaction model. Board should reuse Feed mechanics rather than approximate them.
- Prefer small diffs that respect existing boundaries over broad rewrites.
- Verify every change with tests or targeted checks, then run `neti check`.

## Current Priorities

- Active work starts in `docs/issues/issues-active.md`, then backlog if active is empty.
- Board/Feed parity work should preserve exact vertical drag feel while limiting Board-specific logic to lane targeting and layout.
- When changing issue workflow, update the issue record with a concrete resolution and verification notes.

## Done Criteria

An issue is done only when:

- behavior is implemented, not described,
- verification proves it,
- relevant edge cases are covered,
- the issue record states what changed, why, and how it was verified.

## Constraints

- Do not ignore existing repo state. If `neti check` reports pre-existing failures, distinguish them from regressions you introduced.
- Do not rely on placeholder docs or vague parity claims. Prefer explicit tests, explicit commands, and explicit manual checks where behavior is visual.
