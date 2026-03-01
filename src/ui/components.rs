use dioxus::prelude::*;

#[component]
pub fn NavBtn(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: if active { "nav-btn on" } else { "nav-btn" },
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

#[component]
pub fn StatRow(label: String, count: usize, color: String) -> Element {
    rsx! {
        div { class: "stat",
            span { class: "stat-lbl", "{label}" }
            span { class: "stat-val", style: "color:{color}", "{count}" }
        }
    }
}
