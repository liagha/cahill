// src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::prelude::*;

fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
        LaunchBuilder::new()
            .with_cfg(
                Config::default().with_window(
                    WindowBuilder::new()
                        .with_title("cahill")
                        .with_inner_size(LogicalSize::new(420, 640))
                        .with_decorations(false)
                        .with_resizable(false),
                ),
            )
            .launch(App);
    }

    #[cfg(not(feature = "desktop"))]
    {
        dioxus::launch(App);
    }
}

#[component]
fn App() -> Element {
    let mut playing = use_signal(|| false);
    let mut elapsed = use_signal(|| 0.0f64);
    let duration = use_signal(|| 180.0f64);
    let track = use_signal(|| "Untitled Track".to_string());

    use_future(move || async move {
        if playing() {
            loop {
                gloo_timers::future::TimeoutFuture::new(1000).await;
                if elapsed() < duration() {
                    elapsed.set(elapsed() + 1.0);
                } else {
                    playing.set(false);
                    elapsed.set(0.0);
                }
            }
        }
    });

    let toggle = move |_| {
        if elapsed() >= duration() {
            elapsed.set(0.0);
        }
        playing.set(!playing());
    };

    let rewind = move |_| {
        elapsed.set((elapsed() - 10.0).max(0.0));
    };

    let forward = move |_| {
        elapsed.set((elapsed() + 10.0).min(duration()));
    };

    rsx! {
        style { {include_str!("style.css")} }

        div { class: "shell",
            div { class: "card",
                div { class: "cover",
                    div { class: "cover-fallback" }
                }

                div { class: "meta",
                    h2 { class: "title", "{track}" }
                    p { class: "subtitle", "Unknown Artist" }
                }

                div { class: "seek",
                    input {
                        r#type: "range",
                        class: "seek-bar",
                        min: "0",
                        max: "{duration()}",
                        value: "{elapsed()}",
                        oninput: move |evt| {
                            if let Ok(value) = evt.value().parse::<f64>() {
                                elapsed.set(value);
                            }
                        }
                    }
                    div { class: "timestamps",
                        span { "{format_time(elapsed())}" }
                        span { "{format_time(duration())}" }
                    }
                }

                div { class: "actions",
                    button { class: "action", onclick: rewind,
                        svg {
                            view_box: "0 0 24 24",
                            width: "22",
                            height: "22",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            polyline { points: "11 17 6 12 11 7" }
                            polyline { points: "18 17 13 12 18 7" }
                        }
                    }
                    button { class: "action play", onclick: toggle,
                        if playing() {
                            svg {
                                view_box: "0 0 24 24",
                                width: "28",
                                height: "28",
                                fill: "currentColor",
                                stroke: "none",
                                rect { x: "6", y: "4", width: "4", height: "16" }
                                rect { x: "14", y: "4", width: "4", height: "16" }
                            }
                        } else {
                            svg {
                                view_box: "0 0 24 24",
                                width: "28",
                                height: "28",
                                fill: "currentColor",
                                stroke: "none",
                                polygon { points: "8 5 19 12 8 19" }
                            }
                        }
                    }
                    button { class: "action", onclick: forward,
                        svg {
                            view_box: "0 0 24 24",
                            width: "22",
                            height: "22",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            polyline { points: "13 17 18 12 13 7" }
                            polyline { points: "6 17 11 12 6 7" }
                        }
                    }
                }
            }
        }
    }
}

fn format_time(raw: f64) -> String {
    let mins = (raw / 60.0) as i32;
    let secs = (raw % 60.0) as i32;
    format!("{:02}:{:02}", mins, secs)
}