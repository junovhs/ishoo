mod app;
mod components;
mod toast;
mod views;
mod welcome;

use std::path::PathBuf;
use std::sync::OnceLock;

pub use app::App;

static WORKSPACE_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn launch_dashboard(path: PathBuf) {
    WORKSPACE_ROOT
        .set(path)
        .expect("workspace path already set");
    dioxus::launch(App);
}

/// Returns the discovered workspace root (where issue files live)
pub fn get_workspace_path() -> PathBuf {
    WORKSPACE_ROOT.get().expect("path not set").clone()
}

#[derive(Clone, Copy, PartialEq)]
pub enum View {
    Feed,
    Board,
    Heatmap,
    Graph,
    Timeline,
}