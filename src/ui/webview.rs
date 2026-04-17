use crate::ui::theme::{Theme, ThemeConfig};
use crate::ui::emoji::eh;
pub fn browser_html(theme: &Theme) -> String {
    let css_vars = ThemeConfig::theme_to_css_vars(theme);
    let e_back = eh("back");
    let e_forward = eh("forward");
    let e_refresh = eh("refresh");
    let e_star_empty = eh("star_empty");
    let e_star_solid = eh("star_solid");
    let e_shield = eh("shield");
    let e_split = eh("split");
    let e_key = eh("key");
    let e_palette = eh("palette");
    let e_download = eh("download");
    let e_clock = eh("clock");
    let e_book = eh("book");
    let e_menu = eh("menu");
    let e_close = eh("close");
    let e_up = eh("up");
    let e_down = eh("down");
    let e_middot = eh("middot");
    let e_lock = eh("lock");
    let e_search = eh("search");
    let e_no_entry = eh("no_entry");
    let e_floppy = eh("floppy");
    let e_emdash = eh("emdash");
    let e_gear = eh("gear");
    let e_wrench = eh("wrench");
    let e_puzzle = eh("puzzle");
    let e_person = eh("person");
    let e_memo = eh("memo");
    let e_chart = eh("chart");
    let e_private = eh("private");
    let e_trash = eh("trash");
    let e_clipboard = eh("clipboard");
    let e_check = eh("check");
    let e_cross = eh("cross");
    let e_xr = eh("xr");
    let e_arrow_left = eh("arrow_left");
    let e_arrow_right = eh("arrow_right");
    let e_pause = eh("pause");
    let e_new_doc = eh("new_doc");
    let e_reset = eh("reset");
    let e_broom = eh("broom");
    let e_warning = eh("warning");
    let e_globe = eh("globe");
    let e_apps = eh("rocket");
    let e_bolt = eh("bolt");
    let e_diamond = eh("diamond");
    let e_crown = eh("crown");
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Amni Browse</title>
<style>
    :root {{
        {css_vars}
        --transition: 0.18s ease;
    }}

    * {{ margin: 0; padding: 0; box-sizing: border-box; }}

    body {{
        font-family: var(--font-family);
        background: var(--bg-primary);
        color: var(--text-primary);
        overflow: hidden;
        height: 100vh;
        display: flex;
        flex-direction: column;
        user-select: none;
    }}

    /* ===== TAB BAR ===== */
    #tab-bar {{
        display: flex;
        align-items: center;
        background: var(--bg-secondary);
        border-bottom: 1px solid var(--border);
        height: 38px;
        padding: 0 8px;
        -webkit-app-region: drag;
        app-region: drag;
    }}
    .tab {{
        display: flex; align-items: center; gap: 6px;
        padding: 6px 12px;
        background: var(--tab-inactive);
        border: 1px solid transparent; border-bottom: none;
        border-radius: var(--radius) var(--radius) 0 0;
        color: var(--text-secondary); font-size: 12px; cursor: pointer;
        max-width: 200px; min-width: 80px;
        transition: all var(--transition);
        -webkit-app-region: no-drag; app-region: no-drag;
        white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    }}
    .tab:hover {{ background: var(--bg-hover); color: var(--text-primary); }}
    .tab.active {{ background: var(--tab-active); border-color: var(--border); color: var(--text-primary); border-bottom: 2px solid var(--accent); }}
    .tab-close {{
        display: flex; align-items: center; justify-content: center;
        width: 16px; height: 16px; border-radius: 50%; border: none;
        background: transparent; color: var(--text-secondary); font-size: 14px;
        cursor: pointer; flex-shrink: 0; transition: all var(--transition);
    }}
    .tab-close:hover {{ background: var(--danger); color: white; }}
    #new-tab-btn {{
        display: flex; align-items: center; justify-content: center;
        width: 28px; height: 28px; border-radius: var(--radius); border: none;
        background: transparent; color: var(--text-secondary); font-size: 18px;
        cursor: pointer; margin-left: 4px; transition: all var(--transition);
        -webkit-app-region: no-drag; app-region: no-drag;
    }}
    #new-tab-btn:hover {{ background: var(--bg-hover); color: var(--accent); }}

    /* ===== NAV BAR ===== */
    #nav-bar {{
        display: flex; align-items: center; gap: 4px;
        padding: 5px 10px; background: var(--bg-secondary);
        border-bottom: 1px solid var(--border); height: 44px;
    }}
    .nav-btn {{
        display: flex; align-items: center; justify-content: center;
        width: 32px; height: 32px; border-radius: var(--radius); border: none;
        background: transparent; color: var(--text-secondary); font-size: 15px;
        cursor: pointer; transition: all var(--transition); position: relative;
    }}
    .nav-btn:hover:not(:disabled) {{ background: var(--bg-hover); color: var(--text-primary); }}
    .nav-btn:disabled {{ opacity: 0.3; cursor: not-allowed; }}
    .nav-btn.active {{ color: var(--accent); }}
    #url-bar {{
        flex: 1; height: 32px; background: var(--bg-primary);
        border: 1px solid var(--border); border-radius: 20px;
        padding: 0 16px; color: var(--text-primary); font-size: 13px;
        outline: none; transition: all var(--transition);
    }}
    #url-bar:focus {{ border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent-glow); }}
    .badge {{
        position: absolute; top: 2px; right: 2px;
        background: var(--success); color: white; font-size: 7px;
        padding: 1px 3px; border-radius: 6px; min-width: 12px; text-align: center;
    }}

    /* ===== BOOKMARKS BAR ===== */
    #bookmarks-bar {{
        display: flex; align-items: center; gap: 4px;
        padding: 3px 12px; background: var(--bg-secondary);
        border-bottom: 1px solid var(--border); height: 28px;
        overflow-x: auto; font-size: 12px;
    }}
    .bookmark-item {{
        padding: 3px 10px; border-radius: 4px; background: transparent;
        border: none; color: var(--text-secondary); font-size: 11px;
        cursor: pointer; white-space: nowrap; transition: all var(--transition);
    }}
    .bookmark-item:hover {{ background: var(--bg-hover); color: var(--text-primary); }}

    /* ===== CONTENT AREA ===== */
    #content-area {{
        flex: 1; position: relative; background: var(--bg-primary);
        display: flex; /* for split view */
    }}
    #web-content {{ width: 100%; height: 100%; border: none; background: white; }}
    #split-content {{
        display: none; width: 50%; height: 100%; border: none;
        border-left: 2px solid var(--accent); background: white;
    }}
    #split-content.active {{ display: block; }}
    .split-resize {{
        display: none; width: 4px; cursor: col-resize;
        background: var(--border); transition: background 0.1s;
    }}
    .split-resize:hover {{ background: var(--accent); }}
    .split-resize.active {{ display: block; }}

    /* ===== NEW TAB PAGE ===== */
    #newtab-page {{
        display: none; flex-direction: column; align-items: center;
        justify-content: center; height: 100%; width: 100%; gap: 28px;
        background: linear-gradient(155deg, var(--bg-primary) 0%, var(--bg-secondary) 40%, var(--bg-primary) 100%);
        position: absolute; top: 0; left: 0; z-index: 10;
    }}
    #newtab-page.visible {{ display: flex; }}
    .logo {{
        font-size: 52px; font-weight: 800; letter-spacing: -1px;
        background: linear-gradient(135deg, var(--gradient-start), var(--gradient-mid), var(--gradient-end));
        -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;
    }}
    .logo-sub {{
        font-size: 12px; color: var(--text-secondary); margin-top: -16px;
        text-transform: uppercase; letter-spacing: 3px;
    }}
    #search-box {{
        width: 580px; max-width: 90%; height: 48px;
        background: var(--bg-secondary); border: 1px solid var(--border);
        border-radius: 24px; padding: 0 24px; color: var(--text-primary);
        font-size: 15px; outline: none; transition: all var(--transition);
    }}
    #search-box:focus {{ border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-glow); }}
    .privacy-stats {{ display: flex; gap: 20px; margin-top: 8px; flex-wrap: wrap; justify-content: center; }}
    .stat-card {{
        text-align: center; padding: 14px 22px;
        background: var(--bg-secondary); border: 1px solid var(--border);
        border-radius: var(--radius); min-width: 120px;
        transition: all 0.2s; cursor: default;
    }}
    .stat-card:hover {{ border-color: var(--accent); box-shadow: 0 0 12px var(--accent-glow); }}
    .stat-value {{ font-size: 26px; font-weight: 700; color: var(--accent); }}
    .stat-label {{ font-size: 10px; color: var(--text-secondary); margin-top: 4px; text-transform: uppercase; letter-spacing: 0.5px; }}

    /* ===== STATUS BAR ===== */
    #status-bar {{
        display: flex; align-items: center; justify-content: space-between;
        padding: 0 12px; background: var(--bg-secondary);
        border-top: 1px solid var(--border); height: 22px;
        font-size: 10px; color: var(--text-secondary);
    }}
    .status-left, .status-right {{ display: flex; align-items: center; gap: 10px; }}

    /* ===== SLIDE PANELS ===== */
    .slide-panel {{
        display: none; position: fixed; top: 0; right: 0;
        width: 380px; height: 100vh; background: var(--bg-secondary);
        border-left: 1px solid var(--border); z-index: 500;
        flex-direction: column; box-shadow: -8px 0 32px rgba(0,0,0,0.5);
        transition: transform 0.25s ease;
    }}
    .slide-panel.open {{ display: flex; }}
    .panel-header {{
        display: flex; align-items: center; justify-content: space-between;
        padding: 16px 20px; border-bottom: 1px solid var(--border);
        background: var(--bg-tertiary);
    }}
    .panel-header h2 {{ font-size: 16px; font-weight: 600; }}
    .panel-close {{
        width: 28px; height: 28px; border-radius: 50%; border: none;
        background: transparent; color: var(--text-secondary); font-size: 18px;
        cursor: pointer; transition: all var(--transition);
    }}
    .panel-close:hover {{ background: var(--danger); color: white; }}
    .panel-body {{ flex: 1; overflow-y: auto; padding: 16px 20px; }}

    /* ===== VAULT (Password Manager) ===== */
    .vault-locked {{ display: flex; flex-direction: column; align-items: center; gap: 16px; padding-top: 40px; }}
    .vault-locked input {{
        width: 260px; height: 40px; background: var(--bg-primary);
        border: 1px solid var(--border); border-radius: var(--radius);
        padding: 0 14px; color: var(--text-primary); font-size: 14px; outline: none;
    }}
    .vault-locked input:focus {{ border-color: var(--accent); }}
    .vault-btn {{
        padding: 8px 20px; border-radius: var(--radius); border: none;
        background: var(--accent); color: var(--bg-primary); font-size: 13px;
        font-weight: 600; cursor: pointer; transition: all var(--transition);
    }}
    .vault-btn:hover {{ background: var(--accent-hover); }}
    .vault-btn.danger {{ background: var(--danger); }}
    .vault-btn.danger:hover {{ opacity: 0.85; }}
    .cred-item {{
        display: flex; align-items: center; justify-content: space-between;
        padding: 10px 12px; border: 1px solid var(--border); border-radius: var(--radius);
        margin-bottom: 8px; background: var(--bg-primary); transition: all var(--transition);
    }}
    .cred-item:hover {{ border-color: var(--accent); }}
    .cred-site {{ font-size: 13px; font-weight: 600; }}
    .cred-user {{ font-size: 11px; color: var(--text-secondary); }}
    .cred-actions {{ display: flex; gap: 4px; }}
    .cred-actions button {{
        width: 26px; height: 26px; border-radius: 4px; border: none;
        background: var(--bg-tertiary); color: var(--text-secondary);
        cursor: pointer; font-size: 12px; transition: all var(--transition);
    }}
    .cred-actions button:hover {{ background: var(--accent); color: var(--bg-primary); }}

    /* ===== THEME PANEL ===== */
    .theme-grid {{
        display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 16px;
    }}
    .theme-card {{
        padding: 12px; border: 2px solid var(--border); border-radius: var(--radius);
        cursor: pointer; text-align: center; font-size: 12px; transition: all var(--transition); position: relative;
    }}
    .theme-card:hover {{ border-color: var(--accent); }}
    .theme-card.active {{ border-color: var(--accent); box-shadow: 0 0 12px var(--accent-glow); }}
    .theme-preview {{
        width: 100%; height: 36px; border-radius: 4px; margin-bottom: 6px;
    }}
    .theme-del-btn {{
        position: absolute; top: 6px; right: 6px; width: 22px; height: 22px;
        border: none; border-radius: 50%; background: var(--bg-tertiary); color: var(--text-secondary);
        cursor: pointer; font-size: 11px; line-height: 1; transition: all var(--transition);
    }}
    .theme-del-btn:hover {{ background: var(--danger); color: #fff; }}
    .color-row {{
        display: flex; align-items: center; justify-content: space-between;
        padding: 6px 0;
    }}
    .color-row label {{ font-size: 12px; color: var(--text-secondary); }}
    .color-row input[type="color"] {{
        width: 32px; height: 24px; border: 1px solid var(--border);
        border-radius: 4px; cursor: pointer; background: transparent;
    }}

    /* ===== DATA CLEARING ===== */
    .clear-option {{
        display: flex; align-items: center; justify-content: space-between;
        padding: 10px 0; border-bottom: 1px solid var(--border);
    }}
    .clear-option label {{ font-size: 13px; }}
    .toggle-switch {{
        width: 40px; height: 22px; background: var(--bg-tertiary);
        border-radius: 11px; cursor: pointer; position: relative;
        transition: background 0.2s;
    }}
    .toggle-switch.on {{ background: var(--accent); }}
    .toggle-switch::after {{
        content: ''; position: absolute; top: 2px; left: 2px;
        width: 18px; height: 18px; background: white; border-radius: 50%;
        transition: left 0.2s;
    }}
    .toggle-switch.on::after {{ left: 20px; }}

    /* ===== CONTEXT MENU ===== */
    #context-menu {{
        display: none; position: fixed; background: var(--bg-secondary);
        border: 1px solid var(--border); border-radius: var(--radius);
        padding: 4px; min-width: 200px;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5); z-index: 1000;
    }}
    .ctx-item {{
        display: flex; align-items: center; gap: 8px;
        padding: 7px 12px; border-radius: 4px; border: none;
        background: transparent; color: var(--text-primary); font-size: 12px;
        cursor: pointer; width: 100%; text-align: left; transition: all var(--transition);
    }}
    .ctx-item:hover {{ background: var(--bg-hover); }}
    .ctx-divider {{ height: 1px; background: var(--border); margin: 4px 0; }}

    /* ===== LOADING BAR ===== */
    #loading-bar {{
        height: 2px; position: absolute; top: 0; left: 0; width: 0;
        background: linear-gradient(90deg, var(--gradient-start), var(--gradient-mid), var(--gradient-end));
        z-index: 100; transition: width 0.3s ease;
    }}
    #loading-bar.loading {{ animation: loading 1.5s infinite; }}
    @keyframes loading {{
        0% {{ width: 0; }}
        50% {{ width: 70%; }}
        100% {{ width: 100%; opacity: 0; }}
    }}

    /* ===== SCROLLBAR ===== */
    ::-webkit-scrollbar {{ width: 6px; height: 6px; }}
    ::-webkit-scrollbar-track {{ background: transparent; }}
    ::-webkit-scrollbar-thumb {{ background: var(--bg-hover); border-radius: 3px; }}
    ::-webkit-scrollbar-thumb:hover {{ background: var(--text-secondary); }}

    /* ===== FORM ELEMENTS ===== */
    .form-group {{ margin-bottom: 12px; }}
    .form-group label {{ display: block; font-size: 11px; color: var(--text-secondary); margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.5px; }}
    .form-input {{
        width: 100%; height: 36px; background: var(--bg-primary);
        border: 1px solid var(--border); border-radius: var(--radius);
        padding: 0 12px; color: var(--text-primary); font-size: 13px; outline: none;
    }}
    .form-input:focus {{ border-color: var(--accent); }}
    #find-bar{{display:none;position:absolute;top:0;right:16px;z-index:200;background:var(--bg-secondary);border:1px solid var(--border);border-radius:0 0 var(--radius) var(--radius);padding:6px 10px;gap:6px;align-items:center;box-shadow:0 4px 16px rgba(0,0,0,0.3)}}
    #find-bar.open{{display:flex}}
    #find-bar input{{width:200px;height:28px;background:var(--bg-primary);border:1px solid var(--border);border-radius:4px;padding:0 8px;color:var(--text-primary);font-size:12px;outline:none}}
    #find-bar button{{width:26px;height:26px;border:none;border-radius:4px;background:transparent;color:var(--text-secondary);cursor:pointer;font-size:13px}}
    #find-bar button:hover{{background:var(--bg-hover)}}
    #zoom-toast{{display:none;position:fixed;bottom:32px;left:50%;transform:translateX(-50%);background:var(--bg-secondary);border:1px solid var(--border);border-radius:var(--radius);padding:8px 20px;font-size:14px;font-weight:600;z-index:600;color:var(--text-primary);box-shadow:0 4px 16px rgba(0,0,0,0.4)}}
    .dt-tabs{{display:flex;border-bottom:1px solid var(--border);margin-bottom:8px}}
    .dt-tab{{padding:6px 14px;font-size:12px;border:none;background:transparent;color:var(--text-secondary);cursor:pointer;border-bottom:2px solid transparent}}
    .dt-tab.active{{color:var(--accent);border-bottom-color:var(--accent)}}
    .dt-entry{{font-family:monospace;font-size:11px;padding:3px 6px;border-bottom:1px solid var(--border);white-space:pre-wrap;word-break:break-all}}
    .dt-entry.err{{color:var(--danger)}}
    .dt-entry.wrn{{color:var(--warning)}}
    .priv-badge{{background:var(--warning);color:#000;font-size:9px;padding:1px 4px;border-radius:3px;margin-left:4px;font-weight:600}}
    #cmd-palette{{display:none;position:fixed;top:0;left:0;right:0;bottom:0;z-index:2000;background:rgba(0,0,0,0.55);backdrop-filter:blur(4px);justify-content:center;padding-top:min(18vh,160px)}}
    #cmd-palette.open{{display:flex}}
    #cmd-box{{width:520px;max-width:92%;background:var(--bg-secondary);border:1px solid var(--border);border-radius:12px;box-shadow:0 16px 64px rgba(0,0,0,0.6);overflow:hidden;max-height:420px;display:flex;flex-direction:column}}
    #cmd-input{{width:100%;height:48px;background:transparent;border:none;border-bottom:1px solid var(--border);padding:0 20px;color:var(--text-primary);font-size:15px;outline:none}}
    #cmd-input::placeholder{{color:var(--text-secondary)}}
    #cmd-results{{overflow-y:auto;max-height:360px;padding:4px}}
    .cmd-item{{display:flex;align-items:center;gap:10px;padding:9px 16px;border-radius:6px;cursor:pointer;font-size:13px;color:var(--text-primary);transition:background 0.1s}}
    .cmd-item:hover,.cmd-item.sel{{background:var(--bg-hover)}}
    .cmd-item .ci-icon{{font-size:16px;width:22px;text-align:center;flex-shrink:0}}
    .cmd-item .ci-label{{flex:1}}
    .cmd-item .ci-kbd{{font-size:10px;color:var(--text-secondary);background:var(--bg-tertiary);padding:2px 6px;border-radius:4px;font-family:monospace}}
</style>
</head>
<body>

<!-- Tab Bar -->
<div id="tab-bar">
    <div id="tabs-container" style="display:flex;align-items:center;gap:2px;overflow-x:auto;flex:1;"></div>
    <button id="new-tab-btn" onclick="newTab()" title="New Tab (Ctrl+T)">+</button>
</div>

<!-- Navigation Bar -->
<div id="nav-bar">
    <button class="nav-btn" onclick="goBack()" title="Back (Alt+{e_arrow_left})">{e_back}</button>
    <button class="nav-btn" onclick="goForward()" title="Forward (Alt+{e_arrow_right})">{e_forward}</button>
    <button class="nav-btn" onclick="refresh()" title="Refresh (Ctrl+R)">{e_refresh}</button>
    <input type="text" id="url-bar" placeholder="Search or enter URL..."
           onkeydown="if(event.key==='Enter')navigate(this.value)">
    <button class="nav-btn" id="bookmark-btn" onclick="toggleBookmark()" title="Bookmark (Ctrl+D)">{e_star_empty}</button>
    <button class="nav-btn" id="shield-btn" onclick="toggleShield()" title="Privacy Shield">{e_shield}<span class="badge" id="block-count">0</span></button>
    <button class="nav-btn" onclick="toggleSplit()" title="Split View">{e_split}</button>
    <button class="nav-btn" onclick="openPanel('vault')" title="Password Vault (Ctrl+Shift+P)">{e_key}</button>
    <button class="nav-btn" onclick="openPanel('themes')" title="Themes">{e_palette}</button>
    <button class="nav-btn" onclick="openPanel('downloads')" title="Downloads (Ctrl+J)">{e_download}</button>
    <button class="nav-btn" onclick="openPanel('history')" title="History (Ctrl+H)">{e_clock}</button>
    <button class="nav-btn" id="reader-btn" onclick="toggleReader()" title="Reader Mode">{e_book}</button>
    <button class="nav-btn" id="zoom-display" onclick="zoomReset()" title="Zoom" style="font-size:11px;width:auto;padding:0 6px;">100%</button>
    <button class="nav-btn" id="menu-btn" onclick="toggleMenu(event)" title="Menu">{e_menu}</button>
</div>

<!-- Bookmarks Bar -->
<div id="bookmarks-bar">
    <span style="color:var(--text-secondary);font-size:10px;">{e_star_solid}</span>
    <div id="bookmarks-container" style="display:flex;gap:2px;"></div>
</div>

<!-- Content Area -->
<div id="content-area">
    <div id="loading-bar"></div>
    <div id="find-bar">
        <input type="text" id="find-input" placeholder="Find..." onkeydown="if(event.key==='Enter')findNext();if(event.key==='Escape')findClose()">
        <span id="find-info" style="font-size:11px;color:var(--text-secondary);min-width:40px;text-align:center">0/0</span>
        <button onclick="findPrev()">{e_up}</button>
        <button onclick="findNext()">{e_down}</button>
        <button onclick="findClose()">{e_close}</button>
    </div>
    <div id="zoom-toast"></div>

    <!-- New Tab Page -->
    <div id="newtab-page" class="visible">
        <div class="logo">Amni Browse</div>
        <div class="logo-sub">Independent {e_middot} Private {e_middot} Yours</div>

        <input type="text" id="search-box" placeholder="Search with DuckDuckGo or enter a URL..."
               onkeydown="if(event.key==='Enter')navigate(this.value)" autofocus>

        <div class="privacy-stats">
            <div class="stat-card">
                <div class="stat-value" id="stat-blocked">0</div>
                <div class="stat-label">Ads Blocked</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="stat-tabs">1</div>
                <div class="stat-label">Open Tabs</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="stat-bookmarks">0</div>
                <div class="stat-label">Bookmarks</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="stat-passwords">0</div>
                <div class="stat-label">Saved Passwords</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="stat-history">0</div>
                <div class="stat-label">History</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="stat-downloads">0</div>
                <div class="stat-label">Downloads</div>
            </div>
        </div>

        <div style="margin-top:20px;color:var(--text-secondary);font-size:11px;text-align:center;max-width:520px;line-height:1.7;">
            {e_lock} No telemetry &nbsp;{e_middot}&nbsp; {e_shield} Built-in ad blocker &nbsp;{e_middot}&nbsp; {e_no_entry} No third-party cookies<br>
            {e_search} DuckDuckGo search &nbsp;{e_middot}&nbsp; {e_key} AES-256 encrypted vault &nbsp;{e_middot}&nbsp; {e_floppy} All data stored locally
        </div>

        <div style="margin-top:10px;font-size:10px;color:var(--text-secondary);letter-spacing:1px;text-transform:uppercase;">
            Amni-Scient {e_emdash} Independent Software Studio
        </div>
    </div>

    <iframe id="web-content" style="display:none;flex:1;" referrerpolicy="no-referrer-when-downgrade"></iframe>
</div>

<!-- Status Bar -->
<div id="status-bar">
    <div class="status-left">
        <span id="status-text">Ready</span>
    </div>
    <div class="status-right">
        <span id="xr-status" title="WebXR Status">{e_xr} XR</span>
        <span>{e_shield} Private</span>
        <span id="status-url"></span>
    </div>
</div>

<!-- ===== PASSWORD VAULT PANEL ===== -->
<div class="slide-panel" id="panel-vault">
    <div class="panel-header">
        <h2>{e_key} Password Vault</h2>
        <button class="panel-close" onclick="closePanel('vault')">{e_close}</button>
    </div>
    <div class="panel-body">
        <!-- Vault locked / init -->
        <div id="vault-locked" class="vault-locked">
            <div style="font-size:40px;">{e_lock}</div>
            <div style="font-size:14px;color:var(--text-secondary);text-align:center;">
                AES-256-GCM Encrypted Vault<br>
                <span style="font-size:11px;">PBKDF2-HMAC-SHA256 {e_middot} 600K iterations</span>
            </div>
            <input type="password" id="vault-master" placeholder="Master password..." onkeydown="if(event.key==='Enter')vaultUnlock()">
            <div style="display:flex;gap:8px;">
                <button class="vault-btn" onclick="vaultUnlock()">Unlock</button>
                <button class="vault-btn" style="background:var(--bg-tertiary);color:var(--text-primary);" onclick="vaultInit()">Initialize</button>
            </div>
            <div id="vault-error" style="color:var(--danger);font-size:12px;display:none;"></div>
        </div>
        <!-- Vault unlocked -->
        <div id="vault-unlocked" style="display:none;">
            <div style="display:flex;gap:6px;margin-bottom:12px;">
                <button class="vault-btn" onclick="vaultAddForm()">+ Add</button>
                <button class="vault-btn" style="background:var(--bg-tertiary);color:var(--text-primary);" onclick="vaultGenerate()">Generate</button>
                <button class="vault-btn danger" style="margin-left:auto;" onclick="vaultLock()">Lock</button>
            </div>
            <!-- Add credential form (hidden) -->
            <div id="vault-add-form" style="display:none;margin-bottom:16px;padding:12px;background:var(--bg-primary);border:1px solid var(--border);border-radius:var(--radius);">
                <div class="form-group"><label>Site / URL</label><input class="form-input" id="cred-site" placeholder="example.com"></div>
                <div class="form-group"><label>Username / Email</label><input class="form-input" id="cred-user" placeholder="user@example.com"></div>
                <div class="form-group"><label>Password</label><input class="form-input" id="cred-pass" type="password" placeholder="password"></div>
                <div class="form-group"><label>Notes (optional)</label><input class="form-input" id="cred-notes" placeholder="notes..."></div>
                <div style="display:flex;gap:6px;margin-top:8px;">
                    <button class="vault-btn" onclick="vaultSaveCredential()">Save</button>
                    <button class="vault-btn" style="background:var(--bg-tertiary);color:var(--text-primary);" onclick="document.getElementById('vault-add-form').style.display='none'">Cancel</button>
                </div>
            </div>
            <!-- Generated password display -->
            <div id="vault-generated" style="display:none;margin-bottom:12px;padding:10px;background:var(--bg-primary);border:1px solid var(--accent);border-radius:var(--radius);font-family:monospace;font-size:13px;word-break:break-all;cursor:pointer;" onclick="copyText(this.textContent)" title="Click to copy"></div>
            <!-- Credential list -->
            <div id="vault-list"></div>
        </div>
    </div>
</div>

<!-- ===== THEME PANEL ===== -->
<div class="slide-panel" id="panel-themes">
    <div class="panel-header">
        <h2>{e_palette} Themes</h2>
        <button class="panel-close" onclick="closePanel('themes')">{e_close}</button>
    </div>
    <div class="panel-body">
        <div style="font-size:12px;color:var(--text-secondary);margin-bottom:12px;">Built-in Amni-Scient themes</div>
        <div class="theme-grid" id="theme-grid"></div>

        <div style="margin-top:16px;border-top:1px solid var(--border);padding-top:16px;">
            <div style="font-size:13px;font-weight:600;margin-bottom:12px;">Custom Theme</div>
            <div class="color-row"><label>Primary BG</label><input type="color" id="custom-bg-primary" value="#0a0e14"></div>
            <div class="color-row"><label>Accent</label><input type="color" id="custom-accent" value="#00d4ff"></div>
            <div class="color-row"><label>Text</label><input type="color" id="custom-text" value="#e0e6f0"></div>
            <div class="form-group" style="margin-top:8px;">
                <label>Background Image URL</label>
                <input class="form-input" id="custom-bg-image" placeholder="https://...">
            </div>
            <div class="form-group">
                <label>BG Opacity: <span id="opacity-val">1.0</span></label>
                <input type="range" min="0" max="1" step="0.05" value="1.0" id="custom-opacity"
                       style="width:100%;" oninput="document.getElementById('opacity-val').textContent=this.value">
            </div>
            <button class="vault-btn" onclick="saveCustomTheme()">Save Custom Theme</button>
        </div>
    </div>
</div>

<!-- ===== DATA / SETTINGS PANEL ===== -->
<div class="slide-panel" id="panel-settings">
    <div class="panel-header">
        <h2>{e_gear} Settings & Data</h2>
        <button class="panel-close" onclick="closePanel('settings')">{e_close}</button>
    </div>
    <div class="panel-body">
        <div style="font-size:13px;font-weight:600;margin-bottom:8px;">Clear Browsing Data</div>
        <div class="clear-option"><label>Cache</label><div class="toggle-switch on" data-cat="cache" onclick="this.classList.toggle('on')"></div></div>
        <div class="clear-option"><label>Cookies</label><div class="toggle-switch on" data-cat="cookies" onclick="this.classList.toggle('on')"></div></div>
        <div class="clear-option"><label>History</label><div class="toggle-switch on" data-cat="history" onclick="this.classList.toggle('on')"></div></div>
        <div class="clear-option"><label>Saved Passwords</label><div class="toggle-switch" data-cat="passwords" onclick="this.classList.toggle('on')"></div></div>
        <div style="display:flex;gap:8px;margin-top:16px;">
            <button class="vault-btn" onclick="clearSelectedData()">Clear Selected</button>
            <button class="vault-btn danger" onclick="clearAllData()">Clear Everything</button>
        </div>
        <div style="margin-top:24px;border-top:1px solid var(--border);padding-top:16px;">
            <div style="font-size:13px;font-weight:600;margin-bottom:8px;">Privacy Settings</div>
            <div class="clear-option"><label>Block Ads</label><div class="toggle-switch on" id="toggle-ads" onclick="this.classList.toggle('on');sendIpc({{type:'toggle_adblock'}})"></div></div>
            <div class="clear-option"><label>Enable WebXR / AR</label><div class="toggle-switch" id="toggle-xr" onclick="this.classList.toggle('on')"></div></div>
            <div class="clear-option"><label>Clear Data on Exit</label><div class="toggle-switch" id="toggle-clear-exit" onclick="this.classList.toggle('on')"></div></div>
            <div class="clear-option"><label>DNS over HTTPS</label><div class="toggle-switch" id="toggle-doh" onclick="this.classList.toggle('on');sendIpc({{type:'doh_toggle'}})"></div></div>
            <div class="clear-option"><label>Restore Session on Start</label><div class="toggle-switch on" id="toggle-session" onclick="this.classList.toggle('on')"></div></div>
        </div>
        <div style="margin-top:24px;color:var(--text-secondary);font-size:11px;text-align:center;">
            All data stored locally {e_middot} No telemetry {e_middot} No tracking<br>
            Amni-Scient {e_emdash} Independent Software Studio
        </div>
    </div>
</div>

<!-- ===== DOWNLOADS PANEL ===== -->
<div class="slide-panel" id="panel-downloads">
    <div class="panel-header"><h2>{e_download} Downloads</h2><button class="panel-close" onclick="closePanel('downloads')">{e_close}</button></div>
    <div class="panel-body">
        <div style="display:flex;gap:6px;margin-bottom:12px;">
            <button class="vault-btn" onclick="sendIpc({{type:'download_clear'}});setTimeout(()=>sendIpc({{type:'download_list'}}),200)">Clear Completed</button>
        </div>
        <div id="dl-list"></div>
    </div>
</div>
<!-- ===== HISTORY PANEL ===== -->
<div class="slide-panel" id="panel-history">
    <div class="panel-header"><h2>{e_clock} History</h2><button class="panel-close" onclick="closePanel('history')">{e_close}</button></div>
    <div class="panel-body">
        <div style="display:flex;gap:6px;margin-bottom:12px;">
            <input class="form-input" id="hist-search" placeholder="Search history..." style="flex:1" onkeydown="if(event.key==='Enter')searchHist()">
            <button class="vault-btn danger" onclick="if(confirm('Clear all history?')){{sendIpc({{type:'history_clear'}});setTimeout(()=>sendIpc({{type:'history_list'}}),200)}}">Clear</button>
        </div>
        <div id="hist-list"></div>
    </div>
</div>
<!-- ===== DEVTOOLS PANEL ===== -->
<div class="slide-panel" id="panel-devtools" style="width:480px;">
    <div class="panel-header"><h2>{e_wrench} DevTools</h2><button class="panel-close" onclick="closePanel('devtools')">{e_close}</button></div>
    <div class="panel-body" style="padding:0;">
        <div class="dt-tabs">
            <button class="dt-tab active" onclick="dtSwitch('console',this)">Console</button>
            <button class="dt-tab" onclick="dtSwitch('network',this)">Network</button>
        </div>
        <div style="display:flex;gap:4px;padding:4px 8px;border-bottom:1px solid var(--border);">
            <button class="vault-btn" style="font-size:10px;padding:4px 8px;" onclick="dtClear()">Clear</button>
        </div>
        <div id="dt-console" style="padding:4px;max-height:calc(100vh - 140px);overflow-y:auto;"></div>
        <div id="dt-network" style="padding:4px;max-height:calc(100vh - 140px);overflow-y:auto;display:none;"></div>
    </div>
</div>
<!-- ===== AMNI APPS PANEL ===== -->
<div class="slide-panel" id="panel-amniapps">
    <div class="panel-header"><h2>{e_apps} Amni Apps</h2><button class="panel-close" onclick="closePanel('amniapps')">{e_close}</button></div>
    <div class="panel-body">
        <div style="font-size:12px;color:var(--text-secondary);margin-bottom:12px;">Launch Amni-Scient software</div>
        <div id="amni-app-list"></div>
    </div>
</div>
<!-- ===== EXTENSIONS PANEL ===== -->
<div class="slide-panel" id="panel-extensions">
    <div class="panel-header"><h2>{e_puzzle} Extensions</h2><button class="panel-close" onclick="closePanel('extensions')">{e_close}</button></div>
    <div class="panel-body">
        <div style="display:flex;gap:6px;margin-bottom:12px;">
            <button class="vault-btn" onclick="sendIpc({{type:'ext_scan'}});setTimeout(()=>sendIpc({{type:'ext_list'}}),300)">Scan</button>
        </div>
        <div id="ext-list"></div>
    </div>
</div>
<!-- ===== PROFILES PANEL ===== -->
<div class="slide-panel" id="panel-profiles">
    <div class="panel-header"><h2>{e_person} Profiles</h2><button class="panel-close" onclick="closePanel('profiles')">{e_close}</button></div>
    <div class="panel-body">
        <div style="display:flex;gap:6px;margin-bottom:12px;">
            <input class="form-input" id="new-prof-name" placeholder="New profile name..." style="flex:1">
            <button class="vault-btn" onclick="createProf()">Create</button>
        </div>
        <div id="prof-list"></div>
    </div>
</div>
<!-- ===== AUTOFILL PANEL ===== -->
<div class="slide-panel" id="panel-autofill">
    <div class="panel-header"><h2>{e_memo} Autofill</h2><button class="panel-close" onclick="closePanel('autofill')">{e_close}</button></div>
    <div class="panel-body">
        <div style="font-size:13px;font-weight:600;margin-bottom:8px;">Addresses</div>
        <div id="af-addresses"></div>
        <div style="margin-top:16px;border-top:1px solid var(--border);padding-top:12px;">
            <div style="font-size:13px;font-weight:600;margin-bottom:8px;">Payment Cards</div>
            <div id="af-cards"></div>
        </div>
    </div>
</div>
<!-- ===== PERMISSIONS PANEL ===== -->
<div class="slide-panel" id="panel-permissions">
    <div class="panel-header"><h2>{e_lock} Permissions</h2><button class="panel-close" onclick="closePanel('permissions')">{e_close}</button></div>
    <div class="panel-body">
        <button class="vault-btn danger" style="margin-bottom:12px;" onclick="sendIpc({{type:'permission_reset_all'}});setTimeout(()=>sendIpc({{type:'permission_list'}}),200)">Reset All</button>
        <div id="perm-list"></div>
    </div>
</div>
<!-- Context Menu -->
<div id="context-menu">
    <button class="ctx-item" onclick="newTab()">{e_new_doc} New Tab</button>
    <button class="ctx-item" onclick="newPrivateTab()">{e_private} Private Tab</button>
    <button class="ctx-item" onclick="toggleBookmark()">{e_star_solid} Bookmark Page</button>
    <div class="ctx-divider"></div>
    <button class="ctx-item" onclick="toggleSplit()">{e_split} Split View</button>
    <button class="ctx-item" onclick="openPanel('vault')">{e_key} Password Vault</button>
    <button class="ctx-item" onclick="openPanel('themes')">{e_palette} Themes</button>
    <button class="ctx-item" onclick="openPanel('downloads')">{e_download} Downloads</button>
    <button class="ctx-item" onclick="openPanel('history')">{e_clock} History</button>
    <div class="ctx-divider"></div>
    <button class="ctx-item" onclick="openPanel('devtools')">{e_wrench} DevTools</button>
    <button class="ctx-item" onclick="openPanel('amniapps')">{e_apps} Amni Apps</button>
    <button class="ctx-item" onclick="openPanel('extensions')">{e_puzzle} Extensions</button>
    <button class="ctx-item" onclick="openPanel('profiles')">{e_person} Profiles</button>
    <button class="ctx-item" onclick="openPanel('autofill')">{e_memo} Autofill</button>
    <button class="ctx-item" onclick="openPanel('permissions')">{e_lock} Permissions</button>
    <div class="ctx-divider"></div>
    <button class="ctx-item" onclick="openPanel('settings')">{e_gear} Settings & Data</button>
    <button class="ctx-item" onclick="sendIpc({{type:'get_stats'}})">{e_chart} Refresh Stats</button>
</div>
<div id="cmd-palette" onclick="if(event.target===this)closeCmdPalette()">
    <div id="cmd-box">
        <input id="cmd-input" placeholder="Type a command..." autocomplete="off" spellcheck="false"
               onkeydown="cmdKey(event)" oninput="cmdFilter(this.value)">
        <div id="cmd-results"></div>
    </div>
</div>

<script>
    window.__amni_spa = true;
    let currentTabs = [];
    let currentUrl = 'amnibrowse://newtab';
    let adsBlocked = 0;
    let adBlockEnabled = true;
    let vaultUnlockedState = false;
    let splitActive = false;
    let findVisible = false;
    let zoomLevel = 100;
    let readerActive = false;
    let dtActiveTab = 'console';
    let activeThemeId = 'amni-dark';
    let cachedThemes = [];
    function sendIpc(msg) {{
        if (window.ipc) {{
            window.ipc.postMessage(JSON.stringify(msg));
        }}
    }}

    function webFrame() {{
        let el = document.getElementById('web-content');
        if (!el) return null;
        if (el.tagName === 'IFRAME') return el;
        const frame = document.createElement('iframe');
        frame.id = 'web-content';
        frame.style.cssText = 'display:none;flex:1;width:100%;height:100%;border:none;background:white;';
        frame.referrerPolicy = 'no-referrer-when-downgrade';
        el.replaceWith(frame);
        return frame;
    }}

    window.__amni_receive = function(msg) {{
        try {{
        switch (msg.type) {{
            case 'tabs_updated':
                updateTabs(typeof msg.tabs === 'string' ? JSON.parse(msg.tabs) : msg.tabs);
                break;
            case 'navigate_to':
                loadUrl(msg.url);
                break;
            case 'bookmarks':
                updateBookmarks(JSON.parse(msg.data));
                break;
            case 'stats':
                updateStats(msg);
                break;
            case 'vault_status':
                updateVaultStatus(msg);
                break;
            case 'vault_credentials':
                renderCredentials(JSON.parse(msg.data));
                break;
            case 'vault_password':
                copyText(msg.password);
                setStatus('{e_key} Password copied to clipboard');
                break;
            case 'vault_generated':
                showGenerated(msg.password);
                break;
            case 'themes':
                cachedThemes.length = 0;
                (typeof msg.data === 'string' ? JSON.parse(msg.data) : msg.data || []).forEach(t => cachedThemes.push(t));
                renderThemes(cachedThemes);
                break;
            case 'active_theme':
                const th = typeof msg.data === 'string' ? JSON.parse(msg.data) : msg.data;
                if (th && th.id) activeThemeId = th.id;
                applyTheme(th);
                if (cachedThemes.length) renderThemes(cachedThemes);
                break;
            case 'config':
                applyConfig(JSON.parse(msg.data));
                break;
            case 'success':
                setStatus('{e_check} ' + (msg.message || 'Done'));
                break;
            case 'error':
                setStatus('{e_cross} ' + msg.message);
                break;
            case 'downloads': renderDL(JSON.parse(msg.data)); break;
            case 'download_started': setStatus('{e_download} Download started'); sendIpc({{type:'download_list'}}); break;
            case 'history': renderHist(JSON.parse(msg.data)); break;
            case 'find_result': document.getElementById('find-info').textContent = msg.current + '/' + msg.total; break;
            case 'zoom_level': updZoom(msg.level); break;
            case 'reader_html':
                if (msg.html) {{
                    const frame = webFrame();
                    if (frame) {{
                        document.getElementById('newtab-page').classList.remove('visible');
                        frame.style.display = 'block';
                        frame.srcdoc = msg.html;
                    }}
                }}
                break;
            case 'reader_settings': break;
            case 'permissions': renderPerms(JSON.parse(msg.data)); break;
            case 'permissions_defaults': break;
            case 'doh_status': {{ const d=document.getElementById('toggle-doh'); if(d) d.classList.toggle('on',msg.enabled); }} break;
            case 'devtools_console': renderDTCon(JSON.parse(msg.data)); break;
            case 'devtools_network': renderDTNet(JSON.parse(msg.data)); break;
            case 'devtools_state': setStatus('{e_wrench} DevTools ' + (msg.active_panel || 'console') + ' (' + (msg.console_count||0) + ' logs)'); break;
            case 'extensions': renderExts(JSON.parse(msg.data)); break;
            case 'amni_apps': renderAmniApps(JSON.parse(msg.data)); break;
            case 'app_launched': setStatus('{e_check} ' + msg.message); break;
            case 'app_navigate': sendIpc({{type:'navigate',url:msg.url}}); closeAllPanels(); break;
            case 'profiles': renderProfs(JSON.parse(msg.data)); break;
            case 'session_info': setStatus('{e_clipboard} Session restored'); break;
            case 'autofill_data': renderAFData(msg); break;
            case 'autofill_suggestions': break;
            case 'page_rendered':
                document.getElementById('loading-bar').classList.remove('loading');
                setStatus('{e_check} Engine: ' + msg.title);
                if (msg.html) {{
                    document.getElementById('newtab-page').classList.remove('visible');
                    document.getElementById('url-bar').value = msg.url;
                    var frame = document.getElementById('engine-frame');
                    if (!frame) {{
                        frame = document.createElement('iframe');
                        frame.id = 'engine-frame';
                        frame.style.cssText = 'position:fixed;top:48px;left:0;right:0;bottom:0;width:100%;height:calc(100vh - 48px);border:none;background:white;z-index:999';
                        document.body.appendChild(frame);
                    }}
                    // Use srcdoc for better HTML rendering (executes scripts, handles head/body)
                    frame.srcdoc = msg.html;
                    var viewer = document.getElementById('engine-viewer');
                    if (viewer) viewer.style.display = 'none';
                }}
                break;
            case 'page_meta':
                if (msg.data) {{
                    var meta = JSON.parse(msg.data);
                    setStatus('{e_globe} Meta: ' + meta.description.substring(0, 80));
                }}
                break;
        }}
        }} catch (e) {{
            const m = (e && e.message) ? e.message : 'unknown';
            setStatus('{e_warning} UI error: ' + m);
        }}
    }};

    function navigate(input) {{
        input = input.trim();
        if (!input) return;
        if (input === 'amnibrowse://newtab') {{
            const frame = webFrame();
            document.getElementById('newtab-page').classList.add('visible');
            document.getElementById('url-bar').value = '';
            document.getElementById('loading-bar').classList.remove('loading');
            currentUrl = 'amnibrowse://newtab';
            if (frame) {{
                frame.style.display = 'none';
                frame.src = 'about:blank';
            }}
            const sb = document.getElementById('search-box');
            if (sb) sb.focus();
            return;
        }}
        let url;
        if (input.match(/^https?:\/\//)) {{ url = input; }}
        else if (input.includes('.') && !input.includes(' ')) {{ url = 'https://' + input; }}
        else {{ sendIpc({{ type: 'search', query: input }}); return; }}
        currentUrl = url;
        document.getElementById('url-bar').value = url;
        document.getElementById('loading-bar').classList.add('loading');
        setStatus('Navigating: ' + url);
        sendIpc({{ type: 'navigate', url: url }});
    }}
    function loadUrl(url) {{
        const frame = webFrame();
        if (url === 'amnibrowse://newtab') {{
            document.getElementById('newtab-page').classList.add('visible');
            document.getElementById('url-bar').value = '';
            document.getElementById('loading-bar').classList.remove('loading');
            currentUrl = 'amnibrowse://newtab';
            if (frame) {{
                frame.style.display = 'none';
                frame.src = 'about:blank';
            }}
            return;
        }}
        document.getElementById('newtab-page').classList.remove('visible');
        document.getElementById('url-bar').value = url;
        currentUrl = url;
        setStatus('Loading: ' + url);
        if (/^https?:\/\//i.test(url)) {{
            document.getElementById('loading-bar').classList.add('loading');
            return;
        }}
        if (frame) {{
            var viewer = document.getElementById('engine-viewer');
            if (viewer) viewer.style.display = 'none';
            frame.style.display = 'block';
            if (frame.src !== url && !url.startsWith('http')) frame.src = url;
            frame.onload = () => {{
                document.getElementById('loading-bar').classList.remove('loading');
                setStatus('Ready');
            }};
        }}
    }}
    function goBack() {{ sendIpc({{ type: 'back' }}); }}
    function goForward() {{ sendIpc({{ type: 'forward' }}); }}
    function refresh() {{ sendIpc({{ type: 'refresh' }}); }}

    function updateTabs(tabs) {{
        if (!Array.isArray(tabs)) tabs = [];
        currentTabs = tabs;
        const container = document.getElementById('tabs-container');
        if (!container) return;
        container.innerHTML = '';
        tabs.forEach(tab => {{
            if (!tab || typeof tab !== 'object') return;
            const tabId = (tab.id || '').toString();
            const tabTitle = (tab.title || 'New Tab').toString();
            const tabUrl = (tab.url || 'amnibrowse://newtab').toString();
            const tabActive = !!tab.is_active;
            const tabPrivate = !!tab.is_private;
            const el = document.createElement('div');
            el.className = 'tab' + (tabActive ? ' active' : '');
            el.innerHTML = '<span style="flex:1;overflow:hidden;text-overflow:ellipsis">' + escapeHtml(tabTitle) + (tabPrivate ? '<span class="priv-badge">{e_private}</span>' : '') + '</span>' +
                '<button class="tab-close" onclick="event.stopPropagation();closeTab(\'' + tabId + '\')">{e_close}</button>';
            el.onclick = () => {{ if (tabId) switchTab(tabId); }};
            container.appendChild(el);
        }});
        const active = tabs.find(t => t && t.is_active);
        if (active) {{
            const activeUrl = (active.url || 'amnibrowse://newtab').toString();
            document.getElementById('url-bar').value = activeUrl === 'amnibrowse://newtab' ? '' : activeUrl;
            if (activeUrl !== currentUrl) {{
                /^https?:\/\//.test(activeUrl) ? sendIpc({{ type: 'navigate', url: activeUrl }}) : loadUrl(activeUrl);
            }}
        }}
        const st = document.getElementById('stat-tabs');
        if (st) st.textContent = tabs.length;
    }}

    function newTab() {{ sendIpc({{ type: 'new_tab', url: null }}); }}
    function closeTab(id) {{ sendIpc({{ type: 'close_tab', id: id }}); }}
    function switchTab(id) {{ sendIpc({{ type: 'switch_tab', id: id }}); }}

    function toggleSplit() {{
        if (splitActive) {{
            splitActive = false;
            document.getElementById('split-content').classList.remove('active');
            document.getElementById('split-resize').classList.remove('active');
            document.getElementById('web-content').style.flex = '1';
            sendIpc({{ type: 'close_split' }});
        }} else {{
            splitActive = true;
            document.getElementById('split-content').classList.add('active');
            document.getElementById('split-resize').classList.add('active');
            document.getElementById('web-content').style.flex = '1';
            const splitUrl = currentUrl !== 'amnibrowse://newtab' ? currentUrl : 'about:blank';
            document.getElementById('split-content').src = splitUrl;
            sendIpc({{ type: 'split_tab', mode: 'vertical', url: splitUrl }});
        }}
    }}

    function toggleBookmark() {{
        const url = document.getElementById('url-bar').value || currentUrl;
        if (url && url !== 'amnibrowse://newtab') {{
            sendIpc({{ type: 'bookmark_add', title: document.title || url, url: url }});
            document.getElementById('bookmark-btn').textContent = '{e_star_solid}';
        }}
    }}

    function updateBookmarks(bookmarks) {{
        const container = document.getElementById('bookmarks-container');
        container.innerHTML = '';
        bookmarks.forEach(b => {{
            const el = document.createElement('button');
            el.className = 'bookmark-item';
            el.textContent = b.title || b.url;
            el.onclick = () => navigate(b.url);
            container.appendChild(el);
        }});
        const sb = document.getElementById('stat-bookmarks');
        if (sb) sb.textContent = bookmarks.length;
    }}

    function toggleShield() {{
        adBlockEnabled = !adBlockEnabled;
        document.getElementById('shield-btn').classList.toggle('active', adBlockEnabled);
        sendIpc({{ type: 'toggle_adblock' }});
        setStatus(adBlockEnabled ? '{e_shield} Ad blocker enabled' : '{e_warning} Ad blocker disabled');
    }}

    function updateStats(stats) {{
        adsBlocked = stats.ads_blocked;
        const s = (id, val) => {{ const e = document.getElementById(id); if(e) e.textContent = val; }};
        s('stat-blocked', stats.ads_blocked);
        s('stat-tabs', stats.tabs_open);
        s('stat-bookmarks', stats.bookmarks_count);
        s('stat-passwords', stats.passwords_count || 0);
        s('stat-history', stats.history_count || 0);
        s('stat-downloads', stats.downloads_active || 0);
        s('block-count', stats.ads_blocked > 99 ? '99+' : stats.ads_blocked);
    }}

    function openPanel(name) {{
        closeAllPanels();
        const panel = document.getElementById('panel-' + name);
        if (panel) panel.classList.add('open');
        hideMenu();
        const m = {{vault:'vault_status',themes:'theme_list',downloads:'download_list',history:'history_list',devtools:'devtools_state',extensions:'ext_list',profiles:'profile_list',autofill:'autofill_list',permissions:'permission_list',amniapps:'amni_app_list'}};
        if (name === 'themes') sendIpc({{type:'theme_get_active'}});
        if (m[name]) sendIpc({{type:m[name]}});
    }}

    function closePanel(name) {{
        const panel = document.getElementById('panel-' + name);
        if (panel) panel.classList.remove('open');
    }}

    function closeAllPanels() {{
        document.querySelectorAll('.slide-panel').forEach(p => p.classList.remove('open'));
    }}

    function vaultUnlock() {{
        const pw = document.getElementById('vault-master').value;
        if (!pw) return;
        sendIpc({{ type: 'vault_unlock', master_password: pw }});
        document.getElementById('vault-master').value = '';
    }}

    function vaultInit() {{
        const pw = document.getElementById('vault-master').value;
        if (!pw || pw.length < 8) {{
            showVaultError('Master password must be at least 8 characters');
            return;
        }}
        sendIpc({{ type: 'vault_init', master_password: pw }});
        document.getElementById('vault-master').value = '';
    }}

    function vaultLock() {{
        sendIpc({{ type: 'vault_lock' }});
        vaultUnlockedState = false;
        document.getElementById('vault-locked').style.display = 'flex';
        document.getElementById('vault-unlocked').style.display = 'none';
    }}

    function updateVaultStatus(status) {{
        vaultUnlockedState = status.unlocked;
        if (status.unlocked) {{
            document.getElementById('vault-locked').style.display = 'none';
            document.getElementById('vault-unlocked').style.display = 'block';
            sendIpc({{ type: 'vault_list' }});
        }} else {{
            document.getElementById('vault-locked').style.display = 'flex';
            document.getElementById('vault-unlocked').style.display = 'none';
        }}
    }}

    function showVaultError(msg) {{
        const el = document.getElementById('vault-error');
        el.style.display = 'block';
        el.textContent = msg;
        setTimeout(() => el.style.display = 'none', 4000);
    }}

    function vaultAddForm() {{
        const form = document.getElementById('vault-add-form');
        form.style.display = form.style.display === 'none' ? 'block' : 'none';
    }}

    function vaultSaveCredential() {{
        const site = document.getElementById('cred-site').value.trim();
        const user = document.getElementById('cred-user').value.trim();
        const pass = document.getElementById('cred-pass').value;
        const notes = document.getElementById('cred-notes').value.trim();
        if (!site || !user || !pass) {{ showVaultError('Site, username, and password are required'); return; }}
        sendIpc({{ type: 'vault_add', site: site, username: user, password: pass, notes: notes || null, category: null }});
        document.getElementById('vault-add-form').style.display = 'none';
        ['cred-site','cred-user','cred-pass','cred-notes'].forEach(id => document.getElementById(id).value = '');
        setTimeout(() => sendIpc({{ type: 'vault_list' }}), 200);
    }}

    function vaultGenerate() {{
        sendIpc({{ type: 'vault_generate', length: 24 }});
    }}

    function showGenerated(pw) {{
        const el = document.getElementById('vault-generated');
        el.style.display = 'block';
        el.textContent = pw;
    }}

    function renderCredentials(creds) {{
        const list = document.getElementById('vault-list');
        if (!list) return;
        if (!creds || creds.length === 0) {{
            list.innerHTML = '<div style="text-align:center;color:var(--text-secondary);padding:24px;font-size:13px;">No saved credentials</div>';
            return;
        }}
        list.innerHTML = '';
        creds.forEach(c => {{
            const div = document.createElement('div');
            div.className = 'cred-item';
            div.innerHTML = '<div><div class="cred-site">' + escapeHtml(c.site) + '</div><div class="cred-user">' + escapeHtml(c.username) + '</div></div>' +
                '<div class="cred-actions">' +
                '<button onclick="sendIpc({{type:\'vault_get_password\',id:\'' + c.id + '\'}})" title="Copy password">{e_clipboard}</button>' +
                '<button onclick="sendIpc({{type:\'vault_remove\',id:\'' + c.id + '\'}});setTimeout(()=>sendIpc({{type:\'vault_list\'}}),200)" title="Delete">{e_trash}</button>' +
                '</div>';
            list.appendChild(div);
        }});
    }}

    function renderThemes(themes) {{
        const grid = document.getElementById('theme-grid');
        if (!grid) return;
        grid.innerHTML = '';
        (Array.isArray(themes) ? themes : []).forEach(t => {{
            const card = document.createElement('div');
            card.className = 'theme-card' + (t.id === activeThemeId ? ' active' : '');
            card.innerHTML = '<div class="theme-preview" style="background:linear-gradient(135deg,' + t.gradient_start + ',' + t.gradient_mid + ',' + t.gradient_end + ')"></div>' +
                '<div style="font-weight:600;">' + escapeHtml(t.name) + '</div>';
            card.onclick = () => {{
                sendIpc({{ type: 'theme_set', theme_id: t.id }});
                activeThemeId = t.id;
                renderThemes(cachedThemes);
                setStatus('{e_palette} Theme: ' + t.name);
            }};
            if (t.is_custom) {{
                const del = document.createElement('button');
                del.className = 'theme-del-btn';
                del.title = 'Delete custom theme';
                del.textContent = '{e_trash}';
                del.onclick = (ev) => {{
                    ev.stopPropagation();
                    sendIpc({{ type: 'theme_remove_custom', theme_id: t.id }});
                    if (activeThemeId === t.id) activeThemeId = 'amni-dark';
                    setTimeout(() => {{ sendIpc({{type:'theme_get_active'}}); sendIpc({{type:'theme_list'}}); }}, 120);
                }};
                card.appendChild(del);
            }}
            grid.appendChild(card);
        }});
    }}

    function applyTheme(theme) {{
        const root = document.documentElement;
        root.style.setProperty('--bg-primary', theme.bg_primary);
        root.style.setProperty('--bg-secondary', theme.bg_secondary);
        root.style.setProperty('--bg-tertiary', theme.bg_tertiary);
        root.style.setProperty('--bg-hover', theme.bg_hover);
        root.style.setProperty('--border', theme.border);
        root.style.setProperty('--text-primary', theme.text_primary);
        root.style.setProperty('--text-secondary', theme.text_secondary);
        root.style.setProperty('--accent', theme.accent);
        root.style.setProperty('--accent-hover', theme.accent_hover);
        root.style.setProperty('--accent-glow', theme.accent_glow);
        root.style.setProperty('--danger', theme.danger);
        root.style.setProperty('--success', theme.success);
        root.style.setProperty('--warning', theme.warning);
        root.style.setProperty('--gradient-start', theme.gradient_start);
        root.style.setProperty('--gradient-mid', theme.gradient_mid);
        root.style.setProperty('--gradient-end', theme.gradient_end);
        root.style.setProperty('--tab-active', theme.tab_active);
        root.style.setProperty('--tab-inactive', theme.tab_inactive);
        root.style.setProperty('--font-family', theme.font_family);
        root.style.setProperty('--radius', theme.border_radius);
    }}

    function saveCustomTheme() {{
        const bg = document.getElementById('custom-bg-primary').value;
        const accent = document.getElementById('custom-accent').value;
        const text = document.getElementById('custom-text').value;
        const bgImg = document.getElementById('custom-bg-image').value.trim() || null;
        const opacity = parseFloat(document.getElementById('custom-opacity').value);
        const theme = {{
            id: 'custom-' + Date.now(),
            name: 'My Custom Theme',
            bg_primary: bg,
            bg_secondary: lighten(bg, 5),
            bg_tertiary: lighten(bg, 12),
            bg_hover: lighten(bg, 18),
            border: lighten(bg, 15),
            text_primary: text,
            text_secondary: dim(text, 40),
            accent: accent,
            accent_hover: lighten(accent, 15),
            accent_glow: accent + '26',
            danger: '#ff4757',
            success: '#2ed573',
            warning: '#ffa502',
            gradient_start: accent,
            gradient_mid: '#7c5cfc',
            gradient_end: '#2ed573',
            tab_active: bg,
            tab_inactive: lighten(bg, 5),
            background_image: bgImg,
            background_opacity: opacity,
            font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', Helvetica, Arial, sans-serif",
            border_radius: '8px',
            is_custom: true
        }};
        sendIpc({{ type: 'theme_save_custom', theme: JSON.stringify(theme) }});
        sendIpc({{ type: 'theme_set', theme_id: theme.id }});
        applyTheme(theme);
        setStatus('{e_palette} Custom theme saved!');
    }}

    function clearSelectedData() {{
        const categories = [];
        document.querySelectorAll('.clear-option .toggle-switch.on').forEach(el => {{
            const cat = el.dataset.cat;
            if (cat) categories.push(cat);
        }});
        if (categories.length === 0) {{ setStatus('No categories selected'); return; }}
        sendIpc({{ type: 'clear_data', categories: categories }});
        setStatus('{e_broom} Clearing: ' + categories.join(', '));
    }}

    function clearAllData() {{
        if (confirm('Clear ALL browsing data? This cannot be undone.')) {{
            sendIpc({{ type: 'clear_all_data' }});
            setStatus('{e_broom} All data cleared');
        }}
    }}

    function applyConfig(cfg) {{
        const t = (id, on) => {{ const e=document.getElementById(id); if(e) e.classList.toggle('on',on); }};
        if(cfg.block_ads!==undefined) t('toggle-ads',cfg.block_ads);
        if(cfg.enable_doh!==undefined) t('toggle-doh',cfg.enable_doh);
        if(cfg.restore_session!==undefined) t('toggle-session',cfg.restore_session);
        if(cfg.clear_data_on_exit!==undefined) t('toggle-clear-exit',cfg.clear_data_on_exit);
    }}
    function renderDL(items) {{
        const el = document.getElementById('dl-list'); if(!el) return;
        el.innerHTML = (!items||items.length===0) ? '<div style="text-align:center;color:var(--text-secondary);padding:24px;font-size:13px;">No downloads</div>' : '';
        (items||[]).forEach(d => {{
            const div = document.createElement('div'); div.className = 'cred-item';
            div.innerHTML = '<div><div class="cred-site">'+escapeHtml(d.filename)+'</div><div class="cred-user">'+d.status+(d.total_bytes?' {e_middot} '+fmtB(d.total_bytes):'')+'</div></div>'+
                '<div class="cred-actions">'+(d.status==='Downloading'?'<button onclick="sendIpc({{type:\'download_cancel\',id:\''+d.id+'\'}})">{e_pause}</button>':'')+
                '<button onclick="sendIpc({{type:\'download_remove\',id:\''+d.id+'\'}});setTimeout(()=>sendIpc({{type:\'download_list\'}}),200)">{e_trash}</button></div>';
            el.appendChild(div);
        }});
    }}
    function fmtB(b) {{ return b<1024?b+'B':b<1048576?(b/1024).toFixed(1)+'KB':(b/1048576).toFixed(1)+'MB'; }}
    function renderHist(items) {{
        const el = document.getElementById('hist-list'); if(!el) return;
        el.innerHTML = (!items||items.length===0) ? '<div style="text-align:center;color:var(--text-secondary);padding:24px;font-size:13px;">No history</div>' : '';
        (items||[]).forEach(h => {{
            const div = document.createElement('div'); div.className = 'cred-item'; div.style.cursor='pointer';
            div.innerHTML = '<div data-url="'+escapeHtml(h.url)+'" onclick="navigate(this.dataset.url);closePanel(\'history\')"><div class="cred-site">'+escapeHtml(h.title||h.url)+'</div><div class="cred-user">'+escapeHtml(h.url)+'</div></div>'+
                '<div class="cred-actions"><button onclick="event.stopPropagation();sendIpc({{type:\'history_delete\',id:\''+h.id+'\'}});setTimeout(()=>sendIpc({{type:\'history_list\'}}),200)">{e_trash}</button></div>';
            el.appendChild(div);
        }});
    }}
    function searchHist() {{ const q=document.getElementById('hist-search').value.trim(); sendIpc(q?{{type:'history_search',query:q}}:{{type:'history_list'}}); }}
    function openFind() {{
        findVisible=true; document.getElementById('find-bar').classList.add('open'); document.getElementById('find-input').focus();
    }}
    function findClose() {{
        findVisible=false; document.getElementById('find-bar').classList.remove('open');
        document.getElementById('find-input').value=''; document.getElementById('find-info').textContent='0/0';
        sendIpc({{type:'find_close'}});
    }}
    function findNext() {{ const q=document.getElementById('find-input').value; if(q) sendIpc({{type:'find_next'}}); }}
    function findPrev() {{ const q=document.getElementById('find-input').value; if(q) sendIpc({{type:'find_prev'}}); }}
    function zoomIn() {{ sendIpc({{type:'zoom_in'}}); }}
    function zoomOut() {{ sendIpc({{type:'zoom_out'}}); }}
    function zoomReset() {{ sendIpc({{type:'zoom_reset'}}); }}
    function updZoom(level) {{
        zoomLevel = Math.round(level*100);
        const el=document.getElementById('zoom-display'); if(el) el.textContent=zoomLevel+'%';
        const t=document.getElementById('zoom-toast'); if(t){{t.textContent=zoomLevel+'%';t.style.display='block';setTimeout(()=>t.style.display='none',1500);}}
    }}
    function toggleReader() {{
        readerActive=!readerActive; sendIpc({{type:'reader_toggle'}});
        const b=document.getElementById('reader-btn'); if(b) b.classList.toggle('active',readerActive);
    }}
    function newPrivateTab() {{ sendIpc({{type:'new_private_tab'}}); }}
    function dtSwitch(tab,btn) {{
        dtActiveTab=tab; document.querySelectorAll('.dt-tab').forEach(t=>t.classList.remove('active')); if(btn) btn.classList.add('active');
        document.getElementById('dt-console').style.display=tab==='console'?'block':'none';
        document.getElementById('dt-network').style.display=tab==='network'?'block':'none';
    }}
    function dtClear() {{ sendIpc(dtActiveTab==='console'?{{type:'devtools_clear_console'}}:{{type:'devtools_clear_network'}}); }}
    function renderDTCon(entries) {{
        const el=document.getElementById('dt-console'); if(!el) return; el.innerHTML='';
        (entries||[]).forEach(e => {{ const d=document.createElement('div'); d.className='dt-entry'+(e.level==='error'?' err':'')+(e.level==='warn'?' wrn':''); d.textContent='['+e.level+'] '+e.message; el.appendChild(d); }});
    }}
    function renderDTNet(entries) {{
        const el=document.getElementById('dt-network'); if(!el) return; el.innerHTML='';
        (entries||[]).forEach(e => {{ const d=document.createElement('div'); d.className='dt-entry'; d.textContent=e.method+' '+e.url+' {e_arrow_right} '+(e.status||'pending')+(e.size?' ('+fmtB(e.size)+')':''); el.appendChild(d); }});
    }}
    function renderExts(exts) {{
        const el=document.getElementById('ext-list'); if(!el) return;
        el.innerHTML = (!exts||exts.length===0) ? '<div style="text-align:center;color:var(--text-secondary);padding:24px;font-size:13px;">No extensions</div>' : '';
        (exts||[]).forEach(ext => {{
            const div=document.createElement('div'); div.className='cred-item';
            div.innerHTML='<div><div class="cred-site">'+escapeHtml(ext.name)+' v'+escapeHtml(ext.version)+'</div><div class="cred-user">'+(ext.enabled?'Enabled':'Disabled')+'</div></div>'+
                '<div class="cred-actions"><button onclick="sendIpc({{type:\''+(ext.enabled?'ext_disable':'ext_enable')+'\',id:\''+ext.id+'\'}});setTimeout(()=>sendIpc({{type:\'ext_list\'}}),200)">'+(ext.enabled?'{e_pause}':'{e_forward}')+'</button>'+
                '<button onclick="sendIpc({{type:\'ext_remove\',id:\''+ext.id+'\'}});setTimeout(()=>sendIpc({{type:\'ext_list\'}}),200)">{e_trash}</button></div>';
            el.appendChild(div);
        }});
    }}
    function renderAmniApps(apps) {{
        const el=document.getElementById('amni-app-list'); if(!el) return;
        const emojiMap={{rocket:'{e_apps}',chart:'{e_chart}',inbox:'{e_download}',palette:'{e_palette}',diamond:'{e_diamond}',globe:'{e_globe}',bolt:'{e_bolt}',xr:'{e_xr}',wrench:'{e_wrench}',crown:'{e_crown}'}};
        const iconHtml=(a,fb)=>a.icon_src?('<img src="'+a.icon_src+'" style="width:24px;height:24px;object-fit:contain;border-radius:6px;">'):('<div style="font-size:22px;width:36px;text-align:center;">'+(emojiMap[a.emoji]||fb)+'</div>');
        const locals=(apps||[]).filter(a=>a.category==='Local'),webs=(apps||[]).filter(a=>a.category==='Web');
        let h='<div style="font-size:11px;font-weight:600;color:var(--accent);text-transform:uppercase;letter-spacing:1px;margin-bottom:8px;">Local Apps</div>';
        locals.forEach(a=>{{h+='<div style="display:flex;align-items:center;gap:10px;padding:10px;margin-bottom:6px;background:var(--bg-hover);border-radius:var(--radius);border:1px solid var(--border);">'+
            '<div style="width:36px;display:flex;align-items:center;justify-content:center;">'+iconHtml(a,'{e_apps}')+'</div>'+
            '<div style="flex:1;min-width:0;"><div style="font-size:13px;font-weight:600;">'+escapeHtml(a.name)+'</div><div style="font-size:11px;color:var(--text-secondary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">'+escapeHtml(a.desc)+'</div></div>'+
            '<button class="vault-btn" style="font-size:11px;padding:4px 12px;white-space:nowrap;" onclick="sendIpc({{type:\'launch_app\',id:\''+a.id+'\'}})">Launch</button></div>';}});
        h+='<div style="font-size:11px;font-weight:600;color:var(--accent);text-transform:uppercase;letter-spacing:1px;margin:16px 0 8px;">Web Apps</div>';
        webs.forEach(a=>{{h+='<div style="display:flex;align-items:center;gap:10px;padding:10px;margin-bottom:6px;background:var(--bg-hover);border-radius:var(--radius);border:1px solid var(--border);">'+
            '<div style="width:36px;display:flex;align-items:center;justify-content:center;">'+iconHtml(a,'{e_globe}')+'</div>'+
            '<div style="flex:1;min-width:0;"><div style="font-size:13px;font-weight:600;">'+escapeHtml(a.name)+'</div><div style="font-size:11px;color:var(--text-secondary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">'+escapeHtml(a.desc)+'</div></div>'+
            '<button class="vault-btn" style="font-size:11px;padding:4px 12px;white-space:nowrap;" onclick="sendIpc({{type:\'launch_app\',id:\''+a.id+'\'}})">Open</button></div>';}});
        el.innerHTML=h;
    }}
    function renderProfs(profs) {{
        const el=document.getElementById('prof-list'); if(!el) return; el.innerHTML='';
        (profs||[]).forEach(p => {{
            const div=document.createElement('div'); div.className='cred-item';
            div.innerHTML='<div><div class="cred-site">'+escapeHtml(p.name)+'</div><div class="cred-user">'+(p.is_default?'Active Profile':'')+'</div></div>'+
                '<div class="cred-actions">'+(!p.is_default?'<button onclick="sendIpc({{type:\'profile_switch\',id:\''+p.id+'\'}})">{e_arrow_right}</button>':'')+
                (!p.is_default?'<button onclick="sendIpc({{type:\'profile_delete\',id:\''+p.id+'\'}});setTimeout(()=>sendIpc({{type:\'profile_list\'}}),200)">{e_trash}</button>':'')+'</div>';
            el.appendChild(div);
        }});
    }}
    function createProf() {{
        const n=document.getElementById('new-prof-name').value.trim(); if(!n) return;
        sendIpc({{type:'profile_create',name:n}}); document.getElementById('new-prof-name').value='';
        setTimeout(()=>sendIpc({{type:'profile_list'}}),200);
    }}
    function renderAFData(msg) {{
        const a=document.getElementById('af-addresses'),c=document.getElementById('af-cards');
        if(a) a.innerHTML=msg.addresses||'<div style="color:var(--text-secondary);font-size:12px;">No saved addresses</div>';
        if(c) c.innerHTML=msg.cards||'<div style="color:var(--text-secondary);font-size:12px;">No saved cards</div>';
    }}
    function renderPerms(perms) {{
        const el=document.getElementById('perm-list'); if(!el) return;
        const entries=Object.entries(perms||{{}});
        el.innerHTML = entries.length===0 ? '<div style="color:var(--text-secondary);font-size:12px;text-align:center;padding:24px;">No custom permissions</div>' : '';
        entries.forEach(([site,p]) => {{
            const div=document.createElement('div'); div.className='cred-item';
            div.innerHTML='<div><div class="cred-site">'+escapeHtml(site)+'</div></div>'+
                '<div class="cred-actions"><button onclick="sendIpc({{type:\'permission_reset\',site:\''+escapeHtml(site)+'\'}});setTimeout(()=>sendIpc({{type:\'permission_list\'}}),200)">{e_reset}</button></div>';
            el.appendChild(div);
        }});
    }}

    function toggleMenu(event) {{
        event.stopPropagation();
        const menu = document.getElementById('context-menu');
        if (menu.style.display === 'block') {{ hideMenu(); }}
        else {{
            const rect = event.target.getBoundingClientRect();
            menu.style.display = 'block';
            menu.style.top = rect.bottom + 4 + 'px';
            menu.style.left = (rect.right - 200) + 'px';
        }}
    }}

    function hideMenu() {{ document.getElementById('context-menu').style.display = 'none'; }}
    document.addEventListener('click', (e) => {{
        hideMenu();
        if (!e.target.closest('.slide-panel') && !e.target.closest('.nav-btn') && !e.target.closest('#context-menu')) {{
            closeAllPanels();
        }}
    }});

    function setStatus(text) {{
        const el = document.getElementById('status-text');
        if (el) el.textContent = text;
    }}

    function escapeHtml(str) {{
        const div = document.createElement('div');
        div.textContent = str;
        return div.innerHTML;
    }}

    function copyText(text) {{
        navigator.clipboard.writeText(text).then(() => {{
            setStatus('{e_clipboard} Copied to clipboard');
        }}).catch(() => {{
            const ta = document.createElement('textarea');
            ta.value = text;
            document.body.appendChild(ta);
            ta.select();
            document.execCommand('copy');
            document.body.removeChild(ta);
            setStatus('{e_clipboard} Copied');
        }});
    }}

    function lighten(hex, pct) {{
        hex = hex.replace('#','');
        let r = parseInt(hex.substring(0,2),16);
        let g = parseInt(hex.substring(2,4),16);
        let b = parseInt(hex.substring(4,6),16);
        r = Math.min(255, r + Math.round(255 * pct / 100));
        g = Math.min(255, g + Math.round(255 * pct / 100));
        b = Math.min(255, b + Math.round(255 * pct / 100));
        return '#' + [r,g,b].map(v => v.toString(16).padStart(2,'0')).join('');
    }}

    function dim(hex, pct) {{
        hex = hex.replace('#','');
        let r = parseInt(hex.substring(0,2),16);
        let g = parseInt(hex.substring(2,4),16);
        let b = parseInt(hex.substring(4,6),16);
        r = Math.round(r * (100 - pct) / 100);
        g = Math.round(g * (100 - pct) / 100);
        b = Math.round(b * (100 - pct) / 100);
        return '#' + [r,g,b].map(v => v.toString(16).padStart(2,'0')).join('');
    }}

    document.addEventListener('keydown', function(e) {{
        if (e.ctrlKey || e.metaKey) {{
            switch(e.key) {{
                case 't': e.preventDefault(); newTab(); break;
                case 'w': e.preventDefault(); {{ const a = currentTabs.find(t => t.is_active); if (a) closeTab(a.id); }} break;
                case 'l': e.preventDefault(); document.getElementById('url-bar').focus(); document.getElementById('url-bar').select(); break;
                case 'd': e.preventDefault(); toggleBookmark(); break;
                case 'r': e.preventDefault(); refresh(); break;
                case 'f': e.preventDefault(); openFind(); break;
                case 'h': e.preventDefault(); openPanel('history'); break;
                case 'j': e.preventDefault(); openPanel('downloads'); break;
                case '=': case '+': e.preventDefault(); zoomIn(); break;
                case '-': e.preventDefault(); zoomOut(); break;
                case '0': e.preventDefault(); zoomReset(); break;
                case 'k': e.preventDefault(); openCmdPalette(); break;
            }}
            if (e.shiftKey && e.key === 'P') {{ e.preventDefault(); openPanel('vault'); }}
            if (e.shiftKey && e.key === 'I') {{ e.preventDefault(); openPanel('devtools'); }}
            if (e.shiftKey && e.key === 'N') {{ e.preventDefault(); newPrivateTab(); }}
        }}
        if (e.altKey) {{
            if (e.key === 'ArrowLeft') {{ e.preventDefault(); goBack(); }}
            if (e.key === 'ArrowRight') {{ e.preventDefault(); goForward(); }}
        }}
        if (e.key === 'Escape') {{ findVisible ? findClose() : (document.getElementById('cmd-palette').classList.contains('open') ? closeCmdPalette() : closeAllPanels()); }}
    }});
    const CMD_ITEMS = [
        {{icon:'{e_new_doc}',label:'New Tab',kbd:'Ctrl+T',fn:()=>newTab()}},
        {{icon:'{e_private}',label:'New Private Tab',kbd:'Ctrl+Shift+N',fn:()=>newPrivateTab()}},
        {{icon:'{e_lock}',label:'Password Vault',kbd:'Ctrl+Shift+P',fn:()=>openPanel('vault')}},
        {{icon:'{e_palette}',label:'Themes',kbd:'',fn:()=>openPanel('themes')}},
        {{icon:'{e_download}',label:'Downloads',kbd:'Ctrl+J',fn:()=>openPanel('downloads')}},
        {{icon:'{e_clock}',label:'History',kbd:'Ctrl+H',fn:()=>openPanel('history')}},
        {{icon:'{e_book}',label:'Reader Mode',kbd:'',fn:()=>toggleReader()}},
        {{icon:'{e_wrench}',label:'DevTools',kbd:'Ctrl+Shift+I',fn:()=>openPanel('devtools')}},
        {{icon:'{e_puzzle}',label:'Extensions',kbd:'',fn:()=>openPanel('extensions')}},
        {{icon:'{e_apps}',label:'Amni Apps',kbd:'',fn:()=>openPanel('amniapps')}},
        {{icon:'{e_person}',label:'Profiles',kbd:'',fn:()=>openPanel('profiles')}},
        {{icon:'{e_memo}',label:'Autofill',kbd:'',fn:()=>openPanel('autofill')}},
        {{icon:'{e_shield}',label:'Toggle Ad Blocker',kbd:'',fn:()=>toggleShield()}},
        {{icon:'{e_gear}',label:'Settings & Data',kbd:'',fn:()=>openPanel('settings')}},
        {{icon:'{e_star_empty}',label:'Bookmark Page',kbd:'Ctrl+D',fn:()=>toggleBookmark()}},
        {{icon:'{e_split}',label:'Split View',kbd:'',fn:()=>toggleSplit()}},
        {{icon:'{e_search}',label:'Find in Page',kbd:'Ctrl+F',fn:()=>openFind()}},
        {{icon:'{e_refresh}',label:'Refresh',kbd:'Ctrl+R',fn:()=>refresh()}},
        {{icon:'{e_chart}',label:'Refresh Stats',kbd:'',fn:()=>sendIpc({{type:'get_stats'}})}},
        {{icon:'{e_lock}',label:'Permissions',kbd:'',fn:()=>openPanel('permissions')}},
        {{icon:'{e_globe}',label:'Engine Fetch Page',kbd:'Ctrl+Shift+E',fn:()=>{{ if(currentUrl && currentUrl.startsWith('http')) sendIpc({{type:'fetch_page',url:currentUrl}}); else setStatus('Navigate to a page first'); }}}},
        {{icon:'{e_book}',label:'Reader Fetch (Engine)',kbd:'',fn:()=>{{ if(currentUrl && currentUrl.startsWith('http')) sendIpc({{type:'reader_fetch',url:currentUrl}}); else setStatus('Navigate to a page first'); }}}},
        {{icon:'{e_search}',label:'Page Meta (Engine)',kbd:'',fn:()=>{{ if(currentUrl && currentUrl.startsWith('http')) sendIpc({{type:'page_meta',url:currentUrl}}); else setStatus('Navigate to a page first'); }}}},
    ];
    let cmdSel = 0;
    function openCmdPalette() {{
        closeAllPanels(); hideMenu();
        const p = document.getElementById('cmd-palette');
        p.classList.add('open');
        const inp = document.getElementById('cmd-input');
        inp.value = '';
        cmdFilter('');
        setTimeout(() => inp.focus(), 50);
    }}
    function closeCmdPalette() {{
        document.getElementById('cmd-palette').classList.remove('open');
    }}
    function cmdFilter(q) {{
        const ql = q.toLowerCase();
        const filtered = ql ? CMD_ITEMS.filter(c => c.label.toLowerCase().includes(ql)) : CMD_ITEMS;
        cmdSel = 0;
        const el = document.getElementById('cmd-results');
        el.innerHTML = '';
        filtered.forEach((c, i) => {{
            const d = document.createElement('div');
            d.className = 'cmd-item' + (i === 0 ? ' sel' : '');
            d.innerHTML = '<span class="ci-icon">' + c.icon + '</span><span class="ci-label">' + escapeHtml(c.label) + '</span>' + (c.kbd ? '<span class="ci-kbd">' + c.kbd + '</span>' : '');
            d.onclick = () => {{ closeCmdPalette(); c.fn(); }};
            d.onmouseenter = () => {{ el.querySelectorAll('.cmd-item').forEach(x => x.classList.remove('sel')); d.classList.add('sel'); cmdSel = i; }};
            el.appendChild(d);
        }});
    }}
    function cmdKey(e) {{
        const items = document.querySelectorAll('#cmd-results .cmd-item');
        if (e.key === 'ArrowDown') {{ e.preventDefault(); cmdSel = Math.min(cmdSel + 1, items.length - 1); items.forEach((x,i) => x.classList.toggle('sel', i === cmdSel)); items[cmdSel]?.scrollIntoView({{block:'nearest'}}); }}
        else if (e.key === 'ArrowUp') {{ e.preventDefault(); cmdSel = Math.max(cmdSel - 1, 0); items.forEach((x,i) => x.classList.toggle('sel', i === cmdSel)); items[cmdSel]?.scrollIntoView({{block:'nearest'}}); }}
        else if (e.key === 'Enter') {{ e.preventDefault(); items[cmdSel]?.click(); }}
        else if (e.key === 'Escape') {{ closeCmdPalette(); }}
    }}

    document.getElementById('shield-btn').classList.add('active');
    sendIpc({{ type: 'get_tabs' }});
    sendIpc({{ type: 'bookmark_list' }});
    sendIpc({{ type: 'get_stats' }});
    sendIpc({{ type: 'theme_get_active' }});
    sendIpc({{ type: 'doh_status' }});
    setInterval(() => sendIpc({{ type: 'get_stats' }}), 5000);
</script>
</body>
</html>"##,
        css_vars = css_vars,
        e_back = e_back, e_forward = e_forward, e_refresh = e_refresh,
        e_star_empty = e_star_empty, e_star_solid = e_star_solid, e_shield = e_shield,
        e_split = e_split, e_key = e_key, e_palette = e_palette, e_download = e_download,
        e_clock = e_clock, e_book = e_book, e_menu = e_menu, e_close = e_close,
        e_up = e_up, e_down = e_down, e_middot = e_middot, e_lock = e_lock,
        e_search = e_search, e_no_entry = e_no_entry, e_floppy = e_floppy,
        e_emdash = e_emdash, e_gear = e_gear, e_wrench = e_wrench, e_puzzle = e_puzzle,
        e_person = e_person, e_memo = e_memo, e_chart = e_chart, e_private = e_private,
        e_trash = e_trash, e_clipboard = e_clipboard, e_check = e_check, e_cross = e_cross,
        e_xr = e_xr, e_arrow_left = e_arrow_left, e_arrow_right = e_arrow_right,
        e_pause = e_pause, e_new_doc = e_new_doc, e_reset = e_reset,
        e_broom = e_broom, e_warning = e_warning
    )
}
