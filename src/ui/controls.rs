use dioxus::prelude::*;
use crate::playback::engine::State;

#[component]
pub fn Controls(
    state: State,
    on_toggle: EventHandler<()>,
    on_prev: EventHandler<()>,
    on_next: EventHandler<()>,
) -> Element {
    let playing = state == State::Playing;

    rsx! {
        div { class: "actions",
            button {
                class: "action",
                onclick: move |_| on_prev.call(()),
                svg {
                    view_box: "0 0 24 24", width: "22", height: "22",
                    fill: "none", stroke: "currentColor",
                    stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                    polyline { points: "11 17 6 12 11 7" }
                    polyline { points: "18 17 13 12 18 7" }
                }
            }
            button {
                class: "action play",
                onclick: move |_| on_toggle.call(()),
                if playing {
                    svg {
                        view_box: "0 0 24 24", width: "28", height: "28",
                        fill: "currentColor", stroke: "none",
                        rect { x: "6", y: "4", width: "4", height: "16" }
                        rect { x: "14", y: "4", width: "4", height: "16" }
                    }
                } else {
                    svg {
                        view_box: "0 0 24 24", width: "28", height: "28",
                        fill: "currentColor", stroke: "none",
                        polygon { points: "8 5 19 12 8 19" }
                    }
                }
            }
            button {
                class: "action",
                onclick: move |_| on_next.call(()),
                svg {
                    view_box: "0 0 24 24", width: "22", height: "22",
                    fill: "none", stroke: "currentColor",
                    stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                    polyline { points: "13 17 18 12 13 7" }
                    polyline { points: "6 17 11 12 6 7" }
                }
            }
        }
    }
}