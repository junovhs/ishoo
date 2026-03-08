// neti:allow(LAW OF ATOMICITY)
use super::toast::{Toast, ToastContainer, ToastKind};
use super::welcome::WelcomeScreen;
use super::{components, get_workspace_path, views, View};
use crate::model::{issue_id_sort_key, reinit_workspace, workspace_exists, Issue, Stats, Status, Workspace};
use dioxus::document::eval;
use dioxus::prelude::*;
use notify::{recommended_watcher, Event as NotifyEvent, EventKind, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
struct AppState {
    issues: Signal<Vec<Issue>>,
    toasts: Signal<Vec<Toast>>,
    toast_id: Signal<u64>,
    is_compact: Signal<bool>,
    zoom: Signal<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FeedLens {
    MyOrder,
    NextUp,
    HotPath,
    QuickWins,
}

const STYLESHEET: &str = include_str!("../../assets/style.css");

#[component]
pub fn App() -> Element {
    let ws_path = get_workspace_path();
    let mut initialized = use_signal(|| workspace_exists(&ws_path));

    if !initialized() {
        return rsx! {
            style { "{STYLESHEET}" }
            WelcomeScreen { path: ws_path, on_init: move |_| initialized.set(true) }
        };
    }

    render_dashboard(ws_path)
}

fn render_dashboard(ws_path: std::path::PathBuf) -> Element {
    let initial = Workspace::load(&ws_path).unwrap_or_else(|_| Workspace {
        root: ws_path.clone(),
        issues: vec![],
    });

    let mut issues = use_signal(|| initial.issues);
    let search = use_signal(String::new);
    let active_label = use_signal(|| None::<String>);
    let active_lens = use_signal(|| FeedLens::MyOrder);
    let view = use_signal(|| View::Feed);
    let dirty = use_signal(|| false);
    let edit_epoch = use_signal(|| 0u64);
    let modal = use_signal(|| false);
    let reinit_modal = use_signal(|| false);
    let mut toasts = use_signal(Vec::<Toast>::new);
    let toast_id = use_signal(|| 0u64);
    let is_compact = use_signal(|| false);
    
    // ── Global Scroll physics ──────────────────────────────
    let physics = use_signal(super::scroll::ScrollPhysics::default);
    let animating = use_signal(|| false);

    let zoom = use_signal(|| {
        let p = ws_path.join(".ishoo/zoom");
        std::fs::read_to_string(&p).unwrap_or_else(|_| "1.0".to_string()).parse::<f32>().unwrap_or(1.0)
    });

    use_context_provider(|| edit_epoch);

    use_effect(move || {
        let z = zoom();
        let p = get_workspace_path().join(".ishoo/zoom");
        let _ = std::fs::write(&p, z.to_string());
    });

    let state = AppState {
        issues,
        toasts,
        toast_id,
        is_compact,
        zoom,
    };

    let _watch = {
        let watch_path = ws_path.clone();
        use_coroutine(move |_rx: UnboundedReceiver<()>| {
            let path = watch_path.clone();
            async move {
                let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
                let mut watcher = match recommended_watcher(move |result: notify::Result<NotifyEvent>| {
                    if let Ok(event) = result {
                        if should_reload_for_event(&event) {
                            let _ = event_tx.send(());
                        }
                    }
                }) {
                    Ok(watcher) => watcher,
                    Err(error) => {
                        eprintln!("Failed to create file watcher: {error}");
                        return;
                    }
                };

                if let Err(error) = watcher.watch(&path, RecursiveMode::NonRecursive) {
                    eprintln!("Failed to watch {}: {error}", path.display());
                    return;
                }

                while event_rx.recv().await.is_some() {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    while event_rx.try_recv().is_ok() {}

                    let was_dirty = dirty();
                    let epoch_before = edit_epoch();
                    if let Ok(ws) = Workspace::load(&path) {
                        if can_apply_external_reload(was_dirty, dirty(), epoch_before, edit_epoch()) {
                            issues.set(ws.issues);
                        }
                    }
                }
            }
        })
    };

    let stats = compute_stats(&(state.issues)());
    let sections = section_counts(&(state.issues)());
    let available_labels = collect_labels(&(state.issues)());
    let filtered = apply_feed_lens(
        &(state.issues)(),
        filter_issues(&(state.issues)(), &search(), active_label().as_deref()),
        active_lens(),
    );


    rsx! {
        style { "{STYLESHEET}" }
        
        ToastContainer {
            toasts: toasts(),
            on_dismiss: move |id| { toasts.write().retain(|t| t.id != id); }
        }

        if modal() { NewIssueModal { modal: modal, state: state } }
        if reinit_modal() { ReinitModal { modal: reinit_modal, state: state } }

        div { class: "app",
            {render_sidebar(view, stats.clone(), sections, dirty, modal, reinit_modal, state)}
            main { class: "mn",
                {render_topbar(search, active_label, active_lens, available_labels, state, physics, animating)}
                {render_content(view, filtered, dirty, state, physics, animating)}
            }
        }
    }
}

fn add_toast(state: AppState, msg: String, kind: ToastKind) {
    let id = (state.toast_id)();
    state.toast_id.clone().set(id + 1);
    state.toasts.clone().write().push(Toast {
        id,
        message: msg,
        kind,
    });
    let toasts = state.toasts;
    spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        toasts.clone().write().retain(|t| t.id != id);
    });
}

fn save_workspace(state: AppState, msg: &str) {
    let ws = Workspace {
        root: get_workspace_path(),
        issues: (state.issues)(),
    };
    match ws.save() {
        Ok(()) => {
            if !msg.is_empty() {
                add_toast(state, msg.to_string(), ToastKind::Success);
            }
        },
        Err(e) => add_toast(state, format!("Save failed: {e}"), ToastKind::Error),
    }
}

fn bump_edit_epoch(mut edit_epoch: Signal<u64>) {
    edit_epoch.set(edit_epoch().wrapping_add(1));
}

fn can_apply_external_reload(
    was_dirty: bool,
    is_dirty_now: bool,
    epoch_before: u64,
    epoch_after: u64,
) -> bool {
    !was_dirty && !is_dirty_now && epoch_before == epoch_after
}

fn should_reload_for_event(event: &NotifyEvent) -> bool {
    !matches!(event.kind, EventKind::Access(_))
}

#[component]
fn NewIssueModal(mut modal: Signal<bool>, state: AppState) -> Element {
    let mut title = use_signal(String::new);
    let mut category = use_signal(|| "ISS".to_string());
    let mut status = use_signal(|| "OPEN".to_string());
    let mut labels = use_signal(String::new);
    let mut issues = state.issues;
    let edit_epoch = use_context::<Signal<u64>>();

    rsx! {
        div { class: "modal-overlay", onclick: move |_| modal.set(false),
            div { class: "modal", onclick: move |e| e.stop_propagation(),
                div { class: "m-head",
                    div { class: "m-title", "New Issue" }
                    button { class: "m-close", onclick: move |_| modal.set(false), "×" }
                }
                div { class: "m-divider" }
                div { class: "m-body",
                    div { class: "fgroup",
                        label { class: "flbl", "Title" }
                        input {
                            class: "modal-input", placeholder: "What needs to be done?",
                            value: "{title}", oninput: move |e| title.set(e.value()),
                        }
                    }
                    div { class: "fgroup",
                        label { class: "flbl", "Category" }
                        input {
                            class: "modal-input", placeholder: "BUG",
                            value: "{category}", oninput: move |e| category.set(e.value()),
                        }
                    }
                    div { class: "fgroup",
                        label { class: "flbl", "Status" }
                        select {
                            class: "sel", value: "{status}", onchange: move |e| status.set(e.value()),
                            option { value: "OPEN", "Open" }
                            option { value: "IN PROGRESS", "In Progress" }
                        }
                    }
                    div { class: "fgroup",
                        label { class: "flbl", "Labels" }
                        input {
                            class: "modal-input", placeholder: "core, frontend, ux",
                            value: "{labels}", oninput: move |e| labels.set(e.value()),
                        }
                    }
                }
                div { class: "modal-footer",
                    button { class: "btn-secondary", onclick: move |_| modal.set(false), "Cancel" }
                    button {
                        class: "btn-primary", disabled: title().trim().is_empty(),
                        onclick: move |_| {
                            let t = title().trim().to_string();
                            if !t.is_empty() {
                                let parsed_labels = parse_label_input(&labels());
                                let ws = Workspace {
                                    root: get_workspace_path(),
                                    issues: issues(),
                                };
                                let id = ws.allocate_issue_id(&category()).unwrap_or_else(|_| "ISS-01".to_string());
                                let issue = Issue {
                                    id, title: t.clone(), status: Status::from_str(&status()),
                                    files: vec![], labels: parsed_labels, links: vec![], description: String::new(), resolution: String::new(),
                                    section: "ACTIVE Issues".to_string(), depends_on: vec![],
                                };
                                bump_edit_epoch(edit_epoch);
                                issues.write().insert(0, issue.clone());
                                modal.set(false);
                                save_workspace(state, &format!("Created #{} {}", issue.id, t));
                            }
                        },
                        "Create"
                    }
                }
            }
        }
    }
}

#[component]
fn ReinitModal(mut modal: Signal<bool>, state: AppState) -> Element {
    let mut confirm_text = use_signal(String::new);
    let confirmed = confirm_text().trim().to_lowercase() == "erase my issues";
    let mut issues = state.issues;
    let edit_epoch = use_context::<Signal<u64>>();

    rsx! {
        div { class: "modal-overlay", onclick: move |_| modal.set(false),
            div { class: "modal modal-danger", onclick: move |e| e.stop_propagation(),
                div { class: "m-head",
                    div { class: "m-title", "⚠️ Reinitialize Tracker" }
                    button { class: "m-close", onclick: move |_| modal.set(false), "×" }
                }
                div { class: "m-divider" }
                div { class: "m-body",
                    p { class: "warning-text", "This will permanently delete all issues." }
                    p { class: "hint-text", "💡 Commit to git first so you have history." }
                    div { class: "fgroup",
                        label { class: "flbl", "Type \"erase my issues\" to confirm" }
                        input {
                            class: "modal-input", placeholder: "erase my issues",
                            value: "{confirm_text}", oninput: move |e| confirm_text.set(e.value()),
                        }
                    }
                }
                div { class: "modal-footer",
                    button { class: "btn-secondary", onclick: move |_| modal.set(false), "Cancel" }
                    button {
                        class: "btn-danger", disabled: !confirmed,
                        onclick: move |_| {
                            if confirmed {
                                match reinit_workspace(&get_workspace_path()) {
                                    Ok(()) => {
                                        bump_edit_epoch(edit_epoch);
                                        issues.set(vec![]);
                                        modal.set(false);
                                        add_toast(state, "Reinitialized".to_string(), ToastKind::Success);
                                    }
                                    Err(e) => add_toast(state, format!("Failed: {e}"), ToastKind::Error),
                                }
                            }
                        },
                        "Erase Everything"
                    }
                }
            }
        }
    }
}

fn render_sidebar(
    mut view: Signal<View>,
    stats: Stats,
    sections: Vec<(String, usize)>,
    mut dirty: Signal<bool>,
    mut modal: Signal<bool>,
    mut reinit_modal: Signal<bool>,
    state: AppState,
) -> Element {
    rsx! {
        aside { class: "sb",
            div { class: "logo",
                div { class: "logo-d" }
                em { "Ishoo" }
                button { 
                    class: "dm-toggle", 
                    title: "Toggle dark mode",
                    onclick: move |_| {
                        // We use a small JS eval to toggle the class on the html element
                        let _ = eval("document.documentElement.classList.toggle('dark'); 
                                      let btn = document.getElementById('dm-toggle-btn');
                                      if (document.documentElement.classList.contains('dark')) {
                                          btn.innerHTML = '☀';
                                      } else {
                                          btn.innerHTML = '☽';
                                      }");
                    },
                    id: "dm-toggle-btn",
                    "☽"
                }
            }
            nav { class: "vl",
                div { class: "sl", "Views" }
                components::NavBtn { label: "Feed", active: view() == View::Feed, onclick: move |_| view.set(View::Feed) }
                components::NavBtn { label: "Board", active: view() == View::Board, onclick: move |_| view.set(View::Board) }
                components::NavBtn { label: "Heatmap", active: view() == View::Heatmap, onclick: move |_| view.set(View::Heatmap) }
                components::NavBtn { label: "Graph", active: view() == View::Graph, onclick: move |_| view.set(View::Graph) }
                components::NavBtn { label: "Timeline", active: view() == View::Timeline, onclick: move |_| view.set(View::Timeline) }
            }
            div { class: "vl", style: "margin-top:auto;",
                div { class: "sl", "Breakdown" }
                div { class: "mr",
                    span { class: "l", "Active" }
                    span { class: "v", style: "color: var(--orange)", "{stats.in_progress}" }
                }
                div { class: "mr",
                    span { class: "l", "Backlog" }
                    span { class: "v", style: "color: var(--blue)", "{stats.open}" }
                }
                div { class: "mr",
                    span { class: "l", "Done" }
                    span { class: "v", style: "color: var(--green)", "{stats.done}" }
                }
            }
            if !sections.is_empty() {
                div { class: "vl",
                    div { class: "sl", "Sections" }
                    for (label, count) in sections {
                        components::SectionBadgeRow { key: "{label}", label: label.clone(), count: count }
                    }
                }
            }
            div { class: "sidebar-foot",
                button { class: "btn-n", onclick: move |_| modal.set(true), "+ New Issue" }
                div { class: if dirty() { "sync-status dirty" } else { "sync-status" }, if dirty() { "⚠ Unsaved" } else { "✓ Synced" } }
                if dirty() {
                    button { class: "sync-action", onclick: move |_| { save_workspace(state, "Saved"); dirty.set(false); }, "Save All" }
                }
                button { class: "sync-action", style: "background:transparent; color:var(--ink3); border:1px solid var(--rule);", onclick: move |_| reinit_modal.set(true), "↻ Reset" }
            }
            div { class: "kb-ref",
                kbd { "/" } " search" br {}
                kbd { "j" } kbd { "k" } " navigate" br {}
                kbd { "Enter" } " open" br {}
                kbd { "Esc" } " close" br {}
                kbd { "d" } " toggle density" br {}
                kbd { "t" } " toggle theme"
            }
        }
    }
}

fn render_topbar(
    mut search: Signal<String>, 
    mut active_label: Signal<Option<String>>,
    mut active_lens: Signal<FeedLens>,
    available_labels: Vec<String>,
    state: AppState,
    mut physics: Signal<super::scroll::ScrollPhysics>,
    mut animating: Signal<bool>,
) -> Element {
    let mut is_compact = state.is_compact;
    let mut zoom = state.zoom;
    
    rsx! {
        div { class: "sticky-header",
            div { class: "topbar",
                input { 
                    class: "si", 
                    placeholder: "Search…", 
                    value: "{search}", 
                    oninput: move |e| {
                        search.set(e.value());
                        physics.write().reset();
                        animating.set(true);
                    } 
                }
                
                div { class: "density-toggle", style: "margin-right: 12px;",
                    button { class: "dt-btn", onclick: move |_| zoom.set((zoom() - 0.25).max(1.0)), "-" }
                    button { class: "dt-btn active", style: "width: 50px; text-align: center; pointer-events: none;", "{zoom() * 100.0}%" }
                    button { class: "dt-btn", onclick: move |_| zoom.set((zoom() + 0.25).min(2.5)), "+" }
                }

                div { class: "density-toggle",
                    button { 
                        class: if !is_compact() { "dt-btn active" } else { "dt-btn" }, 
                        onclick: move |_| is_compact.set(false), 
                        "Comfortable" 
                    }
                    button { 
                        class: if is_compact() { "dt-btn active" } else { "dt-btn" }, 
                        onclick: move |_| is_compact.set(true), 
                        "Compact" 
                    }
                }
            }
            div { class: "lens-row",
                button { 
                    class: if active_lens() == FeedLens::MyOrder { "lens active" } else { "lens" },
                    onclick: move |_| active_lens.set(FeedLens::MyOrder), 
                    "My Order" 
                }
                button { 
                    class: if active_lens() == FeedLens::NextUp { "lens active" } else { "lens" },
                    onclick: move |_| active_lens.set(FeedLens::NextUp), 
                    "Next Up" 
                }
                button { 
                    class: if active_lens() == FeedLens::HotPath { "lens active" } else { "lens" },
                    onclick: move |_| active_lens.set(FeedLens::HotPath), 
                    "Hot Path" 
                }
                button { 
                    class: if active_lens() == FeedLens::QuickWins { "lens active" } else { "lens" },
                    onclick: move |_| active_lens.set(FeedLens::QuickWins), 
                    "Quick Wins" 
                }
            }
            if !available_labels.is_empty() {
                div { class: "label-filter-row",
                    button {
                        class: if active_label().is_none() { "label-filter active" } else { "label-filter" },
                        onclick: move |_| active_label.set(None),
                        "All labels"
                    }
                    for label in available_labels {
                        button {
                            key: "label-filter-{label}",
                            class: if active_label().as_deref() == Some(label.as_str()) {
                                "label-filter active {components::label_tone_class(&label)}"
                            } else {
                                "label-filter {components::label_tone_class(&label)}"
                            },
                            onclick: {
                                let label = label.clone();
                                move |_| active_label.set(Some(label.clone()))
                            },
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}

fn render_content(
    view: Signal<View>,
    filtered: Vec<Issue>,
    mut dirty: Signal<bool>,
    state: AppState,
    mut physics: Signal<super::scroll::ScrollPhysics>,
    mut animating: Signal<bool>,
) -> Element {
    let mut issues = state.issues;
    let edit_epoch = use_context::<Signal<u64>>();

    // ── Global Scroll physics ──────────────────────────────
    // physics and animating sigs passed from the root container
    let mut max_scroll = use_signal(|| 0.0f64);
    let mut header_ys = use_signal(Vec::<f64>::new);

    // Animation loop: ticks physics at ~60fps, writes transforms via eval()
    use_effect(move || {
        spawn(async move {
            let mut last_tick = tokio::time::Instant::now();
            let mut frames = 0;
            let mut total_time = 0.0;
            let mut max_frame_time = 0.0;
            let mut was_animating = false;

            loop {
                tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                
                let is_anim = animating();
                
                if is_anim && !was_animating {
                    // Just started scrolling
                    super::scroll::set_is_scrolling(true);
                    last_tick = tokio::time::Instant::now();
                    frames = 0;
                    total_time = 0.0;
                    max_frame_time = 0.0;
                    
                    // Measure geometry once at start of scroll to avoid IPC thrashing
                    let ms = super::scroll::measure_max_scroll().await;
                    if ms > 0.0 { max_scroll.set(ms); }
                    
                    let hys = super::scroll::measure_header_positions().await;
                    if !hys.is_empty() { header_ys.set(hys); }
                } else if !is_anim && was_animating {
                    // Just stopped scrolling
                    super::scroll::set_is_scrolling(false);
                    if frames > 0 {
                        let avg = total_time / (frames as f64);
                        println!("[Scroll Metrics] Frames: {} | Avg: {:.1}ms | Max: {:.1}ms", frames, avg, max_frame_time);
                    }
                }
                
                was_animating = is_anim;

                if !is_anim { continue; }

                let now = tokio::time::Instant::now();
                // Ensure dt is never 0 and capped to prevent tunneling spikes if loop sleeps too long
                let dt = (now - last_tick).as_secs_f64().clamp(0.001, 0.050);
                let dt_ms = dt * 1000.0;
                
                frames += 1;
                total_time += dt_ms;
                if dt_ms > max_frame_time { max_frame_time = dt_ms; }
                
                last_tick = now;

                let still_moving = {
                    let mut p = physics.write();
                    p.tick(dt, max_scroll())
                };

                let vis = physics.read().visual_offset(max_scroll());
                super::scroll::write_transforms(vis, &header_ys());

                if !still_moving {
                    animating.set(false);
                }
            }
        });
    });

    rsx! {
        div {
            class: "content",
            style: "zoom: {(state.zoom)()};",
            onpointerdown: move |_| {
                if animating() {
                    physics.write().velocity = 0.0;
                }
            },
            onpointermove: move |_| {
                if animating() {
                    // Every mouse movement tick drains 15% of the current velocity,
                    // creating an organic, dynamic braking sensation.
                    physics.write().velocity *= 0.85;
                }
            },
            onwheel: move |evt: Event<WheelData>| {
                physics.write().add_wheel_delta(evt.delta().strip_units().y, max_scroll());
                if !animating() {
                    animating.set(true);
                }
            },
            match view() {
                View::Feed => rsx! {
                    views::FeedView {
                        is_compact: (state.is_compact)(),
                        zoom: (state.zoom)(),
                        issues: filtered.clone(),
                        on_status: move |(id, s): (String, String)| {
                            if let Some(i) = issues.write().iter_mut().find(|i| i.id == id) { i.status = Status::from_str(&s); }
                            bump_edit_epoch(edit_epoch);
                            dirty.set(true);
                        },
                        on_resolution: move |(id, t): (String, String)| {
                            if let Some(i) = issues.write().iter_mut().find(|i| i.id == id) { i.resolution = t; }
                            bump_edit_epoch(edit_epoch);
                            dirty.set(true);
                        },
                        on_labels: move |(id, labels): (String, String)| {
                            if let Some(i) = issues.write().iter_mut().find(|i| i.id == id) {
                                i.labels = parse_label_input(&labels);
                            }
                            bump_edit_epoch(edit_epoch);
                            dirty.set(true);
                        },
                        on_reorder: move |(drag, target, after, section): (String, Option<String>, bool, Option<String>)| {
                            if target.as_ref().is_some_and(|target_id| *target_id == drag) {
                                return;
                            }
                            let mut all = issues();
                            if let Some(idx) = all.iter().position(|i| i.id == drag) {
                                let mut iss = all.remove(idx);
                                if let Some(ref target_id) = target {
                                    if let Some(tidx) = all.iter().position(|i| i.id == *target_id) {
                                        iss.section = section.clone().unwrap_or_else(|| all[tidx].section.clone());
                                        let insert_at = if after { tidx + 1 } else { tidx }.min(all.len());
                                        all.insert(insert_at, iss);
                                    } else {
                                        all.insert(idx.min(all.len()), iss);
                                    }
                                } else if let Some(section_name) = section {
                                    iss.section = section_name;
                                    all.push(iss);
                                } else {
                                    all.insert(idx.min(all.len()), iss);
                                }
                                #[cfg(debug_assertions)]
                                {
                                    let mut seen = std::collections::HashSet::new();
                                    for i in all.iter() {
                                        assert!(
                                            seen.insert(i.id.clone()),
                                            "on_reorder: duplicate id {} (drag={} target={:?} after={})",
                                            i.id, drag, target, after
                                        );
                                    }
                                }
                            }
                            bump_edit_epoch(edit_epoch);
                            issues.set(all);
                            save_workspace(state, "");
                        },
                        on_section_toggle: move |_| {
                            physics.write().reset();
                            animating.set(true);
                        },
                    }
                },
                View::Board => rsx! {
                    views::BoardView {
                        issues: filtered,
                        on_status: move |(id, s): (String, String)| {
                            if let Some(i) = issues.write().iter_mut().find(|i| i.id == id) { i.status = Status::from_str(&s); }
                            bump_edit_epoch(edit_epoch);
                            dirty.set(true);
                        },
                        on_resolution: move |(id, t): (String, String)| {
                            if let Some(i) = issues.write().iter_mut().find(|i| i.id == id) { i.resolution = t; }
                            bump_edit_epoch(edit_epoch);
                            dirty.set(true);
                        },
                        on_labels: move |(id, labels): (String, String)| {
                            if let Some(i) = issues.write().iter_mut().find(|i| i.id == id) {
                                i.labels = parse_label_input(&labels);
                            }
                            bump_edit_epoch(edit_epoch);
                            dirty.set(true);
                        },
                        on_reorder: move |payload: (String, Option<String>, bool, Option<String>)| {
                            if payload.1.as_ref().is_some_and(|target| payload.0 == *target) {
                                return;
                            }
                            let (drag, target, after, section) = payload;
                            let mut all = issues();
                            if let Some(idx) = all.iter().position(|i| i.id == drag) {
                                let mut iss = all.remove(idx);
                                if let Some(target_id) = target {
                                    if let Some(tidx) = all.iter().position(|i| i.id == *target_id) {
                                        iss.section = section.clone().unwrap_or_else(|| all[tidx].section.clone());
                                        let insert_at = if after { tidx + 1 } else { tidx }.min(all.len());
                                        all.insert(insert_at, iss);
                                    } else {
                                        all.insert(idx.min(all.len()), iss);
                                    }
                                } else if let Some(section_name) = section {
                                    iss.section = section_name;
                                    all.push(iss);
                                } else {
                                    all.insert(idx.min(all.len()), iss);
                                }
                            }
                            bump_edit_epoch(edit_epoch);
                            issues.set(all);
                            save_workspace(state, "");
                        }
                    }
                },
                View::Heatmap => rsx! { views::HeatmapView { issues: issues() } },
                View::Graph => rsx! { views::GraphView { issues: issues() } },
                View::Timeline => rsx! { views::TimelineView { issues: issues() } },
            }
        }
    }
}

fn compute_stats(issues: &[Issue]) -> Stats {
    let mut s = Stats::default();
    for i in issues {
        match i.status {
            Status::Open => s.open += 1,
            Status::InProgress => s.in_progress += 1,
            Status::Done => s.done += 1,
            Status::Descoped => s.descoped += 1,
        }
    }
    s.total = issues.len();
    s
}

fn section_counts(issues: &[Issue]) -> Vec<(String, usize)> {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for issue in issues {
        *counts.entry(issue.section.clone()).or_default() += 1;
    }

    let mut sections: Vec<_> = counts.into_iter().collect();
    sections.sort_by(|(left, _), (right, _)| {
        section_sort_key(left)
            .cmp(&section_sort_key(right))
            .then_with(|| left.cmp(right))
    });
    sections
}

fn section_sort_key(section: &str) -> (u8, String) {
    let normalized = section.trim().to_ascii_lowercase();
    let rank = if normalized.contains("active") {
        0
    } else if normalized.contains("backlog") {
        1
    } else if normalized.contains("done") {
        2
    } else {
        3
    };
    (rank, normalized)
}

fn apply_feed_lens(all_issues: &[Issue], mut issues: Vec<Issue>, lens: FeedLens) -> Vec<Issue> {
    if lens == FeedLens::MyOrder {
        return issues;
    }

    let metrics = LensMetrics::from_issues(all_issues);
    issues.sort_by(|left, right| {
        metrics
            .sort_key(left, lens)
            .cmp(&metrics.sort_key(right, lens))
            .then_with(|| issue_id_sort_key(&left.id).cmp(&issue_id_sort_key(&right.id)))
    });
    issues
}

#[derive(Debug, Default)]
struct LensMetrics {
    hot_scores: HashMap<String, usize>,
    unblock_scores: HashMap<String, usize>,
    quick_costs: HashMap<String, usize>,
}

impl LensMetrics {
    fn from_issues(issues: &[Issue]) -> Self {
        let ws = Workspace {
            root: PathBuf::new(),
            issues: issues.to_vec(),
        };

        let mut file_weights = HashMap::<String, usize>::new();
        for (file, ids) in ws.file_heatmap() {
            file_weights.insert(file, ids.len());
        }

        let mut dependents = HashMap::<String, Vec<String>>::new();
        for (dependency, dependent) in ws.dependency_edges() {
            dependents.entry(dependency).or_default().push(dependent);
        }

        let active_issue_ids = issues
            .iter()
            .filter(|issue| issue.status != Status::Done && issue.status != Status::Descoped)
            .map(|issue| issue.id.clone())
            .collect::<HashSet<_>>();

        let hot_scores = issues
            .iter()
            .map(|issue| {
                let score = issue
                    .files
                    .iter()
                    .map(|file| file_weights.get(file).copied().unwrap_or(1))
                    .sum::<usize>();
                (issue.id.clone(), score)
            })
            .collect::<HashMap<_, _>>();

        let quick_costs = issues
            .iter()
            .map(|issue| {
                let heat = hot_scores.get(&issue.id).copied().unwrap_or_default();
                let cost = heat + (issue.files.len() * 2) + (issue.depends_on.len() * 3);
                (issue.id.clone(), cost)
            })
            .collect::<HashMap<_, _>>();

        let mut unblock_scores = HashMap::<String, usize>::new();
        for issue in issues {
            let mut visited = HashSet::new();
            let score = transitive_dependents(&issue.id, &dependents, &active_issue_ids, &mut visited);
            unblock_scores.insert(issue.id.clone(), score);
        }

        Self {
            hot_scores,
            unblock_scores,
            quick_costs,
        }
    }

    fn sort_key(&self, issue: &Issue, lens: FeedLens) -> (usize, usize, usize, (String, u32, String)) {
        match lens {
            FeedLens::MyOrder => (0, 0, 0, issue_id_sort_key(&issue.id)),
            FeedLens::NextUp => (
                usize::MAX - self.unblock_scores.get(&issue.id).copied().unwrap_or_default(),
                issue.status_ord() as usize,
                self.quick_costs.get(&issue.id).copied().unwrap_or_default(),
                issue_id_sort_key(&issue.id),
            ),
            FeedLens::HotPath => (
                usize::MAX - self.hot_scores.get(&issue.id).copied().unwrap_or_default(),
                issue.status_ord() as usize,
                issue.files.len(),
                issue_id_sort_key(&issue.id),
            ),
            FeedLens::QuickWins => (
                self.quick_costs.get(&issue.id).copied().unwrap_or_default(),
                issue.status_ord() as usize,
                self.unblock_scores.get(&issue.id).copied().unwrap_or_default(),
                issue_id_sort_key(&issue.id),
            ),
        }
    }
}

fn transitive_dependents(
    id: &str,
    dependents: &HashMap<String, Vec<String>>,
    active_issue_ids: &HashSet<String>,
    visited: &mut HashSet<String>,
) -> usize {
    let Some(children) = dependents.get(id) else {
        return 0;
    };

    let mut total = 0;
    for child in children {
        if !active_issue_ids.contains(child) || !visited.insert(child.clone()) {
            continue;
        }
        total += 1 + transitive_dependents(child, dependents, active_issue_ids, visited);
    }
    total
}

fn collect_labels(issues: &[Issue]) -> Vec<String> {
    let mut labels = issues
        .iter()
        .flat_map(|issue| issue.labels.iter().cloned())
        .collect::<Vec<_>>();
    labels.sort_by_key(|label| label.to_ascii_lowercase());
    labels.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    labels
}

fn parse_label_input(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .collect()
}

fn filter_issues(issues: &[Issue], q: &str, active_label: Option<&str>) -> Vec<Issue> {
    let q = q.to_lowercase();
    let active_label = active_label.map(str::to_ascii_lowercase);
    issues
        .iter()
        .filter(|i| {
            (q.is_empty()
                || i.title.to_lowercase().contains(&q)
                || i.id.to_string().contains(&q)
                || i.labels.iter().any(|label| label.to_lowercase().contains(&q)))
                && active_label.as_ref().is_none_or(|selected| {
                    i.labels
                        .iter()
                        .any(|label| label.eq_ignore_ascii_case(selected))
                })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        apply_feed_lens, can_apply_external_reload, collect_labels, filter_issues,
        parse_label_input, section_counts, should_reload_for_event, FeedLens,
    };
    use crate::model::{Issue, Status};
    use notify::event::{AccessKind, AccessMode, CreateKind, DataChange, ModifyKind};
    use notify::{Event as NotifyEvent, EventKind};
    use std::path::Path;

    fn make_issue(id: &str, title: &str, labels: &[&str]) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            status: Status::Open,
            files: vec![],
            labels: labels.iter().map(|label| label.to_string()).collect(),
            links: vec![],
            description: String::new(),
            resolution: String::new(),
            section: "ACTIVE Issues".to_string(),
            depends_on: vec![],
        }
    }

    fn make_issue_with_graph(
        id: &str,
        title: &str,
        status: Status,
        files: &[&str],
        depends_on: &[&str],
    ) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            status,
            files: files.iter().map(|file| file.to_string()).collect(),
            labels: vec![],
            links: vec![],
            description: String::new(),
            resolution: String::new(),
            section: "ACTIVE Issues".to_string(),
            depends_on: depends_on.iter().map(|dep| dep.to_string()).collect(),
        }
    }

    #[test]
    fn filter_issues_matches_labels() {
        let issues = vec![
            make_issue("BUG-01", "Parser cleanup", &["core", "frontend"]),
            make_issue("BUG-02", "Health pulse", &["ux"]),
        ];

        let filtered = filter_issues(&issues, "front", None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "BUG-01");
    }

    #[test]
    fn filter_issues_excludes_non_matching_labels() {
        let issues = vec![
            make_issue("BUG-01", "Parser cleanup", &["core", "frontend"]),
            make_issue("BUG-02", "Health pulse", &["ux"]),
        ];

        let filtered = filter_issues(&issues, "testing", None);
        assert!(filtered.is_empty());
    }

    #[test]
    fn filter_issues_respects_active_label() {
        let issues = vec![
            make_issue("BUG-01", "Parser cleanup", &["core", "frontend"]),
            make_issue("BUG-02", "Health pulse", &["ux"]),
        ];

        let filtered = filter_issues(&issues, "", Some("ux"));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "BUG-02");
    }

    #[test]
    fn external_reload_requires_clean_unchanged_state() {
        assert!(can_apply_external_reload(false, false, 9, 9));
        assert!(!can_apply_external_reload(true, false, 9, 9));
        assert!(!can_apply_external_reload(false, true, 9, 9));
        assert!(!can_apply_external_reload(false, false, 9, 10));
    }

    #[test]
    fn watcher_ignores_access_only_events() {
        let access_event = NotifyEvent {
            kind: EventKind::Access(AccessKind::Read),
            paths: vec![Path::new("issues-active.md").to_path_buf()],
            attrs: Default::default(),
        };
        let close_event = NotifyEvent {
            kind: EventKind::Access(AccessKind::Close(AccessMode::Write)),
            paths: vec![Path::new("issues-active.md").to_path_buf()],
            attrs: Default::default(),
        };
        let modify_event = NotifyEvent {
            kind: EventKind::Modify(ModifyKind::Data(DataChange::Any)),
            paths: vec![Path::new("issues-active.md").to_path_buf()],
            attrs: Default::default(),
        };
        let create_event = NotifyEvent {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![Path::new("issues-active.md").to_path_buf()],
            attrs: Default::default(),
        };

        assert!(!should_reload_for_event(&access_event));
        assert!(!should_reload_for_event(&close_event));
        assert!(should_reload_for_event(&modify_event));
        assert!(should_reload_for_event(&create_event));
    }

    #[test]
    fn parse_label_input_trims_and_drops_empty_values() {
        assert_eq!(
            parse_label_input(" core, frontend , , ux "),
            vec!["core", "frontend", "ux"]
        );
    }

    #[test]
    fn collect_labels_deduplicates_case_insensitively() {
        let issues = vec![
            make_issue("BUG-01", "Parser cleanup", &["core", "frontend"]),
            make_issue("BUG-02", "Health pulse", &["Core", "ux"]),
        ];

        assert_eq!(collect_labels(&issues), vec!["core", "frontend", "ux"]);
    }

    #[test]
    fn section_counts_groups_and_orders_sections() {
        let mut active = make_issue("BUG-01", "Parser cleanup", &[]);
        active.section = "ACTIVE Issues".to_string();
        let mut done = make_issue("BUG-02", "Resolved bug", &[]);
        done.section = "DONE Issues".to_string();
        let mut custom = make_issue("BUG-03", "Sprint issue", &[]);
        custom.section = "Sprint 42".to_string();

        assert_eq!(
            section_counts(&[custom.clone(), done, active, custom]),
            vec![
                ("ACTIVE Issues".to_string(), 1),
                ("DONE Issues".to_string(), 1),
                ("Sprint 42".to_string(), 2),
            ]
        );
    }

    #[test]
    fn next_up_lens_prioritizes_transitive_unblock_count() {
        let issues = vec![
            make_issue_with_graph("BUG-01", "Base", Status::Open, &["src/main.rs"], &[]),
            make_issue_with_graph("BUG-02", "Middle", Status::Open, &["src/main.rs"], &["BUG-01"]),
            make_issue_with_graph("BUG-03", "Leaf", Status::Open, &["src/ui.rs"], &["BUG-02"]),
        ];

        let sorted = apply_feed_lens(&issues, issues.clone(), FeedLens::NextUp);
        assert_eq!(sorted.iter().map(|issue| issue.id.clone()).collect::<Vec<_>>(), vec!["BUG-01", "BUG-02", "BUG-03"]);
    }

    #[test]
    fn hot_path_lens_prioritizes_hotter_files() {
        let issues = vec![
            make_issue_with_graph("BUG-01", "Shared A", Status::Open, &["src/main.rs"], &[]),
            make_issue_with_graph("BUG-02", "Shared B", Status::Open, &["src/main.rs"], &[]),
            make_issue_with_graph("BUG-03", "Cold", Status::Open, &["src/cold.rs"], &[]),
        ];

        let sorted = apply_feed_lens(&issues, issues.clone(), FeedLens::HotPath);
        assert_eq!(sorted[0].id, "BUG-01");
        assert_eq!(sorted[1].id, "BUG-02");
        assert_eq!(sorted[2].id, "BUG-03");
    }

    #[test]
    fn quick_wins_lens_prefers_lower_cost_work() {
        let issues = vec![
            make_issue_with_graph("BUG-01", "Wide", Status::Open, &["a.rs", "b.rs"], &["BUG-09"]),
            make_issue_with_graph("BUG-02", "Tight", Status::Open, &["solo.rs"], &[]),
        ];

        let sorted = apply_feed_lens(&issues, issues.clone(), FeedLens::QuickWins);
        assert_eq!(sorted[0].id, "BUG-02");
        assert_eq!(sorted[1].id, "BUG-01");
    }
}
