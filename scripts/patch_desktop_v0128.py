def rw(p,f,nl=None):
    s=open(p,encoding='utf-8').read(); n=f(s); assert n!=s,p; open(p,'w',encoding='utf-8',newline=nl).write(n)
def sub1(s,a,b):
    assert s.count(a)==1,(a[:70],s.count(a)); return s.replace(a,b)
rw('Cargo.toml',lambda s: sub1(s,'rfd = { version = "0.15", optional = true }','rfd = { version = "0.15", optional = true }\n[target.\'cfg(windows)\'.dependencies]\nwindows-sys = { version = "0.61", features = ["Win32_Foundation", "Win32_Graphics_Gdi"] }'))
def sess(s):
    s=sub1(s,'    pub window_height: f64,\n    pub saved_at','    pub window_height: f64,\n    #[serde(default)]\n    pub window_x: Option<f64>,\n    #[serde(default)]\n    pub window_y: Option<f64>,\n    #[serde(default)]\n    pub maximized: bool,\n    pub saved_at')
    s=sub1(s,'            window_height: 900.0,\n            saved_at','            window_height: 900.0,\n            window_x: None,\n            window_y: None,\n            maximized: false,\n            saved_at')
    return s
rw('src/storage/session.rs',sess)
def sr(s):
    s=sub1(s,r'''        sm.state.window_width = sz.width as f64;
        sm.state.window_height = sz.height as f64;
        sm.capture(tabs);''',r'''        sm.state.window_width = sz.width as f64;
        sm.state.window_height = sz.height as f64;
        sm.state.maximized = self.window.is_maximized();
        if let Ok(pos) = self.window.outer_position() { sm.state.window_x = Some(pos.x as f64 / scale); sm.state.window_y = Some(pos.y as f64 / scale); }
        sm.capture(tabs);''')
    s=sub1(s,r'''            let (win_w, win_h) = (win_w.min(mon_w).max(720.0), win_h.min(mon_h).max(480.0));
            info!("window size \u{2192} {}x{} logical (monitor cap {:.0}x{:.0})", win_w, win_h, mon_w, mon_h);
            let window = event_loop.create_window(
                Window::default_attributes()
                    .with_title("Amni Browse")
                    .with_decorations(std::env::var("AMNI_DECORATIONS").map(|v| v != "0").unwrap_or(false))
                    .with_transparent(false)
                    .with_min_inner_size(LogicalSize::new(720.0, 480.0))
                    .with_inner_size(LogicalSize::new(win_w, win_h))
            ).expect("window");''',r'''            let (win_w, win_h) = (win_w.min(mon_w).max(720.0), win_h.min(mon_h).max(480.0));
            let scale_guess = event_loop.primary_monitor().map(|m| m.scale_factor()).unwrap_or(1.0);
            let saved_pos = saved.as_ref().and_then(|s| Some((s.window_x?, s.window_y?)));
            let (win_x, win_y) = restore_window_position(saved_pos, (win_w, win_h), scale_guess);
            let maximized = saved.as_ref().map(|s| s.maximized).unwrap_or(false);
            info!("window size \u{2192} {}x{} logical at {:.0},{:.0} (monitor cap {:.0}x{:.0}) maximized={}", win_w, win_h, win_x, win_y, mon_w, mon_h, maximized);
            let window = event_loop.create_window(
                Window::default_attributes()
                    .with_title("Amni Browse")
                    .with_decorations(std::env::var("AMNI_DECORATIONS").map(|v| v != "0").unwrap_or(false))
                    .with_transparent(false)
                    .with_min_inner_size(LogicalSize::new(720.0, 480.0))
                    .with_inner_size(LogicalSize::new(win_w, win_h))
                    .with_position(winit::dpi::LogicalPosition::new(win_x, win_y))
                    .with_maximized(maximized)
            ).expect("window");''')
    s=sub1(s,r'''fn handle_shortcut(key_event: &KeyEvent, state: &AppState) -> bool {
    if key_event.state != ElementState::Pressed { return false; }''',r'''/// Work area (taskbar excluded) of the monitor containing `point`, in physical pixels: (x, y, w, h).
#[cfg(windows)]
fn monitor_work_area(point: (i32, i32)) -> Option<(i32, i32, i32, i32)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    unsafe {
        let m = MonitorFromPoint(POINT { x: point.0, y: point.1 }, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        (GetMonitorInfoW(m, &mut info) != 0).then(|| { let r = info.rcWork; (r.left, r.top, r.right - r.left, r.bottom - r.top) })
    }
}
#[cfg(not(windows))]
fn monitor_work_area(_point: (i32, i32)) -> Option<(i32, i32, i32, i32)> { None }
/// Clamp a saved logical window origin so the whole window sits inside the work area of the
/// monitor it was on; without a saved origin, center on the primary work area.
fn restore_window_position(saved: Option<(f64, f64)>, size: (f64, f64), scale: f64) -> (f64, f64) {
    let (x, y) = saved.unwrap_or((f64::NAN, f64::NAN));
    let probe = match saved { Some((x, y)) => ((x * scale) as i32 + 8, (y * scale) as i32 + 8), None => (0, 0) };
    let Some((ax, ay, aw, ah)) = monitor_work_area(probe) else { return match saved { Some(p) => p, None => (100.0, 60.0) }; };
    let (ax, ay, aw, ah) = (ax as f64 / scale, ay as f64 / scale, aw as f64 / scale, ah as f64 / scale);
    let clamp = |v: f64, lo: f64, span: f64, len: f64| match v.is_nan() { true => lo + ((span - len) / 2.0).max(0.0), false => v.max(lo).min((lo + span - len).max(lo)) };
    (clamp(x, ax, aw, size.0), clamp(y, ay, ah, size.1))
}
fn handle_shortcut(key_event: &KeyEvent, state: &AppState) -> bool {
    if key_event.state != ElementState::Pressed {
        return state.shortcut_keys.borrow_mut().remove(&key_event.physical_key);
    }''')
    s=sub1(s,r'''    /// True while the last cursor we set was a frameless-window resize grip.
    resize_cursor_on: Cell<bool>,''',r'''    /// True while the last cursor we set was a frameless-window resize grip.
    resize_cursor_on: Cell<bool>,
    /// Physical keys whose press was consumed by a chrome shortcut; their release is swallowed too.
    shortcut_keys: RefCell<std::collections::HashSet<winit::keyboard::PhysicalKey>>,''')
    s=sub1(s,r'''                kbd_in_chrome: Cell::new(false),
                paint_logged: Cell::new(false),''',r'''                kbd_in_chrome: Cell::new(false),
                shortcut_keys: RefCell::new(std::collections::HashSet::new()),
                paint_logged: Cell::new(false),''')
    s=sub1(s,r'''                if handle_shortcut(&key_event, state) { if trace {''',r'''                let consumed = handle_shortcut(&key_event, state);
                if consumed && key_event.state == ElementState::Pressed { state.shortcut_keys.borrow_mut().insert(key_event.physical_key); }
                if consumed { if trace {''')
    return s
rw('src/platform/servo_real.rs',sr,'')
print('desktop patched')
