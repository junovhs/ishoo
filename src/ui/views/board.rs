use crate::model::{Issue, Status};
use dioxus::prelude::*;

#[component]
pub fn BoardView(issues: Vec<Issue>) -> Element {
    let open: Vec<Issue> = issues
        .iter()
        .filter(|i| i.status == Status::Open)
        .cloned()
        .collect();
    let wip: Vec<Issue> = issues
        .iter()
        .filter(|i| i.status == Status::InProgress)
        .cloned()
        .collect();
    let done: Vec<Issue> = issues
        .iter()
        .filter(|i| i.status == Status::Done)
        .cloned()
        .collect();

    rsx! {
        div { class: "board",
            BoardCol { title: "Open", color: "var(--c-blue)", items: open }
            BoardCol { title: "In Progress", color: "var(--c-amber)", items: wip }
            BoardCol { title: "Done", color: "var(--c-green)", items: done }
        }
    }
}

#[component]
fn BoardCol(title: String, color: String, items: Vec<Issue>) -> Element {
    rsx! {
        div { class: "bcol",
            div { class: "bcol-hdr",
                div { class: "bcol-dot", style: "background:{color}" }
                span { "{title}" }
                span { class: "bcol-ct", "{items.len()}" }
            }
            div { class: "bcol-cards",
                for i in &items {
                    div { class: "bcard",
                        div { class: "bcard-id", "#{i.id}" }
                        div { class: "bcard-title", "{i.title}" }
                        if !i.files.is_empty() {
                            div { class: "bcard-meta", "{i.files.len()} files" }
                        }
                    }
                }
            }
        }
    }
}
