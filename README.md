# Ishoo

Ishoo is a local-first, Markdown-native issue tracker for human developers and AI agents.

Instead of storing work in a database or SaaS backend, Ishoo keeps issues as clean, Git-tracked `.md` files inside your repository. That makes your issue tracker portable, readable, branch-friendly, and usable even without Ishoo installed.

On top of that repo-native source of truth, Ishoo provides a smooth native desktop app for managing work through a feed, Kanban board, dependency graph, file heatmap, timeline, and inline editing.

## Why Ishoo

Most issue trackers hide your work inside a service or database.

Ishoo does the opposite.

- **Plain Markdown in your repo** — issues live as readable `.md` files
- **No hidden backend** — no SQLite file, no proprietary blob, no SaaS dependency
- **Git-native workflow** — issues branch, diff, merge, and travel with the code
- **AI-friendly by default** — agents can read and update the same issue files humans use
- **Native desktop UI** — fast visual workflow without giving up portable storage

## Philosophy: The “Stealth” Tracker

Your issue tracker should not leave behind weird infrastructure.

With Ishoo, someone can clone your repo without installing anything and still see a clean `docs/issues/` folder full of readable Markdown. It looks like a disciplined project, not a tool-specific data dump.

No database. No lock-in. No “export” step.

Because Ishoo uses standard Markdown:

- issues render naturally in GitHub and GitLab
- they work well with AI coding tools like Cursor, Claude, and Copilot
- they stay inspectable with normal editor, grep, and Git workflows
- they remain useful even if the app disappears

The desktop app is an interaction layer, not the owner of your data.

## Quick Start

```bash
# Initialize an issue tracker in the current repo
ishoo init

# Create a new issue from the terminal
ishoo new "Fix the widget"

# Check issue hygiene before commit or CI
ishoo lint --strict

# Launch the desktop dashboard
ishoo
````

## File Layout

By default, Ishoo stores issues in `docs/issues/` to keep the project root clean.

```text
your-project/
└── docs/
    └── issues/
        ├── issues-active.md
        ├── issues-backlog.md
        └── issues-done.md
```

These three files form the built-in workflow:

* `issues-active.md` — open and in-progress work
* `issues-backlog.md` — future work
* `issues-done.md` — completed or descoped work

Ishoo can also discover issues placed in `.`, `docs`, `issues`, or `.issues`.

Custom files like `issues-graphics.md` are fine for extension workflows, though the current CLI lint pass validates the built-in core set.

## CLI Usage

You can manage the main workflow directly from the terminal:

```bash
ishoo init                                   # Create the docs/issues/ folder structure
ishoo                                        # Launch the desktop UI
ishoo dash                                   # Explicitly launch the desktop UI
ishoo list                                   # List all issues with summary stats
ishoo list --filter "database"               # Filter by title, description, or file path
ishoo show ISS-01                            # Show one issue in the terminal
ishoo set ISS-01 "in progress"               # Update issue status and save
ishoo new "Fix the widget"                   # Create a new OPEN issue in active
ishoo new "Urgent fix" --category bug        # Create an issue with a different ID prefix
ishoo new "Close this out" --status done     # Create an issue with a specific status
ishoo delete ISS-01                          # Prompt before permanent deletion
ishoo delete ISS-01 --force                  # Delete without confirmation
ishoo lint                                   # Report issue-file warnings
ishoo lint --strict                          # Exit non-zero on lint findings
ishoo heatmap                                # CLI visualization of file hotspots
```

## What `ishoo lint` Checks

`ishoo lint` currently validates:

* duplicate issue IDs
* broken dependency references
* missing required authored fields
* empty titles
* coherence across the built-in core files:

  * `issues-active.md`
  * `issues-backlog.md`
  * `issues-done.md`

In the built-in done file, both `DONE` and `DESCOPED` are valid terminal statuses.

## Issue Authoring Format

Ishoo stores issues as Markdown records.

For reliable CLI and AI-agent workflows, keep these fields present on every issue:

* `**Status:**`
* `**Labels:**`
* `**Resolution:**`

`Resolution` may be empty for unfinished work, but the field should still exist so the record shape stays stable.

## Dashboard Features

Run `ishoo` to open the native desktop app.

The app sits on top of your Markdown files and gives you a faster way to browse, edit, and reorganize work without giving up repo-native storage.

### Views

* **Feed** — a linear work queue with smooth drag-and-drop reordering
* **Board** — Kanban-style columns by status
* **Heatmap** — shows which files are touched by the most issues
* **Graph** — issue dependency and file-overlap visualization
* **Timeline** — overall progress and status-oriented project view

### Editing

* **Create & Reset** — create issues quickly or reinitialize the tracker
* **Reordering** — drag cards in Feed view and save changes back to Markdown
* **Inline Editing** — edit description, status, labels, files, and resolution notes inside the app
* **Smart Syncing** — UI edits track unsaved state and can be written back cleanly with Save All

## AI Agent Workflow

Ishoo is designed to work well with AI-assisted development.

Because issues live as plain Markdown in the repo:

* agents can read them without a connector or hidden backend
* issue context travels naturally with the codebase
* task lists remain visible in normal repository exploration
* issue edits can be reviewed in Git like any other change

This makes Ishoo a useful shared medium between humans, editors, terminals, and coding agents.

## Building from Source

Ishoo is built with Rust and [Dioxus](https://dioxuslabs.com/).

```bash
cargo install cargo-binstall
cargo binstall dioxus-cli

# Run in development mode
dx serve --desktop

# Bundle a release binary
dx bundle --release

# Run the verification gate
neti check
```

## License

MIT
