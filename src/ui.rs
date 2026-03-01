use crate::model::{Issue, Status, Stats, Workspace};
use dioxus::prelude::*;
use std::path::PathBuf;

// ── Launch ─────────────────────────────────────────────────────────────

pub fn launch_dashboard(path: PathBuf) {
    let workspace = Workspace::load(&path).unwrap_or_else(|e| {
        eprintln!("Warning: {}", e);
        Workspace {
            root: path.clone(),
            issues: vec![],
        }
    });

    dioxus::launch(move || {
        let workspace_path = path.clone();
        let initial_issues = workspace.issues.clone();
        rsx! { App { initial_issues, workspace_path } }
    });
}

// ── Views ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum View {
    Feed,
    Board,
    Heatmap,
    Graph,
    Timeline,
}

// ── App Root ───────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Props)]
struct AppProps {
    initial_issues: Vec<Issue>,
    workspace_path: PathBuf,
}

#[component]
fn App(props: AppProps) -> Element {
    let mut issues = use_signal(|| props.initial_issues.clone());
    let mut search_query = use_signal(String::new);
    let mut active_issue_id = use_signal(|| None::<u32>);
    let mut active_view = use_signal(|| View::Feed);
    let mut dirty = use_signal(|| false);
    let workspace_path = use_signal(|| props.workspace_path.clone());

    // Periodic file reload
    let _poll = use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let wp = props.workspace_path.clone();
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
        let ws = Workspace {
            root: workspace_path().clone(),
            issues: issues().clone(),
        };
        if let Err(e) = ws.save() {
            eprintln!("Save error: {}", e);
        } else {
            dirty.set(false);
        }
    };

    // Stats
    let stats = {
        let all = issues();
        let mut s = Stats::default();
        for issue in &all {
            match issue.status {
                Status::Open => s.open += 1,
                Status::InProgress => s.in_progress += 1,
                Status::Done => s.done += 1,
                Status::Descoped => s.descoped += 1,
            }
        }
        s.total = all.len();
        s
    };

    // Filter
    let filtered: Vec<Issue> = {
        let q = search_query().to_lowercase();
        let all = issues();
        if q.is_empty() {
            all.clone()
        } else {
            all.iter()
                .filter(|i| {
                    i.title.to_lowercase().contains(&q)
                        || i.id.to_string().contains(&q)
                        || i.description.to_lowercase().contains(&q)
                        || i.files.iter().any(|f| f.to_lowercase().contains(&q))
                })
                .cloned()
                .collect()
        }
    };

    rsx! {
        style { {STYLES} }
        div { class: "shell",

            // ── Sidebar ──
            aside { class: "sidebar",
                div { class: "brand",
                    svg {
                        width: "24", height: "24", view_box: "0 0 24 24",
                        fill: "none", stroke: "currentColor", stroke_width: "2.5",
                        path { d: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" }
                    }
                    span { "Linearis" }
                }

                nav { class: "nav",
                    div { class: "nav-label", "Views" }
                    NavBtn { label: "Feed", active: active_view() == View::Feed, onclick: move |_| active_view.set(View::Feed) }
                    NavBtn { label: "Board", active: active_view() == View::Board, onclick: move |_| active_view.set(View::Board) }
                    NavBtn { label: "Heatmap", active: active_view() == View::Heatmap, onclick: move |_| active_view.set(View::Heatmap) }
                    NavBtn { label: "Graph", active: active_view() == View::Graph, onclick: move |_| active_view.set(View::Graph) }
                    NavBtn { label: "Timeline", active: active_view() == View::Timeline, onclick: move |_| active_view.set(View::Timeline) }
                }

                div { class: "stats-area",
                    div { class: "nav-label", "Metrics" }
                    div { class: "stat-list",
                        StatRow { label: "Backlog", count: stats.open, color: "var(--c-blue)" }
                        StatRow { label: "In Flight", count: stats.in_progress, color: "var(--c-amber)" }
                        StatRow { label: "Resolved", count: stats.done, color: "var(--c-green)" }
                        if stats.descoped > 0 {
                            StatRow { label: "Descoped", count: stats.descoped, color: "var(--c-muted)" }
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

            // ── Main ──
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
                            FeedView {
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
                        View::Board => rsx! { BoardView { issues: filtered.clone() } },
                        View::Heatmap => rsx! { HeatmapView { issues: issues() } },
                        View::Graph => rsx! { GraphView { issues: issues() } },
                        View::Timeline => rsx! { TimelineView { issues: issues() } },
                    }
                }
            }
        }
    }
}

// ── Small Components ───────────────────────────────────────────────────

#[component]
fn NavBtn(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: if active { "nav-btn on" } else { "nav-btn" },
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

#[component]
fn StatRow(label: String, count: usize, color: String) -> Element {
    rsx! {
        div { class: "stat",
            span { class: "stat-lbl", "{label}" }
            span { class: "stat-val", style: "color:{color}", "{count}" }
        }
    }
}

// ── Feed View ──────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Props)]
struct FeedViewProps {
    issues: Vec<Issue>,
    active_id: Option<u32>,
    on_toggle: EventHandler<u32>,
    on_status: EventHandler<(u32, String)>,
    on_resolution: EventHandler<(u32, String)>,
}

#[component]
fn FeedView(props: FeedViewProps) -> Element {
    // Group by section, preserving order
    let mut sections: Vec<(String, Vec<Issue>)> = Vec::new();
    for issue in &props.issues {
        if let Some(sec) = sections.iter_mut().find(|(s, _)| *s == issue.section) {
            sec.1.push(issue.clone());
        } else {
            sections.push((issue.section.clone(), vec![issue.clone()]));
        }
    }

    rsx! {
        div { class: "feed",
            div { class: "feed-inner",
                for (name, items) in sections {
                    div { class: "sec-hdr",
                        span { "{name}" }
                        div { class: "sec-line" }
                        span { class: "sec-ct", "{items.len()}" }
                    }
                    for issue in items {
                        IssueCard {
                            key: "{issue.id}",
                            issue: issue.clone(),
                            expanded: props.active_id == Some(issue.id),
                            on_toggle: props.on_toggle.clone(),
                            on_status: props.on_status.clone(),
                            on_resolution: props.on_resolution.clone(),
                        }
                    }
                }
                div { style: "height:200px;" }
            }
        }
    }
}

// ── Issue Card ─────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Props)]
struct IssueCardProps {
    issue: Issue,
    expanded: bool,
    on_toggle: EventHandler<u32>,
    on_status: EventHandler<(u32, String)>,
    on_resolution: EventHandler<(u32, String)>,
}

#[component]
fn IssueCard(props: IssueCardProps) -> Element {
    let i = &props.issue;
    let id = i.id;

    rsx! {
        div { class: if props.expanded { "card active" } else { "card" },
            div {
                class: "card-hdr",
                onclick: move |_| props.on_toggle.call(id),
                span { class: "cid", "#{id}" }
                span { class: "ctitle", "{i.title}" }
                span { class: "badge b-{i.status.css_class()}", "{i.status.label()}" }
            }
            if props.expanded {
                div { class: "card-body",
                    div { class: "detail-grid",
                        div { class: "detail-l",
                            if !i.description.is_empty() {
                                div { class: "fgroup",
                                    label { class: "flbl", "Description" }
                                    div { class: "desc-block", "{i.description}" }
                                }
                            }
                            div { class: "fgroup",
                                label { class: "flbl", "Resolution Notes" }
                                textarea {
                                    class: "res-input",
                                    rows: "5",
                                    placeholder: "Log your solution…",
                                    value: "{i.resolution}",
                                    oninput: move |e| props.on_resolution.call((id, e.value())),
                                }
                            }
                        }
                        div { class: "detail-r",
                            div { class: "fgroup",
                                label { class: "flbl", "Status" }
                                select {
                                    class: "sel",
                                    value: "{i.status.label()}",
                                    onchange: move |e| props.on_status.call((id, e.value())),
                                    option { value: "OPEN", selected: i.status == Status::Open, "Open" }
                                    option { value: "IN PROGRESS", selected: i.status == Status::InProgress, "In Progress" }
                                    option { value: "DONE", selected: i.status == Status::Done, "Done" }
                                    option { value: "DESCOPED", selected: i.status == Status::Descoped, "Descoped" }
                                }
                            }
                            if !i.files.is_empty() {
                                div { class: "fgroup",
                                    label { class: "flbl", "Files" }
                                    div { class: "chips",
                                        for f in &i.files {
                                            span { class: "chip-file", "{f}" }
                                        }
                                    }
                                }
                            }
                            if !i.depends_on.is_empty() {
                                div { class: "fgroup",
                                    label { class: "flbl", "Depends On" }
                                    div { class: "chips",
                                        for d in &i.depends_on {
                                            span { class: "chip-dep", "#{d}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Board View ─────────────────────────────────────────────────────────

#[component]
fn BoardView(issues: Vec<Issue>) -> Element {
    let open: Vec<Issue> = issues.iter().filter(|i| i.status == Status::Open).cloned().collect();
    let wip: Vec<Issue> = issues.iter().filter(|i| i.status == Status::InProgress).cloned().collect();
    let done: Vec<Issue> = issues.iter().filter(|i| i.status == Status::Done).cloned().collect();

    rsx! {
        div { class: "board",
            BoardCol { title: "Open", color: "var(--c-blue)", items: open }
            BoardCol { title: "In Progress", color: "var(--c-amber)", items: wip }
            BoardCol { title: "Done", color: "var(--c-green)", items: done }
        }
    }
}

#[component]
fn BoardCol(title: String, color: String, items: Vec<Issue>) -> Element {
    rsx! {
        div { class: "bcol",
            div { class: "bcol-hdr",
                div { class: "bcol-dot", style: "background:{color}" }
                span { "{title}" }
                span { class: "bcol-ct", "{items.len()}" }
            }
            div { class: "bcol-cards",
                for i in &items {
                    div { class: "bcard",
                        div { class: "bcard-id", "#{i.id}" }
                        div { class: "bcard-title", "{i.title}" }
                        if !i.files.is_empty() {
                            div { class: "bcard-meta", "{i.files.len()} files" }
                        }
                    }
                }
            }
        }
    }
}

// ── Heatmap View ───────────────────────────────────────────────────────

#[component]
fn HeatmapView(issues: Vec<Issue>) -> Element {
    let ws = Workspace { root: PathBuf::new(), issues: issues.clone() };
    let heatmap = ws.file_heatmap();
    let mut entries: Vec<_> = heatmap.into_iter().collect();
    entries.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    let max = entries.iter().map(|(_, v)| v.len()).max().unwrap_or(1);

    rsx! {
        div { class: "viz",
            div { class: "viz-hdr",
                h2 { class: "viz-title", "File Heatmap" }
                p { class: "viz-sub", "Files touched by the most issues — hotspots in your codebase" }
            }
            div { class: "hm-grid",
                for (file, ids) in &entries {
                    div { class: "hm-row",
                        span { class: "hm-file", "{file}" }
                        div { class: "hm-track",
                            div { class: "hm-bar", style: "width:{(ids.len() as f64 / max as f64 * 100.0) as u32}%" }
                        }
                        span { class: "hm-ct", "{ids.len()}" }
                        div { class: "hm-ids",
                            for id in ids {
                                span { class: "hm-chip", "#{id}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Graph View ─────────────────────────────────────────────────────────

#[component]
fn GraphView(issues: Vec<Issue>) -> Element {
    let ws = Workspace { root: PathBuf::new(), issues: issues.clone() };
    let dep_edges = ws.dependency_edges();

    // File overlap edges (only among active issues)
    let active: Vec<&Issue> = issues.iter().filter(|i| i.status != Status::Done && i.status != Status::Descoped).collect();
    let mut overlaps: Vec<(u32, u32, String)> = Vec::new();
    for a in 0..active.len() {
        for b in (a + 1)..active.len() {
            let shared: Vec<&String> = active[a].files.iter().filter(|f| active[b].files.contains(f)).collect();
            if !shared.is_empty() {
                let names: Vec<&str> = shared.iter().map(|s| s.as_str()).collect();
                overlaps.push((active[a].id, active[b].id, names.join(", ")));
            }
        }
    }

    rsx! {
        div { class: "viz",
            div { class: "viz-hdr",
                h2 { class: "viz-title", "Issue Relationship Graph" }
                p { class: "viz-sub", "Dependencies and shared-file connections between issues" }
            }

            if !dep_edges.is_empty() {
                div { class: "graph-sec",
                    h3 { class: "graph-sec-title", "Explicit Dependencies" }
                    for (from, to) in &dep_edges {
                        div { class: "g-edge",
                            span { class: "g-node g-from", "#{from}" }
                            span { class: "g-arrow", "→" }
                            span { class: "g-node g-to", "#{to}" }
                            {
                                let ft = issues.iter().find(|i| i.id == *from).map(|i| i.title.as_str()).unwrap_or("?");
                                let tt = issues.iter().find(|i| i.id == *to).map(|i| i.title.as_str()).unwrap_or("?");
                                rsx! { span { class: "g-lbl", "{ft} → {tt}" } }
                            }
                        }
                    }
                }
            }

            if !overlaps.is_empty() {
                div { class: "graph-sec",
                    h3 { class: "graph-sec-title", "File Overlaps (Active Issues)" }
                    for (a, b, files) in &overlaps {
                        div { class: "g-edge overlap",
                            span { class: "g-node", "#{a}" }
                            span { class: "g-link", "⟷" }
                            span { class: "g-node", "#{b}" }
                            span { class: "g-files", "{files}" }
                        }
                    }
                }
            }

            if dep_edges.is_empty() && overlaps.is_empty() {
                div { class: "empty", "No dependencies or file overlaps found." }
            }
        }
    }
}

// ── Timeline View ──────────────────────────────────────────────────────

#[component]
fn TimelineView(issues: Vec<Issue>) -> Element {
    let total = issues.len().max(1);
    let done_n = issues.iter().filter(|i| i.status == Status::Done).count();
    let wip_n = issues.iter().filter(|i| i.status == Status::InProgress).count();
    let open_n = issues.iter().filter(|i| i.status == Status::Open).count();
    let done_pct = (done_n as f64 / total as f64 * 100.0) as u32;
    let wip_pct = (wip_n as f64 / total as f64 * 100.0) as u32;
    let open_pct = 100u32.saturating_sub(done_pct).saturating_sub(wip_pct);

    let mut sorted = issues.clone();
    sorted.sort_by_key(|i| (i.status_ord(), i.id));

    rsx! {
        div { class: "viz",
            div { class: "viz-hdr",
                h2 { class: "viz-title", "Progress Overview" }
                p { class: "viz-sub", "Overall project completion at a glance" }
            }
            div { class: "pbar-wrap",
                div { class: "pbar",
                    if done_pct > 0 {
                        div { class: "pseg done", style: "width:{done_pct}%",
                            if done_pct > 8 { "{done_n} done" }
                        }
                    }
                    if wip_pct > 0 {
                        div { class: "pseg wip", style: "width:{wip_pct}%",
                            if wip_pct > 8 { "{wip_n} active" }
                        }
                    }
                    if open_pct > 0 {
                        div { class: "pseg open", style: "width:{open_pct}%",
                            if open_pct > 8 { "{open_n} open" }
                        }
                    }
                }
                div { class: "pbar-lbl", "{done_pct}% complete" }
            }
            div { class: "tl-list",
                for issue in &sorted {
                    div { class: "tl-item st-{issue.status.css_class()}",
                        div { class: "tl-dot" }
                        div { class: "tl-body",
                            span { class: "tl-id", "#{issue.id}" }
                            span { class: "tl-title", "{issue.title}" }
                            span { class: "badge b-{issue.status.css_class()}", "{issue.status.label()}" }
                        }
                    }
                }
            }
        }
    }
}

// ── Stylesheet ─────────────────────────────────────────────────────────

const STYLES: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=DM+Sans:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap');
:root {
    --bg0:#07080a; --bg1:#0f1114; --bg2:#181a1e; --bg3:#1e2024;
    --bd:#25272b; --bd2:#3a3d42;
    --t1:#ecedef; --t2:#95989e; --t3:#5c6066;
    --c-blue:#58a6ff; --c-green:#3fb950; --c-amber:#d29922;
    --c-purple:#a371f7; --c-red:#f85149; --c-muted:#636970;
    --r-lg:12px; --r-md:8px; --r-sm:6px;
    --ff:'DM Sans',-apple-system,sans-serif;
    --fm:'JetBrains Mono','SF Mono',monospace;
    --ease:cubic-bezier(.4,0,.2,1);
}
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:var(--ff);background:var(--bg0);color:var(--t1);overflow:hidden;height:100vh}

/* Shell */
.shell{display:flex;height:100vh;overflow:hidden}

/* Sidebar */
.sidebar{width:272px;min-width:272px;background:var(--bg1);border-right:1px solid var(--bd);display:flex;flex-direction:column;padding:28px 18px;gap:28px}
.brand{display:flex;align-items:center;gap:10px;font-weight:800;font-size:19px;letter-spacing:-.5px;color:var(--t1)}
.brand svg{color:var(--c-purple)}
.nav{display:flex;flex-direction:column;gap:3px}
.nav-label{font-size:10px;font-weight:800;color:var(--t3);text-transform:uppercase;letter-spacing:1.5px;margin-bottom:10px}
.nav-btn{display:block;width:100%;text-align:left;padding:9px 13px;border-radius:var(--r-md);background:0 0;border:none;color:var(--t2);font-size:13.5px;font-weight:500;font-family:var(--ff);cursor:pointer;transition:all .15s var(--ease)}
.nav-btn:hover{background:var(--bg2);color:var(--t1)}
.nav-btn.on{background:var(--bg2);color:var(--t1);font-weight:600;border:1px solid var(--bd)}
.stats-area{margin-top:auto}
.stat-list{display:flex;flex-direction:column;gap:7px}
.stat{display:flex;justify-content:space-between;align-items:center;padding:9px 13px;background:var(--bg2);border-radius:var(--r-md);border:1px solid var(--bd);font-size:13px}
.stat-lbl{color:var(--t2)}
.stat-val{font-weight:700;font-family:var(--fm);font-size:14px}
.sidebar-foot{margin-top:14px}
.sync{text-align:center;padding:11px;border-radius:var(--r-md);background:rgba(255,255,255,.02);border:1px solid var(--bd);font-size:12px;color:var(--t3)}
.sync.dirty{border-color:var(--c-amber);color:var(--c-amber)}
.save-btn{display:block;width:100%;margin-top:7px;padding:9px;border-radius:var(--r-md);border:none;background:var(--c-green);color:#000;font-weight:700;font-size:13px;font-family:var(--ff);cursor:pointer}
.save-btn:hover{opacity:.85}

/* Main */
.main{flex:1;display:flex;flex-direction:column;overflow:hidden}
.topbar{height:66px;padding:0 36px;display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid var(--bd);background:rgba(7,8,10,.85);backdrop-filter:blur(20px);flex-shrink:0}
.search-box{width:440px}
.search{width:100%;background:var(--bg2);border:1px solid var(--bd);padding:10px 18px;border-radius:36px;color:var(--t1);font-size:14px;font-family:var(--ff);outline:none;transition:border-color .2s var(--ease)}
.search:focus{border-color:var(--c-blue);background:var(--bg1)}
.search::placeholder{color:var(--t3)}
.count-pill{font-size:12px;font-weight:600;color:var(--t3);padding:5px 13px;background:var(--bg2);border-radius:18px;border:1px solid var(--bd)}
.content{flex:1;overflow-y:auto;scroll-behavior:smooth}

/* Feed */
.feed{padding:36px}
.feed-inner{max-width:840px;margin:0 auto}
.sec-hdr{margin:44px 0 14px;display:flex;align-items:center;gap:14px;font-size:11px;font-weight:800;color:var(--c-purple);text-transform:uppercase;letter-spacing:2px}
.sec-line{flex:1;height:1px;background:var(--bd)}
.sec-ct{font-family:var(--fm);font-size:11px;color:var(--t3)}

/* Card */
.card{margin-bottom:9px;border-radius:var(--r-lg);background:var(--bg1);border:1px solid var(--bd);overflow:hidden;transition:border-color .2s var(--ease)}
.card:hover{border-color:var(--bd2)}
.card.active{border-color:var(--c-blue)}
.card-hdr{padding:16px 22px;display:flex;align-items:center;gap:18px;cursor:pointer;user-select:none}
.card-hdr:hover{background:var(--bg3)}
.cid{font-family:var(--fm);font-size:12px;color:var(--t3);min-width:40px}
.ctitle{flex:1;font-weight:600;font-size:14.5px;color:var(--t1)}

/* Badges */
.badge{font-size:9px;font-weight:800;padding:3px 9px;border-radius:18px;text-transform:uppercase;letter-spacing:.7px;white-space:nowrap}
.b-open{background:rgba(88,166,255,.1);color:var(--c-blue);border:1px solid rgba(88,166,255,.2)}
.b-in-progress{background:rgba(210,153,34,.1);color:var(--c-amber);border:1px solid rgba(210,153,34,.2)}
.b-done{background:rgba(63,185,80,.1);color:var(--c-green);border:1px solid rgba(63,185,80,.2)}
.b-descoped{background:rgba(99,105,112,.1);color:var(--c-muted);border:1px solid rgba(99,105,112,.2)}

/* Detail */
.card-body{padding:0 22px 22px;border-top:1px solid var(--bd)}
.detail-grid{display:grid;grid-template-columns:1fr 240px;gap:28px;margin-top:18px}
.fgroup{margin-bottom:16px}
.flbl{font-size:9px;font-weight:800;color:var(--t3);text-transform:uppercase;letter-spacing:1.5px;margin-bottom:7px;display:block}
.desc-block{color:var(--t2);font-size:13px;line-height:1.7;white-space:pre-wrap;background:var(--bg0);padding:14px;border-radius:var(--r-md);border:1px solid var(--bd);max-height:280px;overflow-y:auto}
.res-input{width:100%;background:var(--bg2);border:1px solid var(--bd);color:var(--t1);padding:11px;border-radius:var(--r-md);font-family:var(--ff);font-size:13px;line-height:1.6;resize:vertical;outline:none}
.res-input:focus{border-color:var(--c-blue)}
.sel{width:100%;background:var(--bg2);border:1px solid var(--bd);color:var(--t1);padding:9px 11px;border-radius:var(--r-md);font-family:var(--ff);font-size:13px;cursor:pointer;outline:none}
.sel:focus{border-color:var(--c-blue)}
.chips{display:flex;flex-wrap:wrap;gap:5px}
.chip-file{display:inline-block;padding:2px 7px;background:#1a1f27;border-radius:var(--r-sm);font-family:var(--fm);font-size:11px;color:var(--c-blue);border:1px solid var(--bd)}
.chip-dep{display:inline-block;padding:2px 7px;background:rgba(163,113,247,.08);border-radius:var(--r-sm);font-family:var(--fm);font-size:11px;color:var(--c-purple);border:1px solid rgba(163,113,247,.2)}

/* Board */
.board{display:flex;gap:14px;padding:28px;height:100%;overflow-x:auto}
.bcol{flex:1;min-width:240px;max-width:340px;display:flex;flex-direction:column}
.bcol-hdr{display:flex;align-items:center;gap:9px;padding:10px 14px;font-size:13px;font-weight:700;margin-bottom:10px}
.bcol-dot{width:8px;height:8px;border-radius:50%}
.bcol-ct{font-family:var(--fm);font-size:11px;color:var(--t3);margin-left:auto}
.bcol-cards{flex:1;overflow-y:auto;display:flex;flex-direction:column;gap:7px}
.bcard{padding:12px 14px;background:var(--bg1);border:1px solid var(--bd);border-radius:var(--r-md);transition:border-color .15s}
.bcard:hover{border-color:var(--bd2)}
.bcard-id{font-family:var(--fm);font-size:11px;color:var(--t3);margin-bottom:5px}
.bcard-title{font-size:13.5px;font-weight:600;line-height:1.4}
.bcard-meta{margin-top:7px;font-size:11px;color:var(--t3)}

/* Viz common */
.viz{padding:36px;max-width:920px;margin:0 auto}
.viz-hdr{margin-bottom:28px}
.viz-title{font-size:20px;font-weight:800;letter-spacing:-.3px;margin-bottom:5px}
.viz-sub{font-size:13px;color:var(--t2)}

/* Heatmap */
.hm-grid{display:flex;flex-direction:column;gap:5px}
.hm-row{display:grid;grid-template-columns:220px 1fr 36px auto;gap:10px;align-items:center;padding:7px 11px;border-radius:var(--r-md);transition:background .15s}
.hm-row:hover{background:var(--bg2)}
.hm-file{font-family:var(--fm);font-size:12px;color:var(--c-blue);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.hm-track{height:5px;background:var(--bg2);border-radius:3px;overflow:hidden}
.hm-bar{height:100%;background:linear-gradient(90deg,var(--c-amber),var(--c-red));border-radius:3px;transition:width .3s var(--ease)}
.hm-ct{font-family:var(--fm);font-size:12px;font-weight:600;color:var(--t2);text-align:right}
.hm-ids{display:flex;gap:3px;flex-wrap:wrap}
.hm-chip{font-family:var(--fm);font-size:10px;padding:1px 5px;background:var(--bg2);border-radius:4px;color:var(--t3);border:1px solid var(--bd)}

/* Graph */
.graph-sec{margin-bottom:28px}
.graph-sec-title{font-size:11px;font-weight:700;color:var(--t2);text-transform:uppercase;letter-spacing:1px;margin-bottom:10px}
.g-edge{display:flex;align-items:center;gap:9px;padding:9px 12px;border-radius:var(--r-md);margin-bottom:3px;transition:background .15s}
.g-edge:hover{background:var(--bg2)}
.g-node{font-family:var(--fm);font-size:12px;font-weight:600;padding:2px 7px;border-radius:var(--r-sm);background:var(--bg2);border:1px solid var(--bd);color:var(--c-blue)}
.g-from{color:var(--c-green)}
.g-to{color:var(--c-amber)}
.g-arrow{color:var(--t3);font-size:15px}
.g-link{color:var(--c-purple);font-size:15px}
.g-lbl{font-size:12px;color:var(--t3);margin-left:7px}
.g-files{font-family:var(--fm);font-size:11px;color:var(--t3);margin-left:auto}
.empty{text-align:center;padding:50px 18px;color:var(--t3);font-size:14px}

/* Progress / Timeline */
.pbar-wrap{margin-bottom:36px}
.pbar{display:flex;height:26px;border-radius:13px;overflow:hidden;background:var(--bg2);border:1px solid var(--bd)}
.pseg{display:flex;align-items:center;justify-content:center;font-size:10px;font-weight:700;color:rgba(0,0,0,.7);transition:width .4s var(--ease)}
.pseg.done{background:var(--c-green)}
.pseg.wip{background:var(--c-amber)}
.pseg.open{background:rgba(88,166,255,.2);color:var(--c-blue)}
.pbar-lbl{margin-top:9px;font-size:14px;font-weight:700;color:var(--t2)}
.tl-list{display:flex;flex-direction:column;gap:2px}
.tl-item{display:flex;align-items:center;gap:14px;padding:10px 14px;border-radius:var(--r-md);transition:background .15s}
.tl-item:hover{background:var(--bg2)}
.tl-dot{width:7px;height:7px;border-radius:50%;flex-shrink:0}
.st-open .tl-dot{background:var(--c-blue)}
.st-in-progress .tl-dot{background:var(--c-amber)}
.st-done .tl-dot{background:var(--c-green)}
.st-descoped .tl-dot{background:var(--c-muted)}
.tl-body{display:flex;align-items:center;gap:10px;flex:1}
.tl-id{font-family:var(--fm);font-size:11px;color:var(--t3);min-width:32px}
.tl-title{font-size:13.5px;font-weight:500;flex:1}

/* Scrollbar */
::-webkit-scrollbar{width:5px}
::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-thumb{background:var(--bd);border-radius:3px}
::-webkit-scrollbar-thumb:hover{background:var(--bd2)}
"#;
