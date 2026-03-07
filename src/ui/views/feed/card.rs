use crate::model::Issue;
use crate::ui::views::feed::DragState;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct IssueCardProps {
    pub issue: Issue,
    pub idx: usize,
    pub virtual_y: f32, // The pre-calculated absolute Y position of the slot
    pub drag_state: Signal<DragState>,
    pub is_compact: bool,
    pub array_reordered: bool,
}

#[component]
// neti:allow(LAW OF COMPLEXITY)
pub fn IssueCard(props: IssueCardProps) -> Element {
    let i = &props.issue;
    let id = i.id;
    let idx = props.idx;
    let ds = props.drag_state.read();

    let is_dragging = ds.dragging_id == Some(id);
    let array_reordered = props.array_reordered;

    // Compute the effective index for where this card should visually sit right now
    let mut virtual_y = props.virtual_y;
    let slot_h = if props.is_compact { 44.0 } else { 93.0 };
    
    if ds.dragging_id.is_some() && !is_dragging && !array_reordered {
        let start = ds.start_idx as i32;
        let hover = ds.hover_idx as i32;
        let curr = idx as i32;
        
        if curr > start && curr <= hover {
            virtual_y -= slot_h; // Shift up to make room for dragged card moving down
        } else if curr < start && curr >= hover {
            virtual_y += slot_h; // Shift down to make room for dragged card moving up
        }
    }

    let mut actually_dragging = is_dragging;
    let mut effective_offset = ds.offset_y;
    let deadzone = 8.0; // Buttery 8px deadzone for sloppy clicks
    
    if actually_dragging && !ds.releasing {
        if effective_offset.abs() < deadzone {
            actually_dragging = false; 
            effective_offset = 0.0;
        } else {
            effective_offset = effective_offset - (effective_offset.signum() * deadzone);
        }
    }

    let y_pos = if is_dragging && !ds.releasing {
        // Free follow cursor relative to the starting slot
        props.virtual_y + effective_offset
    } else if is_dragging && ds.releasing {
        // Snap/suck into the final hover socket
        if array_reordered {
            virtual_y
        } else {
            ds.hover_y
        }
    } else {
        // Displaced cards or resting cards sit strictly in their assigned socket
        virtual_y
    };

    let transition = if actually_dragging && !ds.releasing {
        "none" // Instantly follow cursor once deadzone broke
    } else {
        "transform 400ms cubic-bezier(0.25, 1, 0.5, 1)" // Match the 0.4s box-shadow / scale release
    };

    let mut cls = "item".to_string();
    if actually_dragging && !ds.releasing {
        cls.push_str(" dragging");
    }
    if ds.releasing && is_dragging {
        cls.push_str(" settling");
    }

    let outer_style = format!(
        "position: absolute; top: 0; left: 0px; right: 0px; transform: translate3d(0, {y_pos}px, 0); transition: {transition};"
    );

    let mut drag_state_signal = props.drag_state;

    let is_done = i.status == crate::model::Status::Done || i.status == crate::model::Status::Descoped;
    let sec_lower = i.section.to_lowercase();
    let is_backlog = !is_done && sec_lower.contains("backlog");
    
    let section_color = if is_done {
        "var(--green)"
    } else if is_backlog {
        "var(--blue)"
    } else {
        "var(--orange)"
    };

    rsx! {
        div { 
            class: "{cls}", 
            style: "{outer_style}",
            div {
                class: "issue-row",
                onpointerdown: move |e| {
                    e.prevent_default();
                    let mut ds_write = drag_state_signal.write();
                    ds_write.dragging_id = Some(id);
                    ds_write.start_idx = idx;
                    ds_write.hover_idx = idx;
                    ds_write.start_y = e.client_coordinates().y as f32;
                    ds_write.start_virtual_y = props.virtual_y;
                    ds_write.offset_y = 0.0;
                    ds_write.hover_y = props.virtual_y;
                    ds_write.releasing = false;
                },
                div { class: "id-badge",
                    span { class: "id-cat", "ISS-" }
                    span { class: "id-num", "{id}" }
                }
                div { class: "issue-body",
                    div { class: "issue-title",
                        "{i.title}"
                    }
                    if !props.is_compact {
                        div { class: "issue-sub",
                            span { "{i.files.len()} file", if i.files.len() != 1 { "s" } }
                            span { class: "sep", "/" }
                            span { "2 days" }
                        }
                        div { class: "labels-row", style: "display:flex;gap:4px;margin-top:4px;",
                            span { class: "label b-{i.status.css_class()}", "{i.status.label()}" }
                            // Mock labels for now since model doesn't have them
                            span { class: "label", style: "color:var(--ink3);border-color:var(--ink3)", "Core" }
                        }
                    }
                }
                div { class: "issue-right", // Empty space on the right, matches the dot and links
                    if !i.depends_on.is_empty() {
                        span { class: "xlink", title: "Linked to {i.depends_on.len()} issue(s)", "↗" }
                    }
                    div { class: "s-dot", style: "background:{section_color}; width:8px; height:8px; border-radius:50%;" }
                }
            }
        }
    }
}
