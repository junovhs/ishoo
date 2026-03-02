# Ishoo

A portable markdown-based issue tracker with a beautiful Dioxus desktop UI.

## Philosophy

Your issues live as plain `.md` files in your repo. No database. No SaaS. Fully portable, fully AI-friendly.

Most of the time, you (or AI) manage issues via CLI. When a human wants to get oriented, `ishoo dash` spawns a native desktop UI with rich visualizations.

## Quick Start

```bash
# Initialize a new issue tracker
ishoo init

# Launch the dashboard (also the default)
ishoo
```

## File Layout

```
your-project/
├── issues-active.md    # In-progress + open issues
├── issues-backlog.md   # Future work
└── issues-done.md      # Completed issues
```

## CLI Usage

```bash
ishoo init                             # Initialize issue tracker
ishoo                                  # Launch dashboard (default)
ishoo dash                             # Launch dashboard (explicit)
ishoo list                             # List all issues
ishoo list --filter "python"           # Filter issues
ishoo show 47                          # Show issue detail
ishoo set 47 done                      # Set status
ishoo new "Fix the widget"             # Create new issue
ishoo new "Urgent fix" --status "in progress"
ishoo heatmap                          # File hotspot visualization
```

## Dashboard Features

### Views
- **Feed** — Linear-style issue list with expand/collapse, drag-to-reorder, inline editing
- **Board** — Kanban columns by status
- **Heatmap** — File hotspot visualization
- **Graph** — Dependency + file-overlap graph
- **Timeline** — Progress bar + sorted issue list

### Editing
- **New Issue** — Click "+ New" in the topbar or sidebar
- **Reorder** — Drag issues to reorder (auto-saves with toast notification)
- **Edit Status/Resolution** — Expand any issue card, changes require manual save
- **Toast Notifications** — Top-right corner feedback for all save operations

## Building

```bash
cargo install cargo-binstall
cargo binstall dioxus-cli
dx serve --desktop        # Development
dx bundle --release       # Release build
```

## License

MIT
