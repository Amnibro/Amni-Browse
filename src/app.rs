use log::{error, info};
use crate::engine::adblocker::AdBlocker;
use crate::engine::drm_fallback::DrmFallbackManager;
use crate::engine::pipeline::{RenderPipeline, PageInteractor};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use crate::crypto::autofill::{AddressProfile, AutofillManager};
use crate::storage::bookmarks::BookmarkManager;
use crate::storage::config::BrowserConfig;
use crate::engine::devtools::{DevToolsState as DTS, LogLevel};
use crate::net::dns::{DohProvider, DohResolver};
use crate::storage::downloads::DownloadManager;
use crate::engine::extensions::ExtensionManager;
use crate::storage::history::HistoryManager;
use crate::net::ipc::{IpcMessage, IpcResponse};
use crate::crypto::vault::PasswordManager;
use crate::engine::permissions::{PermissionState, PermissionType, PermissionsManager};
use crate::storage::profiles::ProfileManager;
use crate::ui::reader::ReaderMode;
use crate::storage::session::SessionManager;
use crate::engine::tabs::{SplitViewMode, TabManager};
use crate::engine::js::JsRuntime;
use crate::media::MediaManager;
use crate::ui::theme::ThemeConfig;
pub struct BrowserState {
    pub tabs: TabManager,
    pub bookmarks: BookmarkManager,
    pub ad_blocker: AdBlocker,
    pub config: BrowserConfig,
    pub passwords: PasswordManager,
    pub themes: ThemeConfig,
    pub downloads: DownloadManager,
    pub history: HistoryManager,
    pub session: SessionManager,
    pub autofill: AutofillManager,
    pub permissions: PermissionsManager,
    pub doh: DohResolver,
    pub devtools: DTS,
    pub extensions: ExtensionManager,
    pub profiles: ProfileManager,
    pub reader: ReaderMode,
    pub pipeline: Arc<TokioMutex<RenderPipeline>>,
    pub js_runtime: JsRuntime,
    pub media: MediaManager,
    pub drm: DrmFallbackManager,
    pub interactor: PageInteractor,
    pub async_tx: Option<std::sync::mpsc::Sender<String>>,
    pub async_notify: Option<Arc<dyn Fn() + Send + Sync>>,
}
impl BrowserState {
    pub fn new() -> Self {
        let config = BrowserConfig::load();
        let themes = ThemeConfig::load();
        info!("Config loaded from {:?}", BrowserConfig::config_dir());
        info!("Ad blocking: {}, Tracker blocking: {}", config.block_ads, config.block_trackers);
        info!("Active theme: {}", themes.active_theme().name);
        let restore_session = config.restore_session;
        let doh_enabled = config.enable_doh;
        let mut state = Self {
            tabs: TabManager::new(),
            bookmarks: BookmarkManager::new(),
            ad_blocker: AdBlocker::new(config.block_ads, config.block_trackers),
            passwords: PasswordManager::new(),
            themes,
            downloads: DownloadManager::new(),
            history: HistoryManager::new(),
            session: SessionManager::new(restore_session),
            autofill: AutofillManager::new(),
            permissions: PermissionsManager::new(),
            doh: DohResolver::new(doh_enabled, DohProvider::default()),
            devtools: DTS::new(),
            extensions: ExtensionManager::new(),
            profiles: ProfileManager::new(),
            reader: ReaderMode::new(),
            pipeline: Arc::new(TokioMutex::new(RenderPipeline::new())),
            js_runtime: JsRuntime::new(),
            media: MediaManager::new(),
            drm: DrmFallbackManager::new(),
            interactor: PageInteractor::new(),
            async_tx: None,
            async_notify: None,
            config,
        };
        state.extensions.scan_extensions();
        if SessionManager::was_crash() { info!("Crash recovery: previous session detected"); }
        SessionManager::create_lock();
        if state.session.restore_on_start {
            let rd = state.session.get_restore_data();
            let mut count = 0usize;
            for stab in &rd {
                if stab.url.is_empty() { continue; }
                let tid = state.tabs.new_tab(&stab.url);
                if let Some(tab) = state.tabs.tabs.iter_mut().find(|t| t.id == tid) {
                    tab.title = stab.title.clone();
                    tab.history = stab.history.clone();
                    tab.history_index = stab.history_index;
                    tab.is_active = stab.is_active;
                }
                count += 1;
            }
            if count > 0 { info!("Restored {} tabs from previous session", count); }
        }
        state
    }
    pub fn shutdown(&mut self) {
        info!("Shutting down Amni Browse...");
        let snap: Vec<crate::storage::session::SessionTab> = self.tabs.tabs.iter().map(|t| crate::storage::session::SessionTab {
            url: t.url.clone(), title: t.title.clone(), is_active: t.is_active,
            history: t.history.clone(), history_index: t.history_index,
        }).collect();
        self.session.capture(snap);
        self.session.save_clean_exit();
        if self.config.clear_data_on_exit { info!("Clearing browsing data..."); self.config.clear_data(); }
        self.config.save();
    }
    pub fn handle_command(&mut self, msg: IpcMessage) -> Option<IpcResponse> {
        match msg {
            IpcMessage::Navigate { url } => {
                let clean = AdBlocker::clean_url(&url);
                info!("Navigate: {}", clean);
                let drm = self.drm.should_use_webview(&clean);
                if drm { info!("DRM domain detected: {} (WebView will handle DRM if needed)", clean); }
                self.drm.log_navigation(&clean, drm);
                let priv_tab = self.tabs.active_tab().map_or(false, |t| t.is_private);
                if !priv_tab && !clean.starts_with("amnibrowse://") { self.history.record_visit(&clean, ""); }
                self.tabs.active_tab_mut().map(|t| t.navigate(&clean));
                Some(IpcResponse::NavigateTo { url: clean })
            }
            IpcMessage::Back => self.tabs.active_tab_mut().and_then(|t| t.go_back().map(|u| u.to_string())).map(|u| { info!("Back to: {}", u); IpcResponse::NavigateTo { url: u } }),
            IpcMessage::Forward => self.tabs.active_tab_mut().and_then(|t| t.go_forward().map(|u| u.to_string())).map(|u| { info!("Forward to: {}", u); IpcResponse::NavigateTo { url: u } }),
            IpcMessage::Refresh => { info!("Refresh"); None }
            IpcMessage::NewTab { url } => {
                let target = url.as_deref().unwrap_or("amnibrowse://newtab");
                let id = self.tabs.new_tab(target);
                info!("New tab: {} ({})", id, target);
                Some(IpcResponse::TabsUpdated { tabs: self.tabs.to_json() })
            }
            IpcMessage::CloseTab { id } => {
                self.tabs.close_tab(&id);
                info!("Closed tab: {}", id);
                if self.tabs.tab_count() == 0 { self.tabs.new_tab("amnibrowse://newtab"); }
                Some(IpcResponse::TabsUpdated { tabs: self.tabs.to_json() })
            }
            IpcMessage::SwitchTab { id } => { self.tabs.switch_tab(&id); info!("Switched to tab: {}", id); Some(IpcResponse::TabsUpdated { tabs: self.tabs.to_json() }) }
            IpcMessage::NewPrivateTab { url } => {
                let target = url.as_deref().unwrap_or("amnibrowse://newtab");
                let id = self.tabs.new_private_tab(target);
                info!("New private tab: {} ({})", id, target);
                Some(IpcResponse::TabsUpdated { tabs: self.tabs.to_json() })
            }
            IpcMessage::SplitTab { mode, url } => {
                let sm = match mode.as_str() { "horizontal" => SplitViewMode::Horizontal, "vertical" => SplitViewMode::Vertical, _ => SplitViewMode::None };
                self.tabs.active_tab_mut().map(|t| { t.set_split(sm, url.as_deref()); info!("Split tab: {}", mode); });
                None
            }
            IpcMessage::CloseSplit => { self.tabs.active_tab_mut().map(|t| { t.clear_split(); info!("Split closed"); }); None }
            IpcMessage::BookmarkAdd { title, url } => { self.bookmarks.add(&title, &url, None); info!("Bookmarked: {} ({})", title, url); None }
            IpcMessage::BookmarkRemove { id } => { self.bookmarks.remove(&id); info!("Removed bookmark: {}", id); None }
            IpcMessage::BookmarkList => Some(IpcResponse::Bookmarks { data: self.bookmarks.to_json() }),
            IpcMessage::GetTabs => Some(IpcResponse::TabsUpdated { tabs: self.tabs.to_json() }),
            IpcMessage::GetStats => Some(IpcResponse::Stats {
                ads_blocked: self.ad_blocker.blocked_count(), tabs_open: self.tabs.tab_count(),
                bookmarks_count: self.bookmarks.bookmarks.len(), passwords_count: self.passwords.list_credentials().len(),
                history_count: self.history.entry_count(), downloads_active: self.downloads.downloads.iter().filter(|d| d.status == crate::storage::downloads::DownloadStatus::Downloading).count(),
            }),
            IpcMessage::Search { query } => {
                let url = format!("{}{}", self.config.search_engine, urlencoding::encode(&query));
                info!("Search: {}", query);
                let priv_tab = self.tabs.active_tab().map_or(false, |t| t.is_private);
                if !priv_tab { self.history.record_visit(&url, &format!("Search: {}", query)); }
                self.tabs.active_tab_mut().map(|t| t.navigate(&url));
                Some(IpcResponse::NavigateTo { url })
            }
            IpcMessage::UpdateTitle { title } => {
                let priv_tab = self.tabs.active_tab().map_or(false, |t| t.is_private);
                let url = self.tabs.active_tab().map(|t| t.url.clone()).unwrap_or_default();
                if !priv_tab && !url.starts_with("amnibrowse://") { self.history.record_visit(&url, &title); }
                self.tabs.active_tab_mut().map(|t| t.title = title);
                None
            }
            IpcMessage::ToggleAdBlock => {
                self.ad_blocker.enabled = !self.ad_blocker.enabled;
                info!("Ad blocker: {}", if self.ad_blocker.enabled { "ON" } else { "OFF" });
                None
            }
            IpcMessage::VaultInit { master_password } => {
                match self.passwords.initialize(&master_password) { Ok(()) => info!("Vault initialized"), Err(e) => error!("Vault init failed: {}", e) }
                Some(IpcResponse::VaultStatus { initialized: self.passwords.is_initialized(), unlocked: self.passwords.is_unlocked(), count: self.passwords.list_credentials().len() })
            }
            IpcMessage::VaultUnlock { master_password } => {
                match self.passwords.unlock(&master_password) {
                    Ok(()) => { info!("Vault unlocked"); if let Some(key) = self.passwords.derived_key() { self.autofill.set_encryption_key(key); } }
                    Err(e) => error!("Vault unlock failed: {}", e),
                }
                Some(IpcResponse::VaultStatus { initialized: self.passwords.is_initialized(), unlocked: self.passwords.is_unlocked(), count: self.passwords.list_credentials().len() })
            }
            IpcMessage::VaultLock => { self.passwords.lock(); info!("Vault locked"); Some(IpcResponse::VaultStatus { initialized: true, unlocked: false, count: 0 }) }
            IpcMessage::VaultStatus => Some(IpcResponse::VaultStatus { initialized: self.passwords.is_initialized(), unlocked: self.passwords.is_unlocked(), count: self.passwords.list_credentials().len() }),
            IpcMessage::VaultAdd { site, username, password, notes, category } => {
                match self.passwords.add_credential(&site, &username, &password, notes.as_deref(), category.as_deref()) {
                    Ok(id) => info!("Credential added: {} for {}", id, site),
                    Err(e) => error!("Add credential failed: {}", e),
                }
                None
            }
            IpcMessage::VaultRemove { id } => { self.passwords.remove_credential(&id); info!("Credential removed: {}", id); None }
            IpcMessage::VaultList => Some(IpcResponse::VaultCredentials { data: self.passwords.to_json() }),
            IpcMessage::VaultGetPassword { id } => match self.passwords.get_password(&id) { Ok(pw) => Some(IpcResponse::VaultPassword { password: pw }), Err(e) => { error!("Get password failed: {}", e); None } },
            IpcMessage::VaultGenerate { length } => Some(IpcResponse::VaultGenerated { password: PasswordManager::generate_password(length.unwrap_or(20)) }),
            IpcMessage::ThemeList => Some(IpcResponse::Themes { data: self.themes.all_themes_json() }),
            IpcMessage::ThemeSet { theme_id } => {
                self.themes.set_theme(&theme_id);
                info!("Theme set to: {}", theme_id);
                Some(IpcResponse::ActiveTheme { data: self.themes.active_theme_json() })
            }
            IpcMessage::ThemeGetActive => Some(IpcResponse::ActiveTheme { data: self.themes.active_theme_json() }),
            IpcMessage::ThemeSaveCustom { theme } => {
                match serde_json::from_str::<crate::ui::theme::Theme>(&theme) {
                    Ok(t) => { info!("Custom theme saved: {}", t.name); self.themes.add_custom_theme(t); }
                    Err(_) => error!("Invalid custom theme JSON"),
                }
                None
            }
            IpcMessage::ThemeRemoveCustom { theme_id } => { self.themes.remove_custom_theme(&theme_id); info!("Custom theme removed: {}", theme_id); None }
            IpcMessage::ClearData { categories } => {
                info!("Clearing data: {:?}", categories);
                for cat in &categories {
                    match cat.as_str() {
                        "cache" => { let d = BrowserConfig::cache_dir(); if d.exists() { std::fs::remove_dir_all(&d).ok(); std::fs::create_dir_all(&d).ok(); } }
                        "cookies" => { std::fs::remove_file(BrowserConfig::config_dir().join("cookies.json")).ok(); }
                        "history" => self.history.clear_all(),
                        "passwords" => self.passwords.wipe_vault(),
                        "downloads" => self.downloads.clear_completed(),
                        _ => {}
                    }
                }
                None
            }
            IpcMessage::ClearAllData => {
                info!("Clearing ALL data");
                BrowserConfig::clear_all_data_now();
                self.passwords.wipe_vault();
                self.history.clear_all();
                self.downloads.clear_completed();
                None
            }
            IpcMessage::GetConfig => Some(IpcResponse::Config { data: serde_json::to_string(&self.config).unwrap_or_default() }),
            IpcMessage::SaveConfig { config } => {
                match serde_json::from_str::<BrowserConfig>(&config) { Ok(nc) => { self.config = nc; self.config.save(); info!("Config saved"); } Err(_) => error!("Invalid config JSON") }
                None
            }
            IpcMessage::SaveLayout { layout } => { std::fs::write(BrowserConfig::config_dir().join("layout.json"), &layout).ok(); info!("Layout saved"); None }
            IpcMessage::GetLayout => { let data = std::fs::read_to_string(BrowserConfig::config_dir().join("layout.json")).unwrap_or_else(|_| "{}".into()); Some(IpcResponse::Layout { data }) }
            IpcMessage::DownloadStart { url } => { let id = self.downloads.start_download(&url); let fname = url.rsplit('/').next().unwrap_or("download").to_string(); info!("Download started: {}", id); Some(IpcResponse::DownloadStarted { id, filename: fname }) }
            IpcMessage::DownloadCancel { id } => { self.downloads.cancel_download(&id); info!("Download cancelled: {}", id); None }
            IpcMessage::DownloadRemove { id } => { self.downloads.remove_download(&id); info!("Download removed: {}", id); None }
            IpcMessage::DownloadClear => { self.downloads.clear_completed(); info!("Downloads cleared"); None }
            IpcMessage::DownloadList => Some(IpcResponse::Downloads { data: self.downloads.to_json() }),
            IpcMessage::HistoryList { limit } => Some(IpcResponse::History { data: self.history.recent_json(limit.unwrap_or(100)) }),
            IpcMessage::HistorySearch { query } => {
                let results: Vec<&crate::storage::history::HistoryEntry> = self.history.search(&query);
                Some(IpcResponse::History { data: serde_json::to_string(&results).unwrap_or_else(|_| "[]".into()) })
            }
            IpcMessage::HistoryDelete { id } => { self.history.delete_entry(&id); info!("History entry deleted: {}", id); None }
            IpcMessage::HistoryClear => { self.history.clear_all(); info!("History cleared"); None }
            IpcMessage::FindInPage { query: _ } => Some(IpcResponse::FindResult { found: true, current: 0, total: 0 }),
            IpcMessage::FindNext | IpcMessage::FindPrev | IpcMessage::FindClose => None,
            IpcMessage::SessionSave => {
                let snap: Vec<crate::storage::session::SessionTab> = self.tabs.tabs.iter().map(|t| crate::storage::session::SessionTab {
                    url: t.url.clone(), title: t.title.clone(), is_active: t.is_active,
                    history: t.history.clone(), history_index: t.history_index,
                }).collect();
                self.session.capture(snap); self.session.save(); info!("Session saved");
                None
            }
            IpcMessage::SessionRestore => {
                let rd = self.session.get_restore_data();
                let mut count = 0usize;
                for stab in &rd {
                    if stab.url.is_empty() { continue; }
                    let tid = self.tabs.new_tab(&stab.url);
                    if let Some(tab) = self.tabs.tabs.iter_mut().find(|t| t.id == tid) {
                        tab.title = stab.title.clone(); tab.history = stab.history.clone();
                        tab.history_index = stab.history_index; tab.is_active = stab.is_active;
                    }
                    count += 1;
                }
                info!("Restored {} tabs", count);
                Some(IpcResponse::TabsUpdated { tabs: self.tabs.to_json() })
            }
            IpcMessage::AutofillAddAddress { label, full_name, street, city, state, zip, country, phone, email } => {
                let addr = AddressProfile::new(&label, &full_name, &street, &city, &state, &zip, &country, &phone, &email);
                let id = self.autofill.add_address(addr);
                info!("Address added: {}", id);
                None
            }
            IpcMessage::AutofillRemoveAddress { id } => { self.autofill.remove_address(&id); info!("Address removed: {}", id); None }
            IpcMessage::AutofillAddCard { label, cardholder, number, expiry, card_type } => {
                match self.autofill.add_card(&label, &cardholder, &number, &expiry, &card_type) { Ok(id) => info!("Card added: {}", id), Err(e) => error!("Add card failed: {}", e) }
                None
            }
            IpcMessage::AutofillRemoveCard { id } => { self.autofill.remove_card(&id); info!("Card removed: {}", id); None }
            IpcMessage::AutofillList => Some(IpcResponse::AutofillData { addresses: self.autofill.addresses_json(), cards: self.autofill.cards_json() }),
            IpcMessage::AutofillSuggest { url } => {
                let sugg = self.autofill.suggest_for_site(&url);
                Some(IpcResponse::AutofillSuggestions { data: serde_json::to_string(&sugg).unwrap_or_else(|_| "{}".into()) })
            }
            IpcMessage::ZoomIn => { self.tabs.zoom_in(); Some(IpcResponse::ZoomLevel { level: self.tabs.active_tab().map_or(1.0, |t| t.zoom_level) }) }
            IpcMessage::ZoomOut => { self.tabs.zoom_out(); Some(IpcResponse::ZoomLevel { level: self.tabs.active_tab().map_or(1.0, |t| t.zoom_level) }) }
            IpcMessage::ZoomReset => { self.tabs.zoom_reset(); Some(IpcResponse::ZoomLevel { level: 1.0 }) }
            IpcMessage::ZoomSet { level } => { self.tabs.zoom_set(level); Some(IpcResponse::ZoomLevel { level }) }
            IpcMessage::ReaderToggle => { let active = self.reader.toggle(); info!("Reader mode: {}", if active { "ON" } else { "OFF" }); None }
            IpcMessage::ReaderSettings { settings } => { if let Ok(s) = serde_json::from_str(&settings) { self.reader.settings = s; } None }
            IpcMessage::ReaderContent { title, content } => {
                let dom = crate::engine::dom::AmniDom::parse(&content);
                let (extracted_title, extracted_content) = dom.extract_reader_content();
                let final_title = if extracted_title.is_empty() { title } else { extracted_title };
                let html = self.reader.render_html(&final_title, &extracted_content);
                Some(IpcResponse::ReaderHtml { html, active: self.reader.active })
            }
            IpcMessage::PermissionSet { url, permission, state: ps } => {
                let ptype = match permission.as_str() {
                    "camera" => Some(PermissionType::Camera), "microphone" => Some(PermissionType::Microphone),
                    "location" => Some(PermissionType::Location), "notifications" => Some(PermissionType::Notifications),
                    "clipboard" => Some(PermissionType::Clipboard), "fullscreen" => Some(PermissionType::Fullscreen),
                    "autoplay" => Some(PermissionType::Autoplay), "popups" => Some(PermissionType::Popups), _ => None,
                };
                let pstate = match ps.as_str() { "allow" => Some(PermissionState::Allow), "deny" => Some(PermissionState::Deny), "ask" => Some(PermissionState::Ask), _ => None };
                if let (Some(pt), Some(pst)) = (ptype, pstate) { self.permissions.set_permission(&url, pt, pst); info!("Permission set for {}: {} = {}", url, permission, ps); }
                None
            }
            IpcMessage::PermissionGet { url } => Some(IpcResponse::Permissions { data: serde_json::to_string(&self.permissions.sites.iter().find(|s| s.site == url)).unwrap_or_else(|_| "null".into()) }),
            IpcMessage::PermissionList => Some(IpcResponse::Permissions { data: self.permissions.to_json() }),
            IpcMessage::PermissionReset { url } => { self.permissions.reset_site(&url); info!("Permissions reset for: {}", url); None }
            IpcMessage::DohToggle => { self.doh.enabled = !self.doh.enabled; info!("DNS-over-HTTPS: {}", if self.doh.enabled { "ON" } else { "OFF" }); None }
            IpcMessage::DohSetProvider { provider } => {
                let p = match provider.as_str() { "cloudflare" => DohProvider::Cloudflare, "google" => DohProvider::Google, "quad9" => DohProvider::Quad9, _ => DohProvider::Custom(provider.clone()) };
                self.doh.set_provider(p); info!("DoH provider set to: {}", provider);
                None
            }
            IpcMessage::DohStatus => Some(IpcResponse::DohStatusResp { enabled: self.doh.enabled, provider: self.doh.provider_json(), cache_size: self.doh.cache_size() }),
            IpcMessage::DevToolsToggle => { let open = self.devtools.toggle(); info!("DevTools: {}", if open { "OPEN" } else { "CLOSED" }); None }
            IpcMessage::DevToolsPanel { panel } => { self.devtools.set_panel(&panel); None }
            IpcMessage::DevToolsConsoleLog { level, message, source, line } => {
                let lvl = match level.as_str() { "warn" => LogLevel::Warn, "error" => LogLevel::Error, "info" => LogLevel::Info, "debug" => LogLevel::Debug, _ => LogLevel::Log };
                self.devtools.log_console(lvl, &message, source.as_deref(), line);
                None
            }
            IpcMessage::DevToolsClearConsole => { self.devtools.clear_console(); None }
            IpcMessage::DevToolsClearNetwork => { self.devtools.clear_network(); None }
            IpcMessage::DevToolsState => Some(IpcResponse::DevToolsStateResp { data: self.devtools.state_json() }),
            IpcMessage::ExtList => Some(IpcResponse::Extensions { data: self.extensions.to_json() }),
            IpcMessage::ExtEnable { id } => { self.extensions.enable(&id); info!("Extension enabled: {}", id); None }
            IpcMessage::ExtDisable { id } => { self.extensions.disable(&id); info!("Extension disabled: {}", id); None }
            IpcMessage::ExtRemove { id } => { self.extensions.remove(&id); info!("Extension removed: {}", id); None }
            IpcMessage::ExtScan => { self.extensions.scan_extensions(); info!("Extensions rescanned"); None }
            IpcMessage::ProfileList => Some(IpcResponse::Profiles { data: self.profiles.to_json(), active_id: self.profiles.active_profile().id.clone() }),
            IpcMessage::ProfileCreate { name, color } => { let id = self.profiles.create_profile(&name, &color); info!("Profile created: {}", id); None }
            IpcMessage::ProfileSwitch { id } => { self.profiles.switch_profile(&id); info!("Switched to profile: {}", id); None }
            IpcMessage::ProfileDelete { id } => { self.profiles.delete_profile(&id); info!("Profile deleted: {}", id); None }
            IpcMessage::ProfileRename { id, name } => { self.profiles.rename_profile(&id, &name); info!("Profile renamed: {}", id); None }
            IpcMessage::FetchPage { url } => {
                let clean = AdBlocker::clean_url(&url);
                info!("Engine fetch: {}", clean);
                let pipe = Arc::clone(&self.pipeline);
                let tx = self.async_tx.clone();
                let notify = self.async_notify.clone();
                let clean2 = clean.clone();
                tokio::spawn(async move {
                    let p = pipe.lock().await;
                    match p.fetch_and_parse(&clean2).await {
                        Ok(result) => {
                            let meta_json = serde_json::json!({
                                "description": result.meta.description,
                                "lang": result.meta.lang,
                                "links": result.meta.link_count,
                                "images": result.meta.image_count,
                                "scripts": result.meta.script_count,
                                "stylesheets": result.meta.stylesheet_count,
                                "headings": result.meta.heading_count,
                                "text_length": result.meta.text_length,
                            }).to_string();
                              
                              let mut html = result.html;
                              if let Ok(rx) = regex::RegexBuilder::new(r#"<meta\s+[^>]*http-equiv\s*=\s*['"]?refresh['"]?[^>]*>"#).case_insensitive(true).build() {
                                  html = rx.replace_all(&html, "").to_string();
                              }
                              if let Ok(rx_base) = regex::RegexBuilder::new(r#"<base\s+[^>]*>"#).case_insensitive(true).build() {
                                  html = rx_base.replace_all(&html, "").to_string();
                              }

                              log::info!("Engine fetched: {} ({}b)", result.title, html.len());
                              let resp = IpcResponse::PageRendered { url: result.url, title: result.title, html, meta: meta_json };
                            if let Some(tx) = tx { tx.send(resp.to_js_call()).ok(); }
                            if let Some(n) = notify { n(); }
                        }
                        Err(e) => log::error!("Engine fetch failed: {}", e),
                    }
                });
                Some(IpcResponse::Success { message: format!("Fetching {}", clean) })
            }
            IpcMessage::PageMetaReq { url } => {
                let clean = AdBlocker::clean_url(&url);
                let pipe = Arc::clone(&self.pipeline);
                let tx = self.async_tx.clone();
                let notify = self.async_notify.clone();
                let clean2 = clean.clone();
                tokio::spawn(async move {
                    let p = pipe.lock().await;
                    match p.fetch_and_parse(&clean2).await {
                        Ok(result) => {
                            let meta_json = serde_json::json!({
                                "description": result.meta.description,
                                "lang": result.meta.lang,
                                "charset": result.meta.charset,
                                "links": result.meta.link_count,
                                "images": result.meta.image_count,
                            }).to_string();
                            log::info!("Meta extracted: {}", result.title);
                            let resp = IpcResponse::PageMetaResp { url: result.url, data: meta_json };
                            if let Some(tx) = tx { tx.send(resp.to_js_call()).ok(); }
                            if let Some(n) = notify { n(); }
                        }
                        Err(e) => log::error!("Meta fetch failed: {}", e),
                    }
                });
                None
            }
            IpcMessage::ReaderFetch { url } => {
                let clean = AdBlocker::clean_url(&url);
                info!("Reader fetch: {}", clean);
                let pipe = Arc::clone(&self.pipeline);
                let tx = self.async_tx.clone();
                let notify = self.async_notify.clone();
                let reader_settings = self.reader.settings.clone();
                let reader_active = self.reader.active;
                let clean2 = clean.clone();
                tokio::spawn(async move {
                    let p = pipe.lock().await;
                    match p.fetch_reader(&clean2).await {
                        Ok((title, content)) => {
                            log::info!("Reader content: {} ({}b)", title, content.len());
                            let rm = crate::ui::reader::ReaderMode { active: reader_active, settings: reader_settings };
                            let html = rm.render_html(&title, &content);
                            let resp = IpcResponse::ReaderHtml { html, active: reader_active };
                            if let Some(tx) = tx { tx.send(resp.to_js_call()).ok(); }
                            if let Some(n) = notify { n(); }
                        }
                        Err(e) => log::error!("Reader fetch failed: {}", e),
                    }
                });
                Some(IpcResponse::Success { message: format!("Reader loading {}", clean) })
            }
            IpcMessage::DrmStats => Some(IpcResponse::DrmStatsResp { data: self.drm.stats_json() }),
            IpcMessage::DrmReport => Some(IpcResponse::DrmReportResp { report: self.drm.concessions_report() }),
            IpcMessage::DrmOverride { domain, use_webview } => { self.drm.add_override(&domain, use_webview); info!("DRM override: {} -> {}", domain, if use_webview { "WebView" } else { "Engine" }); None }
            IpcMessage::AmniAppList => Some(IpcResponse::AmniApps { data: crate::engine::app_launcher::list_apps_json() }),
            IpcMessage::LaunchApp { id } => {
                match crate::engine::app_launcher::launch_app(&id) {
                    Ok(msg) => {
                        let app = crate::engine::app_launcher::AMNI_APPS.iter().find(|a| a.id == id.as_str());
                        let is_web = app.map(|a| matches!(a.launch, crate::engine::app_launcher::LaunchType::Web(_))).unwrap_or(false);
                        if is_web { Some(IpcResponse::AppNavigate { url: msg }) }
                        else { Some(IpcResponse::AppLaunched { message: msg }) }
                    }
                    Err(e) => { error!("App launch failed: {}", e); Some(IpcResponse::Error { message: e }) }
                }
            }
        }
    }
}
