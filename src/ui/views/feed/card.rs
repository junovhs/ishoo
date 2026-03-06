use crate::model::Issue;
use crate::ui::views::feed::DragState;
use dioxus::prelude::*;

pub const CARD_H: f32 = 54.0;
pub const GAP: f32 = 9.0;
pub const SLOT: f32 = CARD_H + GAP;

#[derive(Clone, PartialEq, Props)]
pub struct IssueCardProps {
    pub issue: Issue,
    pub idx: usize,
    pub drag_state: Signal<DragState>,
}

#[component]
pub fn IssueCard(props: IssueCardProps) -> Element {
    let i = &props.issue;
    let id = i.id;
    let idx = props.idx;
    let ds = props.drag_state.read();

    let is_dragging = ds.dragging_id == Some(id);

    // Compute the effective index for where this card should visually sit right now
    let mut target_idx = idx as i32;
    if ds.dragging_id.is_some() && !is_dragging {
        let start = ds.start_idx as i32;
        let hover = ds.hover_idx as i32;
        let curr = idx as i32;
        
        if curr > start && curr <= hover {
            target_idx -= 1; // Shift up to make room for dragged card moving down
        } else if curr < start && curr >= hover {
            target_idx += 1; // Shift down to make room for dragged card moving up
        }
    }

    let y_pos = if is_dragging && !ds.releasing {
        // Free follow cursor relative to the starting slot
        (ds.start_idx as f32 * SLOT) + ds.offset_y
    } else if is_dragging && ds.releasing {
        // Snap/suck into the final hover socket
        ds.hover_idx as f32 * SLOT
    } else {
        // Displaced cards or resting cards sit strictly in their assigned socket
        target_idx as f32 * SLOT
    };

    let transition = if is_dragging && !ds.releasing {
        "none" // Instantly follow cursor
    } else if is_dragging && ds.releasing {
        "transform 200ms cubic-bezier(0.2, 0, 0, 1), box-shadow 200ms ease" // Suck into socket
    } else {
        "transform 200ms ease" // Displaced cards sliding around
    };

    let cls = if is_dragging && !ds.releasing {
        "item dragging"
    } else if is_dragging && ds.releasing {
        "item settling"
    } else {
        "item"
    };

    let outer_style = format!(
        "position: absolute; top: 0; left: 0; right: 0; transform: translate3d(0, {y_pos}px, 0); transition: {transition};"
    );

    let mut drag_state_signal = props.drag_state;

    rsx! {
        div { 
            class: "{cls}", 
            style: "{outer_style}",
            div {
                class: "card",
                onpointerdown: move |e| {
                    e.prevent_default();
                    let mut ds_write = drag_state_signal.write();
                    ds_write.dragging_id = Some(id);
                    ds_write.start_idx = idx;
                    ds_write.hover_idx = idx;
                    ds_write.start_y = e.client_coordinates().y as f32;
                    ds_write.offset_y = 0.0;
                    ds_write.releasing = false;
                },
                div { class: "card-hdr",
                    span { class: "cid", "#{id}" }
                    span { class: "ctitle", "{i.title}" }
                    span { class: "badge b-{i.status.css_class()}", "{i.status.label()}" }
                }
            }
        }
    }
}
