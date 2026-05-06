use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::media::{probe, Meta};
use crate::playback::{engine::State, Player};
use super::{controls::Controls, meta::MetaDisplay};

fn format_time(raw: f64) -> String {
    let mins = (raw / 60.0) as i32;
    let secs = (raw % 60.0) as i32;
    format!("{:02}:{:02}", mins, secs)
}

fn cover_src(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut encoded = String::new();
    let b64 = base64_encode(bytes);
    write!(encoded, "data:image/jpeg;base64,{}", b64).unwrap();
    encoded
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[(n >> 18) & 0x3f] as char);
        out.push(CHARS[(n >> 12) & 0x3f] as char);
        out.push(if chunk.len() > 1 { CHARS[(n >> 6) & 0x3f] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[n & 0x3f] as char } else { '=' });
    }
    out
}

#[component]
pub fn PlayerCard(player: Signal<Arc<Mutex<Player>>>) -> Element {
    let mut state = use_signal(|| State::Stopped);
    let mut meta = use_signal(|| Option::<Meta>::None);
    let mut elapsed = use_signal(|| 0.0f64);
    let mut duration = use_signal(|| 0.0f64);

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;
            if *state.read() == State::Playing {
                let pos = player.read().lock().unwrap().position().as_secs_f64();
                elapsed.set(pos);
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
                    let secs = track.meta.duration
                        .map(|d| d.as_secs_f64())
                        .unwrap_or(0.0);
                    duration.set(secs);
                    elapsed.set(0.0);
                    meta.set(Some(track.meta.clone()));
                    player.read().lock().unwrap().open(&path).ok();
                    state.set(State::Playing);
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
                state.set(State::Playing);
            }
        }
    };

    let cover = meta.read().as_ref().and_then(|m| m.cover.clone());

    rsx! {
        div { class: "card",
            div { class: "cover", onclick: open_file, style: "cursor: pointer;",
                match cover {
                    Some(bytes) => rsx! {
                        img {
                            class: "cover-art",
                            src: cover_src(&bytes),
                        }
                    },
                    None => rsx! {
                        div { class: "cover-fallback" }
                    },
                }
            }

            MetaDisplay { meta }

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