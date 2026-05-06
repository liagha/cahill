use dioxus::prelude::*;

const COVER_LIGHT: &[u8] = include_bytes!("../../assets/cover_light.jpg");
const COVER_DARK: &[u8] = include_bytes!("../../assets/cover_dark.jpg");

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

fn data_uri(bytes: &[u8]) -> String {
    format!("data:image/jpeg;base64,{}", base64_encode(bytes))
}

#[component]
pub fn CoverDisplay(cover: Option<Vec<u8>>, dark: bool, onclick: EventHandler<()>) -> Element {
    let fallback_src = if dark {
        data_uri(COVER_DARK)
    } else {
        data_uri(COVER_LIGHT)
    };

    match cover {
        Some(bytes) => {
            let src = data_uri(&bytes);
            rsx! {
                div { class: "cover", onclick: move |_| onclick.call(()), style: "cursor: pointer;",
                    img { class: "cover-art", src: "{src}" }
                }
            }
        }
        None => rsx! {
            div { class: "cover", onclick: move |_| onclick.call(()), style: "cursor: pointer;",
                img { class: "cover-art", src: "{fallback_src}" }
            }
        }
    }
}