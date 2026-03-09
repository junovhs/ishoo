mod app;
mod components;
pub(crate) mod scroll;
mod toast;
mod views;
mod welcome;

use std::path::PathBuf;
use std::sync::OnceLock;

pub use app::App;
use dioxus::desktop::{Config, WindowBuilder};

static WORKSPACE_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn launch_dashboard(path: PathBuf) {
    WORKSPACE_ROOT
        .set(path)
        .expect("workspace path already set");
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("Ishoo")
                    .with_decorations(false)
                    .with_always_on_top(false),
            ),
        )
        .launch(App);
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
