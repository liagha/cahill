use crate::player::{PlayerCommand, PlayerState};
use dioxus::prelude::*;

fn format_time(dur: std::time::Duration) -> String {
    let secs = dur.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

#[component]
pub fn Seekbar(state: PlayerState, player: Signal<crate::player::Player>) -> Element {
    let total_secs = state.duration.as_secs_f64().max(0.0);
    let current_secs = state.position.as_secs_f64();

    rsx! {
        div { class: "seekbar",
            input {
                r#type: "range",
                min: "0",
                max: "{total_secs}",
                step: "0.01",
                value: "{current_secs}",
                oninput: move |evt| {
                    if let Ok(val) = evt.value().parse::<f64>() {
                        let dur = std::time::Duration::from_secs_f64(val);
                        player.read().send(PlayerCommand::Seek(dur));
                    }
                }
            }
            div { class: "time",
                span { "{format_time(state.position)}" }
                span { "{format_time(state.duration)}" }
            }
        }
    }
}