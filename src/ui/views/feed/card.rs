use crate::model::{split_issue_id, Issue};
use crate::ui::components::LabelList;
use crate::ui::views::feed::{apply_drag_deadzone, DragState, RecentDropState, DRAG_DEADZONE_PX};
use dioxus::document::eval;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct IssueCardProps {
    pub issue: Issue,
    pub incoming_links: Vec<String>,
    pub idx: usize,
    pub virtual_y: f32, // The pre-calculated absolute Y position of the slot
    pub drag_state: Signal<DragState>,
    pub drag_offset: Signal<f32>,
    pub recent_drop: Signal<RecentDropState>,
    pub allow_link_hover: bool,
    pub is_compact: bool,
    pub array_reordered: bool,
    pub is_hidden: bool,
}

const CLEAR_LINK_BRACKETS_SCRIPT: &str = r#"
(() => {
  document.querySelectorAll('.issue-row.link-hl').forEach((row) => row.classList.remove('link-hl'));
  const svg = document.getElementById('link-bracket-overlay');
  if (svg) {
    svg.classList.remove('visible');
    svg.innerHTML = '';
  }
})();
"#;

#[component]
// neti:allow(LAW OF COMPLEXITY)
pub fn IssueCard(props: IssueCardProps) -> Element {
    let i = &props.issue;
    let id = i.id.clone();
    let idx = props.idx;
    let ds = props.drag_state.read();
    let rd = props.recent_drop.read();

    let is_dragging = ds.dragging_id == Some(id.clone());
    let array_reordered = props.array_reordered;
    let is_recent_drop = rd.id == Some(id.clone());
    let hover_armed = rd.hover_armed;

    // Keep live drag displacement local to the card layer instead of simulating
    // array reorder in the parent; this avoids downward cards appearing to jump
    // across the held card mid-flight.
    let mut virtual_y = props.virtual_y;
    let slot_h = if props.is_compact { 44.0 } else { 93.0 };

    if ds.dragging_id.is_some() && !is_dragging && !array_reordered {
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

    let mut actually_dragging = is_dragging;
    let mut effective_offset = if is_dragging && !ds.releasing {
        (props.drag_offset)()
    } else {
        0.0
    };

    if actually_dragging && !ds.releasing {
        if effective_offset.abs() < DRAG_DEADZONE_PX {
            actually_dragging = false;
            effective_offset = 0.0;
        } else {
            effective_offset = apply_drag_deadzone(effective_offset);
        }
    }

    let y_pos = if is_dragging && !ds.releasing {
        // During live drag, stay pinned to the original pickup slot so
        // simulated reordering underneath never pulls the held card away
        // from the cursor.
        ds.start_virtual_y + effective_offset
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
    if is_recent_drop {
        cls.push_str(" recent-drop");
        if hover_armed {
            cls.push_str(" hover-armed");
        }
    }

    let outer_style = format!(
        "position: absolute; top: 0; left: 0px; right: 0px; transform: translate3d(0, {y_pos}px, 0){}; transition: {transition}; opacity: {}; pointer-events: {};",
        if props.is_hidden { " scale(0.8)" } else { "" },
        if props.is_hidden { "0" } else { "1" },
        if props.is_hidden { "none" } else { "auto" },
    );

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
    let row_dom_id = format!("issue-row-{id}");
    let section_dom_key = i.section.to_ascii_lowercase().replace(' ', "-");
    let (id_category, id_number) = split_issue_id(&id);

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
                    let mut ds_write = drag_state_signal.write();
                    ds_write.dragging_id = Some(id.clone());
                    ds_write.start_idx = idx;
                    ds_write.hover_idx = idx;
                    ds_write.hover_after = false;
                    ds_write.start_y = e.client_coordinates().y as f32;
                    ds_write.start_virtual_y = props.virtual_y;
                    drag_offset_signal.set(0.0);
                    ds_write.hover_y = props.virtual_y;
                    ds_write.releasing = false;
                },
                onmouseenter: move |_| {
                    if !props.allow_link_hover || link_count == 0 {
                        return;
                    }
                    let script = format!(
                        r#"
(() => {{
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
                div { class: "id-badge",
                    span { class: "id-cat", "{id_category}-" }
                    span { class: "id-num", "{id_number}" }
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
                            LabelList { labels: i.labels.clone() }
                        }
                    }
                }
                div { class: "issue-right", // Empty space on the right, matches the dot and links
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
                    div { class: "s-dot", style: "background:{section_color}; width:8px; height:8px; border-radius:50%;" }
                }
            }
        }
    }
}
