// neti:allow(LAW OF ATOMICITY)
use crate::model::{Issue, Status};
use crate::ui::components::LabelList;
use dioxus::prelude::*;
use std::collections::BTreeMap;

const BOARD_CARD_H: f32 = 93.0;
const BOARD_DROP_MS: u64 = 400;

#[derive(Clone, PartialEq, Props)]
pub struct BoardViewProps {
    pub issues: Vec<Issue>,
    pub on_status: EventHandler<(u32, String)>,
    pub on_resolution: EventHandler<(u32, String)>,
    pub on_labels: EventHandler<(u32, String)>,
    pub on_reorder: EventHandler<(u32, Option<u32>, bool, Option<String>)>,
}

#[derive(Clone, Default, PartialEq)]
struct BoardDragState {
    dragging_id: Option<u32>,
    start_section: Option<String>,
    hover_section: Option<String>,
    start_idx: usize,
    hover_idx: usize,
    hover_after: bool,
    start_x: f32,
    start_y: f32,
    pointer_x: f32,
    pointer_y: f32,
    offset_x: f32,
    offset_y: f32,
    releasing: bool,
}

#[component]
pub fn BoardView(props: BoardViewProps) -> Element {
    let sections = grouped_sections(&props.issues);
    let mut drag = use_signal(BoardDragState::default);
    let mut modal_id = use_signal(|| None::<u32>);
    let dragged_issue = drag()
        .dragging_id
        .and_then(|id| props.issues.iter().find(|issue| issue.id == id).cloned());
    let active_modal = modal_id()
        .and_then(|id| props.issues.iter().find(|issue| issue.id == id).cloned());

    rsx! {
        div {
            class: "board-view",
            onpointermove: move |e| {
                if drag.read().dragging_id.is_some() {
                    let mut state = drag.write();
                    state.pointer_x = e.client_coordinates().x as f32;
                    state.pointer_y = e.client_coordinates().y as f32;
                }
            },
            onpointerup: move |_| {
                let state = drag();
                if state.dragging_id.is_none() || state.releasing {
                    return;
                }

                let moved = ((state.pointer_x - state.start_x).powi(2) + (state.pointer_y - state.start_y).powi(2)).sqrt() >= 5.0;
                if !moved {
                    if let Some(id) = state.dragging_id {
                        modal_id.set(Some(id));
                    }
                    drag.set(BoardDragState::default());
                    return;
                }

                let drag_id = state.dragging_id;
                let hover_idx = state.hover_idx;
                let hover_after = state.hover_after;
                let hover_section = state.hover_section.clone();
                let dragged_section = state.start_section.clone();
                let issues = props.issues.clone();
                let on_reorder = props.on_reorder;
                drag.write().releasing = true;
                let mut drag_signal = drag;

                spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(BOARD_DROP_MS)).await;
                    if let Some(dragging_id) = drag_id {
                        let target = hover_section.as_ref().and_then(|section| {
                            section_items(&issues, section)
                                .get(hover_idx)
                                .map(|issue| issue.id)
                        });
                        let section_for_drop = hover_section.or(dragged_section);
                        on_reorder.call((dragging_id, target, hover_after, section_for_drop));
                    }
                    drag_signal.set(BoardDragState::default());
                });
            },
            onpointercancel: move |_| drag.set(BoardDragState::default()),
            div { class: "board-grid",
                for (title, items) in sections {
                    BoardLane {
                        key: "lane-{title}",
                        title: title,
                        items: items,
                        drag_state: drag,
                    }
                }
            }

            if let Some(issue) = dragged_issue {
                BoardDragGhost { issue: issue, drag_state: drag() }
            }

            if let Some(issue) = active_modal {
                BoardIssueModal {
                    issue: issue,
                    on_close: move |_| modal_id.set(None),
                    on_status: props.on_status,
                    on_resolution: props.on_resolution,
                    on_labels: props.on_labels,
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct BoardLaneProps {
    title: String,
    items: Vec<Issue>,
    drag_state: Signal<BoardDragState>,
}

#[component]
fn BoardLane(mut props: BoardLaneProps) -> Element {
    let color = section_color(&props.title);
    let lane_active = (props.drag_state)().hover_section.as_deref() == Some(props.title.as_str());

    rsx! {
        section {
            class: if lane_active { "board-lane board-lane-active" } else { "board-lane" },
            onmouseenter: move |_| {
                if props.drag_state.read().dragging_id.is_some() {
                    let mut state = props.drag_state.write();
                    state.hover_section = Some(props.title.clone());
                    if props.items.is_empty() {
                        state.hover_idx = 0;
                        state.hover_after = false;
                    }
                }
            },
            div { class: "board-lane-head",
                div { class: "board-lane-title-row",
                    div { class: "board-lane-dot", style: "background:{color}" }
                    h3 { class: "board-lane-title", "{props.title}" }
                    span { class: "board-lane-count", "{props.items.len()}" }
                }
            }
            div { class: "board-lane-cards",
                if props.items.is_empty() {
                    div { class: "board-empty", div { class: "board-empty-copy", "Drop an issue here" } }
                }
                for (idx, issue) in props.items.iter().enumerate() {
                    BoardCard {
                        key: "board-card-{issue.id}",
                        issue: issue.clone(),
                        lane_title: props.title.clone(),
                        lane_idx: idx,
                        drag_state: props.drag_state,
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct BoardCardProps {
    issue: Issue,
    lane_title: String,
    lane_idx: usize,
    drag_state: Signal<BoardDragState>,
}

#[component]
fn BoardCard(mut props: BoardCardProps) -> Element {
    let ds = (props.drag_state)();
    let is_dragging = ds.dragging_id == Some(props.issue.id);
    let same_hover_lane = ds.hover_section.as_deref() == Some(props.lane_title.as_str());
    let same_start_lane = ds.start_section.as_deref() == Some(props.lane_title.as_str());

    let mut shift = 0.0;
    if ds.dragging_id.is_some() && !is_dragging {
        if same_start_lane && same_hover_lane {
            if ds.hover_idx > ds.start_idx && props.lane_idx > ds.start_idx && props.lane_idx <= ds.hover_idx {
                shift -= BOARD_CARD_H;
            } else if ds.hover_idx < ds.start_idx && props.lane_idx >= ds.hover_idx && props.lane_idx < ds.start_idx {
                shift += BOARD_CARD_H;
            }
        } else if same_start_lane && props.lane_idx > ds.start_idx {
            shift -= BOARD_CARD_H;
        } else if same_hover_lane {
            let insert_idx = if ds.hover_after { ds.hover_idx + 1 } else { ds.hover_idx };
            if props.lane_idx >= insert_idx {
                shift += BOARD_CARD_H;
            }
        }
    }

    let transition = "transform 400ms cubic-bezier(0.25, 1, 0.5, 1), opacity 200ms ease";
    let style = if is_dragging { "opacity:0.14;".to_string() } else { format!("transform: translate3d(0, {shift}px, 0); transition: {transition};") };
    let issue_section = props.issue.section.clone();

    rsx! {
        div {
            class: "board-card-wrap",
            style: "{style}",
            article {
                class: if is_dragging { "board-card board-card-origin" } else { "board-card" },
                onpointerdown: move |e| {
                    e.prevent_default();
                    let rect = e.element_coordinates();
                    props.drag_state.set(BoardDragState {
                        dragging_id: Some(props.issue.id),
                        start_section: Some(issue_section.clone()),
                        hover_section: Some(issue_section.clone()),
                        start_idx: props.lane_idx,
                        hover_idx: props.lane_idx,
                        hover_after: false,
                        start_x: e.client_coordinates().x as f32,
                        start_y: e.client_coordinates().y as f32,
                        pointer_x: e.client_coordinates().x as f32,
                        pointer_y: e.client_coordinates().y as f32,
                        offset_x: rect.x as f32,
                        offset_y: rect.y as f32,
                        releasing: false,
                    });
                },
                onmouseenter: move |e| {
                    if props.drag_state.read().dragging_id.is_some() {
                        let mut state = props.drag_state.write();
                        state.hover_section = Some(props.lane_title.clone());
                        state.hover_idx = props.lane_idx;
                        state.hover_after = e.element_coordinates().y as f32 > (BOARD_CARD_H / 2.0);
                    }
                },
                div { class: "board-card-top",
                    div { class: "board-card-id", "#{props.issue.id}" }
                    span { class: "badge b-{props.issue.status.css_class()}", "{props.issue.status.label()}" }
                }
                div { class: "board-card-section", "{props.issue.section}" }
                div { class: "board-card-title", "{props.issue.title}" }
                if !props.issue.labels.is_empty() {
                    div { class: "labels-row board-labels",
                        span { class: "label b-{props.issue.status.css_class()}", "{props.issue.status.label()}" }
                        LabelList { labels: props.issue.labels.clone() }
                    }
                }
                if !props.issue.files.is_empty() {
                    div { class: "board-card-meta", "{props.issue.files.len()} files touched" }
                } else {
                    div { class: "board-card-meta muted", "No files linked yet" }
                }
            }
            if same_hover_lane && ds.hover_idx == props.lane_idx {
                div {
                    class: "board-drop-indicator",
                    style: if ds.hover_after { "order:2;" } else { "order:-1;" }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct BoardDragGhostProps {
    issue: Issue,
    drag_state: BoardDragState,
}

#[component]
fn BoardDragGhost(props: BoardDragGhostProps) -> Element {
    let left = props.drag_state.pointer_x - props.drag_state.offset_x;
    let top = props.drag_state.pointer_y - props.drag_state.offset_y;

    rsx! {
        article {
            class: if props.drag_state.releasing { "board-card board-card-ghost board-card-ghost-settling" } else { "board-card board-card-ghost" },
            style: "left:{left}px; top:{top}px;",
            div { class: "board-card-top",
                div { class: "board-card-id", "#{props.issue.id}" }
                span { class: "badge b-{props.issue.status.css_class()}", "{props.issue.status.label()}" }
            }
            div { class: "board-card-section", "{props.issue.section}" }
            div { class: "board-card-title", "{props.issue.title}" }
            if !props.issue.labels.is_empty() {
                div { class: "labels-row board-labels",
                    span { class: "label b-{props.issue.status.css_class()}", "{props.issue.status.label()}" }
                    LabelList { labels: props.issue.labels.clone() }
                }
            }
        }
    }
}

fn grouped_sections(issues: &[Issue]) -> Vec<(String, Vec<Issue>)> {
    let mut grouped = BTreeMap::<String, Vec<Issue>>::new();
    for issue in issues {
        grouped.entry(issue.section.clone()).or_default().push(issue.clone());
    }

    let mut sections: Vec<_> = grouped.into_iter().collect();
    sections.sort_by(|(left, _), (right, _)| section_sort_key(left).cmp(&section_sort_key(right)).then_with(|| left.cmp(right)));
    sections
}

fn section_items<'a>(issues: &'a [Issue], section: &str) -> Vec<&'a Issue> {
    issues.iter().filter(|issue| issue.section == section).collect()
}

fn section_sort_key(section: &str) -> (u8, String) {
    let normalized = section.trim().to_ascii_lowercase();
    let rank = if normalized.contains("active") { 0 } else if normalized.contains("backlog") { 1 } else if normalized.contains("done") { 2 } else { 3 };
    (rank, normalized)
}

fn section_color(section: &str) -> &'static str {
    let normalized = section.trim().to_ascii_lowercase();
    if normalized.contains("done") {
        "var(--green)"
    } else if normalized.contains("backlog") {
        "var(--blue)"
    } else if normalized.contains("active") {
        "var(--orange)"
    } else {
        "var(--teal)"
    }
}

#[derive(Clone, PartialEq, Props)]
struct BoardIssueModalProps {
    issue: Issue,
    on_close: EventHandler<()>,
    on_status: EventHandler<(u32, String)>,
    on_resolution: EventHandler<(u32, String)>,
    on_labels: EventHandler<(u32, String)>,
}

#[component]
fn BoardIssueModal(props: BoardIssueModalProps) -> Element {
    let i = &props.issue;
    let id = i.id;
    let mut labels_input = use_signal(|| i.labels.join(", "));
    let section = i.section.to_ascii_lowercase();
    let color = if section.contains("done") || i.status == Status::Done || i.status == Status::Descoped {
        "var(--green)"
    } else if section.contains("backlog") {
        "var(--blue)"
    } else {
        "var(--orange)"
    };

    rsx! {
        div {
            class: "modal-overlay open",
            onclick: move |_| props.on_close.call(()),
            div {
                class: "modal",
                onclick: move |e| e.stop_propagation(),
                div { class: "m-accent", style: "background:{color}" }
                div { class: "m-head",
                    div {
                        div { class: "m-id", "ISSUE-" }
                        div { class: "m-id-num", "{id}" }
                    }
                    button { class: "m-close", onclick: move |_| props.on_close.call(()), "×" }
                }
                div { class: "m-title", "{i.title}" }
                div { class: "m-status-row",
                    div { class: "m-dot", style: "background:{color}" }
                    span { class: "m-status-text", "{i.status.label()}" }
                    div { class: "m-labels",
                        span { class: "label b-{i.status.css_class()}", "{i.status.label()}" }
                        LabelList { labels: i.labels.clone() }
                    }
                }
                hr { class: "m-divider" }
                div { class: "m-body",
                    div { class: "m-body-label", "Description" }
                    p { "{i.description}" }
                    div { style: "margin-top: 16px;",
                        div { class: "m-body-label", "Labels" }
                        input {
                            class: "modal-input",
                            value: "{labels_input}",
                            oninput: move |e| {
                                labels_input.set(e.value().clone());
                                props.on_labels.call((id, e.value()));
                            },
                        }
                    }
                    div { style: "margin-top: 16px;",
                        div { class: "m-body-label", "Resolution" }
                        textarea {
                            class: "res-input",
                            rows: "4",
                            value: "{i.resolution}",
                            oninput: move |e| props.on_resolution.call((id, e.value())),
                        }
                    }
                    div { style: "margin-top: 16px;",
                        div { class: "m-body-label", "Update Status" }
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
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::grouped_sections;
    use crate::model::{Issue, Status};

    fn issue(id: u32, section: &str) -> Issue {
        Issue {
            id,
            title: format!("Issue {id}"),
            status: Status::Open,
            files: vec![],
            labels: vec![],
            links: vec![],
            description: String::new(),
            resolution: String::new(),
            section: section.to_string(),
            depends_on: vec![],
        }
    }

    #[test]
    fn grouped_sections_follow_feed_ordering() {
        let sections = grouped_sections(&[
            issue(1, "DONE Issues"),
            issue(2, "Sprint 42"),
            issue(3, "ACTIVE Issues"),
            issue(4, "BACKLOG Issues"),
        ]);

        let labels: Vec<_> = sections.into_iter().map(|(name, _)| name).collect();
        assert_eq!(
            labels,
            vec![
                "ACTIVE Issues".to_string(),
                "BACKLOG Issues".to_string(),
                "DONE Issues".to_string(),
                "Sprint 42".to_string()
            ]
        );
    }
}
