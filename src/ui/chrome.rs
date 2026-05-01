#[cfg(feature = "servo-engine")]
use egui;
#[cfg(feature = "servo-engine")]
use crate::app::BrowserState;
#[cfg(feature = "servo-engine")]
use crate::net::ipc::IpcMessage;
#[cfg(feature = "servo-engine")]
#[derive(PartialEq, Clone)]
pub enum Panel { None, Vault, Themes, Settings, Downloads, History, DevTools, Extensions, Profiles, Autofill, Permissions }
#[cfg(feature = "servo-engine")]
pub struct BrowserChrome {
    pub url_input: String,
    pub search_input: String,
    pub find_input: String,
    pub find_visible: bool,
    pub active_panel: Panel,
    pub vault_master: String,
    pub hist_search: String,
    pub new_prof_name: String,
    pub cred_site: String,
    pub cred_user: String,
    pub cred_pass: String,
    pub cred_notes: String,
    pub status_text: String,
    pub zoom_pct: u32,
    pub pending_cmds: Vec<IpcMessage>,
    pub tabs_json: String,
    pub bookmarks_json: String,
    pub stats: ChromeStats,
    pub vault_unlocked: bool,
    pub vault_creds_json: String,
    pub downloads_json: String,
    pub history_json: String,
    pub extensions_json: String,
    pub profiles_json: String,
    pub permissions_json: String,
    pub dt_console_json: String,
    pub dt_network_json: String,
    pub dt_tab: String,
}
#[cfg(feature = "servo-engine")]
#[derive(Default, Clone)]
pub struct ChromeStats {
    pub ads_blocked: usize,
    pub tabs_open: usize,
    pub bookmarks_count: usize,
    pub passwords_count: usize,
    pub history_count: usize,
    pub downloads_active: usize,
}
#[cfg(feature = "servo-engine")]
impl Default for BrowserChrome {
    fn default() -> Self {
        Self {
            url_input: String::new(), search_input: String::new(), find_input: String::new(),
            find_visible: false, active_panel: Panel::None, vault_master: String::new(),
            hist_search: String::new(), new_prof_name: String::new(),
            cred_site: String::new(), cred_user: String::new(), cred_pass: String::new(), cred_notes: String::new(),
            status_text: "Ready".into(), zoom_pct: 100, pending_cmds: Vec::new(),
            tabs_json: "[]".into(), bookmarks_json: "[]".into(), stats: ChromeStats::default(),
            vault_unlocked: false, vault_creds_json: "[]".into(), downloads_json: "[]".into(),
            history_json: "[]".into(), extensions_json: "[]".into(), profiles_json: "[]".into(),
            permissions_json: "{}".into(), dt_console_json: "[]".into(), dt_network_json: "[]".into(),
            dt_tab: "console".into(),
        }
    }
}
#[cfg(feature = "servo-engine")]
impl BrowserChrome {
    pub fn new() -> Self { Self::default() }
    pub fn cmd(&mut self, msg: IpcMessage) { self.pending_cmds.push(msg); }
    pub fn drain_commands(&mut self) -> Vec<IpcMessage> { std::mem::take(&mut self.pending_cmds) }
    pub fn render(&mut self, ctx: &egui::Context) {
        self.render_tab_bar(ctx);
        self.render_nav_bar(ctx);
        self.render_status_bar(ctx);
        self.render_panel(ctx);
        if self.find_visible { self.render_find_bar(ctx); }
    }
    fn render_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tab_bar").exact_height(32.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let tabs: Vec<serde_json::Value> = serde_json::from_str(&self.tabs_json).unwrap_or_default();
                for tab in &tabs {
                    let title = tab["title"].as_str().unwrap_or("New Tab");
                    let id = tab["id"].as_str().unwrap_or("");
                    let active = tab["is_active"].as_bool().unwrap_or(false);
                    let is_priv = tab["is_private"].as_bool().unwrap_or(false);
                    let label = if is_priv { format!("🕶 {}", truncate(title, 20)) } else { truncate(title, 20) };
                    let btn = ui.selectable_label(active, &label);
                    if btn.clicked() { self.cmd(IpcMessage::SwitchTab { id: id.into() }); }
                    if btn.secondary_clicked() { self.cmd(IpcMessage::CloseTab { id: id.into() }); }
                }
                if ui.small_button("＋").clicked() { self.cmd(IpcMessage::NewTab { url: None }); }
                ui.separator();
                if ui.small_button("🕶＋").on_hover_text("Private Tab").clicked() { self.cmd(IpcMessage::NewPrivateTab { url: None }); }
            });
        });
    }
    fn render_nav_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("nav_bar").exact_height(36.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("◀").on_hover_text("Back").clicked() { self.cmd(IpcMessage::Back); }
                if ui.button("▶").on_hover_text("Forward").clicked() { self.cmd(IpcMessage::Forward); }
                if ui.button("↻").on_hover_text("Refresh").clicked() { self.cmd(IpcMessage::Refresh); }
                let url_resp = ui.add(egui::TextEdit::singleline(&mut self.url_input).desired_width(ui.available_width() - 360.0).hint_text("Search or enter URL..."));
                if url_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let input = self.url_input.trim().to_string();
                    if !input.is_empty() {
                        let url = if input.starts_with("http://") || input.starts_with("https://") || input.starts_with("amnibrowse://") { input }
                        else if input.contains('.') && !input.contains(' ') { format!("https://{}", input) }
                        else { self.cmd(IpcMessage::Search { query: input }); return; };
                        self.cmd(IpcMessage::Navigate { url });
                    }
                }
                if ui.button("☆").on_hover_text("Bookmark").clicked() {
                    self.cmd(IpcMessage::BookmarkAdd { title: self.url_input.clone(), url: self.url_input.clone() });
                }
                if ui.button("🛡").on_hover_text("Toggle Ad Block").clicked() { self.cmd(IpcMessage::ToggleAdBlock); }
                if ui.button("📖").on_hover_text("Reader Mode").clicked() { self.cmd(IpcMessage::ReaderToggle); }
                let zoom_txt = format!("{}%", self.zoom_pct);
                if ui.button(&zoom_txt).on_hover_text("Reset Zoom").clicked() { self.cmd(IpcMessage::ZoomReset); }
                if ui.button("🔐").on_hover_text("Vault").clicked() { self.toggle_panel(Panel::Vault); }
                if ui.button("🎨").on_hover_text("Themes").clicked() { self.toggle_panel(Panel::Themes); }
                if ui.button("📥").on_hover_text("Downloads").clicked() { self.toggle_panel(Panel::Downloads); }
                if ui.button("🕐").on_hover_text("History").clicked() { self.toggle_panel(Panel::History); }
                if ui.button("🔧").on_hover_text("DevTools").clicked() { self.toggle_panel(Panel::DevTools); }
                if ui.button("🧩").on_hover_text("Extensions").clicked() { self.toggle_panel(Panel::Extensions); }
                if ui.button("👤").on_hover_text("Profiles").clicked() { self.toggle_panel(Panel::Profiles); }
                if ui.button("⚙").on_hover_text("Settings").clicked() { self.toggle_panel(Panel::Settings); }
            });
        });
    }
    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").exact_height(20.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.status_text).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("🛡 {} blocked | {} tabs | {} bookmarks", self.stats.ads_blocked, self.stats.tabs_open, self.stats.bookmarks_count)).small());
                });
            });
        });
    }
    fn render_find_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("find_bar").exact_height(28.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Find:");
                let resp = ui.text_edit_singleline(&mut self.find_input);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.cmd(IpcMessage::FindInPage { query: self.find_input.clone() });
                }
                if ui.small_button("▲").clicked() { self.cmd(IpcMessage::FindPrev); }
                if ui.small_button("▼").clicked() { self.cmd(IpcMessage::FindNext); }
                if ui.small_button("✕").clicked() { self.find_visible = false; self.find_input.clear(); self.cmd(IpcMessage::FindClose); }
            });
        });
    }
    fn toggle_panel(&mut self, panel: Panel) {
        self.active_panel = if self.active_panel == panel { Panel::None } else {
            match &panel {
                Panel::Vault => self.cmd(IpcMessage::VaultStatus),
                Panel::Downloads => self.cmd(IpcMessage::DownloadList),
                Panel::History => self.cmd(IpcMessage::HistoryList { limit: Some(100) }),
                Panel::Extensions => self.cmd(IpcMessage::ExtList),
                Panel::Profiles => self.cmd(IpcMessage::ProfileList),
                Panel::Permissions => self.cmd(IpcMessage::PermissionList),
                Panel::Themes => self.cmd(IpcMessage::ThemeList),
                _ => {}
            }
            panel
        };
    }
    fn render_panel(&mut self, ctx: &egui::Context) {
        if self.active_panel == Panel::None { return; }
        egui::SidePanel::right("side_panel").min_width(340.0).max_width(480.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let title = match self.active_panel {
                    Panel::Vault => "🔐 Password Vault", Panel::Themes => "🎨 Themes",
                    Panel::Settings => "⚙️ Settings & Data", Panel::Downloads => "📥 Downloads",
                    Panel::History => "🕐 History", Panel::DevTools => "🔧 DevTools",
                    Panel::Extensions => "🧩 Extensions", Panel::Profiles => "👤 Profiles",
                    Panel::Autofill => "📝 Autofill", Panel::Permissions => "🔒 Permissions",
                    Panel::None => "",
                };
                ui.heading(title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() { self.active_panel = Panel::None; }
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.active_panel {
                    Panel::Vault => self.render_vault_panel(ui),
                    Panel::Downloads => self.render_downloads_panel(ui),
                    Panel::History => self.render_history_panel(ui),
                    Panel::DevTools => self.render_devtools_panel(ui),
                    Panel::Extensions => self.render_extensions_panel(ui),
                    Panel::Profiles => self.render_profiles_panel(ui),
                    Panel::Permissions => self.render_permissions_panel(ui),
                    Panel::Settings => self.render_settings_panel(ui),
                    Panel::Themes => self.render_themes_panel(ui),
                    _ => {}
                }
            });
        });
    }
    fn render_vault_panel(&mut self, ui: &mut egui::Ui) {
        if !self.vault_unlocked {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("🔒");
                ui.label("AES-256-GCM Encrypted Vault");
                ui.label(egui::RichText::new("PBKDF2-HMAC-SHA256 · 600K iterations").small());
                ui.add_space(8.0);
                ui.add(egui::TextEdit::singleline(&mut self.vault_master).password(true).hint_text("Master password..."));
                ui.horizontal(|ui| {
                    if ui.button("Unlock").clicked() { self.cmd(IpcMessage::VaultUnlock { master_password: self.vault_master.clone() }); self.vault_master.clear(); }
                    if ui.button("Initialize").clicked() { self.cmd(IpcMessage::VaultInit { master_password: self.vault_master.clone() }); self.vault_master.clear(); }
                });
            });
        } else {
            ui.horizontal(|ui| {
                if ui.button("+ Add").clicked() {} 
                if ui.button("Generate").clicked() { self.cmd(IpcMessage::VaultGenerate { length: Some(24) }); }
                if ui.button("🔒 Lock").clicked() { self.cmd(IpcMessage::VaultLock); }
            });
            ui.separator();
            let creds: Vec<serde_json::Value> = serde_json::from_str(&self.vault_creds_json).unwrap_or_default();
            for c in &creds {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(c["site"].as_str().unwrap_or(""));
                            ui.label(egui::RichText::new(c["username"].as_str().unwrap_or("")).small());
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let id = c["id"].as_str().unwrap_or("").to_string();
                            if ui.small_button("🗑").clicked() { self.cmd(IpcMessage::VaultRemove { id: id.clone() }); }
                            if ui.small_button("📋").clicked() { self.cmd(IpcMessage::VaultGetPassword { id }); }
                        });
                    });
                });
            }
        }
    }
    fn render_downloads_panel(&mut self, ui: &mut egui::Ui) {
        if ui.button("Clear Completed").clicked() { self.cmd(IpcMessage::DownloadClear); }
        ui.separator();
        let items: Vec<serde_json::Value> = serde_json::from_str(&self.downloads_json).unwrap_or_default();
        if items.is_empty() { ui.centered_and_justified(|ui| ui.label("No downloads")); return; }
        for d in &items {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.strong(d["filename"].as_str().unwrap_or("unknown"));
                        ui.label(egui::RichText::new(d["status"].as_str().unwrap_or("")).small());
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let id = d["id"].as_str().unwrap_or("").to_string();
                        if ui.small_button("🗑").clicked() { self.cmd(IpcMessage::DownloadRemove { id: id.clone() }); }
                        if d["status"].as_str() == Some("Downloading") && ui.small_button("⏹").clicked() { self.cmd(IpcMessage::DownloadCancel { id }); }
                    });
                });
            });
        }
    }
    fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let resp = ui.add(egui::TextEdit::singleline(&mut self.hist_search).hint_text("Search history...").desired_width(200.0));
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let q = self.hist_search.clone();
                self.cmd(if q.is_empty() { IpcMessage::HistoryList { limit: Some(100) } } else { IpcMessage::HistorySearch { query: q } });
            }
            if ui.button("Clear All").clicked() { self.cmd(IpcMessage::HistoryClear); }
        });
        ui.separator();
        let items: Vec<serde_json::Value> = serde_json::from_str(&self.history_json).unwrap_or_default();
        if items.is_empty() { ui.centered_and_justified(|ui| ui.label("No history")); return; }
        for h in &items {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let url = h["url"].as_str().unwrap_or("").to_string();
                    if ui.link(h["title"].as_str().unwrap_or(&url)).clicked() { self.cmd(IpcMessage::Navigate { url }); self.active_panel = Panel::None; }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("🗑").clicked() { self.cmd(IpcMessage::HistoryDelete { id: h["id"].as_str().unwrap_or("").into() }); }
                    });
                });
            });
        }
    }
    fn render_devtools_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.dt_tab, "console".into(), "Console");
            ui.selectable_value(&mut self.dt_tab, "network".into(), "Network");
            if ui.button("Clear").clicked() {
                self.cmd(if self.dt_tab == "console" { IpcMessage::DevToolsClearConsole } else { IpcMessage::DevToolsClearNetwork });
            }
        });
        ui.separator();
        let json = if self.dt_tab == "console" { &self.dt_console_json } else { &self.dt_network_json };
        let entries: Vec<serde_json::Value> = serde_json::from_str(json).unwrap_or_default();
        for e in &entries {
            let txt = if self.dt_tab == "console" {
                format!("[{}] {}", e["level"].as_str().unwrap_or("log"), e["message"].as_str().unwrap_or(""))
            } else {
                format!("{} {} → {}", e["method"].as_str().unwrap_or("GET"), e["url"].as_str().unwrap_or(""), e["status"].as_u64().unwrap_or(0))
            };
            ui.label(egui::RichText::new(txt).monospace().small());
        }
    }
    fn render_extensions_panel(&mut self, ui: &mut egui::Ui) {
        if ui.button("Scan Extensions").clicked() { self.cmd(IpcMessage::ExtScan); }
        ui.separator();
        let exts: Vec<serde_json::Value> = serde_json::from_str(&self.extensions_json).unwrap_or_default();
        if exts.is_empty() { ui.centered_and_justified(|ui| ui.label("No extensions installed")); return; }
        for ext in &exts {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let id = ext["id"].as_str().unwrap_or("").to_string();
                    let enabled = ext["enabled"].as_bool().unwrap_or(false);
                    ui.vertical(|ui| {
                        ui.strong(format!("{} v{}", ext["name"].as_str().unwrap_or(""), ext["version"].as_str().unwrap_or("")));
                        ui.label(egui::RichText::new(if enabled { "Enabled" } else { "Disabled" }).small());
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("🗑").clicked() { self.cmd(IpcMessage::ExtRemove { id: id.clone() }); }
                        let toggle = if enabled { IpcMessage::ExtDisable { id } } else { IpcMessage::ExtEnable { id } };
                        if ui.small_button(if enabled { "⏸" } else { "▶" }).clicked() { self.cmd(toggle); }
                    });
                });
            });
        }
    }
    fn render_profiles_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.new_prof_name).hint_text("New profile name...").desired_width(180.0));
            if ui.button("Create").clicked() && !self.new_prof_name.is_empty() {
                self.cmd(IpcMessage::ProfileCreate { name: self.new_prof_name.clone(), color: "#00d4ff".into() });
                self.new_prof_name.clear();
            }
        });
        ui.separator();
        let profs: Vec<serde_json::Value> = serde_json::from_str(&self.profiles_json).unwrap_or_default();
        for p in &profs {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let id = p["id"].as_str().unwrap_or("").to_string();
                    let is_default = p["is_default"].as_bool().unwrap_or(false);
                    ui.strong(p["name"].as_str().unwrap_or(""));
                    if is_default { ui.label(egui::RichText::new("(active)").small()); }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !is_default {
                            if ui.small_button("🗑").clicked() { self.cmd(IpcMessage::ProfileDelete { id: id.clone() }); }
                            if ui.small_button("→").on_hover_text("Switch").clicked() { self.cmd(IpcMessage::ProfileSwitch { id }); }
                        }
                    });
                });
            });
        }
    }
    fn render_permissions_panel(&mut self, ui: &mut egui::Ui) {
        if ui.button("Reset All Permissions").clicked() { self.cmd(IpcMessage::PermissionReset { url: "*".into() }); }
        ui.separator();
        ui.label(egui::RichText::new("Per-site permissions are set automatically when sites request access.").small());
    }
    fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Clear Browsing Data").strong());
        ui.separator();
        if ui.button("Clear Cache").clicked() { self.cmd(IpcMessage::ClearData { categories: vec!["cache".into()] }); }
        if ui.button("Clear Cookies").clicked() { self.cmd(IpcMessage::ClearData { categories: vec!["cookies".into()] }); }
        if ui.button("Clear History").clicked() { self.cmd(IpcMessage::ClearData { categories: vec!["history".into()] }); }
        if ui.button("Clear Downloads").clicked() { self.cmd(IpcMessage::ClearData { categories: vec!["downloads".into()] }); }
        ui.add_space(8.0);
        if ui.button(egui::RichText::new("⚠ Clear Everything").color(egui::Color32::RED)).clicked() { self.cmd(IpcMessage::ClearAllData); }
        ui.add_space(16.0);
        ui.separator();
        ui.label(egui::RichText::new("Privacy & Network").strong());
        if ui.button("Toggle Ad Blocker").clicked() { self.cmd(IpcMessage::ToggleAdBlock); }
        if ui.button("Toggle DNS-over-HTTPS").clicked() { self.cmd(IpcMessage::DohToggle); }
        if ui.button("Save Session Now").clicked() { self.cmd(IpcMessage::SessionSave); }
        ui.add_space(16.0);
        ui.separator();
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("Amni Browse — Amni-Scient\nAll data stored locally · Zero telemetry").small());
        });
    }
    fn render_themes_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("Select a theme:");
        ui.separator();
        let themes: &[(&str, &str)] = &[
            ("amni-dark", "Amni Dark"),
            ("amni-cosmos", "Amni Cosmos"),
            ("amni-emerald", "Amni Emerald"),
            ("amni-light", "Amni Light"),
            ("amni-crimson", "Amni Crimson"),
            ("amni-solarflare", "Solar Flare"),
            ("amni-mint-matrix", "Mint Matrix"),
            ("amni-paper-sunset", "Paper Sunset"),
            ("amni-deep-space", "Deep Space"),
        ];
        for (id, name) in themes {
            if ui.button(*name).clicked() { self.cmd(IpcMessage::ThemeSet { theme_id: id.to_string() }); }
        }
    }
    pub fn handle_keyboard(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            let ctrl = i.modifiers.ctrl || i.modifiers.mac_cmd;
            if ctrl && i.key_pressed(egui::Key::T) { self.cmd(IpcMessage::NewTab { url: None }); }
            if ctrl && i.key_pressed(egui::Key::W) {
                let tabs: Vec<serde_json::Value> = serde_json::from_str(&self.tabs_json).unwrap_or_default();
                tabs.iter().find(|t| t["is_active"].as_bool() == Some(true)).and_then(|t| t["id"].as_str()).map(|id| self.pending_cmds.push(IpcMessage::CloseTab { id: id.into() }));
            }
            if ctrl && i.key_pressed(egui::Key::F) { self.find_visible = !self.find_visible; }
            if ctrl && i.key_pressed(egui::Key::H) { self.toggle_panel(Panel::History); }
            if ctrl && i.key_pressed(egui::Key::J) { self.toggle_panel(Panel::Downloads); }
            if ctrl && i.key_pressed(egui::Key::Equals) { self.cmd(IpcMessage::ZoomIn); }
            if ctrl && i.key_pressed(egui::Key::Minus) { self.cmd(IpcMessage::ZoomOut); }
            if ctrl && i.key_pressed(egui::Key::Num0) { self.cmd(IpcMessage::ZoomReset); }
            if ctrl && i.modifiers.shift && i.key_pressed(egui::Key::P) { self.toggle_panel(Panel::Vault); }
            if ctrl && i.modifiers.shift && i.key_pressed(egui::Key::I) { self.toggle_panel(Panel::DevTools); }
            if ctrl && i.modifiers.shift && i.key_pressed(egui::Key::N) { self.cmd(IpcMessage::NewPrivateTab { url: None }); }
            if i.key_pressed(egui::Key::Escape) { self.active_panel = Panel::None; self.find_visible = false; }
        });
    }
}
#[cfg(feature = "servo-engine")]
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}…", &s[..max.min(s.len())]) }
}
