use dioxus::prelude::*;

pub fn label_tone_class(label: &str) -> &'static str {
    let normalized = label.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return "t-ink";
    }

    const TONES: [&str; 6] = ["t-blue", "t-orange", "t-pink", "t-teal", "t-yellow", "t-purple"];
    let hash = normalized
        .bytes()
        .fold(0u64, |acc, byte| acc.wrapping_mul(131).wrapping_add(u64::from(byte)));
    TONES[(hash % TONES.len() as u64) as usize]
}

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

#[cfg(test)]
mod tests {
    use super::label_tone_class;

    #[test]
    fn label_tone_class_is_stable_for_same_label() {
        assert_eq!(label_tone_class("frontend"), label_tone_class("frontend"));
        assert_eq!(label_tone_class("Clown Pass"), label_tone_class("clown pass"));
    }

    #[test]
    fn label_tone_class_uses_palette_for_unknown_labels() {
        let tone = label_tone_class("clown pass");
        assert!(matches!(tone, "t-blue" | "t-orange" | "t-pink" | "t-teal" | "t-yellow" | "t-purple"));
    }
}
