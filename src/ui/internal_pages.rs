use crate::ui::theme::Theme;
use crate::storage::bookmarks::Bookmark;
pub const SETTINGS_TPL: &str = r##"<!DOCTYPE html><html><head><meta charset='utf-8'><title>Settings &#8212; Amni Browse</title><style>
:root{__THEME__}
*{box-sizing:border-box}
body{font:14px/1.5 'Segoe UI Variable Text','Segoe UI',sans-serif;margin:0;color:var(--text);background:var(--bg);min-height:100%;overflow-y:auto}
.wrap{display:flex;min-height:100vh}
nav{width:200px;flex:0 0 200px;border-right:1px solid var(--stroke);padding:28px 14px;background:var(--bg-secondary,#0D0F12)}
nav .mark{width:7px;height:7px;background:var(--accent);display:inline-block;margin-right:8px}
nav h1{font-size:13px;letter-spacing:.16em;text-transform:uppercase;margin:0 0 22px;font-weight:700}
nav button{display:block;width:100%;text-align:left;background:transparent;border:1px solid transparent;color:var(--dim);padding:8px 10px;margin:0 0 4px;border-radius:3px;cursor:pointer;font:650 11px/1.2 inherit;letter-spacing:.12em;text-transform:uppercase}
nav button.on,nav button:hover{color:var(--text);border-color:var(--stroke);background:var(--elev)}
main{flex:1;padding:32px 36px 80px;max-width:720px}
h2{color:var(--accent);font-size:11px;text-transform:uppercase;letter-spacing:.16em;margin:0 0 14px}
.pane{display:none}.pane.on{display:block}
.opt{display:inline-block;padding:7px 12px;margin:0 8px 8px 0;background:var(--elev);border:1px solid var(--stroke);border-radius:3px;cursor:pointer}
.opt:hover{border-color:var(--accent)}
input[type=text],input[type=password],select{width:100%;max-width:460px;padding:8px 12px;background:var(--elev);border:1px solid var(--stroke);border-radius:3px;color:var(--text);font:inherit;margin:6px 0;display:block}
.row{display:flex;justify-content:space-between;align-items:center;padding:9px 0;border-bottom:1px solid var(--stroke)}
.row a{color:var(--text);text-decoration:none}.row a:hover{color:var(--accent)}
.x,.btn{background:var(--elev);border:1px solid var(--stroke);border-radius:3px;color:var(--text);padding:8px 14px;cursor:pointer;font:650 11px inherit;letter-spacing:.1em;text-transform:uppercase;margin:0 8px 8px 0}
.x:hover,.btn:hover{border-color:var(--accent)}
.btn.primary{background:var(--accent);color:#08090B;border-color:transparent}
.dim,.note{color:var(--dim);font-size:13px;margin:8px 0}
.switch{display:flex;align-items:center;padding:8px 0}
.switch input{margin-right:10px}
kbd{background:var(--elev);border:1px solid var(--stroke);border-radius:3px;padding:1px 6px;font:12px Consolas,monospace}
#imp-note{color:var(--accent);min-height:1.3em}
.call{border:1px solid var(--stroke);background:var(--elev);border-radius:3px;padding:14px 16px;margin:0 0 14px}
</style></head><body>
<div class='wrap'>
<nav>
<div><span class='mark'></span><h1 style='display:inline'>Settings</h1></div>
<p class='dim' style='letter-spacing:.1em;text-transform:uppercase;font-size:10px'>v__VER__</p>
<button class='on' data-p='start'>Start</button>
<button data-p='look'>Look</button>
<button data-p='keys'>Passwords</button>
<button data-p='import'>Import</button>
<button data-p='privacy'>Privacy</button>
<button data-p='system'>System</button>
</nav>
<main>
<section class='pane on' id='start'>
<h2>Start</h2>
<p class='note'>Search and the page new tabs open.</p>
<div>__RADIOS__</div>
<input type='text' value='__HOME__' placeholder='Homepage URL (blank = Amni start page)' onchange='set("home_page",this.value)'>
<label>Default zoom<select onchange='set("default_zoom",this.value)'>__ZOOMS__</select></label>
</section>
<section class='pane' id='look'>
<h2>Look</h2>
<p class='note'>Amni Scient is graphite + brass. Pick Amni Light only if you want a pale chrome.</p>
<div>__THEMES__</div>
<label>User-agent override (blank = Servo default — better site CSS)<input type='text' value='__UA__' placeholder='Servo default' onchange='set("custom_user_agent",this.value)'></label>
</section>
<section class='pane' id='keys'>
<h2>Passwords</h2>
<p class='note'>Saved logins live in the Amni vault on this PC, or in a manager you already run. Not passkeys / FIDO — those stay in Chrome or Edge.</p>
<p class='dim'>Status: __VAULT__ &#183; __PMLABEL__</p>
<div>__PMRADIOS__</div>
<input type='password' placeholder='Unlock vault or CLI (type and leave the field)' onchange='set("vault_pw",this.value)'>
<input type='text' value='__PMCLI__' placeholder='CLI path (bw / op / keepassxc-cli)' onchange='set("pm_cli_path",this.value)'>
<input type='text' value='__PMDB__' placeholder='KeePass .kdbx path' onchange='set("pm_keepass_db",this.value)'>
<label class='switch'><input type='checkbox'__AUTOFILL__ onchange='set("autofill_on_load",this.checked)'><span>Fill when exactly one login matches</span></label>
<p class='note'>On a site, the key in the URL bar lists matches. Pick one to fill, same idea as Chrome.</p>
</section>
<section class='pane' id='import'>
<h2>Import from another browser</h2>
<div class='call'>
<p><strong>Bookmarks + history</strong> come over from Chrome, Edge, Brave, or Firefox.</p>
<p><strong>Passwords</strong> (the “keys”) come from Chrome / Edge / Brave only. Unlock the Amni vault on the Passwords pane first. Close the other browser if import says the login file is locked.</p>
<p><strong>Firefox passwords</strong> stay in Firefox (NSS). Firefox bookmarks and history still import.</p>
<p><strong>Passkeys</strong> cannot be pulled. Re-register those sites in your manager.</p>
</div>
<p><button class='btn' onclick='imp("chrome")'>Chrome</button><button class='btn' onclick='imp("edge")'>Edge</button><button class='btn' onclick='imp("brave")'>Brave</button><button class='btn' onclick='imp("firefox")'>Firefox</button><button class='btn primary' onclick='imp("all")'>Everything we find</button></p>
<p id='imp-note'>__IMPORTNOTE__</p>
<p><button class='x' onclick='set("show_tutorial","1")'>Open first-run tutorial</button></p>
<h2>Bookmarks</h2>
<div>__BMS__</div>
</section>
<section class='pane' id='privacy'>
<h2>Privacy</h2>
<label class='switch'><input type='checkbox'__SHIELD__ onchange='set("block_ads",this.checked)'><span>Shield — block ads and trackers</span></label>
<label class='switch'><input type='checkbox'__RESTORE__ onchange='set("restore_session",this.checked)'><span>Restore tabs when Amni starts (Chrome-style)</span></label>
<p class='dim'>__CRASH__</p>
</section>
<section class='pane' id='system'>
<h2>System</h2>
<p class='note'>Registers Amni, then opens Windows default-apps so you can pick HTTP/HTTPS.</p>
<p><button class='btn' onclick='set("default_browser","1")'>Set as default</button></p>
<h2>Updates</h2>
<p class='dim'>This copy: v__VER__ &#183; __UPD__</p>
<label class='switch'><input type='checkbox'__CHKUPD__ onchange='set("check_updates",this.checked)'><span>Check amni-scient.com / GitHub</span></label>
<p><button class='x' onclick='set("update_check","1")'>Check now</button><button class='x' onclick='set("update_now","1")'>Install update</button></p>
<h2>Profiles</h2>
<div>__PROFS__</div>
<input type='text' placeholder='new profile name' onchange='set("profile_new",this.value)'>
<h2>Shortcuts</h2>
<p class='dim'><kbd>Ctrl+L</kbd> URL &#183; <kbd>Ctrl+D</kbd> bookmark &#183; <kbd>Ctrl+T</kbd>/<kbd>W</kbd> tabs &#183; <kbd>Ctrl+Shift+T</kbd> reopen &#183; <kbd>Ctrl+Shift+K</kbd> duplicate &#183; <kbd>Ctrl+Shift+N</kbd> private &#183; <kbd>Ctrl+=</kbd>/<kbd>-</kbd>/<kbd>0</kbd> zoom &#183; <kbd>F11</kbd> fullscreen</p>
</section>
</main>
</div>
<script>
const T='__TOK__';
function set(k,v){fetch('amnibrowse://cmd/setting_set?tok='+T+'&k='+encodeURIComponent(k)+'&v='+encodeURIComponent(v),{mode:'no-cors'}).catch(function(){})}
function rmbm(id){fetch('amnibrowse://cmd/bookmark_remove?tok='+T+'&id='+encodeURIComponent(id),{mode:'no-cors'}).catch(function(){});var e=document.getElementById('bm-'+id);e&&e.remove()}
function imp(src){document.getElementById('imp-note').textContent='Importing '+src+'…';set('import_browser',src);setTimeout(async()=>{try{const r=await fetch('amnibrowse://import/last');const j=await r.json();document.getElementById('imp-note').textContent=(j.source||src)+': '+j.bookmarks+' bookmarks, '+j.history+' history, '+j.passwords+' passwords'+(j.notes&&j.notes[0]?' — '+j.notes[0]:'')}catch(e){document.getElementById('imp-note').textContent='Import finished'}},1400)}
document.querySelectorAll('nav button').forEach(b=>b.onclick=()=>{document.querySelectorAll('nav button').forEach(x=>x.classList.toggle('on',x===b));document.querySelectorAll('.pane').forEach(p=>p.classList.toggle('on',p.id===b.dataset.p))});
</script></body></html>"##;
pub const NEWTAB_TPL: &str = r##"<!DOCTYPE html><html><head><meta charset='utf-8'><title>New Tab</title><style>
:root{__THEME__}
body{font:15px 'Segoe UI Variable Text','Segoe UI',sans-serif;background:var(--bg);color:var(--text);display:flex;flex-direction:column;align-items:center;min-height:100vh;margin:0;padding-top:14vh}
h1{font-size:34px;letter-spacing:.14em;text-transform:uppercase;margin:0 0 6px;color:var(--accent)}
p{color:var(--dim);margin:0 0 40px}
.grid{display:flex;flex-wrap:wrap;gap:14px;justify-content:center;max-width:760px}
.tile{display:flex;flex-direction:column;align-items:center;gap:8px;width:108px;padding:16px 6px;background:var(--elev);border:1px solid var(--stroke);border-radius:4px;text-decoration:none;color:var(--text);font-size:12px;transition:border-color .12s}
.tile:hover{border-color:var(--accent)}
.mono{width:40px;height:40px;border-radius:4px;display:flex;align-items:center;justify-content:center;font-size:18px;font-weight:600;color:#fff}
.tile span{max-width:100px;overflow:hidden;white-space:nowrap}
.dim{color:var(--dim);font-size:13px}
.ver{margin-top:28px;font-size:11px;color:var(--dim);letter-spacing:.6px}
</style></head><body><h1>Amni Browse</h1><p>No Amni product telemetry &#183; local profile &#183; search from the bar above</p><div class='grid'>__TILES__</div><p class='ver'>v__VER__ &#183; __ENGINE__ &#183; Amni-Scient</p></body></html>"##;
pub const TUTORIAL_TPL: &str = r##"<!DOCTYPE html><html><head><meta charset='utf-8'><title>Welcome &#8212; Amni Browse</title><style>
:root{__THEME__}
body{font:15px/1.5 'Segoe UI Variable Text','Segoe UI',sans-serif;background:var(--bg);color:var(--text);margin:0;padding:36px 24px 80px;max-width:640px;margin-left:auto;margin-right:auto}
h1{font-size:26px;color:var(--accent);margin:0 0 8px;letter-spacing:.12em;text-transform:uppercase}
.tag{color:var(--dim);margin:0 0 28px;letter-spacing:.14em;text-transform:uppercase;font-size:11px}
.step{display:none} .step.on{display:block}
.dots{display:flex;gap:6px;margin:28px 0 18px} .dots i{width:18px;height:2px;background:var(--stroke);display:block} .dots i.on{background:var(--accent)}
button{font:inherit;letter-spacing:.1em;text-transform:uppercase;cursor:pointer;border:1px solid var(--stroke);background:var(--elev);color:var(--text);border-radius:3px;padding:10px 16px;margin:0 8px 8px 0}
button.primary{background:var(--accent);color:#08090B;border-color:transparent;font-weight:700}
.card{border:1px solid var(--stroke);background:var(--elev);border-radius:3px;padding:14px 16px;margin:0 0 10px}
.dim{color:var(--dim);font-size:14px}
#note{min-height:1.4em;color:var(--accent);font-size:14px}
kbd{background:var(--elev);border:1px solid var(--stroke);border-radius:4px;padding:1px 6px;font:12px ui-monospace,monospace}
</style></head><body>
<h1>Welcome to Amni Browse</h1>
<p class='tag'>v__VER__ &#183; Real Servo &#183; your data stays on this machine</p>
<div class='dots' id='dots'></div>
<section class='step on' data-s='0'>
<p>This is a real browser engine &#8212; Servo &#8212; not a Chromium wrapper. Tabs, the URL bar, and the shield live in the gold strip above.</p>
<p>Takes about a minute. You can skip anytime.</p>
</section>
<section class='step' data-s='1'>
<p>Bring over bookmarks, history, and (if you unlock the vault) saved passwords from browsers already on this PC.</p>
<div id='found'>__BROWSERS__</div>
<p class='dim'>Close Chrome/Edge first if password import fails &#8212; they lock the login database.</p>
<p id='note'></p>
</section>
<section class='step' data-s='2'>
<p><kbd>Ctrl+L</kbd> jumps to the URL bar. Type a site or a search. <kbd>Ctrl+T</kbd> / <kbd>Ctrl+W</kbd> tabs. The shield strips trackers. The star bookmarks the page.</p>
<p class='dim'>YouTube plays in Servo when we can extract a progressive stream. Netflix-class DRM stays in the same window, same chrome, as an in-tab pane.</p>
</section>
<section class='step' data-s='3'>
<p>Settings &#8594; Password manager: Amni vault, Bitwarden (<code>bw</code>), 1Password (<code>op</code>), or KeePassXC. Unlock once. A key icon appears in the URL bar when a page has matches &#8212; pick one to fill, like Chrome.</p>
</section>
<section class='step' data-s='4'>
<p>You&#8217;re set. New tabs open the start page. Updates check amni-scient.com then GitHub. Set Amni as the default browser from Settings when you&#8217;re ready.</p>
</section>
<p><button id='back'>Back</button><button class='primary' id='next'>Next</button><button id='skip'>Skip tutorial</button></p>
<script>
const T='__TOK__';
let i=0;const steps=[...document.querySelectorAll('.step')];
function paint(){steps.forEach((s,n)=>s.classList.toggle('on',n===i));document.getElementById('dots').innerHTML=steps.map((_,n)=>'<i class="'+(n===i?'on':'')+'"></i>').join('');document.getElementById('next').textContent=i===steps.length-1?'Start browsing':'Next';document.getElementById('back').style.visibility=i?'visible':'hidden'}
function cmd(n,a){const q=a?'?'+new URLSearchParams(Object.assign({tok:T},a)):'?tok='+T;fetch('amnibrowse://cmd/'+n+q,{mode:'no-cors'}).catch(function(){})}
document.getElementById('next').onclick=()=>{if(i<steps.length-1){i++;paint()}else cmd('tutorial_done',{})};
document.getElementById('back').onclick=()=>{if(i){i--;paint()}};
document.getElementById('skip').onclick=()=>cmd('tutorial_done',{});
function imp(src){document.getElementById('note').textContent='Importing '+src+'…';cmd('import_browser',{src:src});setTimeout(async()=>{try{const r=await fetch('amnibrowse://import/last');const j=await r.json();document.getElementById('note').textContent=(j.source||src)+': '+j.bookmarks+' bookmarks, '+j.history+' history, '+j.passwords+' passwords'+(j.notes&&j.notes[0]?' — '+j.notes[0]:'')}catch(e){document.getElementById('note').textContent='Import finished (reload if counts stay blank)'}},1200)}
paint();
</script></body></html>"##;
pub fn esc_html(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;") }

pub fn theme_root_vars(t: &Theme) -> String {
    format!(
        "--bg:{p};--bg-primary:{p};--elev:{e};--bg-tertiary:{e};--bg-secondary:{s};--stroke:{b};--border:{b};--text:{tp};--text-primary:{tp};--dim:{td};--text-secondary:{td};--text-muted:{td};--accent:{a};--accent-dim:{ah};--tab-active:{ta};--tab-inactive:{ti};--chrome:{s}",
        p = t.bg_primary, e = t.bg_tertiary, s = t.bg_secondary, b = t.border, tp = t.text_primary, td = t.text_secondary, a = t.accent, ah = t.accent_hover, ta = t.tab_active, ti = t.tab_inactive
    )
}
pub fn bookmark_tiles(bookmarks: &[Bookmark]) -> String {
    match bookmarks.is_empty() {
        true => "<p class='dim'>Bookmark pages with \u{2606} or Ctrl+D and they land here.</p>".into(),
        false => bookmarks.iter().take(12).map(|bm| {
            let host = url::Url::parse(&bm.url).ok().and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string())).unwrap_or_else(|| bm.title.clone());
            let ch: String = host.chars().next().unwrap_or('\u{2022}').to_uppercase().collect();
            let hue = host.bytes().fold(0u32, |a, x| a.wrapping_mul(31).wrapping_add(x as u32)) % 360;
            format!("<a class='tile' href='{}'><div class='mono' style='background:hsl({},45%,38%)'>{}</div><span>{}</span></a>", esc_html(&bm.url), hue, esc_html(&ch), esc_html(&host))
        }).collect(),
    }
}
pub fn newtab_html(theme: &Theme, bookmarks: &[Bookmark], engine: &str) -> String {
    NEWTAB_TPL.replace("__THEME__", &theme_root_vars(theme)).replace("__TILES__", &bookmark_tiles(bookmarks)).replace("__VER__", env!("CARGO_PKG_VERSION")).replace("__ENGINE__", engine)
}
