mod card;

use crate::model::{Issue, Status};
use card::{IssueCard, SLOT};
use dioxus::prelude::*;

#[derive(Clone, Default, PartialEq)]
pub struct DragState {
    pub dragging_id: Option<u32>,
    pub start_idx: usize,
    pub hover_idx: usize,
    pub start_y: f32,
    pub offset_y: f32,
    pub releasing: bool,
}

#[derive(Clone, PartialEq, Props)]
pub struct FeedViewProps {
    pub issues: Vec<Issue>,
    pub on_status: EventHandler<(u32, String)>,
    pub on_resolution: EventHandler<(u32, String)>,
    pub on_reorder: EventHandler<(u32, u32, bool)>,
}

#[component]
pub fn FeedView(props: FeedViewProps) -> Element {
    let mut drag_state = use_signal(DragState::default);
    let mut modal_id: Signal<Option<u32>> = use_signal(|| None);
    let on_reorder = props.on_reorder;
    
    // Captured for index lookups inside the event handlers
    let issues_len = props.issues.len();
    let issues_for_up = props.issues.clone();
    
    // Compute total absolute container height, plus the 200px scroll padding at the bottom
    let total_height = (issues_len as f32 * SLOT) + 200.0;

    rsx! {
        div {
            class: "feed",
            onpointermove: move |e| {
                let mut ds = drag_state.write();
                if ds.dragging_id.is_some() && !ds.releasing {
                    ds.offset_y = e.client_coordinates().y as f32 - ds.start_y;
                    
                    // What slot grid is the card currently dragged over?
                    let raw_y = (ds.start_idx as f32 * SLOT) + ds.offset_y;
                    let max_idx = issues_len.saturating_sub(1);
                    ds.hover_idx = (raw_y / SLOT).round().clamp(0.0, max_idx as f32) as usize;
                }
            },
            onpointerup: move |_| {
                let mut ds = drag_state.write();
                if let Some(id) = ds.dragging_id {
                    if ds.releasing { return; }

                    // If it was just a tiny click/movement, clear drag and open the issue modal
                    if ds.offset_y.abs() < 5.0 && ds.start_idx == ds.hover_idx {
                        ds.dragging_id = None;
                        drop(ds);
                        modal_id.set(Some(id));
                        return;
                    }

                    // Otherwise, trigger the snap-to-socket animation on the card
                    ds.releasing = true;
                    
                    let drag_id = id;
                    let start_idx = ds.start_idx;
                    let hover_idx = ds.hover_idx;
                    
                    // MUST drop the write lock before we can copy the drag_state signal
                    // into the spawned future
                    drop(ds);
                    
                    let issues_clone = issues_for_up.clone();
                    let on_reorder_clone = on_reorder;
                    let mut ds_signal = drag_state;

                    spawn(async move {
                        // Wait exactly the length of the 200ms CSS transition
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        
                        if start_idx != hover_idx {
                            if let Some(target) = issues_clone.get(hover_idx) {
                                let target_id = target.id;
                                let after = hover_idx > start_idx;
                                on_reorder_clone.call((drag_id, target_id, after));
                            }
                        }
                        ds_signal.set(DragState::default());
                    });
                }
            },
            onpointercancel: move |_| {
                drag_state.set(DragState::default());
            },

            div { 
                class: "feed-inner",
                // absolute container required so cards measure from the top
                style: "position: relative; height: {total_height}px;",
                for (idx, issue) in props.issues.iter().enumerate() {
                    IssueCard {
                        key: "{issue.id}",
                        issue: issue.clone(),
                        idx: idx,
                        drag_state: drag_state,
                    }
                }
            }
        }

        if let Some(id) = modal_id() {
            if let Some(issue) = props.issues.iter().find(|i| i.id == id) {
                IssueModal {
                    issue: issue.clone(),
                    on_close: move |_| modal_id.set(None),
                    on_status: props.on_status,
                    on_resolution: props.on_resolution,
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct IssueModalProps {
    issue: Issue,
    on_close: EventHandler<()>,
    on_status: EventHandler<(u32, String)>,
    on_resolution: EventHandler<(u32, String)>,
}

#[component]
fn IssueModal(props: IssueModalProps) -> Element {
    let i = &props.issue;
    let id = i.id;

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| props.on_close.call(()),
            div {
                class: "modal",
                onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h2 {
                        span { class: "cid", style: "margin-right:12px;", "#{id}" }
                        "{i.title}"
                    }
                    div { style: "display:flex;align-items:center;gap:12px;",
                        span { class: "badge b-{i.status.css_class()}", "{i.status.label()}" }
                        button {
                            class: "modal-close",
                            onclick: move |_| props.on_close.call(()),
                            "×"
                        }
                    }
                }
                div { class: "modal-body",
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
                                        for f in &i.files { span { class: "chip-file", "{f}" } }
                                    }
                                }
                            }
                            if !i.depends_on.is_empty() {
                                div { class: "fgroup",
                                    label { class: "flbl", "Depends On" }
                                    div { class: "chips",
                                        for d in &i.depends_on { span { class: "chip-dep", "#{d}" } }
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