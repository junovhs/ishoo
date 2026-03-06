use crate::model::Issue;
use crate::ui::views::physics::DragState;
use dioxus::prelude::*;

// Must match styles.rs: .card-hdr height + .card margin-bottom
pub const CARD_HDR_HEIGHT: f32 = 54.0;
pub const CARD_GAP: f32 = 9.0;
pub const SLOT_SIZE: f32 = CARD_HDR_HEIGHT + CARD_GAP;

#[derive(Clone, PartialEq, Props)]
pub struct IssueCardProps {
    pub issue: Issue,
    pub drag_state: Signal<DragState>,
    pub issues_for_layout: Signal<Vec<Issue>>,
    pub on_open: EventHandler<u32>,
}

#[component]
pub fn IssueCard(mut props: IssueCardProps) -> Element {
    let i = &props.issue;
    let id = i.id;

    let (item_class, item_style, card_style) = compute_styles(id, &props.drag_state.read());

    rsx! {
        div { class: "{item_class}", style: "{item_style}",
            div {
                class: "card",
                style: "{card_style}",
                onpointerdown: move |e| {
                    e.prevent_default();

                    let coords = e.client_coordinates();
                    let x = coords.x as f32;
                    let y = coords.y as f32;

                    let issues = props.issues_for_layout.read();
                    let layout_ids: Vec<u32> = issues.iter().map(|i| i.id).collect();
                    let orig_idx = layout_ids.iter().position(|&i| i == id).unwrap_or(0);

                    // Anchor: nat_tops[orig_idx] == y exactly.
                    // All slots uniform — SLOT_SIZE is a CSS constant, not measured.
                    let nat_tops: Vec<f32> = (0..layout_ids.len())
                        .map(|i| y + (i as f32 - orig_idx as f32) * SLOT_SIZE)
                        .collect();

                    let mut ds = props.drag_state.write();
                    ds.reset();
                    ds.dragging_id = Some(id);
                    ds.is_dragging = false;
                    ds.start_x = x;
                    ds.start_y = y;
                    ds.cur_x = x;
                    ds.cur_y = y;
                    ds.nat_tops = nat_tops;
                    ds.layout_ids = layout_ids;
                    ds.orig_idx = orig_idx;
                    ds.cur_idx = orig_idx;

                    ds.scale_spring.set(1.0);
                    ds.scale_spring.target = 1.05;
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

fn compute_styles(id: u32, ds: &DragState) -> (String, String, String) {
    let is_dragging = ds.is_dragging && ds.dragging_id == Some(id);
    let is_settling = ds.settling_id == Some(id);

    let mut item_class = "item".to_string();
    if is_dragging { item_class.push_str(" dragging"); }
    if is_settling { item_class.push_str(" settling"); }

    let (item_style, card_style) = if is_dragging {
        let dy = ds.cur_y - ds.start_y;
        let dx = (ds.cur_x - ds.start_x) * 0.4;
        (
            format!("transform: translate3d(0,{}px,0); z-index: 500;", dy),
            format!("transform: translate3d({}px,0,0) scale3d({s},{s},1);", dx, s = ds.scale_spring.pos),
        )
    } else if is_settling {
        (
            format!("transform: translate3d(0,{}px,0); z-index: 400;", ds.y_return.pos),
            format!("transform: translate3d({}px,0,0) scale3d({s},{s},1);", ds.x_return.pos, s = ds.scale_spring.pos),
        )
    } else if let Some(spring) = ds.item_springs.get(&id) {
        if spring.pos.abs() > 0.1 {
            (format!("transform: translate3d(0,{}px,0);", spring.pos), String::new())
        } else {
            (String::new(), String::new())
        }
    } else {
        (String::new(), String::new())
    };

    (item_class, item_style, card_style)
}
