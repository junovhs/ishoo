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

`src/ui/mod.rs`
Implements launch dashboard.
→ Exports: launch_dashboard, View

## Layer 2 -- Domain

`src/model.rs`
Implements cli set status.
→ Exports: cli_set_status, css_class, dependency_edges, from_str

`src/ui.rs`
Implements launch dashboard.
→ Exports: launch_dashboard

`src/ui/components.rs`
Implements nav btn.
→ Exports: NavBtn, StatRow

`src/ui/styles.rs`
Implements styles functionality.

`src/ui/views.rs`
Implements feed view props.
→ Exports: FeedViewProps, BoardView, GraphView, HeatmapView

