use crate::model::{Issue, Status, Workspace};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[component]
pub fn HeatmapView(issues: Vec<Issue>) -> Element {
    let ws = Workspace {
        root: PathBuf::new(),
        issues: issues.clone(),
    };
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
                            div { class: "hm-bar", style: "width:{pct(ids.len(), max)}%" }
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

#[component]
pub fn GraphView(issues: Vec<Issue>) -> Element {
    let ws = Workspace {
        root: PathBuf::new(),
        issues: issues.clone(),
    };
    let dep_edges = ws.dependency_edges();
    let overlaps = compute_overlaps(&issues);

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

#[component]
pub fn TimelineView(issues: Vec<Issue>) -> Element {
    let total = issues.len().max(1);
    let done_n = issues.iter().filter(|i| i.status == Status::Done).count();
    let wip_n = issues
        .iter()
        .filter(|i| i.status == Status::InProgress)
        .count();
    let open_n = issues.iter().filter(|i| i.status == Status::Open).count();
    let done_pct = pct(done_n, total);
    let wip_pct = pct(wip_n, total);
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

fn pct(part: usize, total: usize) -> u32 {
    (part as f64 / total as f64 * 100.0) as u32
}

fn compute_overlaps(issues: &[Issue]) -> Vec<(u32, u32, String)> {
    let active: Vec<&Issue> = issues
        .iter()
        .filter(|i| i.status != Status::Done && i.status != Status::Descoped)
        .collect();

    let mut file_to_issues: HashMap<&str, Vec<u32>> = HashMap::new();
    for i in &active {
        for f in &i.files {
            file_to_issues.entry(f.as_str()).or_default().push(i.id);
        }
    }

    let mut overlap_set: HashMap<(u32, u32), Vec<String>> = HashMap::new();
    for (file, ids) in &file_to_issues {
        for (idx, &a) in ids.iter().enumerate() {
            for &b in &ids[idx + 1..] {
                let key = if a < b { (a, b) } else { (b, a) };
                overlap_set.entry(key).or_default().push((*file).to_owned());
            }
        }
    }

    overlap_set
        .into_iter()
        .map(|((a, b), files)| (a, b, files.join(", ")))
        .collect()
}
