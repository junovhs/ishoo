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

# 2. Launch the desktop dashboard (auto-discovers your issues)
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
        └── issues-done.md      # Completed & Descoped issues
```

## CLI Usage

While the GUI is great for getting oriented, you (or your AI agent) can manage everything directly from the terminal:

```bash
ishoo init                             # Create the docs/issues/ folder structure
ishoo                                  # Launch the desktop UI (default)
ishoo dash                             # Explicitly launch the desktop UI
ishoo list                             # List all issues with stats
ishoo list --filter "database"         # Filter issues by keyword
ishoo show 47                          # View detailed issue card (desc, files, deps)
ishoo set 47 "in progress"             # Update issue status
ishoo new "Fix the widget"             # Create a new OPEN issue
ishoo new "Urgent fix" --status done   # Create an issue with a specific status
ishoo heatmap                          # CLI visualization of file hotspots
```

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
- **Inline Editing** — Expand any issue card to edit its description, status, or resolution notes. 
- **Smart Syncing** — Unsaved UI edits trigger a "⚠ Unsaved" state. Click "Save All" to write your changes cleanly back to Markdown.

## Building from Source

Ishoo is built with Rust and [Dioxus](https://dioxuslabs.com/).

```bash
cargo install cargo-binstall
cargo binstall dioxus-cli

# Run in development mode
dx serve --desktop

# Bundle a release binary for your OS
dx bundle --release
```

## License

MIT
