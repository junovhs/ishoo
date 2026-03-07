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

## Layer 1 -- Domain (Engine)

`README.md`
Handles README.

`SEMMAP.md`
Handles SEMMAP.

`src/model/cli.rs`
Implements cli set status. [TYPE]
→ Exports: cli_set_status, cli_heatmap, cli_list, cli_show

`src/model/workspace.rs`
Implements Workspace functionality. [TYPE]
→ Exports: dependency_edges, file_heatmap, Workspace, load

`src/ui/app.rs`
Implements App functionality.
→ Exports: App

`src/ui/components.rs`
Implements stat row.
→ Exports: NavBtn, StatRow

`src/ui/toast.rs`
Implements toast container.
→ Exports: ToastContainer, ToastKind, class, Toast

`src/ui/views/board.rs`
Implements board view.
→ Exports: BoardView

`src/ui/views/feed.rs`
Implements feed view props.
→ Exports: DragState, FeedViewProps, FeedView

`src/ui/views/feed/card.rs`
Implements issue card props.
→ Exports: IssueCardProps, IssueCard

`src/ui/views/viz.rs`
Implements timeline view.
→ Exports: GraphView, HeatmapView, TimelineView

`src/ui/welcome.rs`
Implements welcome screen.
→ Exports: WelcomeScreen

## Layer 2 -- Adapters / Infra

`src/model/parse.rs`
Parses markdown. [UTIL]
→ Exports: parse_markdown

## Layer 3 -- App / Entrypoints

`assets/style.css`
Implements style functionality. styles.

`dragging-prototype.html`
Smooth Reorder (Engine-grade, no release jitter)

`src/main.rs`
Application entry point. [ENTRY]

`src/model/mod.rs`
Implements status ord. [TYPE]
→ Exports: default_init_path, init_workspace_at, css_class, discover_root

`src/ui/mod.rs`
Gets the workspace path. [ENTRY]
→ Exports: get_workspace_path, launch_dashboard, View

`src/ui/views/mod.rs`
Module definitions for mod. [ENTRY]

## Layer 4 -- Tests

`src/ui/views/physics/tests.rs`
Tests for super.

