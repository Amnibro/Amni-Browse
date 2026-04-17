use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    #[serde(rename = "navigate")]
    Navigate { url: String },
    #[serde(rename = "back")]
    Back,
    #[serde(rename = "forward")]
    Forward,
    #[serde(rename = "refresh")]
    Refresh,
    #[serde(rename = "new_tab")]
    NewTab { url: Option<String> },
    #[serde(rename = "close_tab")]
    CloseTab { id: String },
    #[serde(rename = "switch_tab")]
    SwitchTab { id: String },
    #[serde(rename = "new_private_tab")]
    NewPrivateTab { url: Option<String> },
    #[serde(rename = "bookmark_add")]
    BookmarkAdd { title: String, url: String },
    #[serde(rename = "bookmark_remove")]
    BookmarkRemove { id: String },
    #[serde(rename = "bookmark_list")]
    BookmarkList,
    #[serde(rename = "get_tabs")]
    GetTabs,
    #[serde(rename = "get_stats")]
    GetStats,
    #[serde(rename = "search")]
    Search { query: String },
    #[serde(rename = "update_title")]
    UpdateTitle { title: String },
    #[serde(rename = "toggle_adblock")]
    ToggleAdBlock,
    #[serde(rename = "split_tab")]
    SplitTab { mode: String, url: Option<String> },
    #[serde(rename = "close_split")]
    CloseSplit,
    #[serde(rename = "vault_init")]
    VaultInit { master_password: String },
    #[serde(rename = "vault_unlock")]
    VaultUnlock { master_password: String },
    #[serde(rename = "vault_lock")]
    VaultLock,
    #[serde(rename = "vault_status")]
    VaultStatus,
    #[serde(rename = "vault_add")]
    VaultAdd { site: String, username: String, password: String, notes: Option<String>, category: Option<String> },
    #[serde(rename = "vault_remove")]
    VaultRemove { id: String },
    #[serde(rename = "vault_list")]
    VaultList,
    #[serde(rename = "vault_get_password")]
    VaultGetPassword { id: String },
    #[serde(rename = "vault_generate")]
    VaultGenerate { length: Option<usize> },
    #[serde(rename = "theme_list")]
    ThemeList,
    #[serde(rename = "theme_set")]
    ThemeSet { theme_id: String },
    #[serde(rename = "theme_get_active")]
    ThemeGetActive,
    #[serde(rename = "theme_save_custom")]
    ThemeSaveCustom { theme: String },
    #[serde(rename = "theme_remove_custom")]
    ThemeRemoveCustom { theme_id: String },
    #[serde(rename = "clear_data")]
    ClearData { categories: Vec<String> },
    #[serde(rename = "clear_all_data")]
    ClearAllData,
    #[serde(rename = "get_config")]
    GetConfig,
    #[serde(rename = "save_config")]
    SaveConfig { config: String },
    #[serde(rename = "save_layout")]
    SaveLayout { layout: String },
    #[serde(rename = "get_layout")]
    GetLayout,
    #[serde(rename = "download_start")]
    DownloadStart { url: String },
    #[serde(rename = "download_cancel")]
    DownloadCancel { id: String },
    #[serde(rename = "download_remove")]
    DownloadRemove { id: String },
    #[serde(rename = "download_clear")]
    DownloadClear,
    #[serde(rename = "download_list")]
    DownloadList,
    #[serde(rename = "history_list")]
    HistoryList { limit: Option<usize> },
    #[serde(rename = "history_search")]
    HistorySearch { query: String },
    #[serde(rename = "history_delete")]
    HistoryDelete { id: String },
    #[serde(rename = "history_clear")]
    HistoryClear,
    #[serde(rename = "find_in_page")]
    FindInPage { query: String },
    #[serde(rename = "find_next")]
    FindNext,
    #[serde(rename = "find_prev")]
    FindPrev,
    #[serde(rename = "find_close")]
    FindClose,
    #[serde(rename = "session_save")]
    SessionSave,
    #[serde(rename = "session_restore")]
    SessionRestore,
    #[serde(rename = "autofill_add_address")]
    AutofillAddAddress { label: String, full_name: String, street: String, city: String, state: String, zip: String, country: String, phone: String, email: String },
    #[serde(rename = "autofill_remove_address")]
    AutofillRemoveAddress { id: String },
    #[serde(rename = "autofill_add_card")]
    AutofillAddCard { label: String, cardholder: String, number: String, expiry: String, card_type: String },
    #[serde(rename = "autofill_remove_card")]
    AutofillRemoveCard { id: String },
    #[serde(rename = "autofill_list")]
    AutofillList,
    #[serde(rename = "autofill_suggest")]
    AutofillSuggest { url: String },
    #[serde(rename = "zoom_in")]
    ZoomIn,
    #[serde(rename = "zoom_out")]
    ZoomOut,
    #[serde(rename = "zoom_reset")]
    ZoomReset,
    #[serde(rename = "zoom_set")]
    ZoomSet { level: f64 },
    #[serde(rename = "reader_toggle")]
    ReaderToggle,
    #[serde(rename = "reader_settings")]
    ReaderSettings { settings: String },
    #[serde(rename = "reader_content")]
    ReaderContent { title: String, content: String },
    #[serde(rename = "permission_set")]
    PermissionSet { url: String, permission: String, state: String },
    #[serde(rename = "permission_get")]
    PermissionGet { url: String },
    #[serde(rename = "permission_list")]
    PermissionList,
    #[serde(rename = "permission_reset")]
    PermissionReset { url: String },
    #[serde(rename = "doh_toggle")]
    DohToggle,
    #[serde(rename = "doh_set_provider")]
    DohSetProvider { provider: String },
    #[serde(rename = "doh_status")]
    DohStatus,
    #[serde(rename = "devtools_toggle")]
    DevToolsToggle,
    #[serde(rename = "devtools_panel")]
    DevToolsPanel { panel: String },
    #[serde(rename = "devtools_console_log")]
    DevToolsConsoleLog { level: String, message: String, source: Option<String>, line: Option<u32> },
    #[serde(rename = "devtools_clear_console")]
    DevToolsClearConsole,
    #[serde(rename = "devtools_clear_network")]
    DevToolsClearNetwork,
    #[serde(rename = "devtools_state")]
    DevToolsState,
    #[serde(rename = "ext_list")]
    ExtList,
    #[serde(rename = "ext_enable")]
    ExtEnable { id: String },
    #[serde(rename = "ext_disable")]
    ExtDisable { id: String },
    #[serde(rename = "ext_remove")]
    ExtRemove { id: String },
    #[serde(rename = "ext_scan")]
    ExtScan,
    #[serde(rename = "profile_list")]
    ProfileList,
    #[serde(rename = "profile_create")]
    ProfileCreate { name: String, color: String },
    #[serde(rename = "profile_switch")]
    ProfileSwitch { id: String },
    #[serde(rename = "profile_delete")]
    ProfileDelete { id: String },
    #[serde(rename = "profile_rename")]
    ProfileRename { id: String, name: String },
    #[serde(rename = "fetch_page")]
    FetchPage { url: String },
    #[serde(rename = "page_meta")]
    PageMetaReq { url: String },
    #[serde(rename = "reader_fetch")]
    ReaderFetch { url: String },
    #[serde(rename = "drm_stats")]
    DrmStats,
    #[serde(rename = "drm_report")]
    DrmReport,
    #[serde(rename = "drm_override")]
    DrmOverride { domain: String, use_webview: bool },
    #[serde(rename = "amni_app_list")]
    AmniAppList,
    #[serde(rename = "launch_app")]
    LaunchApp { id: String },
}
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum IpcResponse {
    #[serde(rename = "tabs_updated")]
    TabsUpdated { tabs: String },
    #[serde(rename = "navigate_to")]
    NavigateTo { url: String },
    #[serde(rename = "bookmarks")]
    Bookmarks { data: String },
    #[serde(rename = "stats")]
    Stats { ads_blocked: u64, tabs_open: usize, bookmarks_count: usize, passwords_count: usize, history_count: usize, downloads_active: usize },
    #[serde(rename = "vault_status")]
    VaultStatus { initialized: bool, unlocked: bool, count: usize },
    #[serde(rename = "vault_credentials")]
    VaultCredentials { data: String },
    #[serde(rename = "vault_password")]
    VaultPassword { password: String },
    #[serde(rename = "vault_generated")]
    VaultGenerated { password: String },
    #[serde(rename = "themes")]
    Themes { data: String },
    #[serde(rename = "active_theme")]
    ActiveTheme { data: String },
    #[serde(rename = "config")]
    Config { data: String },
    #[serde(rename = "layout")]
    Layout { data: String },
    #[serde(rename = "downloads")]
    Downloads { data: String },
    #[serde(rename = "download_started")]
    DownloadStarted { id: String, filename: String },
    #[serde(rename = "history")]
    History { data: String },
    #[serde(rename = "find_result")]
    FindResult { found: bool, current: u32, total: u32 },
    #[serde(rename = "session_info")]
    SessionInfo { data: String },
    #[serde(rename = "autofill_data")]
    AutofillData { addresses: String, cards: String },
    #[serde(rename = "autofill_suggestions")]
    AutofillSuggestions { data: String },
    #[serde(rename = "zoom_level")]
    ZoomLevel { level: f64 },
    #[serde(rename = "reader_html")]
    ReaderHtml { html: String, active: bool },
    #[serde(rename = "reader_settings")]
    ReaderSettingsResp { data: String },
    #[serde(rename = "permissions")]
    Permissions { data: String },
    #[serde(rename = "permissions_defaults")]
    PermissionsDefaults { data: String },
    #[serde(rename = "doh_status")]
    DohStatusResp { enabled: bool, provider: String, cache_size: usize },
    #[serde(rename = "devtools_state")]
    DevToolsStateResp { data: String },
    #[serde(rename = "devtools_console")]
    DevToolsConsole { data: String },
    #[serde(rename = "devtools_network")]
    DevToolsNetwork { data: String },
    #[serde(rename = "extensions")]
    Extensions { data: String },
    #[serde(rename = "profiles")]
    Profiles { data: String, active_id: String },
    #[serde(rename = "success")]
    Success { message: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "page_rendered")]
    PageRendered { url: String, title: String, html: String, meta: String },
    #[serde(rename = "page_meta")]
    PageMetaResp { url: String, data: String },
    #[serde(rename = "drm_stats")]
    DrmStatsResp { data: String },
    #[serde(rename = "drm_report")]
    DrmReportResp { report: String },
    #[serde(rename = "amni_apps")]
    AmniApps { data: String },
    #[serde(rename = "app_launched")]
    AppLaunched { message: String },
    #[serde(rename = "app_navigate")]
    AppNavigate { url: String },
    #[serde(rename = "drm_webview_required")]
    DrmWebViewRequired { url: String, reason: String },
}
impl IpcResponse {
    pub fn to_js_call(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string());
        format!("window.__amni_receive({})", json)
    }
}
pub fn parse_ipc_message(raw: &str) -> Result<IpcMessage, String> {
    serde_json::from_str(raw).map_err(|e| format!("IPC parse error: {}", e))
}
