# project -- Semantic Map

**Purpose:** portable issues tracker

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

`[HOTSPOT]` High fan-in file imported by 4+ others — request this file early in any task

## Layer 0 -- Config

`Cargo.toml`
Workspace configuration.

`Dioxus.toml`
Configuration for Dioxus.

`neti.toml`
Configuration for neti.

## Layer 1 -- Core

`src/main.rs`
Orchestrates `clap`.

`src/model/mod.rs`
Searches common subdirectories for issue markdown files.
→ Exports: css_class, dependency_edges, discover_root, file_heatmap

`src/ui/mod.rs`
Implements view functionality.
→ Exports: launch_dashboard, View

`src/ui/views/mod.rs`
Orchestrates `board`, `feed`, `viz`.

## Layer 2 -- Domain

`src/model/cli.rs`
Implements cli set status.
→ Exports: cli_set_status, cli_heatmap, cli_list, cli_show

`src/model/parse.rs`
Parses markdown.
→ Exports: parse_markdown

`src/ui/components.rs`
Implements stat row.
→ Exports: NavBtn, StatRow

`src/ui/styles.rs`
Implements styles functionality.

`src/ui/styles_viz.rs`
Implements styles viz.

`src/ui/views/board.rs`
Implements board view.
→ Exports: BoardView

`src/ui/views/feed.rs`
Implements feed view props.
→ Exports: build_virtual_layout, FeedViewProps, FeedView

`src/ui/views/feed/card.rs`
Implements issue card props.
→ Exports: IssueCardProps, IssueCard

`src/ui/views/physics.rs`
Implements step settle.
→ Exports: is_active, DragState, step_settle, step_drag

`src/ui/views/viz.rs`
Implements heatmap view.
→ Exports: GraphView, HeatmapView, TimelineView

