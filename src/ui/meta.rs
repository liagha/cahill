use dioxus::prelude::*;
use crate::media::Meta;

#[component]
pub fn MetaDisplay(meta: ReadSignal<Option<Meta>>) -> Element {
    let title = meta.read().as_ref()
        .map(|m| m.display_title().to_string())
        .unwrap_or_else(|| "No Track".into());
    let artist = meta.read().as_ref()
        .map(|m| m.display_artist().to_string())
        .unwrap_or_default();

    rsx! {
        div { class: "meta",
            h2 { class: "title", "{title}" }
            p { class: "subtitle", "{artist}" }
        }
    }
}