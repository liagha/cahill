#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod media;
mod playback;
mod ui;

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
                        .with_resizable(false),
                ),
            )
            .launch(app::App);
    }

    #[cfg(not(feature = "desktop"))]
    {
        dioxus::launch(app::App);
    }
}