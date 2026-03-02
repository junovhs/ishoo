# ACTIVE Issues

---

## [6] Move CSS constants to bundled assets
**Status:** OPEN
**Files:** `src/ui/styles.rs`, `src/ui/styles_viz.rs`, `Dioxus.toml`

Injecting massive strings of CSS via `styles.rs` works for the prototype, but it makes editing CSS painful (no syntax highlighting).
Move these to standard `.css` files in an `assets/` folder and let Dioxus bundle them automatically.

**Resolution:** 

---

## [4] Replace polling with OS file system events
**Status:** IN PROGRESS
**Files:** `src/ui/app.rs`, `Cargo.toml`

Currently, the dashboard uses a 3-second `tokio::time::sleep` loop to poll for external changes. We should replace this with the `notify` crate to listen for OS-level file system events (FSEvents/inotify).
This will reduce idle CPU usage and make external CLI edits (or Git branch changes) reflect in the UI instantly.

**Resolution:** 

---
