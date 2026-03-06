use super::toast::{Toast, ToastContainer, ToastKind};
use super::welcome::WelcomeScreen;
use super::{components, get_workspace_path, styles, styles_viz, views, View};
use crate::model::{reinit_workspace, workspace_exists, Issue, Stats, Status, Workspace};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct AppState {
    issues: Signal<Vec<Issue>>,
    toasts: Signal<Vec<Toast>>,
    toast_id: Signal<u64>,
}

#[component]
pub fn App() -> Element {
    let ws_path = get_workspace_path();
    let mut initialized = use_signal(|| workspace_exists(&ws_path));

    if !initialized() {
        return rsx! {
            style { {styles::BASE} }
            style { {styles::MODAL} }
            WelcomeScreen { path: ws_path, on_init: move |_| initialized.set(true) }
        };
    }

    render_dashboard(ws_path)
}

fn render_dashboard(ws_path: std::path::PathBuf) -> Element {
    let initial = Workspace::load(&ws_path).unwrap_or_else(|_| Workspace {
        root: ws_path.clone(),
        issues: vec![],
    });

    let mut issues = use_signal(|| initial.issues);
    let search = use_signal(String::new);
    let view = use_signal(|| View::Feed);
    let dirty = use_signal(|| false);
    let modal = use_signal(|| false);
    let reinit_modal = use_signal(|| false);
    let mut toasts = use_signal(Vec::<Toast>::new);
    let toast_id = use_signal(|| 0u64);

    let state = AppState {
        issues,
        toasts,
        toast_id,
    };

    let _poll = {
        let poll_path = ws_path.clone();
        use_coroutine(move |_rx: UnboundedReceiver<()>| {
            let path = poll_path.clone();
            async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    if let Ok(ws) = Workspace::load(&path) {
                        if !dirty() {
                            issues.set(ws.issues);
                        }
                    }
                }
            }
        })
    };

    let stats = compute_stats(&(state.issues)());
    let filtered = filter_issues(&(state.issues)(), &search());

    let all_styles = format!(
        "{}\n{}\n{}\n{}\n{}",
        styles::BASE,
        styles::CARD,
        styles::DRAG,
        styles::MODAL,
        styles_viz::STYLES_VIZ
    );

    rsx! {
        style { {all_styles} }
        ToastContainer {
            toasts: toasts(),
            on_dismiss: move |id| { toasts.write().retain(|t| t.id != id); }
        }

        if modal() { {render_new_issue_modal(modal, state)} }
        if reinit_modal() { {render_reinit_modal(reinit_modal, state)} }

        div { class: "shell",
            {render_sidebar(view, stats.clone(), dirty, modal, reinit_modal, state)}
            main { class: "main",
                {render_topbar(search, modal, &stats)}
                {render_content(view, filtered, dirty, state)}
            }
        }
    }
}

fn add_toast(state: AppState, msg: String, kind: ToastKind) {
    let id = (state.toast_id)();
    state.toast_id.clone().set(id + 1);
    state.toasts.clone().write().push(Toast {
        id,
        message: msg,
        kind,
    });
    let toasts = state.toasts;
    spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        toasts.clone().write().retain(|t| t.id != id);
    });
}

fn save_workspace(state: AppState, msg: &str) {
    let ws = Workspace {
        root: get_workspace_path(),
        issues: (state.issues)(),
    };
    match ws.save() {
        Ok(()) => add_toast(state, msg.to_string(), ToastKind::Success),
        Err(e) => add_toast(state, format!("Save failed: {e}"), ToastKind::Error),
    }
}

fn render_new_issue_modal(mut modal: Signal<bool>, state: AppState) -> Element {
    let mut title = use_signal(String::new);
    let mut status = use_signal(|| "OPEN".to_string());
    let mut issues = state.issues;

    rsx! {
        div { class: "modal-overlay", onclick: move |_| modal.set(false),
            div { class: "modal", onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h2 { "New Issue" }
                    button { class: "modal-close", onclick: move |_| modal.set(false), "×" }
                }
                div { class: "modal-body",
                    div { class: "fgroup",
                        label { class: "flbl", "Title" }
                        input {
                            class: "modal-input", placeholder: "What needs to be done?",
                            value: "{title}", oninput: move |e| title.set(e.value()),
                        }
                    }
                    div { class: "fgroup",
                        label { class: "flbl", "Status" }
                        select {
                            class: "sel", value: "{status}", onchange: move |e| status.set(e.value()),
                            option { value: "OPEN", "Open" }
                            option { value: "IN PROGRESS", "In Progress" }
                        }
                    }
                }
                div { class: "modal-footer",
                    button { class: "btn-secondary", onclick: move |_| modal.set(false), "Cancel" }
                    button {
                        class: "btn-primary", disabled: title().trim().is_empty(),
                        onclick: move |_| {
                            let t = title().trim().to_string();
                            if !t.is_empty() {
                                let max = issues().iter().map(|i| i.id).max().unwrap_or(0);
                                let issue = Issue {
                                    id: max + 1, title: t.clone(), status: Status::from_str(&status()),
                                    files: vec![], description: String::new(), resolution: String::new(),
                                    section: "ACTIVE Issues".to_string(), depends_on: vec![],
                                };
                                issues.write().insert(0, issue.clone());
                                modal.set(false);
                                save_workspace(state, &format!("Created #{} {}", issue.id, t));
                            }
                        },
                        "Create"
                    }
                }
            }
        }
    }
}

fn render_reinit_modal(mut modal: Signal<bool>, state: AppState) -> Element {
    let mut confirm_text = use_signal(String::new);
    let confirmed = confirm_text().trim().to_lowercase() == "erase my issues";
    let mut issues = state.issues;

    rsx! {
        div { class: "modal-overlay", onclick: move |_| modal.set(false),
            div { class: "modal modal-danger", onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h2 { "⚠️ Reinitialize Tracker" }
                    button { class: "modal-close", onclick: move |_| modal.set(false), "×" }
                }
                div { class: "modal-body",
                    p { class: "warning-text", "This will permanently delete all issues." }
                    p { class: "hint-text", "💡 Commit to git first so you have history." }
                    div { class: "fgroup",
                        label { class: "flbl", "Type \"erase my issues\" to confirm" }
                        input {
                            class: "modal-input", placeholder: "erase my issues",
                            value: "{confirm_text}", oninput: move |e| confirm_text.set(e.value()),
                        }
                    }
                }
                div { class: "modal-footer",
                    button { class: "btn-secondary", onclick: move |_| modal.set(false), "Cancel" }
                    button {
                        class: "btn-danger", disabled: !confirmed,
                        onclick: move |_| {
                            if confirmed {
                                match reinit_workspace(&get_workspace_path()) {
                                    Ok(()) => {
                                        issues.set(vec![]);
                                        modal.set(false);
                                        add_toast(state, "Reinitialized".to_string(), ToastKind::Success);
                                    }
                                    Err(e) => add_toast(state, format!("Failed: {e}"), ToastKind::Error),
                                }
                            }
                        },
                        "Erase Everything"
                    }
                }
            }
        }
    }
}

fn render_sidebar(
    mut view: Signal<View>,
    stats: Stats,
    mut dirty: Signal<bool>,
    mut modal: Signal<bool>,
    mut reinit_modal: Signal<bool>,
    state: AppState,
) -> Element {
    rsx! {
        aside { class: "sidebar",
            div { class: "brand",
                svg { width: "24", height: "24", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.5",
                    path { d: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" }
                }
                span { "Ishoo" }
            }
            nav { class: "nav",
                div { class: "nav-label", "Views" }
                components::NavBtn { label: "Feed", active: view() == View::Feed, onclick: move |_| view.set(View::Feed) }
                components::NavBtn { label: "Board", active: view() == View::Board, onclick: move |_| view.set(View::Board) }
                components::NavBtn { label: "Heatmap", active: view() == View::Heatmap, onclick: move |_| view.set(View::Heatmap) }
                components::NavBtn { label: "Graph", active: view() == View::Graph, onclick: move |_| view.set(View::Graph) }
                components::NavBtn { label: "Timeline", active: view() == View::Timeline, onclick: move |_| view.set(View::Timeline) }
            }
            div { class: "stats-area",
                div { class: "nav-label", "Metrics" }
                div { class: "stat-list",
                    components::StatRow { label: "Backlog", count: stats.open, color: "var(--c-blue)" }
                    components::StatRow { label: "In Flight", count: stats.in_progress, color: "var(--c-amber)" }
                    components::StatRow { label: "Resolved", count: stats.done, color: "var(--c-green)" }
                }
            }
            div { class: "sidebar-foot",
                button { class: "new-issue-btn", onclick: move |_| modal.set(true), "+ New Issue" }
                div { class: if dirty() { "sync dirty" } else { "sync" }, if dirty() { "⚠ Unsaved" } else { "✓ Synced" } }
                if dirty() {
                    button { class: "save-btn", onclick: move |_| { save_workspace(state, "Saved"); dirty.set(false); }, "Save All" }
                }
                button { class: "reinit-btn", onclick: move |_| reinit_modal.set(true), "↻ Reset" }
            }
        }
    }
}

fn render_topbar(mut search: Signal<String>, mut modal: Signal<bool>, stats: &Stats) -> Element {
    rsx! {
        div { class: "topbar",
            div { class: "search-box",
                input { class: "search", placeholder: "Search…", value: "{search}", oninput: move |e| search.set(e.value()) }
            }
            button { class: "topbar-new-btn", onclick: move |_| modal.set(true), "+ New" }
            span { class: "count-pill", "{stats.total} issues" }
        }
    }
}

// Replace render_content in app.rs with this.
// Removed: active_id signal, on_toggle, on_collapse_all — all moved into FeedView.

fn render_content(
    view: Signal<View>,
    filtered: Vec<Issue>,
    mut dirty: Signal<bool>,
    state: AppState,
) -> Element {
    let mut issues = state.issues;
    rsx! {
        div { class: "content",
            match view() {
                View::Feed => rsx! {
                    views::FeedView {
                        issues: filtered.clone(),
                        on_status: move |(id, s): (u32, String)| {
                            if let Some(i) = issues.write().iter_mut().find(|i| i.id == id) { i.status = Status::from_str(&s); }
                            dirty.set(true);
                        },
                        on_resolution: move |(id, t): (u32, String)| {
                            if let Some(i) = issues.write().iter_mut().find(|i| i.id == id) { i.resolution = t; }
                            dirty.set(true);
                        },
                        on_reorder: move |(drag, target, after): (u32, u32, bool)| {
                            let mut all = issues();
                            if let Some(idx) = all.iter().position(|i| i.id == drag) {
                                let mut iss = all.remove(idx);
                                if let Some(tidx) = all.iter().position(|i| i.id == target) {
                                    iss.section = all[tidx].section.clone();
                                    all.insert(if after { tidx + 1 } else { tidx }.min(all.len()), iss);
                                }
                            }
                            issues.set(all);
                            save_workspace(state, "Reordered");
                        },
                    }
                },
                View::Board => rsx! { views::BoardView { issues: filtered } },
                View::Heatmap => rsx! { views::HeatmapView { issues: issues() } },
                View::Graph => rsx! { views::GraphView { issues: issues() } },
                View::Timeline => rsx! { views::TimelineView { issues: issues() } },
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

fn filter_issues(issues: &[Issue], q: &str) -> Vec<Issue> {
    if q.is_empty() {
        return issues.to_vec();
    }
    let q = q.to_lowercase();
    issues
        .iter()
        .filter(|i| i.title.to_lowercase().contains(&q) || i.id.to_string().contains(&q))
        .cloned()
        .collect()
}
