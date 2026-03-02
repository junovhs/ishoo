use crate::model::init_workspace_at;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn WelcomeScreen(path: PathBuf, on_init: EventHandler<()>) -> Element {
    let mut error = use_signal(|| None::<String>);

    rsx! {
        div { class: "welcome-screen",
            div { class: "welcome-card",
                div { class: "welcome-icon",
                    svg {
                        width: "64", height: "64", view_box: "0 0 24 24",
                        fill: "none", stroke: "currentColor", stroke_width: "1.5",
                        path { d: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" }
                    }
                }
                h1 { "Welcome to Ishoo" }
                p { class: "welcome-desc",
                    "No issue tracker found. Initialize one to get started."
                }
                p { class: "welcome-path", "{path.display()}" }

                if let Some(err) = error() {
                    div { class: "welcome-error", "{err}" }
                }

                button {
                    class: "btn-primary btn-lg",
                    onclick: move |_| {
                        match init_workspace_at(&path) {
                            Ok(()) => on_init.call(()),
                            Err(e) => error.set(Some(e)),
                        }
                    },
                    "Initialize Issue Tracker"
                }

                p { class: "welcome-hint",
                    "Creates: issues-active.md, issues-backlog.md, issues-done.md"
                }
            }
        }
    }
}
