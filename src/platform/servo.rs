#[cfg(feature = "servo-engine")]
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId}};
#[cfg(feature = "servo-engine")]
use winit::event::{ElementState, MouseButton, KeyEvent};
#[cfg(feature = "servo-engine")]
use winit::keyboard::{Key, NamedKey};
#[cfg(feature = "servo-engine")]
use std::sync::Arc;
#[cfg(feature = "servo-engine")]
use std::collections::HashMap;
#[cfg(feature = "servo-engine")]
use crate::app::BrowserState;
#[cfg(feature = "servo-engine")]
use crate::ui::chrome::{BrowserChrome, ChromeStats};
#[cfg(feature = "servo-engine")]
use crate::net::ipc::IpcResponse;
#[cfg(feature = "servo-engine")]
use crate::engine::layout::LayoutRect;
#[cfg(feature = "servo-engine")]
use crate::engine::pipeline::{extract_scripts, extract_external_script_urls};
#[cfg(feature = "servo-engine")]
use crate::engine::paint::RenderTree;
#[cfg(feature = "servo-engine")]
struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}
#[cfg(feature = "servo-engine")]
struct AmniApp {
    state: BrowserState,
    chrome: BrowserChrome,
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,
    needs_initial_data: bool,
    page_texture: Option<egui::TextureHandle>,
    rendered_url: String,
    render_pending: bool,
    async_rx: Option<std::sync::mpsc::Receiver<String>>,
    cursor_pos: (f64, f64),
    page_html: String,
}
#[cfg(feature = "servo-engine")]
impl ApplicationHandler for AmniApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(format!("Amni Browse v{} — Servo Engine", crate::storage::config::APP_VERSION))
            .with_inner_size(winit::dpi::LogicalSize::new(1400.0, 900.0))
            .with_min_inner_size(winit::dpi::LogicalSize::new(640.0, 400.0));
        let window = Arc::new(event_loop.create_window(attrs).expect("window creation failed"));
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), ..Default::default() });
        let surface = instance.create_surface(window.clone()).expect("surface creation failed");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).expect("no suitable GPU adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("amni_device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        }, None)).expect("device creation failed");
        let size = window.inner_size();
        let config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).expect("surface config failed");
        surface.configure(&device, &config);
        self.egui_state = Some(egui_winit::State::new(
            self.egui_ctx.clone(), egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None, None,
        ));
        self.egui_renderer = Some(egui_wgpu::Renderer::new(&device, config.format, None, 1, false));
        self.window = Some(window);
        self.gpu = Some(GpuState { surface, device, queue, config });
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (Some(window), Some(egui_state)) = (self.window.as_ref(), self.egui_state.as_mut()) else { return; };
        let resp = egui_state.on_window_event(window, &event);
        if resp.consumed { return; }
        match event {
            WindowEvent::CloseRequested => { self.state.shutdown(); event_loop.exit(); }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.config.width = size.width.max(1);
                    gpu.config.height = size.height.max(1);
                    gpu.surface.configure(&gpu.device, &gpu.config);
                }
                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x, position.y);
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                if !self.state.interactor.current_layouts.is_empty() {
                    let (cx, cy) = self.cursor_pos;
                    let scale = window.scale_factor();
                    let x = (cx / scale) as f32;
                    let y = (cy / scale) as f32;
                    if let Some(node_id) = self.state.interactor.dispatch_click(x, y) {
                        self.state.interactor.focus_node(node_id);
                    }
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. }, .. } => {
                if self.state.interactor.focus_manager.current_focus.is_some() {
                    let (key_str, code) = match &logical_key {
                        Key::Character(c) => (c.to_string(), 0u32),
                        Key::Named(named) => match named {
                            NamedKey::Enter => ("Enter".into(), 13),
                            NamedKey::Tab => ("Tab".into(), 9),
                            NamedKey::Backspace => ("Backspace".into(), 8),
                            NamedKey::Escape => ("Escape".into(), 27),
                            NamedKey::Space => (" ".into(), 32),
                            NamedKey::ArrowLeft => ("ArrowLeft".into(), 37),
                            NamedKey::ArrowUp => ("ArrowUp".into(), 38),
                            NamedKey::ArrowRight => ("ArrowRight".into(), 39),
                            NamedKey::ArrowDown => ("ArrowDown".into(), 40),
                            NamedKey::Delete => ("Delete".into(), 46),
                            NamedKey::Home => ("Home".into(), 36),
                            NamedKey::End => ("End".into(), 35),
                            _ => ("".into(), 0),
                        },
                        _ => ("".into(), 0),
                    };
                    if !key_str.is_empty() {
                        let mods = self.egui_state.as_ref().map(|s| s.egui_input().modifiers).unwrap_or_default();
                        self.state.interactor.dispatch_key(&key_str, code, mods.shift, mods.ctrl, mods.alt);
                        if matches!(logical_key, Key::Named(NamedKey::Tab)) {
                            self.state.interactor.tab_focus_next();
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => self.render(),
            _ => {}
        }
    }
}
#[cfg(feature = "servo-engine")]
fn apply_to_chrome(chrome: &mut BrowserChrome, resp: Option<IpcResponse>) {
    let Some(r) = resp else { return; };
    match r {
        IpcResponse::TabsUpdated { tabs } => chrome.tabs_json = tabs,
        IpcResponse::NavigateTo { url } => { chrome.url_input = url; chrome.status_text = "Loading...".into(); }
        IpcResponse::Bookmarks { data } => chrome.bookmarks_json = data,
        IpcResponse::Stats { ads_blocked, tabs_open, bookmarks_count, passwords_count, history_count, downloads_active } => {
            chrome.stats = ChromeStats { ads_blocked: ads_blocked as usize, tabs_open, bookmarks_count, passwords_count, history_count, downloads_active };
        }
        IpcResponse::VaultStatus { unlocked, .. } => chrome.vault_unlocked = unlocked,
        IpcResponse::VaultCredentials { data } => chrome.vault_creds_json = data,
        IpcResponse::Downloads { data } => chrome.downloads_json = data,
        IpcResponse::History { data } => chrome.history_json = data,
        IpcResponse::Extensions { data } => chrome.extensions_json = data,
        IpcResponse::Profiles { data, .. } => chrome.profiles_json = data,
        IpcResponse::Permissions { data } => chrome.permissions_json = data,
        IpcResponse::ZoomLevel { level } => chrome.zoom_pct = (level * 100.0) as u32,
        IpcResponse::DevToolsStateResp { data } => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                chrome.dt_console_json = v["console"].to_string();
                chrome.dt_network_json = v["network"].to_string();
            }
        }
        IpcResponse::DrmWebViewRequired { url, reason } => {
            chrome.url_input = url.clone();
            chrome.status_text = format!("DRM: Opening in WebView — {}", reason);
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", &url])
                    .spawn();
            }
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg(&url)
                    .spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&url)
                    .spawn();
            }
        }
        _ => {}
    }
}
#[cfg(feature = "servo-engine")]
impl AmniApp {
    fn render(&mut self) {
        if self.window.is_none() || self.gpu.is_none() || self.egui_state.is_none() || self.egui_renderer.is_none() { return; }
        if self.needs_initial_data {
            self.needs_initial_data = false;
            let r1 = self.state.handle_command(crate::net::ipc::IpcMessage::GetTabs);
            apply_to_chrome(&mut self.chrome, r1);
            let r2 = self.state.handle_command(crate::net::ipc::IpcMessage::GetStats);
            apply_to_chrome(&mut self.chrome, r2);
            let r3 = self.state.handle_command(crate::net::ipc::IpcMessage::BookmarkList);
            apply_to_chrome(&mut self.chrome, r3);
        }
        self.check_rendered_pages();
        let raw_input = self.egui_state.as_mut().unwrap().take_egui_input(self.window.as_ref().unwrap());
        self.egui_ctx.begin_pass(raw_input);
        self.chrome.handle_keyboard(&self.egui_ctx);
        self.chrome.render(&self.egui_ctx);
        let active_url = self.state.tabs.active_tab().map(|t| t.url.clone()).unwrap_or_default();
        let stats = self.chrome.stats.clone();
        let bmarks_json = self.chrome.bookmarks_json.clone();
        let mut new_search = self.chrome.search_input.clone();
        let mut content_cmds: Vec<crate::net::ipc::IpcMessage> = Vec::new();
        let ctx = self.egui_ctx.clone();
        egui::CentralPanel::default().show(&ctx, |ui| {
            if active_url.is_empty() || active_url.starts_with("amnibrowse://newtab") {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.15);
                    ui.heading(egui::RichText::new("Amni Browse").size(48.0).color(egui::Color32::from_rgb(0, 212, 255)));
                    ui.label(egui::RichText::new("Privacy-First · Zero Telemetry · Servo Powered").size(14.0).color(egui::Color32::GRAY));
                    ui.add_space(24.0);
                    let resp = ui.add(egui::TextEdit::singleline(&mut new_search).desired_width(500.0).hint_text("Search the web or enter URL...").font(egui::TextStyle::Heading));
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !new_search.is_empty() {
                        let q = new_search.clone(); new_search.clear();
                        content_cmds.push(if q.contains('.') && !q.contains(' ') { crate::net::ipc::IpcMessage::Navigate { url: format!("https://{}", q) } } else { crate::net::ipc::IpcMessage::Search { query: q } });
                    }
                    ui.add_space(32.0);
                    ui.columns(3, |cols| {
                        cols[0].vertical_centered(|ui| { ui.label(egui::RichText::new(format!("{}", stats.ads_blocked)).size(32.0).strong()); ui.label(egui::RichText::new("Ads Blocked").small()); });
                        cols[1].vertical_centered(|ui| { ui.label(egui::RichText::new(format!("{}", stats.tabs_open)).size(32.0).strong()); ui.label(egui::RichText::new("Tabs Open").small()); });
                        cols[2].vertical_centered(|ui| { ui.label(egui::RichText::new(format!("{}", stats.bookmarks_count)).size(32.0).strong()); ui.label(egui::RichText::new("Bookmarks").small()); });
                    });
                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("Quick Links").strong());
                    let bookmarks: Vec<serde_json::Value> = serde_json::from_str(&bmarks_json).unwrap_or_default();
                    ui.horizontal_wrapped(|ui| {
                        for bm in bookmarks.iter().take(12) {
                            let title = bm["title"].as_str().unwrap_or("Bookmark");
                            let url = bm["url"].as_str().unwrap_or("").to_string();
                            if ui.button(truncate_str(title, 18)).clicked() { content_cmds.push(crate::net::ipc::IpcMessage::Navigate { url }); }
                        }
                    });
                });
            } else {
                if self.rendered_url != active_url && !self.render_pending {
                    self.render_pending = true;
                    let pipe = std::sync::Arc::clone(&self.state.pipeline);
                    let url_clone = active_url.clone();
                    let tx = self.state.async_tx.clone();
                    let notify = self.state.async_notify.clone();
                    tokio::spawn(async move {
                        let mut p = pipe.lock().await;
                        match p.fetch_and_render(&url_clone, 1280.0, 2048.0).await {
                            Ok((page, rendered)) => {
                                let script_urls = extract_external_script_urls(&page.html);
                                let base = url::Url::parse(&url_clone).ok();
                                let mut external_scripts: Vec<String> = Vec::new();
                                for script_url in &script_urls {
                                    let resolved = base.as_ref()
                                        .and_then(|b| b.join(script_url).ok())
                                        .map(|u| u.to_string())
                                        .unwrap_or_else(|| script_url.clone());
                                    match p.client.get(&resolved).await {
                                        Ok(resp) => {
                                            if let Ok(text) = resp.text() { external_scripts.push(text); }
                                        }
                                        Err(e) => log::warn!("Script fetch failed {}: {}", resolved, e),
                                    }
                                }
                                let layouts_json: HashMap<String, serde_json::Value> = rendered.layouts.iter()
                                    .map(|(k, r)| (k.to_string(), serde_json::json!({"x": r.x, "y": r.y, "w": r.w, "h": r.h})))
                                    .collect();
                                let resp = serde_json::json!({
                                    "type": "page_painted",
                                    "url": page.url,
                                    "title": page.title,
                                    "html": page.html,
                                    "width": rendered.width,
                                    "height": rendered.height,
                                    "nodes": rendered.node_count,
                                    "commands": rendered.command_count,
                                    "pixels_b64": base64::Engine::encode(
                                        &base64::engine::general_purpose::STANDARD,
                                        &rendered.pixels
                                    ),
                                    "layouts": layouts_json,
                                    "external_scripts": external_scripts,
                                });
                                if let Some(tx) = tx { tx.send(resp.to_string()).ok(); }
                                if let Some(n) = notify { n(); }
                            }
                            Err(e) => log::error!("Render failed: {}", e),
                        }
                    });
                }
                if let Some(tex) = &self.page_texture {
                    let size = tex.size_vec2();
                    let available = ui.available_size();
                    let scale = (available.x / size.x).min(1.0);
                    let display_size = egui::vec2(size.x * scale, size.y * scale);
                    egui::ScrollArea::both().show(ui, |ui| {
                        ui.image(egui::load::SizedTexture::new(tex.id(), display_size));
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.3);
                        ui.spinner();
                        ui.add_space(16.0);
                        ui.label(egui::RichText::new(format!("Rendering: {}", truncate_str(&active_url, 80))).color(egui::Color32::GRAY));
                        ui.label(egui::RichText::new("Amni paint engine active").size(12.0).color(egui::Color32::from_rgb(0, 212, 255)));
                    });
                }
            }
        });
        self.chrome.search_input = new_search;
        for cmd in content_cmds { self.chrome.cmd(cmd); }
        let full_output = self.egui_ctx.end_pass();
        let cmds: Vec<crate::net::ipc::IpcMessage> = self.chrome.drain_commands();
        for cmd in cmds {
            let resp = self.state.handle_command(cmd);
            apply_to_chrome(&mut self.chrome, resp);
        }
        let clipped = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        let gpu = self.gpu.as_ref().unwrap();
        let window = self.window.as_ref().unwrap();
        let frame = match gpu.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                gpu.surface.configure(&gpu.device, &gpu.config); return;
            }
            Err(e) => { log::error!("Surface error: {:?}", e); return; }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("egui_enc") });
        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gpu.config.width, gpu.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        let mut renderer = self.egui_renderer.take().unwrap();
        for (id, delta) in &full_output.textures_delta.set { renderer.update_texture(&gpu.device, &gpu.queue, *id, delta); }
        renderer.update_buffers(&gpu.device, &gpu.queue, &mut encoder, &clipped, &screen_desc);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view, resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.06, g: 0.06, b: 0.09, a: 1.0 }), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        }).forget_lifetime();
        renderer.render(&mut rpass, &clipped, &screen_desc);
        drop(rpass);
        for id in &full_output.textures_delta.free { renderer.free_texture(id); }
        self.egui_renderer = Some(renderer);
        gpu.queue.submit([encoder.finish()]);
        frame.present();
        window.request_redraw();
    }

    fn check_rendered_pages(&mut self) {
        let msgs: Vec<String> = match &self.async_rx {
            Some(r) => r.try_iter().collect(),
            None => return,
        };
        for msg in msgs {
            let val = match serde_json::from_str::<serde_json::Value>(&msg) { Ok(v) => v, Err(_) => continue };
            if val["type"].as_str() != Some("page_painted") { continue; }
            let w = val["width"].as_u64().unwrap_or(0) as u32;
            let h = val["height"].as_u64().unwrap_or(0) as u32;
            let url = val["url"].as_str().unwrap_or("").to_string();
            let title = val["title"].as_str().unwrap_or("").to_string();
            let html = val["html"].as_str().unwrap_or("").to_string();
            let b64 = match val["pixels_b64"].as_str() { Some(s) => s.to_string(), None => continue };
            use base64::Engine;
            let pixels = match base64::engine::general_purpose::STANDARD.decode(&b64) { Ok(p) => p, Err(_) => continue };
            if pixels.len() != (w * h * 4) as usize || w == 0 || h == 0 { continue; }
            let img = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
            self.page_texture = Some(self.egui_ctx.load_texture("page_content", img, egui::TextureOptions::LINEAR));
            self.rendered_url = url.clone();
            self.render_pending = false;
            if !title.is_empty() {
                self.state.handle_command(crate::net::ipc::IpcMessage::UpdateTitle { title });
            }
            if let Some(layouts_val) = val.get("layouts") {
                let mut layouts: HashMap<usize, LayoutRect> = HashMap::new();
                if let Some(obj) = layouts_val.as_object() {
                    for (k, v) in obj {
                        if let Ok(id) = k.parse::<usize>() {
                            layouts.insert(id, LayoutRect {
                                x: v["x"].as_f64().unwrap_or(0.0) as f32,
                                y: v["y"].as_f64().unwrap_or(0.0) as f32,
                                w: v["w"].as_f64().unwrap_or(0.0) as f32,
                                h: v["h"].as_f64().unwrap_or(0.0) as f32,
                            });
                        }
                    }
                }
                self.state.interactor.current_layouts = layouts;
            }
            self.state.interactor.set_page_origin(&url);
            self.page_html = html.clone();
            let ext_scripts: Vec<String> = val.get("external_scripts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            self.run_page_scripts(&html, &ext_scripts);
            log::info!("Page painted: {}x{} ({})", w, h, url);
        }
    }

    fn run_page_scripts(&mut self, html: &str, external_scripts: &[String]) {
        let inline_scripts = extract_scripts(html);
        if inline_scripts.is_empty() && external_scripts.is_empty() { return; }
        let dom = crate::engine::dom::AmniDom::parse(html);
        let sheets = Vec::new();
        let mut counter = 0usize;
        let render_tree = RenderTree::build_from_dom(&dom.dom.document, &sheets, &mut counter);
        self.state.interactor.js_bridge.snapshot_dom(&render_tree);
        for script in external_scripts {
            let result = self.state.interactor.js_bridge.exec_script(script);
            if let Some(err) = &result.error {
                log::warn!("External script error: {}", err);
            }
        }
        for script in &inline_scripts {
            let result = self.state.interactor.js_bridge.exec_script(script);
            if let Some(err) = &result.error {
                log::warn!("Script error: {}", err);
            }
        }
    }
}
#[cfg(feature = "servo-engine")]
fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}…", &s[..max.min(s.len())]) }
}
#[cfg(feature = "servo-engine")]
pub fn run(mut state: BrowserState) {
    let event_loop = EventLoop::new().expect("event loop creation failed");
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    state.async_tx = Some(tx);
    let mut app = AmniApp {
        state, chrome: BrowserChrome::new(), window: None, gpu: None,
        egui_ctx: egui::Context::default(), egui_state: None, egui_renderer: None,
        needs_initial_data: true,
        page_texture: None, rendered_url: String::new(), render_pending: false,
        async_rx: Some(rx),
        cursor_pos: (0.0, 0.0),
        page_html: String::new(),
    };
    log::info!("Amni Browse Servo backend starting...");
    event_loop.run_app(&mut app).expect("event loop failed");
}
