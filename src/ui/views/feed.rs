mod card;

use super::physics::{DragState, PendingReorder};
use crate::model::Issue;
use card::IssueCard;
use dioxus::prelude::*;
use std::collections::HashMap;

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
    let mut card_screen_tops: Signal<Vec<(u32, f32)>> = use_signal(Vec::new);

    // Keep layout signal in sync with props
    use_effect(move || {
        issues_for_layout.set(issues_clone.clone());
    });

    // Measure real card positions after render
    use_effect(move || {
        let _ = issues_for_layout.read();
        spawn(async move {
            let js = "Array.from(document.querySelectorAll('[data-card-id]')).map(el => [parseInt(el.dataset.cardId), el.getBoundingClientRect().top])";
            let ev = document::eval(js);
            if let Ok(val) = ev.await {
                if let Some(arr) = val.as_array() {
                    let tops: Vec<(u32, f32)> = arr
                        .iter()
                        .filter_map(|v| {
                            let pair = v.as_array()?;
                            let id = pair.first()?.as_f64()? as u32;
                            let top = pair.get(1)?.as_f64()? as f32;
                            Some((id, top))
                        })
                        .collect();
                    card_screen_tops.set(tops);
                }
            }
        });
    });

    let (section_order, section_map) = group_by_section(&props.issues);

    let on_reorder = props.on_reorder;

    // Physics loop — spawns ONCE per component mount via use_coroutine
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
                        ds.reset();
                        drop(ds);
                        props.on_toggle.call(id);
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
                                    card_screen_tops: card_screen_tops,
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

fn exceeds_threshold(ds: &DragState) -> bool {
    (ds.cur_x - ds.start_x).abs() > DRAG_THRESHOLD
        || (ds.cur_y - ds.start_y).abs() > DRAG_THRESHOLD
}
