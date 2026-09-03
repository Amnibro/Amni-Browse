p='src/platform/servo_real.rs'; s=open(p,encoding='utf-8').read()
def sub1(s,a,b):
    assert s.count(a)==1,(a[:70],s.count(a)); return s.replace(a,b)
s=sub1(s,r'''    fn notify_page_title_changed(&self, webview: WebView, title: Option<String>) {
        if let Some(idx) = self.tab_index_for_webview(&webview) {
            if let Some(t) = title.as_deref() {
                self.remember_tab_title(idx, t);
            }
        }
        let is_active = self.active_content().map(|a| a.id() == webview.id()).unwrap_or(false);
        if !is_active { return; }
        let t = title.unwrap_or_default();''',r'''    fn sync_window_title(&self) {
        if let Some(c) = self.active_content() { self.set_window_title(c.page_title()); }
    }
    fn notify_page_title_changed(&self, webview: WebView, title: Option<String>) {
        if let Some(idx) = self.tab_index_for_webview(&webview) {
            if let Some(t) = title.as_deref() {
                self.remember_tab_title(idx, t);
            }
        }
        let is_active = self.active_content().map(|a| a.id() == webview.id()).unwrap_or(false);
        if !is_active { return; }
        self.set_window_title(title);
    }
    fn set_window_title(&self, title: Option<String>) {
        let t = title.unwrap_or_default();''')
s=sub1(s,r'''                if idx < len { self.active_content_index.set(idx); self.apply_media_visibility(); info!("cmd switch_tab \u{2192} idx {}", idx); self.window.request_redraw(); }''',r'''                if idx < len { self.active_content_index.set(idx); self.apply_media_visibility(); self.sync_window_title(); info!("cmd switch_tab \u{2192} idx {}", idx); self.window.request_redraw(); }''')
open(p,'w',encoding='utf-8',newline='').write(s); print('title ok')
