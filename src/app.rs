#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::prelude::*;
use std::sync::{Arc, Mutex};

use crate::playback::Player;
use crate::ui::player::PlayerCard;

#[component]
pub fn App() -> Element {
    let player = use_signal(|| Arc::new(Mutex::new(Player::new().expect("audio engine failed"))));

    rsx! {
        style { {include_str!("style.css")} }
        PlayerCard { player }
    }
}