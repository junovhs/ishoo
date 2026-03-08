use dioxus::prelude::*;

pub fn label_tone_class(label: &str) -> &'static str {
    let normalized = label.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return "t-ink";
    }

    const TONES: [[&str; 3]; 6] = [
        ["t-blue-a", "t-blue-b", "t-blue-c"],
        ["t-orange-a", "t-orange-b", "t-orange-c"],
        ["t-pink-a", "t-pink-b", "t-pink-c"],
        ["t-teal-a", "t-teal-b", "t-teal-c"],
        ["t-yellow-a", "t-yellow-b", "t-yellow-c"],
        ["t-purple-a", "t-purple-b", "t-purple-c"],
    ];

    let hash = normalized
        .bytes()
        .fold(0u64, |acc, byte| acc.wrapping_mul(131).wrapping_add(u64::from(byte)));
    let hue = (hash % TONES.len() as u64) as usize;
    let variant = ((hash / TONES.len() as u64) % 3) as usize;
    TONES[hue][variant]
}

#[component]
pub fn LabelChip(label: String) -> Element {
    rsx! {
        span { class: "label {label_tone_class(&label)}", "{label}" }
    }
}

#[component]
pub fn LabelList(labels: Vec<String>) -> Element {
    rsx! {
        for (idx, label) in labels.into_iter().enumerate() {
            LabelChip { key: "{idx}-{label}", label: label }
        }
    }
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

#[component]
pub fn SectionBadgeRow(label: String, count: usize) -> Element {
    rsx! {
        div { class: "section-badge-row",
            span { class: "section-badge-label", "{label}" }
            span { class: "section-badge-count", "{count}" }
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
        assert!(matches!(
            tone,
            "t-blue-a" | "t-blue-b" | "t-blue-c"
                | "t-orange-a" | "t-orange-b" | "t-orange-c"
                | "t-pink-a" | "t-pink-b" | "t-pink-c"
                | "t-teal-a" | "t-teal-b" | "t-teal-c"
                | "t-yellow-a" | "t-yellow-b" | "t-yellow-c"
                | "t-purple-a" | "t-purple-b" | "t-purple-c"
        ));
    }
}
