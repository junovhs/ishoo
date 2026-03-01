use crate::model::{Issue, Status, Workspace};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

// ── Feed View ──────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Props)]
pub struct FeedViewProps {
    pub issues: Vec<Issue>,
    pub active_id: Option<u32>,
    pub on_toggle: EventHandler<u32>,
    pub on_status: EventHandler<(u32, String)>,
    pub on_resolution: EventHandler<(u32, String)>,
}

#[component]
pub fn FeedView(props: FeedViewProps) -> Element {
    // Group by section preserving insertion order via index map
    let mut section_order: Vec<String> = Vec::new();
    let mut section_map: HashMap<String, Vec<Issue>> = HashMap::new();

    for issue in &props.issues {
        if !section_map.contains_key(&issue.section) {
            section_order.push(issue.section.clone());
        }
        section_map.entry(issue.section.clone()).or_default().push(issue.clone());
    }

    rsx! {
        div { class: "feed",
            div { class: "feed-inner",
                for name in &section_order {
                    {
                        let items = &section_map[name];
                        rsx! {
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
pub fn BoardView(issues: Vec<Issue>) -> Element {
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
pub fn HeatmapView(issues: Vec<Issue>) -> Element {
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
pub fn GraphView(issues: Vec<Issue>) -> Element {
    let ws = Workspace { root: PathBuf::new(), issues: issues.clone() };
    let dep_edges = ws.dependency_edges();

    // File overlaps among active issues — use HashMap for O(n) lookups
    let active: Vec<&Issue> = issues.iter()
        .filter(|i| i.status != Status::Done && i.status != Status::Descoped)
        .collect();

    let mut file_to_issues: HashMap<&str, Vec<u32>> = HashMap::new();
    for i in &active {
        for f in &i.files {
            file_to_issues.entry(f.as_str()).or_default().push(i.id);
        }
    }

    // Deduplicate overlap pairs
    let mut overlap_set: HashMap<(u32, u32), Vec<String>> = HashMap::new();
    for (file, ids) in &file_to_issues {
        for (idx, &a) in ids.iter().enumerate() {
            for &b in &ids[idx + 1..] {
                let key = if a < b { (a, b) } else { (b, a) };
                overlap_set.entry(key).or_default().push(file.to_string());
            }
        }
    }
    let overlaps: Vec<(u32, u32, String)> = overlap_set
        .into_iter()
        .map(|((a, b), files)| (a, b, files.join(", ")))
        .collect();

    rsx! {
        div { class: "viz",
            div { class: "viz-hdr",
                h2 { class: "viz-title", "Issue Relationship Graph" }
                p { class: "viz-sub", "Dependencies and shared-file connections" }
            }

            if !dep_edges.is_empty() {
                div { class: "graph-sec",
                    h3 { class: "graph-sec-title", "Dependencies" }
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
                    h3 { class: "graph-sec-title", "File Overlaps (Active)" }
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
pub fn TimelineView(issues: Vec<Issue>) -> Element {
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
