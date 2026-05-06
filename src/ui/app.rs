// src/ui/app.rs
use crate::player::{Player, PlayerCommand, PlayerEvent, PlayerState};
use crate::ui::controls::Controls;
use crate::ui::playlist::Playlist;
use crate::ui::seekbar::Seekbar;
use crate::ui::style;
use crate::media::MediaInfo;
use base64::Engine;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
async fn wait(ms: u64) {
    gloo_timers::future::TimeoutFuture::new(ms as u32).await;
}
#[cfg(not(target_arch = "wasm32"))]
async fn wait(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

#[component]
pub fn app() -> Element {
    let player_signal = use_signal(Player::new);
    let state_signal = use_signal(PlayerState::default);
    let playlist_signal = use_signal(Vec::<MediaInfo>::new);
    let current_path_signal = use_signal(|| Option::<String>::None);

    {
        let player = player_signal.clone();
        let mut state = state_signal.clone();
        let mut playlist = playlist_signal.clone();
        let mut current_path = current_path_signal.clone();
        use_coroutine(move |_rx: UnboundedReceiver<()>| {
            async move {
                loop {
                    while let Some(event) = player.read().try_recv() {
                        match event {
                            PlayerEvent::State(new_state) => {
                                if let Some(ref media) = new_state.media {
                                    *current_path.write() = Some(media.path.clone());
                                    let path = media.path.clone();
                                    playlist.write().retain(|m| m.path != path);
                                    playlist.write().push(media.clone());
                                }
                                *state.write() = new_state;
                            }
                            PlayerEvent::Loaded(media) => {
                                let path = media.path.clone();
                                playlist.write().retain(|m| m.path != path);
                                playlist.write().push(media.clone());
                                *current_path.write() = Some(path);
                            }
                            PlayerEvent::Ended => {}
                        }
                    }
                    wait(16).await;
                }
            }
        });
    }

    let state = state_signal.read().clone();
    let player_for_seekbar = player_signal.clone();

    let cover_src = state.media.as_ref().and_then(|m| m.cover.as_ref()).map(|cover| {
        let encoded = base64::engine::general_purpose::STANDARD.encode(&cover.data);
        format!("data:{};base64,{}", cover.mime_type, encoded)
    });

    rsx! {
        style { {style::css()} }
        div { class: "app",
            div { class: "artwork",
                if let Some(src) = &cover_src {
                    img {
                        src: "{src}",
                        style: "width:100%;height:100%;object-fit:cover;border-radius:16px;",
                    }
                } else {
                    div { class: "artwork_fallback", "🎵" }
                }
            }

            div { class: "track_info",
                if let Some(ref media) = state.media {
                    div { class: "title", "{media.title}" }
                    div { class: "artist", "{media.artist} · {media.album}" }
                } else {
                    div { class: "title", "No Track" }
                    div { class: "artist", "Add some music to get started" }
                }
            }

            Seekbar {
                state: state.clone(),
                player: player_for_seekbar.clone(),
            }

            Controls {
                player_signal: player_signal.clone(),
                state: state.clone(),
            }

            div { class: "toolbar",
                button {
                    onclick: {
                        let sender = player_signal.read().sender.clone();
                        move |_| {
                            let sender = sender.clone();
                            spawn(async move {
                                let file = rfd::AsyncFileDialog::new()
                                    .add_filter("Audio", &["mp3", "wav", "flac", "m4a", "ogg"])
                                    .pick_file()
                                    .await;
                                if let Some(handle) = file {
                                    let path = handle.path().to_string_lossy().to_string();
                                    sender.send(PlayerCommand::Load(path));
                                }
                            });
                        }
                    },
                    "Open File"
                }
                button {
                    onclick: {
                        let player = player_signal.clone();
                        let playlist = playlist_signal.clone();
                        move |_| {
                            let player = player.clone();
                            let mut playlist = playlist.clone();
                            spawn(async move {
                                let folder = rfd::AsyncFileDialog::new().pick_folder().await;
                                if let Some(handle) = folder {
                                    let dir = handle.path().to_string_lossy().to_string();
                                    let tracks = crate::library::scan_directory(&dir);
                                    for track in tracks {
                                        playlist.write().retain(|m| m.path != track.path);
                                        playlist.write().push(track.clone());
                                    }
                                    if let Some(first) = playlist.read().first() {
                                        player.read().send(PlayerCommand::Load(first.path.clone()));
                                    }
                                }
                            });
                        }
                    },
                    "Scan Folder"
                }
            }

            Playlist {
                list: playlist_signal,
                current_path: current_path_signal,
                on_select: move |path: String| {
                    player_signal.read().send(PlayerCommand::Load(path));
                },
            }
        }
    }
}