// neti:allow(LAW OF ATOMICITY)
mod card;

use crate::model::{split_issue_id, Issue, Status};
use crate::ui::components::LabelList;
use dioxus::prelude::*;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
struct HoverTarget {
    idx: usize,
    after: bool,
    y: f32,
}

pub(super) const DRAG_DEADZONE_PX: f32 = 8.0;
const HOVER_RECONCILE_STEP_PX: f32 = 4.0;

pub(crate) fn issue_instance_key(issue: &Issue) -> String {
    format!("{}::{}", issue.section.to_ascii_lowercase(), issue.id)
}

pub(super) fn apply_drag_deadzone(offset_y: f32) -> f32 {
    if offset_y.abs() < DRAG_DEADZONE_PX {
        0.0
    } else {
        offset_y - (offset_y.signum() * DRAG_DEADZONE_PX)
    }
}

fn drag_logical_y(start_virtual_y: f32, offset_y: f32) -> f32 {
    start_virtual_y + apply_drag_deadzone(offset_y)
}

fn compute_hover_target(
    start_idx: usize,
    start_virtual_y: f32,
    offset_y: f32,
    candidates: &[(usize, f32)],
) -> HoverTarget {
    let logical_y = drag_logical_y(start_virtual_y, offset_y);
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
pub struct DragPresence {
    pub dragging_key: Option<String>,
    pub releasing: bool,
}

#[derive(Clone, Default, PartialEq)]
pub struct DragState {
    pub start_idx: usize,
    pub hover_idx: usize,
    pub hover_after: bool,
    pub hover_y: f32,
    pub start_y: f32,
    pub start_virtual_y: f32,
    pub last_layout_probe_y: f32,
}

#[derive(Clone, Default, PartialEq)]
pub struct RecentDropState {
    pub key: Option<String>,
    pub release_x: f32,
    pub release_y: f32,
    pub hover_armed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DragDebugState {
    pub active: bool,
    pub issue_key: Option<String>,
    pub elapsed_ms: f64,
    pub pointer_events: u32,
    pub hover_updates: u32,
    pub avg_pointer_gap_ms: f64,
    pub max_pointer_gap_ms: f64,
    pub avg_frame_ms: f64,
    pub max_frame_ms: f64,
    pub frame_samples: u32,
    pub logical_y: f32,
    pub live_y: f32,
    pub snapped_y: f32,
    pub hover_y: f32,
}

impl Default for DragDebugState {
    fn default() -> Self {
        Self {
            active: false,
            issue_key: None,
            elapsed_ms: 0.0,
            pointer_events: 0,
            hover_updates: 0,
            avg_pointer_gap_ms: 0.0,
            max_pointer_gap_ms: 0.0,
            avg_frame_ms: 0.0,
            max_frame_ms: 0.0,
            frame_samples: 0,
            logical_y: 0.0,
            live_y: 0.0,
            snapped_y: 0.0,
            hover_y: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
struct DragMetrics {
    issue_key: Option<String>,
    started_at: Instant,
    last_pointer_at: Option<Instant>,
    last_frame_at: Option<Instant>,
    last_log_at: Instant,
    pointer_events: u32,
    hover_updates: u32,
    pointer_gap_total_ms: f64,
    max_pointer_gap_ms: f64,
    frame_samples: u32,
    frame_total_ms: f64,
    max_frame_ms: f64,
}

impl Default for DragMetrics {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            issue_key: None,
            started_at: now,
            last_pointer_at: None,
            last_frame_at: None,
            last_log_at: now,
            pointer_events: 0,
            hover_updates: 0,
            pointer_gap_total_ms: 0.0,
            max_pointer_gap_ms: 0.0,
            frame_samples: 0,
            frame_total_ms: 0.0,
            max_frame_ms: 0.0,
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct FeedViewProps {
    pub is_compact: bool,
    pub zoom: f32,
    pub issues: Vec<Issue>,
    pub on_status: EventHandler<(String, String)>,
    pub on_resolution: EventHandler<(String, String)>,
    pub on_labels: EventHandler<(String, String)>,
    pub on_reorder: EventHandler<(String, Option<String>, bool, Option<String>)>,
    pub on_section_toggle: EventHandler<()>,
}

fn reverse_links(issues: &[Issue]) -> std::collections::HashMap<String, Vec<String>> {
    let mut reverse = std::collections::HashMap::<String, Vec<String>>::new();
    for issue in issues {
        for target in &issue.links {
            reverse
                .entry(target.clone())
                .or_default()
                .push(issue.id.clone());
        }
    }
    reverse
}

fn modal_neighbor_id(issues: &[Issue], current_id: &str, delta: isize) -> Option<String> {
    let current_idx = issues
        .iter()
        .position(|issue| issue_instance_key(issue) == current_id)?;
    let next_idx = current_idx as isize + delta;
    if next_idx < 0 {
        return None;
    }
    issues.get(next_idx as usize).map(issue_instance_key)
}

#[component]
pub fn FeedView(props: FeedViewProps) -> Element {
    let mut drag_presence = use_signal(DragPresence::default);
    let mut drag_state = use_signal(DragState::default);
    let mut drag_offset = use_signal(|| 0.0f32);
    let mut recent_drop = use_signal(RecentDropState::default);
    let mut drag_debug = use_signal(DragDebugState::default);
    let mut drag_metrics = use_signal(DragMetrics::default);
    let mut modal_id: Signal<Option<String>> = use_signal(|| None);
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
    let allow_link_hover = {
        let dp = drag_presence.read();
        let rd = recent_drop.read();
        dp.dragging_key.is_none() && !dp.releasing && (rd.key.is_none() || rd.hover_armed)
    };
    let mentioned_by = reverse_links(&props.issues);
    let active_modal = modal_id().and_then(|instance_key| {
        props
            .issues
            .iter()
            .find(|issue| issue_instance_key(issue) == instance_key)
            .cloned()
            .map(|issue| {
                (
                    issue.clone(),
                    mentioned_by.get(&issue.id).cloned().unwrap_or_default(),
                    modal_neighbor_id(&props.issues, &instance_key, -1),
                    modal_neighbor_id(&props.issues, &instance_key, 1),
                )
            })
    });
    let dragged_issue = drag_presence().dragging_key.as_ref().and_then(|drag_key| {
        props
            .issues
            .iter()
            .find(|issue| issue_instance_key(issue) == *drag_key)
            .cloned()
            .map(|issue| {
                let incoming_links = mentioned_by.get(&issue.id).cloned().unwrap_or_default();
                (issue, incoming_links)
            })
    });
    let drag_debug_state = drag_debug();
    let debug_issue_key = drag_debug_state
        .issue_key
        .clone()
        .unwrap_or_else(|| "-".to_string());

    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(16)).await;

                let dp = drag_presence.read().clone();
                if dp.dragging_key.is_none() {
                    continue;
                }

                let now = Instant::now();
                let ds = drag_state.read().clone();
                let offset = drag_offset();
                let logical_y = drag_logical_y(ds.start_virtual_y, offset);
                let live_y = if dp.releasing {
                    ds.hover_y
                } else {
                    ds.start_virtual_y + apply_drag_deadzone(offset)
                };
                let snapped_y = live_y.round();

                {
                    let mut metrics = drag_metrics.write();
                    if metrics.issue_key.is_none() {
                        metrics.issue_key = dp.dragging_key.clone();
                        metrics.started_at = now;
                        metrics.last_frame_at = Some(now);
                        metrics.last_log_at = now;
                    } else if let Some(last_frame_at) = metrics.last_frame_at.replace(now) {
                        let frame_gap_ms = now.duration_since(last_frame_at).as_secs_f64() * 1000.0;
                        metrics.frame_samples += 1;
                        metrics.frame_total_ms += frame_gap_ms;
                        metrics.max_frame_ms = metrics.max_frame_ms.max(frame_gap_ms);
                    }

                    let elapsed_ms = now.duration_since(metrics.started_at).as_secs_f64() * 1000.0;
                    let avg_pointer_gap_ms = if metrics.pointer_events > 1 {
                        metrics.pointer_gap_total_ms / (metrics.pointer_events - 1) as f64
                    } else {
                        0.0
                    };
                    let avg_frame_ms = if metrics.frame_samples > 0 {
                        metrics.frame_total_ms / metrics.frame_samples as f64
                    } else {
                        0.0
                    };

                    drag_debug.set(DragDebugState {
                        active: true,
                        issue_key: dp.dragging_key.clone(),
                        elapsed_ms,
                        pointer_events: metrics.pointer_events,
                        hover_updates: metrics.hover_updates,
                        avg_pointer_gap_ms,
                        max_pointer_gap_ms: metrics.max_pointer_gap_ms,
                        avg_frame_ms,
                        max_frame_ms: metrics.max_frame_ms,
                        frame_samples: metrics.frame_samples,
                        logical_y,
                        live_y,
                        snapped_y,
                        hover_y: ds.hover_y,
                    });

                    if now.duration_since(metrics.last_log_at).as_millis() >= 250 {
                        metrics.last_log_at = now;
                        println!(
                            "[Drag Metrics] key={} t={:.0}ms ptr={} avg_ptr={:.1}ms max_ptr={:.1}ms frames={} avg_frame={:.1}ms max_frame={:.1}ms hover={} logical={:.1} live={:.1} snapped={:.1} hover_y={:.1}",
                            dp.dragging_key.as_deref().unwrap_or("-"),
                            elapsed_ms,
                            metrics.pointer_events,
                            avg_pointer_gap_ms,
                            metrics.max_pointer_gap_ms,
                            metrics.frame_samples,
                            avg_frame_ms,
                            metrics.max_frame_ms,
                            metrics.hover_updates,
                            logical_y,
                            live_y,
                            snapped_y,
                            ds.hover_y
                        );
                    }
                }
            }
        });
    });

    rsx! {
        div {
            class: if props.is_compact { "feed compact" } else { "feed" },
            onpointermove: move |e| {
                {
                    let rd = recent_drop.read();
                    if let Some(_key) = &rd.key {
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
                let dp = drag_presence.read();
                if dp.dragging_key.is_none() || dp.releasing {
                    return;
                }
                let ds_read = drag_state.read();
                let next_offset = (e.client_coordinates().y as f32 - ds_read.start_y) / props.zoom;
                let now = Instant::now();
                {
                    let mut metrics = drag_metrics.write();
                    if let Some(last_pointer_at) = metrics.last_pointer_at.replace(now) {
                        let pointer_gap_ms =
                            now.duration_since(last_pointer_at).as_secs_f64() * 1000.0;
                        metrics.pointer_gap_total_ms += pointer_gap_ms;
                        metrics.max_pointer_gap_ms =
                            metrics.max_pointer_gap_ms.max(pointer_gap_ms);
                    }
                    metrics.pointer_events += 1;
                }
                drop(dp);
                drop(ds_read);
                drag_offset.set(next_offset);

                let ds = drag_state.read();
                let logical_y = drag_logical_y(ds.start_virtual_y, next_offset);
                if (logical_y - ds.last_layout_probe_y).abs() < HOVER_RECONCILE_STEP_PX {
                    return;
                }

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
                    next_offset,
                    &insertion_slots.iter().map(|(idx, y, _)| (*idx, *y)).collect::<Vec<_>>(),
                );

                let hover_after = insertion_slots
                    .iter()
                    .find(|(idx, y, _)| *idx == hover.idx && (*y - hover.y).abs() < f32::EPSILON)
                    .map(|(_, _, after)| *after)
                    .unwrap_or(false);
                let needs_layout_update =
                    ds.hover_idx != hover.idx || ds.hover_y != hover.y || ds.hover_after != hover_after;
                drop(ds);

                if needs_layout_update {
                    let mut ds = drag_state.write();
                    ds.hover_idx = hover.idx;
                    ds.hover_y = hover.y;
                    ds.hover_after = hover_after;
                    ds.last_layout_probe_y = logical_y;
                    drag_metrics.write().hover_updates += 1;
                } else {
                    drag_state.write().last_layout_probe_y = logical_y;
                }
            },
            onpointerup: move |e| {
                let mut dp = drag_presence.write();
                if let Some(drag_key) = dp.dragging_key.clone() {
                    if dp.releasing { return; }
                    let ds = drag_state.read();

                    // If it was just a tiny click/movement, clear drag and open the issue modal
                    if drag_offset() .abs() < 5.0 && ds.start_idx == ds.hover_idx {
                        dp.dragging_key = None;
                        drag_offset.set(0.0);
                        drop(ds);
                        drop(dp);
                        modal_id.set(Some(drag_key));
                        return;
                    }

                    // Otherwise, trigger the snap-to-socket animation on the card
                    dp.releasing = true;
                    let start_idx = ds.start_idx;
                    let hover_idx = ds.hover_idx;
                    let hover_after = ds.hover_after;
                    recent_drop.set(RecentDropState {
                        key: Some(drag_key.clone()),
                        release_x: e.client_coordinates().x as f32,
                        release_y: e.client_coordinates().y as f32,
                        hover_armed: false,
                    });
                    // MUST drop the write lock before we can copy the drag_state signal
                    // into the spawned future
                    drop(ds);
                    drop(dp);
                    let issues_clone = issues_for_up.clone();
                    let on_reorder_clone = on_reorder;
                    let mut ds_signal = drag_state;
                    let mut dp_signal = drag_presence;
                    let mut drag_offset_signal = drag_offset;
                    let mut debug_signal = drag_debug;
                    let mut metrics_signal = drag_metrics;
                    let metrics_snapshot = drag_metrics.read().clone();

                    println!(
                        "[Drag Metrics] release key={} ptr={} avg_ptr={:.1}ms max_ptr={:.1}ms frames={} avg_frame={:.1}ms max_frame={:.1}ms hover={}",
                        drag_key,
                        metrics_snapshot.pointer_events,
                        if metrics_snapshot.pointer_events > 1 {
                            metrics_snapshot.pointer_gap_total_ms
                                / (metrics_snapshot.pointer_events - 1) as f64
                        } else {
                            0.0
                        },
                        metrics_snapshot.max_pointer_gap_ms,
                        metrics_snapshot.frame_samples,
                        if metrics_snapshot.frame_samples > 0 {
                            metrics_snapshot.frame_total_ms / metrics_snapshot.frame_samples as f64
                        } else {
                            0.0
                        },
                        metrics_snapshot.max_frame_ms,
                        metrics_snapshot.hover_updates
                    );

                    spawn(async move {
                        // Wait exactly the length of the 400ms CSS transition
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                        if start_idx != hover_idx {
                            if let Some(target) = issues_clone.get(hover_idx) {
                                let target_key = issue_instance_key(target);
                                on_reorder_clone.call((drag_key, Some(target_key), hover_after, None));
                            }
                        }
                        drag_offset_signal.set(0.0);
                        ds_signal.set(DragState::default());
                        dp_signal.set(DragPresence::default());
                        debug_signal.set(DragDebugState::default());
                        metrics_signal.set(DragMetrics::default());
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
                let dp = drag_presence.read();
                if dp.dragging_key.is_some() && !dp.releasing {
                    drop(dp);
                    drag_offset.set(0.0);
                    drag_state.set(DragState::default());
                    drag_presence.set(DragPresence::default());
                    drag_debug.set(DragDebugState::default());
                    drag_metrics.set(DragMetrics::default());
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
                            let issue_key = issue_instance_key(issue);
                            let target_y = if is_collapsed { current_y - 45.0 } else { current_y };

                            if !is_collapsed {
                                elements.push(rsx! {
                                    div { key: "spacer-{issue_key}", style: "height: {slot_h}px; width: 100%;" }
                                });
                            }

                            elements.push(rsx! {
                                card::IssueCard {
                                    key: "card-{issue_key}",
                                    issue: issue.clone(),
                                    issue_key: issue_key,
                                    incoming_links: mentioned_by.get(&issue.id).cloned().unwrap_or_default(),
                                    idx: idx,
                                    virtual_y: target_y,
                                    drag_presence: drag_presence,
                                    drag_state: drag_state,
                                    drag_offset: drag_offset,
                                    recent_drop: recent_drop,
                                    allow_link_hover: allow_link_hover,
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

                if let Some((issue, incoming_links)) = dragged_issue {
                    card::DragOverlay {
                        key: "drag-overlay-{issue_instance_key(&issue)}",
                        issue: issue.clone(),
                        issue_key: issue_instance_key(&issue),
                        incoming_links: incoming_links,
                        drag_presence: drag_presence,
                        drag_state: drag_state,
                        drag_offset: drag_offset,
                        is_compact: props.is_compact,
                    }
                }

                if drag_debug_state.active {
                    div {
                        class: "drag-debug-hud",
                        div { class: "drag-debug-hud__title", "drag telemetry" }
                        div { class: "drag-debug-hud__row", "issue {debug_issue_key}" }
                        div { class: "drag-debug-hud__row", "t {drag_debug_state.elapsed_ms.round()}ms  ptr {drag_debug_state.pointer_events}  hover {drag_debug_state.hover_updates}" }
                        div { class: "drag-debug-hud__row", "ptr avg {drag_debug_state.avg_pointer_gap_ms:.1}ms  max {drag_debug_state.max_pointer_gap_ms:.1}ms" }
                        div { class: "drag-debug-hud__row", "frame avg {drag_debug_state.avg_frame_ms:.1}ms  max {drag_debug_state.max_frame_ms:.1}ms" }
                        div { class: "drag-debug-hud__row", "logical {drag_debug_state.logical_y:.1}  live {drag_debug_state.live_y:.1}" }
                        div { class: "drag-debug-hud__row", "snapped {drag_debug_state.snapped_y:.1}  slot {drag_debug_state.hover_y:.1}" }
                    }
                }
            }
        }

        if let Some((issue, incoming_links, prev_id, next_id)) = active_modal {
                IssueModal {
                    issue: issue,
                    incoming_links: incoming_links,
                    prev_id: prev_id.clone(),
                    next_id: next_id.clone(),
                    on_close: move |_| modal_id.set(None),
                    on_prev: move |_| {
                        if let Some(prev_id) = prev_id.clone() {
                            modal_id.set(Some(prev_id));
                        }
                    },
                    on_next: move |_| {
                        if let Some(next_id) = next_id.clone() {
                            modal_id.set(Some(next_id));
                        }
                    },
                    on_status: props.on_status,
                    on_resolution: props.on_resolution,
                    on_labels: props.on_labels,
                }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct IssueModalProps {
    issue: Issue,
    incoming_links: Vec<String>,
    prev_id: Option<String>,
    next_id: Option<String>,
    on_close: EventHandler<()>,
    on_prev: EventHandler<()>,
    on_next: EventHandler<()>,
    on_status: EventHandler<(String, String)>,
    on_resolution: EventHandler<(String, String)>,
    on_labels: EventHandler<(String, String)>,
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
    let id = i.id.clone();
    let mut labels_input = use_signal(|| i.labels.join(", "));
    let modal_dom_id = format!("issue-modal-{}", i.id);
    let (id_category, id_number) = split_issue_id(&id);
    let labels_id = id.clone();
    let resolution_id = id.clone();
    let status_id = id.clone();

    let section = i.section.to_ascii_lowercase();
    let color =
        if section.contains("done") || i.status == Status::Done || i.status == Status::Descoped {
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

    use_effect({
        let modal_dom_id = modal_dom_id.clone();
        move || {
            let modal_dom_id = modal_dom_id.clone();
            spawn(async move {
                let script = format!(
                    "const el = document.getElementById({modal_dom_id:?}); if (el) el.focus();"
                );
                let _ = document::eval(&script);
            });
        }
    });

    rsx! {
        div {
            class: "modal-overlay open", // We use Dioxus conditional rendering so it's always open when mounted
            onclick: move |_| props.on_close.call(()),
            div {
                id: "{modal_dom_id}",
                class: "modal",
                tabindex: 0,
                onkeydown: move |e| match e.key() {
                    Key::ArrowUp if props.prev_id.is_some() => props.on_prev.call(()),
                    Key::ArrowDown if props.next_id.is_some() => props.on_next.call(()),
                    Key::Escape => props.on_close.call(()),
                    _ => {}
                },
                onclick: move |e| e.stop_propagation(),
                div { class: "m-accent", style: "background:{color}" }
                div { class: "m-head",
                    div {
                        div { class: "m-id", "{id_category}-" }
                        div { class: "m-id-num", "{id_number}" }
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
                        LabelList { labels: i.labels.clone() }
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
                if !i.links.is_empty() || !props.incoming_links.is_empty() {
                    div { class: "m-links",
                        hr { class: "m-divider", style: "margin:12px 0;" }
                        if !i.links.is_empty() {
                            div { class: "m-link-label", "Mentions" }
                            for d in &i.links {
                                span { class: "m-link-item", "↗ {d}" }
                            }
                        }
                        if !props.incoming_links.is_empty() {
                            div { class: "m-link-label", "Mentioned By" }
                            for d in &props.incoming_links {
                                span { class: "m-link-item", "↙ {d}" }
                            }
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
                                props.on_labels.call((labels_id.clone(), e.value()));
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
                            oninput: move |e| props.on_resolution.call((resolution_id.clone(), e.value())),
                        }
                    }
                    div { style: "margin-top: 16px;",
                        div { class: "m-body-label", "Update Status" }
                        select {
                            class: "sel",
                            value: "{i.status.label()}",
                            onchange: move |e| props.on_status.call((status_id.clone(), e.value())),
                            option { value: "OPEN", selected: i.status == Status::Open, "Open" }
                            option { value: "IN PROGRESS", selected: i.status == Status::InProgress, "In Progress" }
                            option { value: "DONE", selected: i.status == Status::Done, "Done" }
                            option { value: "DESCOPED", selected: i.status == Status::Descoped, "Descoped" }
                        }
                    }
                }
                div { class: "m-nav",
                    span {
                        if props.prev_id.is_some() || props.next_id.is_some() {
                            kbd { "↑" } kbd { "↓" } " prev / next"
                        } else {
                            kbd { "↑" } kbd { "↓" } " no neighbors"
                        }
                    }
                    span { kbd { "Esc" } " close" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_hover_target, modal_neighbor_id, reverse_links, HoverTarget};
    use crate::model::{Issue, Status};

    fn make_issue(id: &str) -> Issue {
        Issue {
            id: id.to_string(),
            title: format!("Issue {id}"),
            status: Status::Open,
            files: vec![],
            labels: vec![],
            links: vec![],
            description: String::new(),
            resolution: String::new(),
            section: "ACTIVE Issues".to_string(),
            depends_on: vec![],
        }
    }

    fn make_issue_in_section(id: &str, section: &str) -> Issue {
        let mut issue = make_issue(id);
        issue.section = section.to_string();
        issue
    }

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

    #[test]
    fn modal_neighbor_id_tracks_previous_and_next_in_filtered_order() {
        let issues = vec![
            make_issue("BUG-10"),
            make_issue("BUG-20"),
            make_issue("BUG-30"),
        ];
        assert_eq!(
            modal_neighbor_id(&issues, "active issues::BUG-20", -1),
            Some("active issues::BUG-10".to_string())
        );
        assert_eq!(
            modal_neighbor_id(&issues, "active issues::BUG-20", 1),
            Some("active issues::BUG-30".to_string())
        );
    }

    #[test]
    fn modal_neighbor_id_stops_at_list_edges() {
        let issues = vec![make_issue("BUG-10"), make_issue("BUG-20")];
        assert_eq!(
            modal_neighbor_id(&issues, "active issues::BUG-10", -1),
            None
        );
        assert_eq!(modal_neighbor_id(&issues, "active issues::BUG-20", 1), None);
    }

    #[test]
    fn issue_instance_key_disambiguates_duplicate_visible_ids_across_sections() {
        let active = make_issue_in_section("132", "ACTIVE Issues");
        let done = make_issue_in_section("132", "DONE Issues");

        assert_ne!(
            super::issue_instance_key(&active),
            super::issue_instance_key(&done)
        );
    }

    #[test]
    fn reverse_links_collects_incoming_mentions() {
        let mut first = make_issue("BUG-10");
        first.links = vec!["BUG-30".to_string()];
        let mut second = make_issue("BUG-20");
        second.links = vec!["BUG-30".to_string()];
        let reverse = reverse_links(&[first, second, make_issue("BUG-30")]);
        assert_eq!(
            reverse.get("BUG-30"),
            Some(&vec!["BUG-10".to_string(), "BUG-20".to_string()])
        );
    }
}
