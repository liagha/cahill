use crate::media::MediaInfo;
use dioxus::prelude::*;

#[component]
pub fn Playlist(
    list: Signal<Vec<MediaInfo>>,
    current_path: Signal<Option<String>>,
    on_select: EventHandler<String>,
) -> Element {
    let items = list.read();
    let current = current_path.read();

    rsx! {
        ul { class: "playlist",
            for item in items.iter() {
                {
                    let path = item.path.clone();
                    let is_active = current.as_ref().map_or(false, |p| p == &path);
                    rsx! {
                        li {
                            class: if is_active { "active" },
                            onclick: move |_| on_select.call(path.clone()),
                            "{item.title}",
                            div { style: "font-size: 11px; color: #8e8e93;", "{item.artist} · {item.album}" }
                        }
                    }
                }
            }
        }
    }
}