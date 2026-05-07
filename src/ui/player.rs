use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::media::{probe, Meta};
use crate::playback::{engine::State, Player};
use super::{controls::Controls, cover::CoverDisplay, meta::MetaDisplay};

fn format_time(raw: f64) -> String {
    let mins = (raw / 60.0) as i32;
    let secs = (raw % 60.0) as i32;
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

    let player_clone = player.clone();
    use_future(move || async move {
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        loop {
            interval.tick().await;
            let binding = player_clone.read();
            let p = binding.lock().unwrap();

            if p.state() != State::Playing {
                continue;
            }

            let pos = p.position().as_secs_f64();
            if (pos - elapsed()).abs() > 0.01 {
                elapsed.set(pos);
            }

            if p.finished() && !p.queue.is_empty() {
                drop(p);
                drop(binding);
                let binding = player_clone.read();
                let mut p = binding.lock().unwrap();
                p.queue.advance();
                if let Some(current) = p.queue.current() {
                    let path = current.meta.path.clone();
                    let meta_val = current.meta.clone();
                    let dur = meta_val.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);
                    let _ = p.open(&path);
                    meta.set(Some(meta_val));
                    duration.set(dur);
                    elapsed.set(0.0);
                    state.set(State::Playing);
                }
            }
        }
    });

    let player_clone = player.clone();
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_millis(16)).await;
            let dur = duration();
            if dur > 0.0 {
                let fill = (elapsed() / dur) * 100.0;
                seek_fill.set(fill);
            }
        }
    });

    let open_file = {
        let player = player.clone();
        move |_| {
            let picked = rfd::FileDialog::new()
                .add_filter("Audio", &["mp3", "flac", "wav", "ogg", "aac", "m4a"])
                .pick_file();

            if let Some(path) = picked {
                if let Some(track) = probe::load(&path) {
                    let binding = player.read();
                    let mut p = binding.lock().unwrap();
                    let index = p.queue.len();
                    p.queue.push(track);
                    p.queue.set_cursor(index);

                    if let Some(current) = p.queue.current() {
                        let meta_val = current.meta.clone();
                        let dur = meta_val.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);
                        let path = current.meta.path.clone();

                        let _ = p.open(&path);
                        meta.set(Some(meta_val));
                        duration.set(dur);
                        elapsed.set(0.0);
                        state.set(State::Playing);
                    }
                }
            }
        }
    };

    let switch_track = move |player: Signal<Arc<Mutex<Player>>>, direction: i32| {
        let binding = player.read();
        let mut p = binding.lock().unwrap();
        if direction < 0 {
            p.queue.retreat();
        } else {
            p.queue.advance();
        }
        if let Some(current) = p.queue.current() {
            let path = current.meta.path.clone();
            let meta_val = current.meta.clone();
            let dur = meta_val.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);

            let _ = p.open(&path);
            meta.set(Some(meta_val));
            duration.set(dur);
            elapsed.set(0.0);
            state.set(State::Playing);
        }
    };

    let toggle = {
        let player = player.clone();
        move |_| {
            let binding = player.read();
            let mut p = binding.lock().unwrap();
            if let Ok(new_state) = p.toggle() {
                state.set(new_state);
            }
        }
    };

    let prev = {
        let player = player.clone();
        let mut switch = switch_track.clone();
        move |_| switch(player.clone(), -1)
    };

    let next = {
        let player = player.clone();
        let mut switch = switch_track.clone();
        move |_| switch(player.clone(), 1)
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
                    onclick: open_file,
                }

                MetaDisplay { meta }

                div { class: "seek",
                    input {
                        r#type: "range",
                        class: "seek-bar",
                        min: "0",
                        max: "{duration()}",
                        value: "{elapsed()}",
                        style: "--seek-fill: {seek_fill}%",
                        oninput: move |evt| {
                            if let Ok(value) = evt.value().parse::<f64>() {
                                let binding = player.read();
                                binding.lock().unwrap().seek(Duration::from_secs_f64(value));
                            }
                        }
                    }
                    div { class: "timestamps",
                        span { "{format_time(elapsed())}" }
                        span { "{format_time(duration())}" }
                    }
                }

                Controls {
                    state,
                    on_toggle: toggle,
                    on_prev: prev,
                    on_next: next,
                }
            }
        }
    }
}