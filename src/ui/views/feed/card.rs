use crate::model::{split_issue_id, Issue};
use crate::ui::components::LabelList;
use crate::ui::views::feed::{
    apply_drag_deadzone, DragPresence, DragState, RecentDropState, DRAG_DEADZONE_PX,
};
use dioxus::document::eval;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct IssueCardProps {
    pub issue: Issue,
    pub issue_key: String,
    pub incoming_links: Vec<String>,
    pub idx: usize,
    pub virtual_y: f32, // The pre-calculated absolute Y position of the slot
    pub drag_presence: Signal<DragPresence>,
    pub drag_state: Signal<DragState>,
    pub drag_offset: Signal<f32>,
    pub recent_drop: Signal<RecentDropState>,
    pub allow_link_hover: bool,
    pub is_compact: bool,
    pub array_reordered: bool,
    pub is_hidden: bool,
}

#[derive(Clone, PartialEq, Props)]
pub struct DragOverlayProps {
    pub issue: Issue,
    pub issue_key: String,
    pub incoming_links: Vec<String>,
    pub drag_presence: Signal<DragPresence>,
    pub drag_state: Signal<DragState>,
    pub drag_offset: Signal<f32>,
    pub is_compact: bool,
}

const CLEAR_LINK_BRACKETS_SCRIPT: &str = r#"
(() => {
  const clearRows = () => {
    document.querySelectorAll('.issue-row.link-hl').forEach((row) => row.classList.remove('link-hl'));
  };
  if (window.__ishooBracketHideTimer) {
    clearTimeout(window.__ishooBracketHideTimer);
  }
  const svg = document.getElementById('link-bracket-overlay');
  if (svg) {
    window.__ishooBracketHideTimer = setTimeout(() => {
      clearRows();
      svg.classList.remove('visible');
      svg.innerHTML = '';
      window.__ishooBracketHideTimer = null;
    }, 200);
  } else {
    clearRows();
  }
})();
"#;

#[component]
// neti:allow(LAW OF COMPLEXITY)
pub fn IssueCard(props: IssueCardProps) -> Element {
    let i = &props.issue;
    let id = i.id.clone();
    let issue_key = props.issue_key.clone();
    let idx = props.idx;
    let dp = props.drag_presence.read();
    let ds = props.drag_state.read();
    let rd = props.recent_drop.read();

    let is_dragging = dp.dragging_key == Some(issue_key.clone());
    let array_reordered = props.array_reordered;
    let is_recent_drop = rd.key == Some(issue_key.clone());
    let hover_armed = rd.hover_armed;

    // Keep live drag displacement local to the card layer instead of simulating
    // array reorder in the parent; this avoids downward cards appearing to jump
    // across the held card mid-flight.
    let mut virtual_y = props.virtual_y;
    let slot_h = if props.is_compact { 44.0 } else { 93.0 };

    if dp.dragging_key.is_some() && !is_dragging && !array_reordered {
        let start_y = ds.start_virtual_y;
        let hover_y = ds.hover_y;

        if hover_y > start_y {
            if props.virtual_y > start_y && props.virtual_y <= hover_y {
                virtual_y -= slot_h;
            }
        } else if hover_y < start_y && props.virtual_y < start_y && props.virtual_y >= hover_y {
            virtual_y += slot_h;
        }
    }

    let y_pos = if is_dragging && dp.releasing {
        ds.hover_y
    } else {
        virtual_y
    };
    let transition = "transform 400ms cubic-bezier(0.25, 1, 0.5, 1)";

    let mut cls = "item".to_string();
    if is_recent_drop {
        cls.push_str(" recent-drop");
        if hover_armed {
            cls.push_str(" hover-armed");
        }
    }

    let outer_style = format!(
        "position: absolute; top: 0; left: 0px; right: 0px; transform: translate3d(0, {y_pos}px, 0){}; transition: {transition}; opacity: {}; pointer-events: {};",
        if props.is_hidden { " scale(0.8)" } else { "" },
        if props.is_hidden || is_dragging { "0" } else { "1" },
        if props.is_hidden || is_dragging { "none" } else { "auto" },
    );

    let mut drag_presence_signal = props.drag_presence;
    let mut drag_state_signal = props.drag_state;
    let mut drag_offset_signal = props.drag_offset;
    let mut recent_drop_signal = props.recent_drop;

    let is_done =
        i.status == crate::model::Status::Done || i.status == crate::model::Status::Descoped;
    let sec_lower = i.section.to_lowercase();
    let is_backlog = !is_done && sec_lower.contains("backlog");

    let section_color = if is_done {
        "var(--green)"
    } else if is_backlog {
        "var(--blue)"
    } else {
        "var(--orange)"
    };
    let outgoing_count = i.links.len();
    let incoming_count = props.incoming_links.len();
    let link_count = outgoing_count + incoming_count;
    let link_ids = if props.allow_link_hover && link_count > 0 {
        let mut related_links = i.links.clone();
        for incoming in &props.incoming_links {
            if !related_links.contains(incoming) {
                related_links.push(incoming.clone());
            }
        }
        related_links.join(",")
    } else {
        String::new()
    };
    let row_dom_id = format!("issue-row-{}", issue_key.replace(':', "-"));
    let section_dom_key = i.section.to_ascii_lowercase().replace(' ', "-");
    rsx! {
        div {
            class: "{cls}",
            style: "{outer_style}",
            div {
                id: "{row_dom_id}",
                class: "issue-row",
                "data-issue-id": "{id}",
                "data-section-key": "{section_dom_key}",
                onpointerdown: move |e| {
                    e.prevent_default();
                    let _ = eval(CLEAR_LINK_BRACKETS_SCRIPT);
                    recent_drop_signal.set(RecentDropState::default());
                    drag_presence_signal.set(DragPresence {
                        dragging_key: Some(issue_key.clone()),
                        releasing: false,
                    });
                    let mut ds_write = drag_state_signal.write();
                    ds_write.start_idx = idx;
                    ds_write.hover_idx = idx;
                    ds_write.hover_after = false;
                    ds_write.start_y = e.client_coordinates().y as f32;
                    ds_write.start_virtual_y = props.virtual_y;
                    ds_write.last_layout_probe_y = props.virtual_y;
                    drag_offset_signal.set(0.0);
                    ds_write.hover_y = props.virtual_y;
                },
                onmouseenter: move |_| {
                    if !props.allow_link_hover || link_count == 0 {
                        return;
                    }
                    let script = format!(
                        r#"
(() => {{
  if (window.__ishooBracketHideTimer) {{
    clearTimeout(window.__ishooBracketHideTimer);
    window.__ishooBracketHideTimer = null;
  }}
  const source = document.getElementById({row_id:?});
  if (!source) return;
  const container = document.getElementById("scroll-content");
  if (!container) return;
  const sectionKey = source.dataset.sectionKey;
  const linkIds = {link_ids:?}.split(',').filter(Boolean);
  const targets = linkIds
    .map((linkId) => document.querySelector(`.issue-row[data-issue-id="${{linkId}}"][data-section-key="${{sectionKey}}"]`))
    .filter(Boolean);
  if (!targets.length) return;

  document.querySelectorAll('.issue-row.link-hl').forEach((row) => row.classList.remove('link-hl'));
  let svg = document.getElementById('link-bracket-overlay');
  if (!svg) {{
    svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('id', 'link-bracket-overlay');
    svg.classList.add('bracket-svg');
    container.appendChild(svg);
  }}

  const containerRect = container.getBoundingClientRect();
  const sourceRect = source.getBoundingClientRect();
  const srcY = sourceRect.top + sourceRect.height / 2 - containerRect.top;
  const x = sourceRect.left - containerRect.left - 10;
  const tickLen = 8;

  source.classList.add('link-hl');
  svg.innerHTML = '';
  svg.setAttribute('width', String(Math.max(22, Math.ceil(x + tickLen + 4))));
  svg.setAttribute('height', String(container.scrollHeight));
  svg.style.width = `${{Math.max(22, Math.ceil(x + tickLen + 4))}}px`;
  svg.style.height = `${{container.scrollHeight}}px`;
  svg.style.left = '0px';

  targets.forEach((target) => {{
    target.classList.add('link-hl');
    const targetRect = target.getBoundingClientRect();
    const tgtY = targetRect.top + targetRect.height / 2 - containerRect.top;
    const topY = Math.min(srcY, tgtY);
    const botY = Math.max(srcY, tgtY);

    const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    line.setAttribute('x1', String(x));
    line.setAttribute('y1', String(topY));
    line.setAttribute('x2', String(x));
    line.setAttribute('y2', String(botY));
    line.classList.add('bracket-line');
    svg.appendChild(line);

    const topTick = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    topTick.setAttribute('x1', String(x));
    topTick.setAttribute('y1', String(topY));
    topTick.setAttribute('x2', String(x + tickLen));
    topTick.setAttribute('y2', String(topY));
    topTick.classList.add('bracket-tick');
    svg.appendChild(topTick);

    const bottomTick = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    bottomTick.setAttribute('x1', String(x));
    bottomTick.setAttribute('y1', String(botY));
    bottomTick.setAttribute('x2', String(x + tickLen));
    bottomTick.setAttribute('y2', String(botY));
    bottomTick.classList.add('bracket-tick');
    svg.appendChild(bottomTick);
  }});

  svg.classList.add('visible');
}})();
"#,
                        row_id = row_dom_id,
                        link_ids = link_ids,
                    );
                    let _ = eval(&script);
                },
                onmouseleave: move |_| {
                    let _ = eval(CLEAR_LINK_BRACKETS_SCRIPT);
                },
                IssueRowContent {
                    issue: i.clone(),
                    incoming_links: props.incoming_links.clone(),
                    is_compact: props.is_compact,
                    section_color: section_color.to_string(),
                }
            }
        }
    }
}

#[component]
pub fn DragOverlay(props: DragOverlayProps) -> Element {
    let dp = props.drag_presence.read();
    let ds = props.drag_state.read();
    if dp.dragging_key.as_deref() != Some(props.issue_key.as_str()) {
        return rsx! {};
    }

    let mut effective_offset = if dp.releasing {
        0.0
    } else {
        (props.drag_offset)()
    };
    let mut cls = "item dragging".to_string();
    let transition = if dp.releasing {
        cls.push_str(" settling");
        "transform 400ms cubic-bezier(0.25, 1, 0.5, 1)"
    } else {
        if effective_offset.abs() < DRAG_DEADZONE_PX {
            effective_offset = 0.0;
        } else {
            effective_offset = apply_drag_deadzone(effective_offset);
        }
        "none"
    };

    let y_pos = if dp.releasing {
        ds.hover_y
    } else {
        ds.start_virtual_y + effective_offset
    };

    let outer_style = format!(
        "position: absolute; top: 0; left: 0; right: 0; transform: translate3d(0, {y_pos}px, 0); transition: {transition}; pointer-events: none;"
    );

    let is_done = props.issue.status == crate::model::Status::Done
        || props.issue.status == crate::model::Status::Descoped;
    let sec_lower = props.issue.section.to_lowercase();
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
                IssueRowContent {
                    issue: props.issue,
                    incoming_links: props.incoming_links,
                    is_compact: props.is_compact,
                    section_color: section_color.to_string(),
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct IssueRowContentProps {
    issue: Issue,
    incoming_links: Vec<String>,
    is_compact: bool,
    section_color: String,
}

#[component]
fn IssueRowContent(props: IssueRowContentProps) -> Element {
    let (id_category, id_number) = split_issue_id(&props.issue.id);
    let outgoing_count = props.issue.links.len();
    let incoming_count = props.incoming_links.len();

    rsx! {
        div { class: "id-badge",
            span { class: "id-cat", "{id_category}-" }
            span { class: "id-num", "{id_number}" }
        }
        div { class: "issue-body",
            div { class: "issue-title",
                "{props.issue.title}"
            }
            if !props.is_compact {
                div { class: "issue-sub",
                    span { "{props.issue.files.len()} file", if props.issue.files.len() != 1 { "s" } }
                    span { class: "sep", "/" }
                    span { "2 days" }
                }
                div { class: "labels-row", style: "display:flex;gap:4px;margin-top:4px;",
                    span { class: "label b-{props.issue.status.css_class()}", "{props.issue.status.label()}" }
                    LabelList { labels: props.issue.labels.clone() }
                }
            }
        }
        div { class: "issue-right",
            if outgoing_count > 0 || incoming_count > 0 {
                span {
                    class: "xlink",
                    title: if outgoing_count > 0 && incoming_count > 0 {
                        "Mentions {outgoing_count} issue(s) and is mentioned by {incoming_count}"
                    } else if outgoing_count > 0 {
                        "Mentions {outgoing_count} linked issue(s)"
                    } else {
                        "Mentioned by {incoming_count} issue(s)"
                    },
                    if outgoing_count > 0 && incoming_count > 0 {
                        "↕"
                    } else if outgoing_count > 0 {
                        "↗"
                    } else {
                        "↙"
                    }
                }
            }
            div { class: "s-dot", style: "background:{props.section_color}; width:8px; height:8px; border-radius:50%;" }
        }
    }
}
