# project -- Semantic Map

**Purpose:** portable issues tracker

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

`[HOTSPOT]` High fan-in file imported by 4+ others - request this file early in any task

`[GLOBAL-UTIL]` High fan-in utility imported from 3+ distinct domains

`[DOMAIN-CONTRACT]` Shared contract imported mostly by one subsystem

`[ROLE:model]` Primary domain model or state-holding data structure.

`[ROLE:controller]` Coordinates commands, events, or request handling.

`[ROLE:rendering]` Produces visual output or drawing behavior.

`[ROLE:view]` Represents a reusable UI view or presentation component.

`[ROLE:dialog]` Implements dialog-oriented interaction flow.

`[ROLE:config]` Defines configuration loading or configuration schema behavior.

`[ROLE:os-integration]` Bridges the application to OS-specific APIs or services.

`[ROLE:utility]` Provides cross-cutting helper logic without owning core flow.

`[ROLE:bootstrap]` Initializes the application or wires subsystem startup.

`[ROLE:build-only]` Supports the build toolchain rather than runtime behavior.

`[COUPLING:pure]` Logic stays within the language/runtime without external surface coupling.

`[COUPLING:mixed]` Blends pure logic with side effects or boundary interactions.

`[COUPLING:ui-coupled]` Depends directly on UI framework, rendering, or windowing APIs.

`[COUPLING:os-coupled]` Depends directly on operating-system services or platform APIs.

`[COUPLING:build-only]` Only relevant during build, generation, or compilation steps.

`[BEHAVIOR:owns-state]` Maintains durable in-memory state for a subsystem.

`[BEHAVIOR:mutates]` Changes application or model state in response to work.

`[BEHAVIOR:renders]` Produces rendered output, drawing commands, or visual layout.

`[BEHAVIOR:dispatches]` Routes commands, events, or control flow to other units.

`[BEHAVIOR:observes]` Listens to callbacks, notifications, or external signals.

`[BEHAVIOR:persists]` Reads from or writes to durable storage.

`[BEHAVIOR:spawns-worker]` Creates background workers, threads, or async jobs.

`[BEHAVIOR:sync-primitives]` Coordinates execution with locks, channels, or wait primitives.

`[SURFACE:filesystem]` Touches filesystem paths, files, or directory traversal.

`[SURFACE:ntfs]` Uses NTFS-specific filesystem semantics or metadata.

`[SURFACE:win32]` Touches Win32 platform APIs or Windows-native handles.

`[SURFACE:shell]` Integrates with shell commands, shell UX, or command launch surfaces.

`[SURFACE:clipboard]` Reads from or writes to the system clipboard.

`[SURFACE:gdi]` Uses GDI drawing primitives or related graphics APIs.

`[SURFACE:control]` Represents or manipulates widget/control surfaces.

`[SURFACE:view]` Represents a view-level presentation surface.

`[SURFACE:dialog]` Represents a dialog/window interaction surface.

`[SURFACE:document]` Represents document-oriented editing or display surfaces.

`[SURFACE:frame]` Represents application frame/window chrome surfaces.

## Layer 0 -- Config

`Cargo.toml`
Workspace configuration.

`Dioxus.toml`
Configuration for Dioxus.

`README.md`
Project overview and usage guide.

`SEMMAP.md`
Generated semantic map.

`neti.toml`
Configuration for neti.

## Layer 1 -- Domain (Engine)

`src/model/cli.rs`
Implements cli set status. [TYPE]
Exports: cli_set_status, cli_delete, cli_heatmap, cli_lint
Touch: Contains inline Rust tests alongside runtime code.

`src/model/lint.rs`
Implements lint workspace. [TYPE]
Exports: lint_markdown, lint_workspace

`src/model/workspace.rs`
Removes issue. [TYPE]
Exports: allocate_issue_id, dependency_edges, file_heatmap, delete_issue
Touch: Contains inline Rust tests alongside runtime code.

`src/ui/app.rs`
Implements App functionality.
Exports: App
Touch: Contains inline Rust tests alongside runtime code.

`src/ui/components.rs`
Implements label tone class.
Exports: SectionBadgeRow, label_tone_class, NavBtn, StatRow
Touch: Contains inline Rust tests alongside runtime code.

`src/ui/feed_lens.rs`
Implements feed lens.
Touch: Contains inline Rust tests alongside runtime code.

`src/ui/scroll.rs`
Scroll physics engine - pure Rust.
Exports: add_wheel_delta, jump_to_top, set_is_scrolling, ScrollPhysics
Touch: Contains inline Rust tests alongside runtime code.

`src/ui/toast.rs`
Implements toast container.
Exports: ToastContainer, ToastKind, class, Toast

`src/ui/views/board.rs`
Implements board view props.
Exports: BoardViewProps, BoardView
Touch: Contains inline Rust tests alongside runtime code.

`src/ui/views/feed.rs`
Implements drag debug state. [TYPE]
Exports: RecentDropState, FeedViewProps, DragDebugState, DragPresence
Touch: Contains inline Rust tests alongside runtime code.

`src/ui/views/feed/card.rs`
Implements drag overlay props.
Exports: DragOverlayProps, IssueCardProps, DragOverlay, IssueCard

`src/ui/views/viz.rs`
Implements heatmap view.
Exports: GraphView, HeatmapView, TimelineView
Touch: Contains inline Rust tests alongside runtime code.

`src/ui/welcome.rs`
Implements welcome screen.
Exports: WelcomeScreen

## Layer 2 -- Adapters / Infra

`src/model/parse.rs`
Parses markdown. [UTIL]
Exports: parse_markdown
Touch: Contains inline Rust tests alongside runtime code.

## Layer 3 -- App / Entrypoints

`assets/style.css`
Implements style functionality. styles.

`dragging-prototype.html`
Smooth Reorder (Engine-grade, no release jitter)

`src/main.rs`
Application entry point. [ENTRY]

`src/model/mod.rs`
Defines shared mod for the model subsystem. [TYPE] [HOTSPOT] [DOMAIN-CONTRACT]
Exports: issue_id_sort_key, parse_categorical_issue_id, default_init_path, init_workspace_at
Touch: Contains inline Rust tests alongside runtime code.

`src/ui/mod.rs`
Gets the workspace path. [ENTRY] [HOTSPOT]
Exports: get_workspace_path, launch_dashboard, View

`src/ui/views/mod.rs`
Module definitions for mod. [ENTRY]

## Layer 4 -- Tests

`src/ui/views/physics/tests.rs`
Tests for super.

