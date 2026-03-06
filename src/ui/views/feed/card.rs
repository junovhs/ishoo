use crate::model::{Issue, Status};
use crate::ui::views::physics::DragState;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct IssueCardProps {
    pub issue: Issue,
    pub expanded: bool,
    pub drag_state: Signal<DragState>,
    pub issues_for_layout: Signal<Vec<Issue>>,
    /// Real screen tops of all cards, measured by JS eval after each render.
    /// Sorted by top (ascending). Used for slot detection — no hardcoded heights.
    pub card_screen_tops: Signal<Vec<(u32, f32)>>,
    pub on_collapse_all: EventHandler<()>,
    pub on_status: EventHandler<(u32, String)>,
    pub on_resolution: EventHandler<(u32, String)>,
}

#[component]
pub fn IssueCard(mut props: IssueCardProps) -> Element {
    let i = &props.issue;
    let id = i.id;

    let (item_class, item_style, card_style) = compute_styles(id, &props.drag_state.read());

    rsx! {
        div { class: "{item_class}", style: "{item_style}",
            "data-card-id": "{id}",
            div {
                class: if props.expanded { "card active" } else { "card" },
                style: "{card_style}",
                div {
                    class: "card-hdr",
                    onpointerdown: move |e| {
                        e.prevent_default();
                        props.on_collapse_all.call(());

                        let coords = e.client_coordinates();
                        let x = coords.x as f32;
                        let y = coords.y as f32;

                        // Get layout from props directly — no JS needed
                        let issues = props.issues_for_layout.read();
                        let layout_ids: Vec<u32> = issues.iter().map(|i| i.id).collect();
                        let orig_idx = layout_ids.iter().position(|&i| i == id).unwrap_or(0);

                        // CSS: card header padding 16px top + 16px bottom + ~22px line-height
                        // + 1px border top + 1px border bottom + 9px margin-bottom = 65px/slot.
                        // Anchored so nat_tops[orig_idx] == start_y exactly, making slot
                        // detection independent of where within the card the pointer lands.
                        let slot_size = 65.0_f32;
                        let nat_tops: Vec<f32> = (0..layout_ids.len())
                            .map(|i| y + (i as f32 - orig_idx as f32) * slot_size)
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
                    span { class: "cid", "#{id}" }
                    span { class: "ctitle", "{i.title}" }
                    span { class: "badge b-{i.status.css_class()}", "{i.status.label()}" }
                }

                if props.expanded {
                    {render_card_body(i, props.on_status, props.on_resolution)}
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

    // Cards never rotate. Outer wrapper handles vertical translation only.
    // Inner card handles horizontal float + scale.
    // translate3d forces GPU 3D compositing path unconditionally.
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

fn render_card_body(
    i: &Issue,
    on_status: EventHandler<(u32, String)>,
    on_resolution: EventHandler<(u32, String)>,
) -> Element {
    let id = i.id;
    rsx! {
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
                            oninput: move |e| on_resolution.call((id, e.value())),
                        }
                    }
                }
                div { class: "detail-r",
                    div { class: "fgroup",
                        label { class: "flbl", "Status" }
                        select {
                            class: "sel",
                            value: "{i.status.label()}",
                            onchange: move |e| on_status.call((id, e.value())),
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
