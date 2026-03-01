# Linearis

A portable markdown-based issue tracker with a beautiful Dioxus desktop UI.

## Philosophy

Your issues live as plain `.md` files in your repo. No database. No SaaS. Fully portable, fully AI-friendly.

Most of the time, you (or AI) manage issues via CLI. When a human wants to get oriented, `linearis dash` spawns a native desktop UI with rich visualizations.

## File Layout

```
your-project/
├── issues-active.md    # In-progress + open issues
├── issues-backlog.md   # Future work
└── issues-done.md      # Completed issues
```

## CLI Usage

```bash
linearis list                          # List all issues
linearis list --filter "python"        # Filter issues
linearis show 47                       # Show issue detail
linearis set 47 done                   # Set status
linearis new "Fix the widget"          # Create new issue
linearis heatmap                       # File hotspot visualization
linearis dash                          # Launch desktop dashboard
linearis                               # Also launches dashboard
```

## Dashboard Views

- **Feed** — Linear-style issue list with expand/collapse, search, inline editing
- **Board** — Kanban columns by status
- **Heatmap** — File hotspot visualization
- **Graph** — Dependency + file-overlap graph
- **Timeline** — Progress bar + sorted issue list

## Building

```bash
cargo install cargo-binstall
cargo binstall dioxus-cli
dx serve --desktop        # Development
dx bundle --release       # Release build
```
