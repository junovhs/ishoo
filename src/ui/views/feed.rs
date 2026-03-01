use crate::model::{Issue, Status};
use dioxus::prelude::*;
use std::collections::HashMap;

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
    let mut section_order: Vec<String> = Vec::new();
    let mut section_map: HashMap<String, Vec<Issue>> = HashMap::new();

    for issue in &props.issues {
        if !section_map.contains_key(&issue.section) {
            section_order.push(issue.section.clone());
        }
        section_map
            .entry(issue.section.clone())
            .or_default()
            .push(issue.clone());
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
                                    on_toggle: props.on_toggle,
                                    on_status: props.on_status,
                                    on_resolution: props.on_resolution,
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
