use crate::model::{Issue, Status, Workspace};
use crate::ui::components::LabelList;
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

    let file_overlaps = shared_file_overlaps(&issues);


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

            if !file_overlaps.is_empty() {
                div { class: "graph-sec",
                    h3 { class: "graph-sec-title", "File Overlaps (Active)" }
                    for (file, ids) in &file_overlaps {
                        div { class: "g-edge overlap",
                            span { class: "g-files", "{file}" }
                            span { class: "g-link", ":" }
                            div { class: "hm-ids",
                                for id in ids {
                                    span { class: "g-node", "#{id}" }
                                }
                            }
                        }
                    }
                }
            }

            if dep_edges.is_empty() && file_overlaps.is_empty() {
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
                            if !issue.labels.is_empty() {
                                div { class: "labels-row", style: "display:flex;gap:4px;margin-top:6px;",
                                    LabelList { labels: issue.labels.clone() }
                                }
                            }
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

/// Build a file→issue-IDs map for active (non-done, non-descoped) issues,
/// returning only files touched by two or more issues (i.e. real overlaps).
fn shared_file_overlaps(issues: &[Issue]) -> Vec<(String, Vec<u32>)> {
    let mut map: HashMap<&str, Vec<u32>> = HashMap::new();
    issues
        .iter()
        .filter(|i| i.status != Status::Done && i.status != Status::Descoped)
        .flat_map(|i| i.files.iter().map(move |f| (f.as_str(), i.id)))
        .for_each(|(f, id)| map.entry(f).or_default().push(id));
    map.into_iter()
        .filter(|(_, ids)| ids.len() >= 2)
        .map(|(f, ids)| (f.to_string(), ids))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Issue, Status};

    fn make_issue(id: u32, status: Status, files: &[&str]) -> Issue {
        Issue {
            id,
            title: format!("Issue {id}"),
            status,
            files: files.iter().map(|s| s.to_string()).collect(),
            labels: vec![],
            description: String::new(),
            resolution: String::new(),
            section: "ACTIVE Issues".to_string(),
            depends_on: vec![],
        }
    }

    // Issue [48]: shared_file_overlaps must return a file when two or more active
    // issues reference it, and must include all relevant issue IDs.
    #[test]
    fn overlaps_found_when_two_active_issues_share_file() {
        let issues = vec![
            make_issue(1, Status::Open, &["src/main.rs", "src/lib.rs"]),
            make_issue(2, Status::InProgress, &["src/main.rs"]),
        ];
        let result = shared_file_overlaps(&issues);
        let main_entry = result.iter().find(|(f, _)| f == "src/main.rs");
        assert!(main_entry.is_some(), "shared file must appear in overlaps");
        let ids = &main_entry.unwrap().1;
        assert!(ids.contains(&1) && ids.contains(&2), "both issue IDs must be present");
        // src/lib.rs is only touched by issue 1 — must NOT appear.
        assert!(result.iter().all(|(f, _)| f != "src/lib.rs"), "single-issue file must be excluded");
    }

    // Negative case: a file shared between one active and one Done issue must NOT
    // appear — Done issues are excluded from the graph overlap section.
    // Pre-refactor the old compute_overlaps included Done issues in the active filter
    // only for the heatmap but had a separate active filter for overlaps; this
    // test pins that the filter is correct in the new implementation.
    #[test]
    fn done_issue_not_counted_toward_overlap() {
        let issues = vec![
            make_issue(1, Status::Open, &["src/model.rs"]),
            make_issue(2, Status::Done, &["src/model.rs"]),
        ];
        let result = shared_file_overlaps(&issues);
        assert!(result.is_empty(), "file shared with only one active issue must not appear");
    }

    // Edge case: descoped issues are also excluded.
    #[test]
    fn descoped_issue_not_counted_toward_overlap() {
        let issues = vec![
            make_issue(1, Status::Open, &["src/ui.rs"]),
            make_issue(2, Status::Descoped, &["src/ui.rs"]),
        ];
        let result = shared_file_overlaps(&issues);
        assert!(result.is_empty(), "file shared with a Descoped issue must not appear");
    }
}
