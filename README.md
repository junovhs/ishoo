# Ishoo

Ishoo is a local-first Markdown issue tracker built for both AI agents and human developers. It stores issues as clean, Git-tracked Markdown files inside the repository rather than in a database or SaaS backend, so the system stays portable, readable, and usable even without the app installed. On top of that source of truth, Ishoo provides a native desktop dashboard for visually managing work through Kanban boards, drag-and-drop feeds, dependency graphs, file heatmaps, timelines, and inline editing.

## Philosophy: The "Stealth" Tracker

Your issues live as plain `.md` files in your repo. No database. No SaaS. Fully portable, fully AI-friendly.

**Zero Tooling Footprint:** If someone clones your repo without Ishoo installed, they won't see a proprietary `.sqlite` database or a massive JSON blob. They just see a beautifully organized `docs/issues/` folder that looks like it was manually typed by an incredibly disciplined developer. 

Because it uses standard markdown, your issue tracker renders natively in the GitHub/GitLab web UI, works flawlessly with AI coding assistants (Cursor, Copilot, Claude), and branches perfectly with your Git workflow.

## Quick Start

```bash
# 1. Initialize a new issue tracker in your current project
ishoo init

# 2. Create an issue from the terminal
ishoo new "Fix the widget"

# 3. Check issue hygiene before committing
ishoo lint --strict

# 4. Launch the desktop dashboard (auto-discovers your issues)
ishoo
```

## File Layout

By default, Ishoo creates and manages files inside a `docs/issues/` directory to keep your project root clean (though it auto-discovers issues placed in `.`, `docs`, `issues`, or `.issues`).

```text
your-project/
└── docs/
    └── issues/
        ├── issues-active.md    # Open + In-Progress issues
        ├── issues-backlog.md   # Future work
        └── issues-done.md      # Completed issues
```

The built-in CLI and lint flow operate on those three core files. Custom files such as `issues-graphics.md` are fine for extension workflows, but the current CLI lint pass only validates the built-in set.

## CLI Usage

While the GUI is great for getting oriented, you can manage the current core workflow directly from the terminal:

```bash
ishoo init                                   # Create the docs/issues/ folder structure
ishoo                                        # Launch the desktop UI (default)
ishoo dash                                   # Explicitly launch the desktop UI
ishoo list                                   # List all issues with summary stats
ishoo list --filter "database"               # Filter list output by title, description, or file path
ishoo show ISS-01                            # View one issue in the terminal
ishoo set ISS-01 "in progress"               # Update issue status and save
ishoo new "Fix the widget"                   # Create a new OPEN issue in the active section
ishoo new "Urgent fix" --category bug        # Create a new issue with a different ID prefix
ishoo new "Close this out" --status done     # Create an issue with a specific status
ishoo delete ISS-01                          # Prompt before permanent deletion
ishoo delete ISS-01 --force                  # Delete without confirmation
ishoo lint                                   # Report issue-file warnings
ishoo lint --strict                          # Exit non-zero on lint findings
ishoo heatmap                                # CLI visualization of file hotspots
```

`ishoo lint` currently checks for duplicate IDs, broken dependency references, missing required authored fields, empty titles, and core-file coherence for `issues-active.md`, `issues-backlog.md`, and `issues-done.md`.

## Issue Authoring

Ishoo stores issues as Markdown records. For reliable CLI and AI-agent workflows, keep these fields present on every issue:

- `**Status:**`
- `**Labels:**`
- `**Resolution:**`

The `Resolution` field may be empty for unfinished work, but the field should still exist so the record structure stays stable.

## Dashboard Features

When you run `ishoo`, you get a native, hardware-accelerated desktop app with rich visualizations and editing tools.

### Views
- **Feed** — A linear task list with a custom spring-physics engine for buttery-smooth drag-and-drop reordering.
- **Board** — A standard Kanban board organized by status (Open, In Progress, Done).
- **Heatmap** — Codebase hotspot visualization (shows which files are touched by the most issues).
- **Graph** — A node graph mapping out issue dependencies and file-overlap connections.
- **Timeline** — An overall project progress bar and status-sorted list.

### Editing
- **Create & Reset** — Add new issues instantly, or wipe the board clean with the Reinitialize modal.
- **Frictionless Reordering** — Drag and drop cards in the Feed view; changes auto-save with a toast notification.
- **Inline Editing** — Expand any issue card to edit description, status, labels, files, and resolution notes.
- **Smart Syncing** — Unsaved UI edits trigger an unsaved state. Use Save All to write changes cleanly back to Markdown.

## Building from Source

Ishoo is built with Rust and [Dioxus](https://dioxuslabs.com/).

```bash
cargo install cargo-binstall
cargo binstall dioxus-cli

# Run in development mode
dx serve --desktop

# Bundle a release binary for your OS
dx bundle --release

# Run the repo verification gate
neti check
```

## License

MIT
