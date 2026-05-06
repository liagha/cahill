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

    let mut state_writer = state.clone();
    let mut meta_writer = meta.clone();
    let mut duration_writer = duration.clone();

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;
            let binding = player.read();
            let mut p = binding.lock().unwrap();
            if p.state() == State::Playing {
                elapsed.set(p.position().as_secs_f64());

                if p.finished() {
                    let next_path = p.queue.next().map(|t| t.meta.path.clone());
                    if let Some(path) = next_path {
                        let _ = p.open(&path);
                        if let Some(current_track) = p.queue.current() {
                            let meta_val = current_track.meta.clone();
                            meta_writer.set(Some(meta_val.clone()));
                            let dur = meta_val.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);
                            duration_writer.set(dur);
                            elapsed.set(0.0);
                            state_writer.set(State::Playing);
                        }
                    } else {
                        state_writer.set(State::Stopped);
                    }
                }
            }
        }
    });

    let toggle = {
        let player = player.clone();
        move |_| {
            let binding = player.read();
            let mut p = binding.lock().unwrap();
            p.toggle();
            state.set(p.state());
        }
    };

    let open_file = {
        let player = player.clone();
        move |_| {
            let picked = rfd::FileDialog::new()
                .add_filter("Audio", &["mp3", "flac", "wav", "ogg", "aac", "m4a"])
                .pick_file();

            if let Some(path) = picked {
                if let Some(track) = probe::load(&path) {
                    let meta_clone = track.meta.clone();
                    let dur = meta_clone.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);

                    let (new_index, open_path) = {
                        let binding = player.read();
                        let mut p = binding.lock().unwrap();
                        let idx = p.queue.len();
                        p.queue.push(track);
                        p.queue.jump_to(idx);
                        let current = p.queue.current().map(|t| t.meta.path.clone());
                        (idx, current)
                    };

                    if let Some(path) = open_path {
                        let binding = player.read();
                        let mut p = binding.lock().unwrap();
                        let _ = p.open(&path);
                        state.set(State::Playing);
                        meta.set(Some(meta_clone));
                        duration.set(dur);
                        elapsed.set(0.0);
                    }
                }
            }
        }
    };

    let prev = {
        let player = player.clone();
        move |_| {
            let binding = player.read();
            let mut p = binding.lock().unwrap();
            if let Some(path) = p.queue.prev().map(|t| t.meta.path.clone()) {
                p.open(&path).ok();
                if let Some(current) = p.queue.current() {
                    let meta_val = current.meta.clone();
                    meta.set(Some(meta_val.clone()));
                    duration.set(meta_val.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0));
                    elapsed.set(0.0);
                }
                state.set(State::Playing);
            }
        }
    };

    let next = {
        let player = player.clone();
        move |_| {
            let binding = player.read();
            let mut p = binding.lock().unwrap();
            if let Some(path) = p.queue.next().map(|t| t.meta.path.clone()) {
                p.open(&path).ok();
                if let Some(current) = p.queue.current() {
                    let meta_val = current.meta.clone();
                    meta.set(Some(meta_val.clone()));
                    duration.set(meta_val.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0));
                    elapsed.set(0.0);
                }
                state.set(State::Playing);
            }
        }
    };

    let ratio = if duration() > 0.0 {
        (elapsed() / duration()) * 100.0
    } else {
        0.0
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
                        style: "--seek-fill: {ratio}%",
                        oninput: move |evt| {
                            if let Ok(value) = evt.value().parse::<f64>() {
                                elapsed.set(value);
                                player.read().lock().unwrap().seek(Duration::from_secs_f64(value));
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