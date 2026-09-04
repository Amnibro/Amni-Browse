def rw(p,f):
    s=open(p,encoding='utf-8').read(); n=f(s); assert n!=s,p; open(p,'w',encoding='utf-8',newline='\n').write(n)
def sub1(s,a,b):
    assert s.count(a)==1,(a[:70],s.count(a)); return s.replace(a,b)
def ch(s):
    i=s.index('#[cfg(not(windows))]\nfn compile_filter()'); j=s.index('/// Everything wry does not expose')
    new='''#[cfg(not(windows))]
unsafe extern "C" fn on_filter_saved(src: *mut webkit2gtk::glib::gobject_ffi::GObject, res: *mut webkit2gtk::gio::ffi::GAsyncResult, data: webkit2gtk::glib::ffi::gpointer) {
    let mut err: *mut webkit2gtk::glib::ffi::GError = std::ptr::null_mut();
    let f = webkit2gtk_sys::webkit_user_content_filter_store_save_finish(src as *mut webkit2gtk_sys::WebKitUserContentFilterStore, res, &mut err);
    if !err.is_null() { let e: webkit2gtk::glib::Error = webkit2gtk::glib::translate::from_glib_full(err); warn!("shield: content rule list failed: {}", e); }
    let done: Rc<Cell<Option<usize>>> = Rc::from_raw(data as *const Cell<Option<usize>>);
    done.set(Some(f as usize));
}
#[cfg(not(windows))]
fn compile_filter() -> Option<usize> {
    let dir = dirs::config_dir()?.join("amni-browse").join("filters");
    std::fs::create_dir_all(&dir).ok();
    let path = std::ffi::CString::new(dir.to_str()?).ok()?;
    let id = std::ffi::CString::new("amni-shield").ok()?;
    let rules = AdBlocker::content_rules();
    let done: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
    unsafe {
        let store = webkit2gtk_sys::webkit_user_content_filter_store_new(path.as_ptr());
        let bytes = webkit2gtk::glib::ffi::g_bytes_new(rules.as_ptr() as *const _, rules.len());
        webkit2gtk_sys::webkit_user_content_filter_store_save(store, id.as_ptr(), bytes, std::ptr::null_mut(), Some(on_filter_saved), Rc::into_raw(done.clone()) as *mut _);
        let ctx = webkit2gtk::glib::MainContext::default();
        while done.get().is_none() { ctx.iteration(true); }
        webkit2gtk::glib::ffi::g_bytes_unref(bytes);
    }
    let f = done.get()?;
    match f { 0 => None, _ => { info!("shield: content rule list compiled ({} bytes)", rules.len()); Some(f) } }
}
#[cfg(not(windows))]
fn attach_filter(view: &WebView, filter: Option<usize>, on: bool) {
    use webkit2gtk::glib::translate::ToGlibPtr;
    use webkit2gtk::WebViewExt;
    use wry::WebViewExtUnix;
    if let (Some(f), Some(m)) = (filter, view.webview().user_content_manager()) {
        let mp: *mut webkit2gtk_sys::WebKitUserContentManager = m.to_glib_none().0;
        unsafe { match on { true => webkit2gtk_sys::webkit_user_content_manager_add_filter(mp, f as *mut _), false => webkit2gtk_sys::webkit_user_content_manager_remove_filter(mp, f as *mut _) } }
    }
}
'''
    s=s[:i]+new+s[j:]
    s=sub1(s,'''    filter: Option<webkit2gtk::UserContentFilter>,''','''    filter: Option<usize>,''')
    s=sub1(s,'''attach_filter(&view, self.filter.as_ref(), self.shield.get());''','''attach_filter(&view, self.filter, self.shield.get());''')
    s=sub1(s,'''attach_filter(&t.view, self.filter.as_ref(), self.shield.get()); }''','''attach_filter(&t.view, self.filter, self.shield.get()); }''')
    return s
rw('src/platform/chromium.rs',ch)
def cg(s):
    return sub1(s,'''webkit2gtk = { version = "=2.0.1", features = ["v2_38"] }''','''webkit2gtk = { version = "=2.0.1", features = ["v2_38"] }
webkit2gtk-sys = { version = "=2.0.1", features = ["v2_38"] }''')
rw('Cargo.toml',cg)
print('ffi shield patched')
