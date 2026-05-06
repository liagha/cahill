use crate::player::{Player, PlayerCommand, PlayerState};
use dioxus::prelude::*;

#[component]
pub fn Controls(player_signal: Signal<Player>, state: PlayerState) -> Element {
    let mut volume = use_signal(|| state.volume);

    rsx! {
        div { class: "controls",
            button {
                onclick: {
                    let sender = player_signal.read().sender.clone();
                    move |_| { sender.send(PlayerCommand::Play).ok(); }
                },
                "▶"
            }
            button {
                onclick: {
                    let sender = player_signal.read().sender.clone();
                    move |_| { sender.send(PlayerCommand::Pause).ok(); }
                },
                "⏸"
            }
            button {
                onclick: {
                    let sender = player_signal.read().sender.clone();
                    move |_| { sender.send(PlayerCommand::Stop).ok(); }
                },
                "⏹"
            }
        }
        div { class: "volume",
            span { "🔊" }
            input {
                r#type: "range",
                min: "0",
                max: "1",
                step: "0.01",
                value: "{volume}",
                oninput: {
                    let sender = player_signal.read().sender.clone();
                    move |evt| {
                        if let Ok(val) = evt.value().parse::<f32>() {
                            volume.set(val);
                            sender.send(PlayerCommand::Volume(val)).ok();
                        }
                    }
                }
            }
        }
    }
}