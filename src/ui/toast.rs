use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
}

impl ToastKind {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Success => "toast-success",
            Self::Error => "toast-error",
        }
    }
}

#[component]
pub fn ToastContainer(toasts: Vec<Toast>, on_dismiss: EventHandler<u64>) -> Element {
    rsx! {
        div { class: "toast-container",
            for toast in toasts {
                div {
                    key: "{toast.id}",
                    class: "toast {toast.kind.class()}",
                    onclick: move |_| on_dismiss.call(toast.id),
                    "{toast.message}"
                }
            }
        }
    }
}
