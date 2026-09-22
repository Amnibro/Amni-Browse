//! Ask-AI: hand a query to the AI chat the user already pays for / is signed into.
//! Nothing goes through Amni: the browser just opens the provider's own site with
//! the prompt prefilled (URL `q=` where the site supports it, otherwise a page
//! script that types the prompt into the composer once it appears).
use crate::storage::config::BrowserConfig;
pub struct AiProvider {
    pub id: &'static str,
    pub name: &'static str,
    /// Chat URL with `%s` for the encoded query. Empty = open `home` and inject.
    pub url: &'static str,
    /// Landing page used for injection and for the toolbar button with an empty query.
    pub home: &'static str,
    /// The site only fills the composer; the user still presses Enter.
    pub fill_only: bool,
    /// The site ignores `q=`; type the prompt in with a page script instead.
    pub inject: bool,
    pub note: &'static str,
}
pub const PROVIDERS: &[AiProvider] = &[
    AiProvider { id: "claude", name: "Claude", url: "https://claude.ai/new?q=%s", home: "https://claude.ai/new", fill_only: true, inject: false, note: "Prefills the prompt; press Enter to send." },
    AiProvider { id: "chatgpt", name: "ChatGPT", url: "https://chatgpt.com/?q=%s", home: "https://chatgpt.com/", fill_only: false, inject: false, note: "Sends the prompt on open." },
    AiProvider { id: "chatgpt-search", name: "ChatGPT (web search)", url: "https://chatgpt.com/?q=%s&hints=search", home: "https://chatgpt.com/", fill_only: false, inject: false, note: "Sends the prompt with web search on." },
    AiProvider { id: "perplexity", name: "Perplexity", url: "https://www.perplexity.ai/search?q=%s", home: "https://www.perplexity.ai/", fill_only: false, inject: false, note: "Runs the search on open." },
    AiProvider { id: "grok", name: "Grok", url: "https://grok.com/?q=%s", home: "https://grok.com/", fill_only: false, inject: false, note: "Sends the prompt on open." },
    AiProvider { id: "gemini", name: "Google Gemini", url: "", home: "https://gemini.google.com/app", fill_only: false, inject: true, note: "Gemini has no prompt link; Amni types the prompt into the composer for you." },
    AiProvider { id: "copilot", name: "Microsoft Copilot", url: "", home: "https://copilot.microsoft.com/", fill_only: false, inject: true, note: "Copilot dropped its prompt link; Amni types the prompt into the composer for you." },
    AiProvider { id: "aimode", name: "Google AI Mode", url: "https://www.google.com/search?udm=50&q=%s", home: "https://www.google.com/search?udm=50", fill_only: false, inject: false, note: "Google's AI Mode results." },
    AiProvider { id: "duckai", name: "Duck.ai", url: "https://duckduckgo.com/?ia=chat&q=%s", home: "https://duck.ai/", fill_only: false, inject: false, note: "DuckDuckGo's private AI chat." },
    AiProvider { id: "mistral", name: "Mistral Le Chat", url: "https://chat.mistral.ai/chat?q=%s", home: "https://chat.mistral.ai/chat", fill_only: false, inject: false, note: "Opens Le Chat with the prompt." },
    AiProvider { id: "kagi", name: "Kagi Assistant", url: "https://kagi.com/assistant?q=%s", home: "https://kagi.com/assistant", fill_only: false, inject: false, note: "Needs a Kagi subscription; sends on open." },
];
pub fn provider(id: &str) -> &'static AiProvider { PROVIDERS.iter().find(|p| p.id == id).unwrap_or(&PROVIDERS[0]) }
pub fn provider_name(cfg: &BrowserConfig) -> String {
    match cfg.ai_provider.as_str() { "custom" => "AI".to_string(), id => provider(id).name.to_string() }
}
/// Page script that waits for the chat composer, types `q`, and submits.
fn inject_script(q: &str) -> String {
    let jq = serde_json::to_string(q).unwrap_or_else(|_| "\"\"".into());
    format!(r#"(function(){{var q={jq},t0=Date.now();function box(){{var s=['rich-textarea .ql-editor','div[contenteditable="true"][role="textbox"]','textarea#userInput','textarea[placeholder]','textarea','div[contenteditable="true"]'];for(var i=0;i<s.length;i++){{var els=document.querySelectorAll(s[i]);for(var j=0;j<els.length;j++){{var e=els[j];if(e.offsetParent!==null||e.getClientRects().length)return e}}}}return null}}function fill(e){{e.focus();if(e.tagName==='TEXTAREA'){{var set=Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set;set.call(e,q)}}else{{e.textContent='';document.execCommand&&document.execCommand('insertText',false,q);if(!e.textContent)e.textContent=q}}e.dispatchEvent(new InputEvent('input',{{bubbles:true,data:q,inputType:'insertText'}}));e.dispatchEvent(new Event('change',{{bubbles:true}}))}}function send(e){{var b=document.querySelector('button[aria-label*="Send" i]:not([disabled]),button[data-testid*="send" i]:not([disabled]),button[type="submit"]:not([disabled]),button.send-button:not([disabled])');if(b){{b.click();return}}['keydown','keypress','keyup'].forEach(function(k){{e.dispatchEvent(new KeyboardEvent(k,{{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}}))}})}}(function tick(){{var e=box();if(e){{fill(e);setTimeout(function(){{send(e)}},450);return}}if(Date.now()-t0<20000)setTimeout(tick,250)}})()}})()"#)
}
/// Where to go and what (if anything) to run once the page is up.
pub fn ask(cfg: &BrowserConfig, query: &str) -> (String, Option<String>) {
    let q = query.trim();
    if cfg.ai_provider == "custom" {
        let tpl = cfg.ai_custom_url.clone().unwrap_or_default();
        return match tpl.contains("%s") {
            true => (tpl.replace("%s", &urlencoding::encode(q)), None),
            false => (tpl, (!q.is_empty()).then(|| inject_script(q))),
        };
    }
    let p = provider(&cfg.ai_provider);
    if q.is_empty() { return (p.home.to_string(), None); }
    match p.inject || p.url.is_empty() {
        true => (p.home.to_string(), Some(inject_script(q))),
        false => (p.url.replace("%s", &urlencoding::encode(q)), None),
    }
}
/// A prompt about the page the user is on. The AI site fetches the URL itself; Amni sends no page content.
pub fn page_prompt(title: &str, url: &str) -> String {
    match title.trim().is_empty() { true => format!("Read {} and summarize it. Then answer my follow-up questions about it.", url), false => format!("Read \"{}\" at {} and summarize it. Then answer my follow-up questions about it.", title.trim(), url) }
}
pub fn selection_prompt(sel: &str, url: &str) -> String {
    let s: String = sel.trim().chars().take(4000).collect();
    format!("Explain this, from {}:\n\n{}", url, s)
}
pub fn providers_json(cfg: &BrowserConfig) -> serde_json::Value {
    serde_json::json!(PROVIDERS.iter().map(|p| serde_json::json!({"id": p.id, "name": p.name, "note": p.note, "fill_only": p.fill_only, "inject": p.inject, "active": p.id == cfg.ai_provider})).collect::<Vec<_>>())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn claude_prefills() {
        let mut c = BrowserConfig::default();
        c.ai_provider = "claude".into();
        let (u, js) = ask(&c, "hello world");
        assert_eq!(u, "https://claude.ai/new?q=hello%20world");
        assert!(js.is_none());
    }
    #[test]
    fn gemini_injects() {
        let mut c = BrowserConfig::default();
        c.ai_provider = "gemini".into();
        let (u, js) = ask(&c, "x");
        assert_eq!(u, "https://gemini.google.com/app");
        assert!(js.unwrap().contains("\"x\""));
    }
    #[test]
    fn custom_template() {
        let mut c = BrowserConfig::default();
        c.ai_provider = "custom".into();
        c.ai_custom_url = Some("https://ai.example/?p=%s".into());
        assert_eq!(ask(&c, "a b").0, "https://ai.example/?p=a%20b");
    }
}
