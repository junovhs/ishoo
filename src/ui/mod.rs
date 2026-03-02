// src/ui/mod.rs
mod components;
mod styles;
mod styles_viz;
mod views;

use crate::model::{init_workspace, workspace_exists, Issue, Stats, Status, Workspace};
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

// ── Toast System ───────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

impl ToastKind {
    fn class(&self) -> &'static str {
        match self {
            Self::Success => "toast-success",
            Self::Error => "toast-error",
            Self::Info => "toast-info",
        }
    }
}

#[component]
fn ToastContainer(toasts: Vec<Toast>, on_dismiss: EventHandler<u64>) -> Element {
    rsx! {
        div { class: "toast-container",
            for toast in toasts {
                div {
                    key: "{toast.id}",
                    class: "toast {toast.kind.class()}",
                    onclick: move |_| on_dismiss.call(toast.id),
                    "{toast.message}"
                }
            }
        }
    }
}

// ── New Issue Modal ────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Props)]
struct NewIssueModalProps {
    on_create: EventHandler<(String, String)>,
    on_cancel: EventHandler<()>,
}

#[component]
fn NewIssueModal(props: NewIssueModalProps) -> Element {
    let mut title = use_signal(String::new);
    let mut status = use_signal(|| "OPEN".to_string());

    let can_create = !title().trim().is_empty();

    rsx! {
        div { class: "modal-overlay", onclick: move |_| props.on_cancel.call(()),
            div {
                class: "modal",
                onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h2 { "New Issue" }
                    button {
                        class: "modal-close",
                        onclick: move |_| props.on_cancel.call(()),
                        "×"
                    }
                }
                div { class: "modal-body",
                    div { class: "fgroup",
                        label { class: "flbl", "Title" }
                        input {
                            class: "modal-input",
                            r#type: "text",
                            placeholder: "What needs to be done?",
                            value: "{title}",
                            autofocus: true,
                            oninput: move |e| title.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter && can_create {
                                    props.on_create.call((title(), status()));
                                }
                            },
                        }
                    }
                    div { class: "fgroup",
                        label { class: "flbl", "Initial Status" }
                        select {
                            class: "sel",
                            value: "{status}",
                            onchange: move |e| status.set(e.value()),
                            option { value: "OPEN", "Open" }
                            option { value: "IN PROGRESS", "In Progress" }
                        }
                    }
                }
                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| props.on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "btn-primary",
                        disabled: !can_create,
                        onclick: move |_| {
                            if can_create {
                                props.on_create.call((title(), status()));
                            }
                        },
                        "Create Issue"
                    }
                }
            }
        }
    }
}

// ── Welcome / Init Screen ──────────────────────────────────────────────

#[derive(Clone, PartialEq, Props)]
struct WelcomeScreenProps {
    path: PathBuf,
    on_init: EventHandler<()>,
}

#[component]
fn WelcomeScreen(props: WelcomeScreenProps) -> Element {
    let mut error = use_signal(|| None::<String>);

    rsx! {
        div { class: "welcome-screen",
            div { class: "welcome-card",
                div { class: "welcome-icon",
                    svg {
                        width: "64", height: "64", view_box: "0 0 24 24",
                        fill: "none", stroke: "currentColor", stroke_width: "1.5",
                        path { d: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" }
                    }
                }
                h1 { "Welcome to Ishoo" }
                p { class: "welcome-desc",
                    "No issue tracker found in this directory. Initialize one to get started."
                }
                p { class: "welcome-path", "{props.path.display()}" }

                if let Some(err) = error() {
                    div { class: "welcome-error", "{err}" }
                }

                button {
                    class: "btn-primary btn-lg",
                    onclick: move |_| {
                        match init_workspace(&props.path) {
                            Ok(()) => props.on_init.call(()),
                            Err(e) => error.set(Some(e)),
                        }
                    },
                    "Initialize Issue Tracker"
                }

                p { class: "welcome-hint",
                    "This will create: issues-active.md, issues-backlog.md, issues-done.md"
                }
            }
        }
    }
}

// ── Main App ───────────────────────────────────────────────────────────

#[component]
fn App() -> Element {
    let ws_path = WORKSPACE_PATH
        .get()
        .expect("workspace path not set")
        .clone();

    let mut initialized = use_signal(|| workspace_exists(&ws_path));

    // If not initialized, show welcome screen
    if !initialized() {
        return rsx! {
            style { {styles::STYLES} }
            style { {styles::STYLES_MODAL} }
            style { {styles::STYLES_WELCOME} }
            WelcomeScreen {
                path: ws_path.clone(),
                on_init: move |_| initialized.set(true),
            }
        };
    }

    let initial = Workspace::load(&ws_path).unwrap_or_else(|_| Workspace {
        root: ws_path.clone(),
        issues: vec![],
    });

    let mut issues = use_signal(|| initial.issues);
    let mut search_query = use_signal(String::new);
    let mut active_issue_id = use_signal(|| None::<u32>);
    let mut active_view = use_signal(|| View::Feed);
    let mut dirty = use_signal(|| false);
    let mut show_new_modal = use_signal(|| false);
    let mut toasts = use_signal(Vec::<Toast>::new);
    let mut toast_counter = use_signal(|| 0u64);

    let add_toast = move |message: String, kind: ToastKind| {
        let id = toast_counter();
        toast_counter.set(id + 1);
        let mut t = toasts();
        t.push(Toast { id, message, kind });
        toasts.set(t);

        // Auto-dismiss after 3 seconds
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            let mut t = toasts();
            t.retain(|toast| toast.id != id);
            toasts.set(t);
        });
    };

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

    let save_and_notify = move |msg: &str| {
        let wp = WORKSPACE_PATH.get().expect("path not set").clone();
        let ws = Workspace {
            root: wp,
            issues: issues(),
        };
        if let Err(e) = ws.save() {
            add_toast(format!("Save failed: {e}"), ToastKind::Error);
        } else {
            dirty.set(false);
            add_toast(msg.to_string(), ToastKind::Success);
        }
    };

    let save = move |_| {
        save_and_notify("Changes saved");
    };

    let stats = compute_stats(&issues());
    let filtered: Vec<Issue> = filter_issues(&issues(), &search_query());
    let all_styles = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        styles::STYLES,
        styles::STYLES_CARD,
        styles::STYLES_DRAG,
        styles::STYLES_MODAL,
        styles::STYLES_TOAST,
        styles_viz::STYLES_VIZ
    );

    rsx! {
        style { {all_styles} }

        ToastContainer {
            toasts: toasts(),
            on_dismiss: move |id| {
                let mut t = toasts();
                t.retain(|toast| toast.id != id);
                toasts.set(t);
            }
        }

        if show_new_modal() {
            NewIssueModal {
                on_create: move |(title, status): (String, String)| {
                    let max_id = issues().iter().map(|i| i.id).max().unwrap_or(0);
                    let issue = Issue {
                        id: max_id + 1,
                        title: title.clone(),
                        status: Status::from_str(&status),
                        files: vec![],
                        description: String::new(),
                        resolution: String::new(),
                        section: "ACTIVE Issues".to_string(),
                        depends_on: vec![],
                    };
                    let mut all = issues();
                    all.insert(0, issue.clone());
                    issues.set(all);
                    show_new_modal.set(false);

                    // Auto-save new issue
                    let wp = WORKSPACE_PATH.get().expect("path not set").clone();
                    let ws = Workspace { root: wp, issues: issues() };
                    if let Err(e) = ws.save() {
                        add_toast(format!("Save failed: {e}"), ToastKind::Error);
                    } else {
                        add_toast(format!("Created #{} {}", issue.id, title), ToastKind::Success);
                    }
                },
                on_cancel: move |_| show_new_modal.set(false),
            }
        }

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
                    button {
                        class: "new-issue-btn",
                        onclick: move |_| show_new_modal.set(true),
                        "+ New Issue"
                    }
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
                    button {
                        class: "topbar-new-btn",
                        onclick: move |_| show_new_modal.set(true),
                        "+ New"
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
                                    if active_issue_id() == Some(id) {
                                        active_issue_id.set(None);
                                    } else {
                                        active_issue_id.set(Some(id));
                                    }
                                },
                                on_status: move |(id, s): (u32, String)| {
                                    let mut all = issues();
                                    if let Some(i) = all.iter_mut().find(|i| i.id == id) {
                                        i.status = Status::from_str(&s);
                                    }
                                    issues.set(all);
                                    dirty.set(true);
                                },
                                on_resolution: move |(id, t): (u32, String)| {
                                    let mut all = issues();
                                    if let Some(i) = all.iter_mut().find(|i| i.id == id) {
                                        i.resolution = t;
                                    }
                                    issues.set(all);
                                    dirty.set(true);
                                },
                                on_collapse_all: move |_| {
                                    active_issue_id.set(None);
                                },
                                on_reorder: move |(dragged_id, target_id, insert_after): (u32, u32, bool)| {
                                    let mut all = issues();
                                    if let Some(dragged_idx) = all.iter().position(|i| i.id == dragged_id) {
                                        let mut dragged_issue = all.remove(dragged_idx);
                                        if let Some(target_idx) = all.iter().position(|i| i.id == target_id) {
                                            dragged_issue.section = all[target_idx].section.clone();
                                            let insert_idx = if insert_after { target_idx + 1 } else { target_idx };
                                            all.insert(insert_idx.min(all.len()), dragged_issue);
                                        } else {
                                            all.insert(dragged_idx, dragged_issue);
                                        }
                                    }
                                    issues.set(all);

                                    // Auto-save on reorder
                                    let wp = WORKSPACE_PATH.get().expect("path not set").clone();
                                    let ws = Workspace { root: wp, issues: issues() };
                                    if let Err(e) = ws.save() {
                                        add_toast(format!("Save failed: {e}"), ToastKind::Error);
                                    } else {
                                        add_toast("Reordered".to_string(), ToastKind::Success);
                                    }
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
