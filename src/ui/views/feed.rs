mod card;

use super::physics::{DragState, PendingReorder};
use crate::model::{Issue, Status};
use card::IssueCard;
use dioxus::prelude::*;

const DRAG_THRESHOLD: f32 = 5.0;

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
    let issues_clone = props.issues.clone();
    let mut issues_for_layout = use_signal(|| issues_clone.clone());
    let mut modal_id: Signal<Option<u32>> = use_signal(|| None);

    // Keep layout signal in sync with props
    use_effect(move || {
        issues_for_layout.set(issues_clone.clone());
    });

    let on_reorder = props.on_reorder;

    // Physics loop — spawns ONCE per component mount
    use_coroutine(move |_rx: UnboundedReceiver<()>| async move {
        let mut last_time = tokio::time::Instant::now();
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(8)).await;
            let now = tokio::time::Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32();
            last_time = now;

            let is_active = drag_state.read().is_active();
            if is_active {
                let mut ds = drag_state.write();
                if ds.is_dragging && ds.dragging_id.is_some() {
                    ds.step_drag(dt);
                } else if ds.settling_id.is_some() {
                    if let Some(reorder) = ds.step_settle(dt) {
                        drop(ds);
                        on_reorder.call((reorder.drag_id, reorder.target_id, reorder.insert_after));
                    }
                }
            }
        }
    });

    rsx! {
        div {
            class: "feed",
            onpointermove: move |e| {
                let coords = e.client_coordinates();
                let x = coords.x as f32;
                let y = coords.y as f32;

                let mut ds = drag_state.write();
                if ds.dragging_id.is_some() {
                    let dt = 1.0 / 60.0;
                    ds.update_velocity(x, y, dt);

                    if !ds.is_dragging && exceeds_threshold(&ds) {
                        ds.is_dragging = true;
                    }
                }
            },
            onpointerup: move |_| {
                let mut ds = drag_state.write();
                if let Some(id) = ds.dragging_id {
                    if !ds.is_dragging {
                        // Tap, not drag — open modal
                        ds.reset();
                        drop(ds);
                        modal_id.set(Some(id));
                    } else {
                        let dy = ds.cur_y - ds.start_y;
                        let dx = (ds.cur_x - ds.start_x) * 0.6;
                        let new_idx = ds.cur_idx;
                        let orig_idx = ds.orig_idx;

                        if new_idx != orig_idx {
                            let target_id = ds.layout_ids.get(new_idx).copied().unwrap_or(0);
                            let insert_after = new_idx > orig_idx;
                            ds.pending_reorder = Some(PendingReorder {
                                drag_id: id,
                                target_id,
                                insert_after,
                            });
                        }

                        ds.begin_settle(dy, dx, new_idx);
                        ds.settling_id = Some(id);
                        ds.dragging_id = None;
                    }
                }
            },
            onpointercancel: move |_| {
                drag_state.write().reset();
            },

            div { class: "feed-inner",
                for issue in &props.issues {
                    IssueCard {
                        key: "{issue.id}",
                        issue: issue.clone(),
                        drag_state: drag_state,
                        issues_for_layout: issues_for_layout,
                        on_open: move |id| modal_id.set(Some(id)),
                    }
                }
                div { style: "height:200px;" }
            }
        }

        // Detail modal
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

// ─── Modal ───────────────────────────────────────────────────────────────────

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

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn exceeds_threshold(ds: &DragState) -> bool {
    (ds.cur_x - ds.start_x).abs() > DRAG_THRESHOLD
        || (ds.cur_y - ds.start_y).abs() > DRAG_THRESHOLD
}
