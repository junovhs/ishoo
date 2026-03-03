# project -- Semantic Map

**Purpose:** portable issues tracker

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

## Layer 0 -- Config

`Cargo.toml`
Rust package manifest and dependencies. Centralizes project configuration.

`Dioxus.toml`
Configuration for Dioxus. Centralizes project configuration.

`neti.toml`
Configuration for neti. Centralizes project configuration.

## Layer 1 -- Core

`src/main.rs`
Orchestrates `clap`. Provides application entry point.

`src/model/mod.rs`
Module providing `Issue`, `Stats`, `Status`. Defines domain data structures.
→ Exports: Issue, Stats, Status, css_class, default_init_path, discover_root, from_str, init_workspace, init_workspace_at, label, reinit_workspace, status_ord, workspace_exists

`src/ui/mod.rs`
Module providing `View`, `get_workspace_path`, `launch_dashboard`. Supports application functionality.
→ Exports: View, get_workspace_path, launch_dashboard

`src/ui/views/mod.rs`
Orchestrates `board`, `feed`, `viz`. Supports application functionality.

## Layer 2 -- Domain

`src/model/cli.rs`
Module providing `cli_heatmap`, `cli_list`, `cli_set_status`. Defines domain data structures.
→ Exports: cli_heatmap, cli_list, cli_set_status, cli_show

`src/model/parse.rs`
Module providing `parse_markdown`. Parses input into structured data.
→ Exports: parse_markdown

`src/model/workspace.rs`
Module providing `Workspace`, `dependency_edges`, `file_heatmap`. Defines domain data structures.
→ Exports: Workspace, dependency_edges, file_heatmap, load, save, stats

`src/ui/app.rs`
Module providing `App`. Supports application functionality.
→ Exports: App

`src/ui/components.rs`
Module providing `NavBtn`, `StatRow`. Supports application functionality.
→ Exports: NavBtn, StatRow

`src/ui/styles.rs`
Implements styles functionality. Supports application functionality.

`src/ui/styles_viz.rs`
Implements styles viz. Supports application functionality.

`src/ui/toast.rs`
Module providing `Toast`, `ToastContainer`, `ToastKind`. Supports application functionality.
→ Exports: Toast, ToastContainer, ToastKind, class

`src/ui/views/board.rs`
Module providing `BoardView`. Supports application functionality.
→ Exports: BoardView

`src/ui/views/feed.rs`
Module providing `FeedView`, `FeedViewProps`, `build_virtual_layout`. Supports application functionality.
→ Exports: FeedView, FeedViewProps, build_virtual_layout

`src/ui/views/feed/card.rs`
Module providing `IssueCard`, `IssueCardProps`. Supports application functionality.
→ Exports: IssueCard, IssueCardProps

`src/ui/views/physics.rs`
Module providing `DragState`, `Spring`, `done`. Supports application functionality.
→ Exports: DragState, Spring, done, is_active, new, set, step, step_drag, step_settle

`src/ui/views/viz.rs`
Module providing `GraphView`, `HeatmapView`, `TimelineView`. Supports application functionality.
→ Exports: GraphView, HeatmapView, TimelineView

`src/ui/welcome.rs`
Module providing `WelcomeScreen`. Supports application functionality.
→ Exports: WelcomeScreen

