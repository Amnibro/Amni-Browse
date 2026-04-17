use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderSettings {
    pub font_size: u32,
    pub font_family: String,
    pub theme: ReaderTheme,
    pub line_height: f32,
    pub max_width: u32,
}
impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            font_size: 18, font_family: "Georgia, 'Times New Roman', serif".into(),
            theme: ReaderTheme::Light, line_height: 1.8, max_width: 680,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReaderTheme { Light, Dark, Sepia }
impl ReaderTheme {
    pub fn css(&self) -> (&str, &str) {
        match self {
            Self::Light => ("#ffffff", "#1a1a1a"),
            Self::Dark => ("#1a1a2e", "#e0e0e0"),
            Self::Sepia => ("#f4ecd8", "#5b4636"),
        }
    }
}
pub struct ReaderMode {
    pub active: bool,
    pub settings: ReaderSettings,
}
impl ReaderMode {
    pub fn new() -> Self { Self { active: false, settings: ReaderSettings::default() } }
    pub fn toggle(&mut self) -> bool { self.active = !self.active; self.active }
    pub fn extraction_js() -> &'static str {
        r#"(function(){
var a=document.querySelector('article')||document.querySelector('[role="main"]')||document.querySelector('.post-content,.entry-content,.article-body,main,.content');
if(!a){var ps=document.querySelectorAll('p');var best=document.body;var bestLen=0;var containers=new Map();ps.forEach(function(p){var par=p.parentElement;if(!par)return;var len=(containers.get(par)||0)+p.textContent.length;containers.set(par,len);if(len>bestLen){bestLen=len;best=par;}});a=best;}
var t=document.title||'';var c=a?a.innerHTML:'';return JSON.stringify({title:t,content:c});})()"#
    }
    pub fn render_html(&self, title: &str, content: &str) -> String {
        let (bg, fg) = self.settings.theme.css();
        format!(r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{background:{bg};color:{fg};font-family:{ff};font-size:{fs}px;line-height:{lh};padding:40px 20px;display:flex;justify-content:center}}
.reader-container{{max-width:{mw}px;width:100%}}
h1{{font-size:2em;margin-bottom:0.5em;line-height:1.3}}
h2,h3,h4{{margin:1.2em 0 0.5em}}
p{{margin:0.8em 0}}
img{{max-width:100%;height:auto;border-radius:8px;margin:1em 0}}
a{{color:#0066cc;text-decoration:underline}}
blockquote{{border-left:3px solid #ccc;padding-left:1em;margin:1em 0;font-style:italic}}
pre,code{{background:rgba(128,128,128,0.1);padding:2px 6px;border-radius:4px;font-size:0.9em}}
pre{{padding:1em;overflow-x:auto}}
ul,ol{{margin:0.8em 0;padding-left:1.5em}}
.reader-exit{{position:fixed;top:20px;right:20px;background:#333;color:#fff;border:none;padding:8px 16px;border-radius:20px;cursor:pointer;font-size:14px;z-index:9999}}
.reader-exit:hover{{background:#555}}
</style></head><body>
<button class="reader-exit" onclick="window.ipc.postMessage(JSON.stringify({{type:'reader_toggle'}}))">✕ Exit Reader</button>
<div class="reader-container"><h1>{title}</h1>{content}</div></body></html>"#,
            bg=bg, fg=fg, ff=self.settings.font_family, fs=self.settings.font_size,
            lh=self.settings.line_height, mw=self.settings.max_width, title=title, content=content)
    }
    pub fn settings_json(&self) -> String {
        serde_json::to_string(&self.settings).unwrap_or_else(|_| "{}".into())
    }
}
