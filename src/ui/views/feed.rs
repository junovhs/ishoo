// neti:allow(LAW OF ATOMICITY)
mod card;

use crate::model::{Issue, Status};
use crate::ui::components::label_tone_class;
use card::IssueCard;
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct HoverTarget {
    idx: usize,
    after: bool,
    y: f32,
}

pub(super) const DRAG_DEADZONE_PX: f32 = 8.0;

pub(super) fn apply_drag_deadzone(offset_y: f32) -> f32 {
    if offset_y.abs() < DRAG_DEADZONE_PX {
        0.0
    } else {
        offset_y - (offset_y.signum() * DRAG_DEADZONE_PX)
    }
}

fn compute_hover_target(
    start_idx: usize,
    start_virtual_y: f32,
    offset_y: f32,
    candidates: &[(usize, f32)],
) -> HoverTarget {
    let logical_y = start_virtual_y + apply_drag_deadzone(offset_y);
    let mut closest = HoverTarget {
        idx: start_idx,
        after: false,
        y: start_virtual_y,
    };
    let mut min_dist = f32::MAX;

    for (idx, insertion_y) in candidates {
        let dist = (logical_y - *insertion_y).abs();

        if dist < min_dist {
            min_dist = dist;
            closest = HoverTarget {
                idx: *idx,
                after: false,
                y: *insertion_y,
            };
        }
    }

    closest
}

#[derive(Clone, Default, PartialEq)]
pub struct DragState {
    pub dragging_id: Option<u32>,
    pub start_idx: usize,
    pub hover_idx: usize,
    pub hover_after: bool,
    pub hover_y: f32,
    pub start_y: f32,
    pub start_virtual_y: f32,
    pub offset_y: f32,
    pub releasing: bool,
}

#[derive(Clone, Default, PartialEq)]
pub struct RecentDropState {
    pub id: Option<u32>,
    pub release_x: f32,
    pub release_y: f32,
    pub hover_armed: bool,
}

#[derive(Clone, PartialEq, Props)]
pub struct FeedViewProps {
    pub is_compact: bool,
    pub zoom: f32,
    pub issues: Vec<Issue>,
    pub on_status: EventHandler<(u32, String)>,
    pub on_resolution: EventHandler<(u32, String)>,
    pub on_labels: EventHandler<(u32, String)>,
    pub on_reorder: EventHandler<(u32, u32, bool)>,
    pub on_section_toggle: EventHandler<()>,
}

#[component]
pub fn FeedView(props: FeedViewProps) -> Element {
    let mut drag_state = use_signal(DragState::default);
    let mut recent_drop = use_signal(RecentDropState::default);
    let mut modal_id: Signal<Option<u32>> = use_signal(|| None);
    let on_reorder = props.on_reorder;
    let on_section_toggle = props.on_section_toggle;
    
    // Captured for index lookups inside the event handlers
    let issues_len = props.issues.len();
    let issues_for_up = props.issues.clone();
    let is_compact = props.is_compact;
    let slot_h = if is_compact { 44.0 } else { 93.0 };
    
    // Compute total absolute container height, plus the 200px scroll padding at the bottom
    // We add 45.0 px worth of height for each of the 3 section headers
    let total_height = (issues_len as f32 * slot_h) + (3.0 * 45.0) + 200.0;
    
    // We need to track which sections are currently collapsed
    let mut collapsed = use_signal(std::collections::HashSet::<String>::new);

    rsx! {
        div {
            class: if props.is_compact { "feed compact" } else { "feed" },
            onpointermove: move |e| {
                {
                    let rd = recent_drop.read();
                    if let Some(_id) = rd.id {
                        if !rd.hover_armed {
                            let dx = e.client_coordinates().x as f32 - rd.release_x;
                            let dy = e.client_coordinates().y as f32 - rd.release_y;
                            if (dx * dx + dy * dy).sqrt() >= 20.0 {
                                drop(rd);
                                let mut rdw = recent_drop.write();
                                rdw.hover_armed = true;
                            }
                        }
                    }
                }

                // BUG FIX: Do not unconditionally call drag_state.write() here!
                // Calling .write() dirties the signal and forces every single IssueCard
                // to re-render on *every single mouse move*. Read first, write only if dragging.
                let ds_read = drag_state.read();
                if ds_read.dragging_id.is_none() || ds_read.releasing {
                    return;
                }
                drop(ds_read);

                let mut ds = drag_state.write();
                ds.offset_y = (e.client_coordinates().y as f32 - ds.start_y) / props.zoom;

                let sections = [
                    ("Active", "active", "var(--orange)"),
                    ("Backlog", "backlog", "var(--blue)"),
                    ("Done", "done", "var(--green)"),
                ];

                let mut insertion_slots = vec![(ds.start_idx, ds.start_virtual_y, false)];
                let mut current_y = 0.0;

                for (_label, key, _color) in sections {
                    let section_items: Vec<(usize, &Issue)> = props
                        .issues
                        .iter()
                        .enumerate()
                        .filter(|(idx, _)| *idx != ds.start_idx)
                        .filter(|(_, i)| {
                            let sec = i.section.to_lowercase();
                            let is_done =
                                sec.contains("done") || i.status == Status::Done || i.status == Status::Descoped;
                            let is_backlog = !is_done && sec.contains("backlog");
                            match key {
                                "done" => is_done,
                                "backlog" => is_backlog,
                                "active" => !is_done && !is_backlog,
                                _ => false,
                            }
                        })
                        .collect();

                    if section_items.is_empty() {
                        continue;
                    }

                    current_y += 45.0;

                    if !collapsed.read().contains(key) {
                        let mut prev_idx: Option<usize> = None;
                        for (idx, _) in section_items {
                            if current_y != ds.start_virtual_y {
                                insertion_slots.push((idx, current_y, false));
                            }
                            prev_idx = Some(idx);
                            current_y += slot_h;
                        }
                        if let Some(last_idx) = prev_idx {
                            insertion_slots.push((last_idx, current_y, true));
                        }
                    }
                }

                let hover = compute_hover_target(
                    ds.start_idx,
                    ds.start_virtual_y,
                    ds.offset_y,
                    &insertion_slots.iter().map(|(idx, y, _)| (*idx, *y)).collect::<Vec<_>>(),
                );

                ds.hover_idx = hover.idx;
                ds.hover_y = hover.y;
                ds.hover_after = insertion_slots
                    .iter()
                    .find(|(idx, y, _)| *idx == hover.idx && (*y - hover.y).abs() < f32::EPSILON)
                    .map(|(_, _, after)| *after)
                    .unwrap_or(false);
            },
            onpointerup: move |e| {
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
                    let hover_after = ds.hover_after;
                    recent_drop.set(RecentDropState {
                        id: Some(id),
                        release_x: e.client_coordinates().x as f32,
                        release_y: e.client_coordinates().y as f32,
                        hover_armed: false,
                    });
                    
                    // MUST drop the write lock before we can copy the drag_state signal
                    // into the spawned future
                    drop(ds);
                    
                    let issues_clone = issues_for_up.clone();
                    let on_reorder_clone = on_reorder;
                    let mut ds_signal = drag_state;

                    spawn(async move {
                        // Wait exactly the length of the 400ms CSS transition
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                        
                        if start_idx != hover_idx {
                            if let Some(target) = issues_clone.get(hover_idx) {
                                let target_id = target.id;
                                on_reorder_clone.call((drag_id, target_id, hover_after));
                            }
                        }
                        
                        ds_signal.set(DragState::default());
                    });
                }
            },
            onpointercancel: move |_| {
                // BUG FIX: Window managers (especially X11/Wayland via winit) frequently fire 
                // PointerCancel immediately after PointerUp if the cursor moves slightly 
                // during the physical button release, or if the window loses exact grab focus.
                // We MUST NOT clear the drag state if we are currently animating the release
                // drop sequence (`ds.releasing == true`), otherwise the state zeros out instantly
                // and the card snaps back to origin, breaking the 200ms easing transition.
                let mut ds = drag_state.write();
                if ds.dragging_id.is_some() && !ds.releasing {
                    *ds = DragState::default();
                }
            },

            div { 
                id: "scroll-content",
                class: "feed-inner",
                // absolute container required so cards measure from the top
                style: "position: relative; height: {total_height}px;",
                {
                    // Group issues into sections in the exact order they appear in the array
                    // so that `idx` strictly maps 1:1 with `props.issues[idx]`.
                    let mut elements = vec![];
                    
                    let sections = [
                        ("Active", "active", "var(--orange)"),
                        ("Backlog", "backlog", "var(--blue)"),
                        ("Done", "done", "var(--green)"),
                    ];
                    
                    let array_reordered = false;
                    
                    let mut current_y = 0.0;
                    
                    for (label, key, color) in sections {
                        let section_items: Vec<(usize, &Issue)> = props.issues.iter().enumerate()
                            .filter(|(_, i)| {
                                let sec = i.section.to_lowercase();
                                let is_done = sec.contains("done") || i.status == Status::Done || i.status == Status::Descoped;
                                let is_backlog = !is_done && sec.contains("backlog");
                                let is_active = !is_done && !is_backlog;
                                
                                match key {
                                    "done" => is_done,
                                    "backlog" => is_backlog,
                                    "active" => is_active,
                                    _ => false
                                }
                            })
                            .collect();
                            
                        if section_items.is_empty() {
                            continue;
                        }
                        
                        let is_collapsed = collapsed.read().contains(key);
                        let key_clone = key.to_string();
                        let count = section_items.len();
                        
                        // Push the header element into the container natively, taking up 1 slot.
                        // For the section class, we append " collapsed" if true, though the CSS 
                        // handles the chevron separately via the outer class. We'll just style the chevron directly.
                        elements.push(rsx! {
                            div {
                                key: "header-{key}",
                                id: "sh-{key}",
                                class: "section-head s-{key}",
                                onclick: move |_| {
                                    let mut c = collapsed.write();
                                    if c.contains(&key_clone) {
                                        c.remove(&key_clone);
                                    } else {
                                        c.insert(key_clone.clone());
                                    }
                                    on_section_toggle.call(());
                                },
                                span { 
                                    class: if is_collapsed { "chevron collapsed" } else { "chevron" }, 
                                    "▼" 
                                }
                                div { class: "section-dot", style: "background:{color}" }
                                span { class: "section-title", "{label}" }
                                span { class: "section-count", "{count}" }
                                div { class: "section-line" }
                            }
                        });
                        
                        current_y += 45.0; // Math matched to the 45px section-head constraint.
                        
                        for (idx, issue) in section_items {
                            let target_y = if is_collapsed { current_y - 45.0 } else { current_y };
                            
                            if !is_collapsed {
                                elements.push(rsx! {
                                    div { key: "spacer-{issue.id}", style: "height: {slot_h}px; width: 100%;" }
                                });
                            }
                            
                            elements.push(rsx! {
                                IssueCard {
                                    key: "card-{issue.id}",
                                    issue: issue.clone(),
                                    idx: idx,
                                    virtual_y: target_y,
                                    drag_state: drag_state,
                                    recent_drop: recent_drop,
                                    is_compact: props.is_compact,
                                    array_reordered: array_reordered,
                                    is_hidden: is_collapsed,
                                }
                            });
                            
                            if !is_collapsed {
                                current_y += slot_h;
                            }
                        }
                    }
                    
                    elements.into_iter()
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
                    on_labels: props.on_labels,
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
    on_labels: EventHandler<(u32, String)>,
}

fn render_markdown(text: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    let parser = pulldown_cmark::Parser::new_ext(text, options);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output
}

#[component]
fn IssueModal(props: IssueModalProps) -> Element {
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
    
    // We don't have age or comments on the backend yet, use placeholders
    let age = "2 days";
    let comments_len = 0;
    
    let html_desc = render_markdown(&i.description);

    rsx! {
        div {
            class: "modal-overlay open", // We use Dioxus conditional rendering so it's always open when mounted
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
                    div { class: "m-actions",
                        button { class: "m-btn", title: "Edit description", "✎ Edit" }
                        button { class: "m-close", id: "mc", onclick: move |_| props.on_close.call(()), "×" }
                    }
                }
                div { class: "m-title", "{i.title}" }
                div { class: "m-status-row",
                    div { class: "m-dot", style: "background:{color}" }
                    span { class: "m-status-text", "{i.status.label()}" }
                    div { class: "m-labels",
                        span { class: "label b-{i.status.css_class()}", "{i.status.label()}" }
                        for label in &i.labels {
                            span { class: "label {label_tone_class(label)}", "{label}" }
                        }
                    }
                }
                hr { class: "m-divider" }
                div { class: "m-grid",
                    div { class: "m-cell", 
                        div { class: "m-cell-l", "Age" } 
                        div { class: "m-cell-v", "{age}" } 
                    }
                    div { class: "m-cell", 
                        div { class: "m-cell-l", "Files" } 
                        div { class: "m-cell-v", "{i.files.len()}" } 
                    }
                    div { class: "m-cell", 
                        div { class: "m-cell-l", "Notes" } 
                        div { class: "m-cell-v", "{comments_len}" } 
                    }
                }
                div { style: "padding:0 28px;",
                    div { class: "m-cell-l", style: "margin:8px 0 4px;", "Files" }
                    div { class: "m-cell-v files",
                        for f in &i.files { code { "{f}" } br {} }
                    }
                }
                if !i.depends_on.is_empty() {
                    div { class: "m-links",
                        hr { class: "m-divider", style: "margin:12px 0;" }
                        div { class: "m-link-label", "Linked Issues" }
                        for d in &i.depends_on {
                            span { class: "m-link-item", "↗ ISSUE-{d}" }
                        }
                    }
                }
                hr { class: "m-divider" }
                div { class: "m-body",
                    div { class: "m-body-label", "Description" }
                    div { dangerous_inner_html: "{html_desc}" }
                    
                    div { style: "margin-top: 16px;",
                        div { class: "m-body-label", "Labels" }
                        input {
                            class: "modal-input",
                            placeholder: "core, frontend, ux",
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
                            style: "width: 100%; border: 1px solid var(--rule); background: transparent; color: inherit; font: inherit; padding: 8px;",
                            rows: "4",
                            placeholder: "Log your solution...",
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
                div { class: "m-nav",
                    span { kbd { "↑" } kbd { "↓" } " prev / next" }
                    span { kbd { "Esc" } " close" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_hover_target, HoverTarget};

    #[test]
    fn tiny_downward_motion_does_not_reorder() {
        let hover = compute_hover_target(0, 45.0, 1.0, &[(0, 45.0), (1, 138.0), (2, 231.0)]);
        assert_eq!(
            hover,
            HoverTarget {
                idx: 0,
                after: false,
                y: 45.0
            }
        );
    }

    #[test]
    fn downward_reorders_after_crossing_first_boundary() {
        let hover = compute_hover_target(0, 45.0, 70.0, &[(0, 45.0), (1, 138.0), (2, 231.0)]);
        assert_eq!(
            hover,
            HoverTarget {
                idx: 1,
                after: false,
                y: 138.0
            }
        );
    }

    #[test]
    fn upward_reorders_after_crossing_first_boundary() {
        let hover = compute_hover_target(2, 231.0, -70.0, &[(0, 45.0), (1, 138.0), (2, 231.0)]);
        assert_eq!(
            hover,
            HoverTarget {
                idx: 1,
                after: false,
                y: 138.0
            }
        );
    }
}
