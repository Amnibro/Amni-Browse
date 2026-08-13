pub fn is_pdf_url(url: &str) -> bool {
    let l = url.to_lowercase();
    let path = l.split(['?', '#']).next().unwrap_or(&l);
    path.ends_with(".pdf")
}
pub fn is_download_url(url: &str) -> bool {
    if is_pdf_url(url) { return true; }
    let l = url.to_lowercase();
    let path = l.split(['?', '#']).next().unwrap_or(&l);
    const EXT: &[&str] = &[".zip",".exe",".msi",".dmg",".7z",".rar",".gz",".bz2",".xz",".iso",".apk",".doc",".docx",".xls",".xlsx",".ppt",".pptx",".csv",".torrent"];
    EXT.iter().any(|e| path.ends_with(e))
}
pub fn find_script(query: &str, dir: i32) -> String {
    let q = query.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', " ");
    format!("(function(){{var q='{}';var dir={};try{{if(!q){{if(window.getSelection)window.getSelection().removeAllRanges();return {{found:false,current:0,total:0}};}}if(typeof window.find==='function'){{var ok=window.find(q,false,dir<0,true,false,false,false);return {{found:!!ok,current:0,total:0}};}}return {{found:false,current:0,total:0}};}}catch(e){{return {{found:false,current:0,total:0}};}}}})()", q, dir)
}
pub fn print_script() -> &'static str { "try{window.print()}catch(e){}" }
pub fn autofill_script(user: &str, pass: &str) -> String {
    let u = user.replace('\\', "\\\\").replace('\'', "\\'");
    let p = pass.replace('\\', "\\\\").replace('\'', "\\'");
    format!("(function(){{try{{var u='{}',p='{}';var inputs=document.querySelectorAll('input');var userEl=null,passEl=null;for(var i=0;i<inputs.length;i++){{var el=inputs[i];var t=(el.type||'').toLowerCase();var n=(el.name||el.id||el.autocomplete||'').toLowerCase();if(!passEl&&t==='password')passEl=el;if(!userEl&&(t==='email'||t==='text'||t==='tel')&&(n.indexOf('user')>=0||n.indexOf('email')>=0||n.indexOf('login')>=0||el.autocomplete==='username'))userEl=el;}}if(!userEl){{for(var j=0;j<inputs.length;j++){{var e2=inputs[j];var t2=(e2.type||'').toLowerCase();if(t2==='text'||t2==='email'){{userEl=e2;break;}}}}}}if(userEl&&u){{userEl.value=u;userEl.dispatchEvent(new Event('input',{{bubbles:true}}));userEl.dispatchEvent(new Event('change',{{bubbles:true}}));}}if(passEl&&p){{passEl.value=p;passEl.dispatchEvent(new Event('input',{{bubbles:true}}));passEl.dispatchEvent(new Event('change',{{bubbles:true}}));}}}}catch(e){{}}}})()", u, p)
}
pub fn inject_css_script(css: &str) -> String {
    let c = css.replace('\\', "\\\\").replace('`', "\\`").replace("</", "<\\/");
    format!("(function(){{try{{var s=document.createElement('style');s.setAttribute('data-amni-ext','1');s.textContent=`{}`;(document.head||document.documentElement).appendChild(s);}}catch(e){{}}}})()", c)
}
pub fn pdf_viewer_html(url: &str, theme_vars: &str, tok: &str) -> String {
    let safe = url.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
    format!("<!DOCTYPE html><html><head><meta charset='utf-8'><title>PDF</title><style>:root{{{theme}}}body{{font:15px/1.5 -apple-system,'Segoe UI',sans-serif;background:var(--bg,#08090B);color:var(--text,#EDEFF2);margin:0;padding:48px 24px;text-align:center}}h1{{font-size:22px;color:var(--accent,#C89B4E);margin:0 0 8px}}p{{color:var(--dim,#A7ADB6);max-width:520px;margin:0 auto 22px;word-break:break-all}}button,a.btn{{display:inline-block;margin:0 6px 8px;padding:10px 18px;border-radius:8px;border:1px solid var(--stroke,#20242B);background:var(--elev,#111418);color:var(--text,#EDEFF2);text-decoration:none;cursor:pointer;font:inherit}}button.primary,a.primary{{background:var(--accent,#C89B4E);color:#08090B;border-color:transparent;font-weight:600}}</style></head><body><h1>PDF</h1><p>{safe}</p><p>Servo does not paint PDF pages. Amni saved a copy to Downloads and can open it in the system viewer.</p><button class='primary' onclick=\"fetch('amnibrowse://cmd/open_download?tok={tok}&url='+encodeURIComponent('{js}'),{{mode:'no-cors'}})\">Open with system</button><button onclick=\"fetch('amnibrowse://cmd/download?tok={tok}&url='+encodeURIComponent('{js}'),{{mode:'no-cors'}})\">Save again</button></body></html>", theme = theme_vars, safe = safe, tok = tok, js = url.replace('\\', "\\\\").replace('\'', "\\'"))
}
pub fn omnibox_rank(query: &str, url: &str, title: &str, visits: u32) -> i64 {
    let q = query.to_lowercase();
    if q.is_empty() { return visits as i64; }
    let u = url.to_lowercase();
    let t = title.to_lowercase();
    let mut s = visits as i64;
    if u.starts_with(&q) || t.starts_with(&q) { s += 50; }
    if u.contains(&q) { s += 20; }
    if t.contains(&q) { s += 15; }
    s
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pdf_detects_query_and_hash() {
        assert!(is_pdf_url("https://ex.com/a.pdf"));
        assert!(is_pdf_url("https://ex.com/a.PDF?dl=1"));
        assert!(is_pdf_url("https://ex.com/a.pdf#page=2"));
        assert!(!is_pdf_url("https://ex.com/pdfs/list"));
        assert!(!is_pdf_url("https://ex.com/notpdf.html"));
    }
    #[test]
    fn download_detects_archives() {
        assert!(is_download_url("https://ex.com/app.exe"));
        assert!(is_download_url("https://ex.com/pack.zip?x=1"));
        assert!(!is_download_url("https://ex.com/index.html"));
    }
    #[test]
    fn omnibox_prefers_prefix() {
        let a = omnibox_rank("git", "https://github.com", "GitHub", 1);
        let b = omnibox_rank("git", "https://example.com/blog", "gardening tips", 40);
        assert!(a > b);
    }
    #[test]
    fn find_script_escapes() {
        let s = find_script("it's", 1);
        assert!(s.contains("it\\'s"));
        assert!(s.contains("window.find"));
    }
}
