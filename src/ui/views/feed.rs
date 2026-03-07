// neti:allow(LAW OF ATOMICITY)
mod card;

use crate::model::{Issue, Status};
use card::IssueCard;
use dioxus::prelude::*;

#[derive(Clone, Default, PartialEq)]
pub struct DragState {
    pub dragging_id: Option<u32>,
    pub start_idx: usize,
    pub hover_idx: usize,
    pub hover_y: f32,
    pub start_y: f32,
    pub start_virtual_y: f32,
    pub offset_y: f32,
    pub releasing: bool,
}

#[derive(Clone, PartialEq, Props)]
pub struct FeedViewProps {
    pub is_compact: bool,
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
    
    let slot_h = if props.is_compact { 40.0 } else { 63.0 };
    
    // Compute total absolute container height, plus the 200px scroll padding at the bottom
    // We add 1 slot_h worth of height for each of the 3 section headers
    let total_height = (issues_len as f32 * slot_h) + (3.0 * slot_h) + 200.0;
    
    // We need to track which sections are currently collapsed
    let mut collapsed = use_signal(std::collections::HashSet::<String>::new);

    rsx! {
        div {
            class: if props.is_compact { "feed compact" } else { "feed" },
            onpointermove: move |e| {
                let mut ds = drag_state.write();
                if ds.dragging_id.is_some() && !ds.releasing {
                    ds.offset_y = e.client_coordinates().y as f32 - ds.start_y;
                    
                    let logical_y = ds.start_virtual_y + ds.offset_y;
                    let sl = if props.is_compact { 40.0 } else { 63.0 };
                    
                    let sections = [
                        ("Active", "active", "var(--orange)"),
                        ("Backlog", "backlog", "var(--blue)"),
                        ("Done", "done", "var(--green)"),
                    ];
                    
                    let mut closest_idx = ds.start_idx;
                    let mut min_dist = f32::MAX;
                    let mut v_idx = 0;
                    
                    for (_label, key, _color) in sections {
                        let section_items: Vec<(usize, &Issue)> = props.issues.iter().enumerate()
                            .filter(|(_, i)| {
                                let sec = i.section.to_lowercase();
                                let is_done = sec.contains("done") || i.status == Status::Done || i.status == Status::Descoped;
                                let is_backlog = !is_done && sec.contains("backlog");
                                match key {
                                    "done" => is_done,
                                    "backlog" => is_backlog,
                                    "active" => !is_done && !is_backlog,
                                    _ => false,
                                }
                            })
                            .collect();
                            
                        if section_items.is_empty() { continue; }
                        v_idx += 1;
                        if !collapsed.read().contains(key) {
                            for (idx, _) in section_items {
                                let vy = v_idx as f32 * sl;
                                let dist = (vy - logical_y).abs();
                                if dist < min_dist {
                                    min_dist = dist;
                                    closest_idx = idx;
                                    ds.hover_y = vy;
                                }
                                v_idx += 1;
                            }
                        }
                    }
                    ds.hover_idx = closest_idx;
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
                        
                        // Critical logic: Wait 50ms for the workspace array mutation to propagate 
                        // down into the FeedView properties AND re-render visually before we kill the 
                        // hovering/settling transition CSS. This perfectly eliminates the origin snapback.
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
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
                    
                    // We must track the `virtual_idx` which is the actual vertical slot.
                    // Every section header takes up 1 slot. Cards take up 1 slot each.
                    let mut virtual_idx = 0;
                    
                    let mut array_reordered = false;
                    {
                        let ds_read = drag_state.read();
                        if let Some(drag_id) = ds_read.dragging_id {
                            if let Some(curr) = props.issues.iter().position(|i| i.id == drag_id) {
                                if curr != ds_read.start_idx {
                                    array_reordered = true;
                                }
                            }
                        }
                    }
                    
                    for (_label, key, _color) in sections {
                        // Find all issues belonging to this section, preserving their original index in `props.issues`
                        let _section_items: Vec<(usize, &Issue)> = props.issues.iter().enumerate()
                            .filter(|(_, i)| {
                                if key == "active" { i.status == Status::Open || i.status == Status::InProgress }
                                else if key == "done" { i.status == Status::Done || i.status == Status::Descoped }
                                else { i.status == Status::Open /* fallback, wait actually we should check the actual section string */ }
                            })
                            .collect();
                            
                        // Wait, the spike just groups by status. Let's do a strict pass based on `i.status`
                        let _section_items: Vec<(usize, &Issue)> = props.issues.iter().enumerate()
                            .filter(|(_, i)| {
                                match key {
                                    "active" => i.status == Status::InProgress || i.status == Status::Open,
                                    "done" => i.status == Status::Done || i.status == Status::Descoped,
                                    "backlog" => false, // We'll fix this, Issue struct doesn't have "backlog" status.
                                    _ => false
                                }
                            })
                            .collect();
                    }
                    
                    // Rewrite this logic properly:
                    // In `workspace.rs`, `Issue` has `pub section: String`. We should use that!
                    // Let's iterate the sections and pull out items where `i.section.to_lowercase() == key`
                    for (label, key, color) in sections {
                        let section_items: Vec<(usize, &Issue)> = props.issues.iter().enumerate()
                            .filter(|(_, i)| {
                                let sec = i.section.to_lowercase();
                                let is_done = sec.contains("done") || i.status == Status::Done;
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
                        let y_pos = virtual_idx as f32 * slot_h;
                        
                        let key_clone = key.to_string();
                        let count = section_items.len();
                        
                        // Push the header element into the container natively, taking up 1 slot.
                        // For the section class, we append " collapsed" if true, though the CSS 
                        // handles the chevron separately via the outer class. We'll just style the chevron directly.
                        elements.push(rsx! {
                            div {
                                key: "header-{key}",
                                class: "section-head",
                                style: "position: absolute; top: {y_pos}px; left: 0; right: 0; height: {slot_h}px;",
                                onclick: move |_| {
                                    let mut c = collapsed.write();
                                    if c.contains(&key_clone) {
                                        c.remove(&key_clone);
                                    } else {
                                        c.insert(key_clone.clone());
                                    }
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
                        
                        virtual_idx += 1;
                        
                        if !is_collapsed {
                            for (idx, issue) in section_items {
                                elements.push(rsx! {
                                    IssueCard {
                                        key: "card-{issue.id}",
                                        issue: issue.clone(),
                                        idx: idx,
                                        virtual_y: virtual_idx as f32 * slot_h, // We need to modify IssueCard to take `virtual_y`
                                        drag_state: drag_state,
                                        is_compact: props.is_compact,
                                        array_reordered: array_reordered,
                                    }
                                });
                                virtual_idx += 1;
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

    let color = match i.section.as_str() {
        "active" => "var(--orange)",
        "done" => "var(--green)",
        _ => "var(--blue)",
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
                        // Real tags would map here in the future
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