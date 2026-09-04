def rw(p,f):
    s=open(p,encoding='utf-8').read(); n=f(s); assert n!=s,p; open(p,'w',encoding='utf-8',newline='\n').write(n)
def sub1(s,a,b):
    assert s.count(a)==1,(a[:70],s.count(a)); return s.replace(a,b)
def ab(s):
    s=sub1(s,'''    pub fn blocked_count(&self) -> u64 {''','''    /// WebKit content-blocker JSON (Safari/WebKitGTK rule list) mirroring the Windows request shield:
    /// block on the domain list + tracker paths, ignore-previous-rules on the auth allowlist, and hide
    /// the iframes/containers the blocked ads leave behind.
    pub fn content_rules() -> String {
        let esc = |s: &str| s.replace('.', "\\.").replace('/', "\\/").replace('?', "\\?");
        let mut rules: Vec<String> = BLOCKED_DOMAINS.iter().map(|d| format!(r#"{{"trigger":{{"url-filter":"{}","url-filter-is-case-sensitive":false}},"action":{{"type":"block"}}}}"#, esc(d))).collect();
        for p in ["[?&]utm_[a-z]+=", "\\/ads?\\/", "\\/adserv", "\\/pixel[./]", "\\/beacon[./]", "[?&]fbclid=", "[?&]gclid=", "[?&]mc_[a-z]+=", "\\/tracking?[./]", "\\/track[./]", "\\/collect\\?", "\\/__utm\\.gif", "\\/piwik\\.", "\\/matomo\\."] {
            rules.push(format!(r#"{{"trigger":{{"url-filter":"{}","url-filter-is-case-sensitive":false,"resource-type":["image","script","raw","popup","document","style-sheet","font","media","svg-document"]}},"action":{{"type":"block"}}}}"#, p));
        }
        for a in AUTH_ALLOW.iter().map(|s| esc(s)).chain(["\\/o\\/oauth2", "\\/oauth2\\/", "ux_mode=popup", "gsiwebsdk", "redirect_uri=gis_"].into_iter().map(String::from)) {
            rules.push(format!(r#"{{"trigger":{{"url-filter":"{}","url-filter-is-case-sensitive":false}},"action":{{"type":"ignore-previous-rules"}}}}"#, a));
        }
        let hide: Vec<String> = BLOCKED_DOMAINS.iter().take(160).map(|d| format!("iframe[src*=\\"{}\\"]", d)).chain(["ins.adsbygoogle", "[id^='google_ads_iframe']", "[id^='div-gpt-ad']", "[data-ad-slot]", "[data-google-query-id]", ".adsbygoogle", "iframe[id^='google_ads']", "iframe[name^='google_ads']", "[class~='ad-slot']", "[class~='ad-container']", "[id^='ad-slot']"].into_iter().map(String::from)).collect();
        rules.push(format!(r#"{{"trigger":{{"url-filter":".*"}},"action":{{"type":"css-display-none","selector":"{}"}}}}"#, hide.join(", ")));
        format!("[{}]", rules.join(","))
    }
    pub fn blocked_count(&self) -> u64 {''')
    return s
rw('src/engine/adblocker.rs',ab)
def ch(s):
    s=sub1(s,'''    blocker: Rc<RefCell<AdBlocker>>,
    shield: Rc<Cell<bool>>,
''','''    blocker: Rc<RefCell<AdBlocker>>,
    shield: Rc<Cell<bool>>,
    #[cfg(not(windows))]
    filter: Option<webkit2gtk::UserContentFilter>,
''')
    s=sub1(s,'''/// Everything wry does not expose: request-level shield + DNT/GPC headers, real history state,
/// favicons, HTML5 fullscreen, audio state, download progress, password/form autofill.
#[cfg(not(windows))]
fn wire_engine(''','''/// WebKitGTK request shield: the ad/tracker lists compiled once into a WebKit content rule list
/// (blocks subresources like the Windows `WebResourceRequested` hook) and attached to every tab.
#[cfg(not(windows))]
fn compile_filter() -> Option<webkit2gtk::UserContentFilter> {
    use webkit2gtk::prelude::*;
    let dir = dirs::config_dir()?.join("amni-browse").join("filters");
    std::fs::create_dir_all(&dir).ok();
    let store = webkit2gtk::UserContentFilterStore::new(dir.to_str()?);
    let bytes = webkit2gtk::glib::Bytes::from_owned(AdBlocker::content_rules().into_bytes());
    let done: Rc<RefCell<Option<Result<webkit2gtk::UserContentFilter, webkit2gtk::glib::Error>>>> = Rc::new(RefCell::new(None));
    let d = done.clone();
    store.save("amni-shield", &bytes, None::<&webkit2gtk::gio::Cancellable>, move |r| { *d.borrow_mut() = Some(r); });
    let ctx = webkit2gtk::glib::MainContext::default();
    while done.borrow().is_none() { ctx.iteration(true); }
    let r = done.borrow_mut().take()?;
    match r { Ok(f) => { info!("shield: content rule list compiled"); Some(f) }, Err(e) => { warn!("shield: content rule list failed: {}", e); None } }
}
#[cfg(not(windows))]
fn attach_filter(view: &WebView, filter: Option<&webkit2gtk::UserContentFilter>, on: bool) {
    use webkit2gtk::prelude::*;
    use wry::WebViewExtUnix;
    if let (Some(f), Some(m)) = (filter, view.webview().user_content_manager()) { match on { true => m.add_filter(f), false => m.remove_filter(f) } }
}
/// Everything wry does not expose: request-level shield + DNT/GPC headers, real history state,
/// favicons, HTML5 fullscreen, audio state, download progress, password/form autofill.
#[cfg(not(windows))]
fn wire_engine(''')
    s=sub1(s,'''        let view = match view { Ok(v) => v, Err(e) => { warn!("webview2 tab failed: {}", e); return self.active; } };
''','''        let view = match view { Ok(v) => v, Err(e) => { warn!("webview2 tab failed: {}", e); return self.active; } };
        #[cfg(not(windows))]
        attach_filter(&view, self.filter.as_ref(), self.shield.get());
''')
    s=sub1(s,'''            "block_ads" | "shield" => { self.state.config.block_ads = on; self.shield.set(on); }''','''            "block_ads" | "shield" => { self.state.config.block_ads = on; self.shield.set(on); self.reshield(); }''')
    s=sub1(s,'''            "shield" | "block_ads" => { let on = !self.shield.get(); self.shield.set(on); self.state.config.block_ads = on; self.state.config.save(); }''','''            "shield" | "block_ads" => { let on = !self.shield.get(); self.shield.set(on); self.state.config.block_ads = on; self.state.config.save(); self.reshield(); }''')
    s=sub1(s,'''    fn spawn_tab(&mut self, url: &str, private: bool, at: Option<usize>) -> usize {''','''    fn reshield(&mut self) {
        #[cfg(not(windows))]
        for t in &self.tabs { attach_filter(&t.view, self.filter.as_ref(), self.shield.get()); }
    }
    fn spawn_tab(&mut self, url: &str, private: bool, at: Option<usize>) -> usize {''')
    s=sub1(s,'''blocker, shield, collapsed: Vec::new(), ephemeral };''','''blocker, shield, #[cfg(not(windows))] filter: compile_filter(), collapsed: Vec::new(), ephemeral };''')
    return s
rw('src/platform/chromium.rs',ch)
def cg(s):
    s=sub1(s,'''[target.'cfg(windows)'.dependencies]''','''[target.'cfg(not(windows))'.dependencies]
webkit2gtk = { version = "=2.0.1", features = ["v2_38"] }

[target.'cfg(windows)'.dependencies]''')
    return s
rw('Cargo.toml',cg)
print('linux shield patched')
