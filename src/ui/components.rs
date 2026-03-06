use dioxus::prelude::*;

#[component]
pub fn NavBtn(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: if active { "vb active" } else { "vb" },
            onclick: move |e| onclick.call(e),
            div { class: "p" }
            "{label}"
        }
    }
}

#[component]
pub fn StatRow(label: String, count: usize, color: String) -> Element {
    rsx! {
        div { class: "mr",
            span { class: "l", "{label}" }
            span { class: "v", style: "color:{color}", "{count}" }
        }
    }
}
