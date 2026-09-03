use serde_json::Value;
#[derive(Debug, Clone)]
pub struct Progressive {
    pub page_url: String,
    pub title: String,
    pub media_url: String,
    pub mime: String,
    pub quality: String,
}
pub fn youtube_id(url: &str) -> Option<String> {
    let l = url;
    let take = |s: &str| -> Option<String> {
        let id: String = s.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect();
        (id.len() == 11).then_some(id)
    };
    if let Some(i) = l.find("youtu.be/") { return take(&l[i + 9..]); }
    for key in ["v=", "vi="] {
        if let Some(i) = l.find(key) { return take(&l[i + key.len()..]); }
    }
    for key in ["/embed/", "/shorts/", "/live/"] {
        if let Some(i) = l.find(key) { return take(&l[i + key.len()..]); }
    }
    None
}
pub fn is_progressive_host(url: &str) -> bool { youtube_id(url).is_some() }
fn pick_format(formats: &Value) -> Option<(String, String, String)> {
    let arr = formats.as_array()?;
    let mut best: Option<(i64, String, String, String)> = None;
    for f in arr {
        let url = f.get("url").and_then(|v| v.as_str()).unwrap_or("");
        if url.is_empty() { continue; }
        if f.get("audioQuality").is_none() && f.get("audioSampleRate").is_none() { continue; }
        let h = f.get("height").and_then(|v| v.as_i64()).unwrap_or(0);
        let mime = f.get("mimeType").and_then(|v| v.as_str()).unwrap_or("video/mp4").split(';').next().unwrap_or("video/mp4").to_string();
        let q = f.get("qualityLabel").and_then(|v| v.as_str()).unwrap_or("progressive").to_string();
        let score = h;
        if best.as_ref().map(|b| score > b.0).unwrap_or(true) { best = Some((score, url.to_string(), mime, q)); }
    }
    best.map(|(_, u, m, q)| (u, m, q))
}
pub fn extract(url: &str) -> Result<Progressive, String> {
    let id = youtube_id(url).ok_or_else(|| "not a progressive extract host".to_string())?;
    let body = serde_json::json!({"context":{"client":{"clientName":"ANDROID_VR","clientVersion":"1.60.19","hl":"en","gl":"US"}},"videoId":id,"contentCheckOk":true,"racyCheckOk":true});
    let client = reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(12)).user_agent("com.google.android.apps.youtube.vr.oculus/1.60.19 (Linux; U; Android 12) gzip").build().map_err(|e| e.to_string())?;
    let resp = client.post("https://www.youtube.com/youtubei/v1/player?prettyPrint=false").header("Content-Type", "application/json").json(&body).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() { return Err(format!("player http {}", resp.status())); }
    let v: Value = resp.json().map_err(|e| e.to_string())?;
    let title = v.pointer("/videoDetails/title").and_then(|x| x.as_str()).unwrap_or("YouTube").to_string();
    let formats = v.pointer("/streamingData/formats").cloned().unwrap_or(Value::Array(vec![]));
    let (media_url, mime, quality) = pick_format(&formats).ok_or_else(|| "no muxed progressive format (adaptive-only; Servo has no MSE yet)".to_string())?;
    Ok(Progressive { page_url: url.to_string(), title, media_url, mime, quality })
}
pub fn player_html(hit: &Progressive, theme_vars: &str) -> String {
    let title = esc(&hit.title);
    let media = esc(&hit.media_url);
    let page = esc(&hit.page_url);
    let mime = esc(&hit.mime);
    let q = esc(&hit.quality);
    format!("<!DOCTYPE html><html><head><meta charset='utf-8'><title>{title}</title><style>:root{{{theme}}}body{{margin:0;background:var(--bg,#08090B);color:var(--text,#EDEFF2);font:15px/1.45 -apple-system,'Segoe UI',sans-serif}}header{{padding:14px 18px 8px;border-bottom:1px solid var(--stroke,#20242B)}}h1{{margin:0;font-size:18px;color:var(--accent,#C89B4E)}}p{{margin:6px 0 0;color:var(--dim,#A7ADB6);font-size:12px}}video{{display:block;width:100%;max-height:calc(100vh - 88px);background:#000}}a{{color:var(--accent,#C89B4E)}}</style></head><body><header><h1>{title}</h1><p>Amni progressive player · {q} · {mime} · Servo &lt;video&gt; · <a href='{page}'>original page</a></p></header><video controls autoplay playsinline src='{media}'></video></body></html>", theme = theme_vars)
}
fn esc(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;") }
pub fn try_player_html(url: &str, theme_vars: &str) -> Option<String> {
    if !is_progressive_host(url) { return None; }
    match extract(url) {
        Ok(hit) => { log::info!("stream_extract ok {} → {} {}", hit.title, hit.quality, hit.mime); Some(player_html(&hit, theme_vars)) }
        Err(e) => { log::warn!("stream_extract miss {}: {}", url, e); None }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_watch_short_embed() {
        assert_eq!(youtube_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").as_deref(), Some("dQw4w9WgXcQ"));
        assert_eq!(youtube_id("https://youtu.be/dQw4w9WgXcQ").as_deref(), Some("dQw4w9WgXcQ"));
        assert_eq!(youtube_id("https://www.youtube.com/embed/dQw4w9WgXcQ").as_deref(), Some("dQw4w9WgXcQ"));
        assert_eq!(youtube_id("https://www.youtube.com/shorts/dQw4w9WgXcQ").as_deref(), Some("dQw4w9WgXcQ"));
        assert!(youtube_id("https://vimeo.com/123").is_none());
    }
}
