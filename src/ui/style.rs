// src/ui/style.rs
pub fn css() -> &'static str {
    r#"
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}
body {
    background: #f5f5f7;
    color: #1d1d1f;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    margin: 0;
}
.app {
    width: 400px;
    background: rgba(255,255,255,0.8);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
    border-radius: 24px;
    padding: 28px 24px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.08);
    display: flex;
    flex-direction: column;
    gap: 20px;
}
.artwork {
    width: 100%;
    height: 220px;
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 56px;
    color: #8e8e93;
    box-shadow: 0 2px 8px rgba(0,0,0,0.05);
    overflow: hidden;
    background: linear-gradient(135deg, #e2e2e2, #f0f0f0);
}
.artwork_fallback {
    font-size: 56px;
    color: #8e8e93;
}
.track_info {
    text-align: center;
    margin-top: 4px;
}
.track_info .title {
    font-size: 22px;
    font-weight: 600;
    line-height: 1.3;
    word-break: break-word;
}
.track_info .artist {
    font-size: 15px;
    color: #6e6e73;
    margin-top: 4px;
}
.controls {
    display: flex;
    justify-content: center;
    gap: 16px;
    margin: 8px 0 4px;
}
.controls button {
    background: none;
    border: none;
    font-size: 28px;
    color: #1d1d1f;
    cursor: pointer;
    transition: color 0.2s, transform 0.1s;
    padding: 8px;
    border-radius: 50%;
}
.controls button:hover {
    color: #ff3b30;
}
.controls button:active {
    transform: scale(0.9);
}
.volume {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 8px;
    color: #8e8e93;
    font-size: 18px;
}
.volume input[type="range"] {
    flex: 1;
    accent-color: #ff3b30;
    height: 4px;
}
.seekbar {
    display: flex;
    flex-direction: column;
    gap: 4px;
}
.seekbar input[type="range"] {
    width: 100%;
    accent-color: #ff3b30;
    height: 4px;
}
.seekbar .time {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: #8e8e93;
}
.toolbar {
    display: flex;
    gap: 8px;
    justify-content: center;
}
.toolbar button {
    background: transparent;
    border: 1px solid #d2d2d7;
    color: #1d1d1f;
    padding: 6px 14px;
    border-radius: 20px;
    font-size: 13px;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s;
}
.toolbar button:hover {
    background: #e5e5ea;
    border-color: #c7c7cc;
}
.playlist {
    list-style: none;
    background: rgba(255,255,255,0.7);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-radius: 16px;
    padding: 8px 0;
    max-height: 180px;
    overflow-y: auto;
    border: 1px solid rgba(0,0,0,0.05);
}
.playlist li {
    padding: 10px 16px;
    cursor: pointer;
    transition: background 0.2s;
    border-radius: 8px;
    margin: 0 8px;
    font-size: 14px;
    font-weight: 500;
    color: #1d1d1f;
}
.playlist li:hover {
    background: rgba(0,0,0,0.05);
}
.playlist li.active {
    background: rgba(255,59,48,0.1);
    color: #ff3b30;
}
    "#
}