mod card;

use super::physics::DragState;
use crate::model::Issue;
use card::IssueCard;
use dioxus::prelude::*;
use std::collections::HashMap;

const HEADER_HEIGHT: f32 = 73.0;
const CARD_HEIGHT: f32 = 62.0;
const DRAG_THRESHOLD: f32 = 5.0;

#[derive(Clone, PartialEq, Props)]
pub struct FeedViewProps {
    pub issues: Vec<Issue>,
    pub active_id: Option<u32>,
    pub on_toggle: EventHandler<u32>,
    pub on_status: EventHandler<(u32, String)>,
    pub on_resolution: EventHandler<(u32, String)>,
    pub on_collapse_all: EventHandler<()>,
    pub on_reorder: EventHandler<(u32, u32, bool)>,
}

#[component]
pub fn FeedView(props: FeedViewProps) -> Element {
    let mut drag_state = use_signal(DragState::default);
    let issues_clone = props.issues.clone();
    let mut issues_for_layout = use_signal(|| issues_clone.clone());

    // Keep layout data in sync
    use_effect(move || {
        issues_for_layout.set(issues_clone.clone());
    });

    let (section_order, section_map) = group_by_section(&props.issues);

    spawn_physics_loop(drag_state);

    rsx! {
        div {
            class: "feed",
            onpointermove: move |e| {
                let mut ds = drag_state.write();
                if ds.dragging_id.is_some() {
                    let coords = e.client_coordinates();
                    ds.cur_x = coords.x as f32;
                    ds.cur_y = coords.y as f32;
                    if !ds.is_dragging && exceeds_threshold(&ds) {
                        ds.is_dragging = true;
                    }
                }
            },
            onpointerup: move |_| {
                let result = finalize_drag(&mut drag_state.write());
                if let Some((id, is_click, do_reorder, target_id, insert_after)) = result {
                    if is_click {
                        props.on_toggle.call(id);
                    } else if do_reorder {
                        props.on_reorder.call((id, target_id, insert_after));
                    }
                }
            },
            onpointercancel: move |_| {
                let mut ds = drag_state.write();
                ds.dragging_id = None;
                ds.is_dragging = false;
            },

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
                                    drag_state: drag_state,
                                    issues_for_layout: issues_for_layout,
                                    on_collapse_all: props.on_collapse_all,
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

fn group_by_section(issues: &[Issue]) -> (Vec<String>, HashMap<String, Vec<Issue>>) {
    let mut section_order: Vec<String> = Vec::new();
    let mut section_map: HashMap<String, Vec<Issue>> = HashMap::new();

    for issue in issues {
        if !section_map.contains_key(&issue.section) {
            section_order.push(issue.section.clone());
        }
        section_map
            .entry(issue.section.clone())
            .or_default()
            .push(issue.clone());
    }

    (section_order, section_map)
}

pub fn build_virtual_layout(issues: &[Issue]) -> (Vec<f32>, Vec<u32>) {
    let mut y_pos = 0.0;
    let mut nat_tops = Vec::new();
    let mut layout_ids = Vec::new();
    let mut current_section: Option<&str> = None;

    for issue in issues {
        if current_section != Some(&issue.section) {
            y_pos += HEADER_HEIGHT;
            current_section = Some(&issue.section);
        }
        nat_tops.push(y_pos);
        layout_ids.push(issue.id);
        y_pos += CARD_HEIGHT;
    }

    (nat_tops, layout_ids)
}

fn exceeds_threshold(ds: &DragState) -> bool {
    (ds.cur_x - ds.start_x).abs() > DRAG_THRESHOLD || (ds.cur_y - ds.start_y).abs() > DRAG_THRESHOLD
}

fn finalize_drag(ds: &mut DragState) -> Option<(u32, bool, bool, u32, bool)> {
    let id = ds.dragging_id?;

    let dy = ds.cur_y - ds.start_y;
    let dx = (ds.cur_x - ds.start_x) * 0.4;
    let is_click = !ds.is_dragging;

    let do_reorder = !is_click && ds.cur_idx != ds.orig_idx;
    let target_id = ds.layout_ids.get(ds.cur_idx).copied().unwrap_or(0);
    let insert_after = ds.cur_idx > ds.orig_idx;

    if !is_click {
        ds.x_return.pos = dx;
        ds.x_return.target = 0.0;
        ds.scale_spring.target = 1.0;

        if do_reorder {
            let old_top = *ds.nat_tops.get(ds.orig_idx).unwrap_or(&0.0);
            let new_top = *ds.nat_tops.get(ds.cur_idx).unwrap_or(&0.0);
            ds.y_return.pos = old_top + dy - new_top;
        } else {
            ds.y_return.pos = dy;
        }
        ds.y_return.target = 0.0;

        for spring in ds.item_springs.values_mut() {
            spring.target = 0.0;
        }

        ds.is_dragging = false;
        ds.settling_id = Some(id);
    }

    ds.dragging_id = None;
    Some((id, is_click, do_reorder, target_id, insert_after))
}

fn spawn_physics_loop(mut drag_state: Signal<DragState>) {
    use_coroutine(move |_rx: UnboundedReceiver<()>| async move {
        let mut last_time = tokio::time::Instant::now();
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(16)).await;
            let now = tokio::time::Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32();
            last_time = now;

            let is_active = drag_state.read().is_active();

            if is_active {
                let mut ds = drag_state.write();
                if ds.is_dragging && ds.dragging_id.is_some() {
                    ds.step_drag(dt);
                } else if ds.settling_id.is_some() {
                    ds.step_settle(dt);
                }
            }
        }
    });
}
