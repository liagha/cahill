use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::media::{probe, Meta};
use crate::playback::{engine::State, Player};
use super::{controls::Controls, cover::CoverDisplay, meta::MetaDisplay};

fn format_time(raw: f64) -> String {
    let total_secs = raw as u64;
    let mins = (total_secs / 60) as u32;
    let secs = (total_secs % 60) as u32;
    format!("{:02}:{:02}", mins, secs)
}

#[component]
pub fn PlayerCard(player: Signal<Arc<Mutex<Player>>>) -> Element {
    let mut dark = use_signal(|| false);
    let mut state = use_signal(|| State::Stopped);
    let mut meta = use_signal(|| Option::<Meta>::None);
    let mut elapsed = use_signal(|| 0.0f64);
    let mut duration = use_signal(|| 0.0f64);
    let mut seek_fill = use_signal(|| 0.0f64);
    let mut seeking = use_signal(|| false);

    use_future({
        let player = player.clone();
        move || async move {
            let mut tick = tokio::time::interval(Duration::from_millis(100));
            loop {
                tick.tick().await;
                let binding = player.read();
                let mut p = binding.lock().unwrap();

                let current_state = p.state();
                state.set(current_state.clone());

                if !seeking() {
                    let pos = p.position().as_secs_f64();
                    elapsed.set(pos);

                    let dur = duration();
                    if dur > 0.0 {
                        seek_fill.set((pos / dur) * 100.0);
                    }
                }

                if p.finished() && !p.queue.is_empty() {
                    if let Some(next) = p.queue.advance() {
                        let meta_clone = next.meta.clone();
                        let dur = meta_clone.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);
                        let path = next.meta.path.clone();
                        let _ = p.open(&path);
                        drop(p);
                        meta.set(Some(meta_clone));
                        duration.set(dur);
                        elapsed.set(0.0);
                        seek_fill.set(0.0);
                        state.set(State::Playing);
                        continue;
                    }
                }
            }
        }
    });

    let load_track = {
        let player = player.clone();
        move |file_path: &std::path::Path| {
            if let Some(track) = probe::load(file_path) {
                let binding = player.read();
                let mut p = binding.lock().unwrap();
                let idx = p.queue.len();
                p.queue.push(track);
                p.queue.set_cursor(idx);

                if let Some(current) = p.queue.current() {
                    let meta_clone = current.meta.clone();
                    let dur = meta_clone.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);
                    let path = current.meta.path.clone();
                    let _ = p.open(&path);
                    meta.set(Some(meta_clone));
                    duration.set(dur);
                    elapsed.set(0.0);
                    seek_fill.set(0.0);
                    state.set(State::Playing);
                }
            }
        }
    };

    let mut open_file = {
        let mut load_track = load_track.clone();
        move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio", &["mp3", "flac", "wav", "ogg", "aac", "m4a"])
                .pick_file()
            {
                load_track(&path);
            }
        }
    };

    let switch = {
        let player = player.clone();
        move |direction: i32| {
            let binding = player.read();
            let mut p = binding.lock().unwrap();
            let jumped = if direction < 0 {
                p.queue.retreat()
            } else {
                p.queue.advance()
            };
            if let Some(current) = jumped.cloned().or_else(|| p.queue.current().cloned()) {
                let path = current.meta.path.clone();
                let meta_clone = current.meta.clone();
                let dur = meta_clone.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);
                let _ = p.open(&path);
                meta.set(Some(meta_clone));
                duration.set(dur);
                elapsed.set(0.0);
                seek_fill.set(0.0);
                state.set(State::Playing);
            }
        }
    };

    let toggle = {
        let player = player.clone();
        move |_| {
            let binding = player.read();
            let mut p = binding.lock().unwrap();
            if let Ok(s) = p.toggle() {
                state.set(s);
            }
        }
    };

    let prev = {
        let mut switch = switch.clone();
        move |_| switch(-1)
    };

    let next = {
        let mut switch = switch.clone();
        move |_| switch(1)
    };

    let seek_start = {
        let mut seeking = seeking.clone();
        move |_| {
            seeking.set(true);
        }
    };

    let mut seek_move = {
        let mut elapsed = elapsed.clone();
        let mut seek_fill = seek_fill.clone();
        let duration = duration.clone();
        move |value: f64| {
            elapsed.set(value);
            let dur = duration();
            if dur > 0.0 {
                seek_fill.set((value / dur) * 100.0);
            }
        }
    };

    let mut seek_end = {
        let player = player.clone();
        let mut seeking = seeking.clone();
        move |value: f64| {
            let binding = player.read();
            let mut p = binding.lock().unwrap();
            p.seek(Duration::from_secs_f64(value));
            let actual_pos = p.position().as_secs_f64();
            elapsed.set(actual_pos);
            let dur = duration();
            if dur > 0.0 {
                seek_fill.set((actual_pos / dur) * 100.0);
            }
            seeking.set(false);
        }
    };

    let cover_data = meta.read().as_ref().and_then(|m| m.cover.clone());

    rsx! {
        div {
            class: "shell",
            class: if dark() { "dark" },

            div { class: "card",
                div { class: "theme-toggle",
                    button {
                        class: "action",
                        onclick: move |_| dark.toggle(),
                        if dark() {
                            svg {
                                view_box: "0 0 24 24",
                                width: "20",
                                height: "20",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
                            }
                        } else {
                            svg {
                                view_box: "0 0 24 24",
                                width: "20",
                                height: "20",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                circle { cx: "12", cy: "12", r: "5" }
                                line { x1: "12", y1: "1", x2: "12", y2: "3" }
                                line { x1: "12", y1: "21", x2: "12", y2: "23" }
                                line { x1: "4.22", y1: "4.22", x2: "5.64", y2: "5.64" }
                                line { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }
                                line { x1: "1", y1: "12", x2: "3", y2: "12" }
                                line { x1: "21", y1: "12", x2: "23", y2: "12" }
                                line { x1: "4.22", y1: "19.78", x2: "5.64", y2: "18.36" }
                                line { x1: "18.36", y1: "5.64", x2: "19.78", y2: "4.22" }
                            }
                        }
                    }
                }

                CoverDisplay {
                    cover: cover_data,
                    dark: dark(),
                    onclick: move |_| open_file(),
                }

                MetaDisplay { meta: meta.read().clone() }

                div { class: "seek",
                    input {
                        r#type: "range",
                        class: "seek-bar",
                        min: "0",
                        max: "{duration()}",
                        value: "{elapsed()}",
                        step: "0.01",
                        style: "--seek-fill: {seek_fill}%",
                        onmousedown: seek_start,
                        oninput: move |evt| {
                            if let Ok(v) = evt.value().parse::<f64>() {
                                seek_move(v);
                            }
                        },
                        onchange: move |evt| {
                            if let Ok(v) = evt.value().parse::<f64>() {
                                seek_end(v);
                            }
                        }
                    }
                    div { class: "timestamps",
                        span { "{format_time(elapsed())}" }
                        span { "{format_time(duration())}" }
                    }
                }

                Controls {
                    state: state(),
                    on_toggle: toggle,
                    on_prev: prev,
                    on_next: next,
                }
            }
        }
    }
}