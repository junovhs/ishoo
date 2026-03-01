mod components;
mod styles;
mod styles_viz;
mod views;

use crate::model::{Issue, Stats, Status, Workspace};
use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::OnceLock;

static WORKSPACE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn launch_dashboard(path: PathBuf) {
    WORKSPACE_PATH
        .set(path)
        .expect("workspace path already set");
    dioxus::launch(App);
}

#[derive(Clone, Copy, PartialEq)]
pub enum View {
    Feed,
    Board,
    Heatmap,
    Graph,
    Timeline,
}

#[component]
fn App() -> Element {
    let ws_path = WORKSPACE_PATH
        .get()
        .expect("workspace path not set")
        .clone();

    let initial = Workspace::load(&ws_path).unwrap_or_else(|_| Workspace {
        root: ws_path.clone(),
        issues: vec![],
    });

    let mut issues = use_signal(|| initial.issues);
    let mut search_query = use_signal(String::new);
    let mut active_issue_id = use_signal(|| None::<u32>);
    let mut active_view = use_signal(|| View::Feed);
    let mut dirty = use_signal(|| false);

    let _poll = use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let wp = ws_path.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                if let Ok(ws) = Workspace::load(&wp) {
                    if !dirty() {
                        issues.set(ws.issues);
                    }
                }
            }
        }
    });

    let save = move |_| {
        let wp = WORKSPACE_PATH.get().expect("path not set").clone();
        let ws = Workspace {
            root: wp,
            issues: issues(),
        };
        if let Err(e) = ws.save() {
            eprintln!("Save error: {e}");
        } else {
            dirty.set(false);
        }
    };

    let stats = compute_stats(&issues());
    let filtered: Vec<Issue> = filter_issues(&issues(), &search_query());
    let all_styles = format!("{}\n{}", styles::STYLES, styles_viz::STYLES_VIZ);

    rsx! {
        style { {all_styles} }
        div { class: "shell",
            aside { class: "sidebar",
                div { class: "brand",
                    svg {
                        width: "24", height: "24", view_box: "0 0 24 24",
                        fill: "none", stroke: "currentColor", stroke_width: "2.5",
                        path { d: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" }
                    }
                    span { "Ishoo" }
                }

                nav { class: "nav",
                    div { class: "nav-label", "Views" }
                    components::NavBtn { label: "Feed", active: active_view() == View::Feed, onclick: move |_| active_view.set(View::Feed) }
                    components::NavBtn { label: "Board", active: active_view() == View::Board, onclick: move |_| active_view.set(View::Board) }
                    components::NavBtn { label: "Heatmap", active: active_view() == View::Heatmap, onclick: move |_| active_view.set(View::Heatmap) }
                    components::NavBtn { label: "Graph", active: active_view() == View::Graph, onclick: move |_| active_view.set(View::Graph) }
                    components::NavBtn { label: "Timeline", active: active_view() == View::Timeline, onclick: move |_| active_view.set(View::Timeline) }
                }

                div { class: "stats-area",
                    div { class: "nav-label", "Metrics" }
                    div { class: "stat-list",
                        components::StatRow { label: "Backlog", count: stats.open, color: "var(--c-blue)" }
                        components::StatRow { label: "In Flight", count: stats.in_progress, color: "var(--c-amber)" }
                        components::StatRow { label: "Resolved", count: stats.done, color: "var(--c-green)" }
                        if stats.descoped > 0 {
                            components::StatRow { label: "Descoped", count: stats.descoped, color: "var(--c-muted)" }
                        }
                    }
                }

                div { class: "sidebar-foot",
                    div { class: if dirty() { "sync dirty" } else { "sync" },
                        if dirty() { "⚠ Unsaved" } else { "✓ Synced" }
                    }
                    if dirty() {
                        button { class: "save-btn", onclick: save, "Save All" }
                    }
                }
            }

            main { class: "main",
                div { class: "topbar",
                    div { class: "search-box",
                        input {
                            class: "search",
                            r#type: "text",
                            placeholder: "Search issues, files, or #id…",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }
                    span { class: "count-pill", "{stats.total} issues" }
                }

                div { class: "content",
                    match active_view() {
                        View::Feed => rsx! {
                            views::FeedView {
                                issues: filtered.clone(),
                                active_id: active_issue_id(),
                                on_toggle: move |id: u32| {
                                    if active_issue_id() == Some(id) { active_issue_id.set(None); }
                                    else { active_issue_id.set(Some(id)); }
                                },
                                on_status: move |(id, s): (u32, String)| {
                                    let mut all = issues();
                                    if let Some(i) = all.iter_mut().find(|i| i.id == id) { i.status = Status::from_str(&s); }
                                    issues.set(all);
                                    dirty.set(true);
                                },
                                on_resolution: move |(id, t): (u32, String)| {
                                    let mut all = issues();
                                    if let Some(i) = all.iter_mut().find(|i| i.id == id) { i.resolution = t; }
                                    issues.set(all);
                                    dirty.set(true);
                                },
                            }
                        },
                        View::Board => rsx! { views::BoardView { issues: filtered.clone() } },
                        View::Heatmap => rsx! { views::HeatmapView { issues: issues() } },
                        View::Graph => rsx! { views::GraphView { issues: issues() } },
                        View::Timeline => rsx! { views::TimelineView { issues: issues() } },
                    }
                }
            }
        }
    }
}

fn compute_stats(issues: &[Issue]) -> Stats {
    let mut s = Stats::default();
    for i in issues {
        match i.status {
            Status::Open => s.open += 1,
            Status::InProgress => s.in_progress += 1,
            Status::Done => s.done += 1,
            Status::Descoped => s.descoped += 1,
        }
    }
    s.total = issues.len();
    s
}

fn filter_issues(issues: &[Issue], query: &str) -> Vec<Issue> {
    let q = query.to_lowercase();
    if q.is_empty() {
        return issues.to_vec();
    }
    issues
        .iter()
        .filter(|i| {
            i.title.to_lowercase().contains(&q)
                || i.id.to_string().contains(&q)
                || i.files.iter().any(|f| f.to_lowercase().contains(&q))
        })
        .cloned()
        .collect()
}
