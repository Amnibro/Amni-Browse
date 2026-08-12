use crate::ui::theme::{{Theme, ThemeConfig}};
use crate::storage::config::APP_VERSION;
use crate::engine::extensions::ExtensionManager;
pub fn developer_html(theme: &Theme) -> String {
    let css = ThemeConfig::theme_to_css_vars(theme);
    let ext_dir = ExtensionManager::extensions_dir_public();
    format!(r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="UTF-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>Developer — Amni Browse</title>
<style>
:root{{{css}}}
*{{box-sizing:border-box;margin:0;padding:0}}
body{{font:14px/1.5 var(--font-family,system-ui,sans-serif);background:var(--bg-primary,#0a0e14);color:var(--text-primary,#e0e6f0);min-height:100vh}}
header{{padding:20px 28px;border-bottom:1px solid var(--border,#1a2332);background:var(--bg-secondary,#0f1419);display:flex;align-items:center;gap:16px;flex-wrap:wrap}}
header h1{{font-size:18px;font-weight:700;letter-spacing:-.3px}}
header .ver{{color:var(--text-secondary,#6b7d99);font-size:12px}}
.tabs{{display:flex;gap:4px;padding:12px 28px 0;background:var(--bg-secondary,#0f1419);border-bottom:1px solid var(--border,#1a2332);flex-wrap:wrap}}
.tab{{background:transparent;border:none;color:var(--text-secondary,#6b7d99);padding:10px 14px;border-radius:8px 8px 0 0;cursor:pointer;font-weight:600;font-size:13px;font-family:inherit}}
.tab:hover{{color:var(--text-primary);background:var(--bg-hover,#1a1f2e)}}
.tab.on{{color:var(--accent,#C89B4E);border-bottom:2px solid var(--accent,#C89B4E)}}
main{{max-width:960px;margin:0 auto;padding:24px 28px 48px}}
.panel{{display:none}}
.panel.on{{display:block}}
h2{{font-size:15px;margin:0 0 10px;color:var(--accent,#C89B4E);text-transform:uppercase;letter-spacing:1px}}
p.dim{{color:var(--text-secondary,#6b7d99);font-size:13px;margin-bottom:16px}}
.card{{background:var(--bg-secondary,#0f1419);border:1px solid var(--border,#1a2332);border-radius:12px;padding:16px;margin-bottom:14px}}
.row{{display:flex;gap:10px;flex-wrap:wrap;align-items:center;margin:8px 0}}
label{{font-size:12px;color:var(--text-secondary);min-width:90px}}
input[type=text],input[type=search],textarea,select{{flex:1;min-width:180px;background:var(--bg-primary,#0a0e14);border:1px solid var(--border,#1a2332);color:var(--text-primary);border-radius:8px;padding:8px 12px;font-size:13px;font-family:inherit}}
textarea{{min-height:110px;width:100%;resize:vertical}}
button.btn{{background:var(--accent,#C89B4E);color:#08090B;border:none;border-radius:8px;padding:8px 14px;font-weight:600;font-size:12px;font-family:inherit;cursor:pointer}}
button.btn:hover{{filter:brightness(1.08)}}
button.btn.ghost{{background:transparent;color:var(--text-primary);border:1px solid var(--border,#1a2332)}}
button.btn.danger{{background:var(--danger,#ff4757);color:#fff}}
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:10px}}
.tcard{{border:1px solid var(--border,#1a2332);border-radius:10px;padding:10px;cursor:pointer;background:var(--bg-primary)}}
.tcard:hover,.tcard.on{{border-color:var(--accent)}}
.swatch{{height:36px;border-radius:6px;margin-bottom:8px}}
.badge{{display:inline-block;padding:2px 8px;border-radius:999px;font-size:11px;font-weight:700}}
.badge.safe{{background:#1a3d2a;color:#2ed573}}
.badge.low{{background:#3d3a1a;color:#e6c84a}}
.badge.medium{{background:#3d2a1a;color:#ff9f43}}
.badge.high,.badge.critical{{background:#3d1a1a;color:#ff4757}}
pre,code{{font-family:ui-monospace,Consolas,monospace;font-size:12px}}
pre{{background:var(--bg-primary);border:1px solid var(--border);border-radius:8px;padding:12px;overflow:auto;max-height:220px;white-space:pre-wrap}}
.list li{{margin:6px 0 6px 18px;color:var(--text-secondary)}}
.ext{{display:flex;justify-content:space-between;gap:12px;align-items:flex-start;padding:10px 0;border-bottom:1px solid var(--border)}}
.ext:last-child{{border-bottom:0}}
.muted{{color:var(--text-secondary);font-size:12px}}
.ok{{color:var(--success,#2ed573)}}
.err{{color:var(--danger,#ff4757)}}
</style></head><body>
<header>
  <h1>Developer</h1>
  <span class="ver">Amni Browse v{ver} · theming · extensions · security · bug reports</span>
  <div style="margin-left:auto;display:flex;gap:8px">
    <button class="btn ghost" onclick="ipc({{type:'navigate',url:'amnibrowse://newtab'}})">Home</button>
  </div>
</header>
<nav class="tabs" id="tabs">
  <button class="tab on" data-p="themes">Themes</button>
  <button class="tab" data-p="ext">Extensions</button>
  <button class="tab" data-p="sec">Security</button>
  <button class="tab" data-p="bug">Bug report</button>
</nav>
<main>
<section class="panel on" id="p-themes">
  <h2>Theme studio</h2>
  <p class="dim">Pick a built-in theme, craft a custom one, or export/import JSON for sharing and git-friendly skin packs.</p>
  <div class="card">
    <div class="grid" id="theme-grid"></div>
  </div>
  <div class="card">
    <h2 style="margin-bottom:12px">Custom colors</h2>
    <div class="row"><label>Name</label><input type="text" id="th-name" value="My Theme"/></div>
    <div class="row"><label>Primary BG</label><input type="color" id="th-bg" value="#0a0e14"/><input type="text" id="th-bg-t" value="#0a0e14" style="max-width:110px"/></div>
    <div class="row"><label>Accent</label><input type="color" id="th-ac" value="#C89B4E"/><input type="text" id="th-ac-t" value="#C89B4E" style="max-width:110px"/></div>
    <div class="row"><label>Text</label><input type="color" id="th-tx" value="#e0e6f0"/><input type="text" id="th-tx-t" value="#e0e6f0" style="max-width:110px"/></div>
    <div class="row">
      <button class="btn" onclick="saveTheme()">Save &amp; apply</button>
      <button class="btn ghost" onclick="exportThemes()">Export all JSON</button>
      <button class="btn ghost" onclick="importThemes()">Import JSON</button>
    </div>
    <textarea id="th-json" placeholder="Export appears here · paste JSON to import"></textarea>
  </div>
</section>
<section class="panel" id="p-ext">
  <h2>Extensions</h2>
  <p class="dim">Sideload simple Amni extensions from <code>{ext_dir}</code>. Each folder needs a <code>manifest.json</code>. Scan after adding files.</p>
  <div class="card">
    <div class="row">
      <button class="btn" onclick="scanExt()">Scan folder</button>
      <button class="btn ghost" onclick="openExtDir()">Open extensions dir</button>
      <button class="btn ghost" onclick="writeSample()">Install sample extension</button>
    </div>
    <div id="ext-list" class="muted" style="margin-top:12px">Loading…</div>
  </div>
  <div class="card">
    <h2>Manifest sketch</h2>
    <pre id="manifest-sample">{{
  "id": "hello-amni",
  "name": "Hello Amni",
  "version": "0.1.0",
  "description": "Sample content-script extension",
  "permissions": ["activeTab"],
  "content_scripts": [{{
    "matches": ["*://*/*"],
    "js": ["content.js"],
    "css": [],
    "run_at": "document_idle"
  }}]
}}</pre>
  </div>
</section>
<section class="panel" id="p-sec">
  <h2>Security awareness</h2>
  <p class="dim">Heuristic checks for the current or any URL — not a full Safe Browsing feed. Treat High/Critical as “do not sign in” until you verify the domain yourself.</p>
  <div class="card">
    <div class="row">
      <input type="text" id="sec-url" placeholder="https://…"/>
      <button class="btn" onclick="checkUrl()">Assess</button>
      <button class="btn ghost" onclick="checkCurrent()">Current tab</button>
    </div>
    <div id="sec-out" style="margin-top:12px" class="muted">Enter a URL to assess.</div>
  </div>
  <div class="card">
    <h2>What we flag</h2>
    <ul class="list">
      <li>HTTP (no TLS) especially on login-like paths</li>
      <li>Raw IP hosts, user:pass@ in URL, punycode (xn--) domains</li>
      <li>Lookalike host patterns (paypa1, g00gle, …)</li>
      <li>Risky free TLDs and deep subdomain chains</li>
    </ul>
    <p class="muted" style="margin-top:10px">Chrome toolbar shows a live risk chip on external pages.</p>
  </div>
</section>
<section class="panel" id="p-bug">
  <h2>Bug report</h2>
  <p class="dim">Opens a pre-filled GitHub issue on <code>Amnibro/Amni-Browse</code> with version, OS, and page diagnostics. Nothing is auto-uploaded without you submitting the issue.</p>
  <div class="card">
    <div class="row"><label>Title</label><input type="text" id="bug-title" placeholder="Short summary"/></div>
    <div class="row" style="align-items:flex-start"><label>Details</label><textarea id="bug-body" placeholder="What you expected vs what happened"></textarea></div>
    <div class="row">
      <button class="btn" onclick="sendBug()">Open GitHub issue</button>
      <button class="btn ghost" onclick="previewBug()">Preview diagnostics</button>
    </div>
    <pre id="bug-diag" class="muted">Diagnostics preview…</pre>
  </div>
</section>
</main>
<script>
function ipc(o){{try{{window.ipc&&window.ipc.postMessage(JSON.stringify(o))}}catch(e){{}}}}
function $(id){{return document.getElementById(id)}}
document.querySelectorAll('.tab').forEach(function(t){{
  t.onclick=function(){{
    document.querySelectorAll('.tab').forEach(function(x){{x.classList.remove('on')}});
    document.querySelectorAll('.panel').forEach(function(x){{x.classList.remove('on')}});
    t.classList.add('on');
    var p=$('p-'+t.getAttribute('data-p'));
    if(p)p.classList.add('on');
    location.hash=t.getAttribute('data-p');
  }};
}});
(function(){{
  var h=(location.hash||'').replace('#','');
  if(h){{var b=document.querySelector('.tab[data-p="'+h+'"]');if(b)b.click();}}
}})();
function syncColor(c,t){{c.oninput=function(){{t.value=c.value}};t.oninput=function(){{if(/^#[0-9a-fA-F]{{6}}$/.test(t.value))c.value=t.value}};}}
syncColor($('th-bg'),$('th-bg-t'));syncColor($('th-ac'),$('th-ac-t'));syncColor($('th-tx'),$('th-tx-t'));
var activeId='';
function renderThemes(list){{
  var g=$('theme-grid');g.innerHTML='';
  (list||[]).forEach(function(t){{
    var d=document.createElement('div');
    d.className='tcard'+(t.id===activeId?' on':'');
    d.innerHTML='<div class="swatch" style="background:linear-gradient(135deg,'+t.gradient_start+','+t.gradient_mid+','+t.gradient_end+')"></div><div style="font-weight:600;font-size:12px">'+t.name+'</div><div class="muted">'+t.id+'</div>';
    d.onclick=function(){{ipc({{type:'theme_set',theme_id:t.id}});activeId=t.id;renderThemes(list);}};
    g.appendChild(d);
  }});
}}
function saveTheme(){{
  var bg=$('th-bg-t').value,ac=$('th-ac-t').value,tx=$('th-tx-t').value;
  var id='custom-'+Date.now().toString(36);
  var theme={{
    id:id,name:$('th-name').value||'Custom',bg_primary:bg,bg_secondary:bg,bg_tertiary:'#1a1f2e',bg_hover:'#1a1f2e',
    border:'#1a2332',text_primary:tx,text_secondary:'#8a96a8',accent:ac,accent_hover:ac,accent_glow:'rgba(0,212,255,0.15)',
    danger:'#ff4757',success:'#2ed573',warning:'#ffa502',gradient_start:ac,gradient_mid:tx,gradient_end:bg,
    tab_active:bg,tab_inactive:'#0f1419',background_image:null,background_opacity:1,font_family:'system-ui,sans-serif',
    border_radius:'8px',is_custom:true
  }};
  ipc({{type:'theme_save_custom',theme:JSON.stringify(theme)}});
  setTimeout(function(){{ipc({{type:'theme_set',theme_id:id}});ipc({{type:'theme_list'}});}},80);
}}
function exportThemes(){{ipc({{type:'theme_export'}});}}
function importThemes(){{
  var raw=$('th-json').value.trim();
  if(!raw){{alert('Paste theme JSON first');return;}}
  ipc({{type:'theme_import',data:raw}});
  setTimeout(function(){{ipc({{type:'theme_list'}});}},100);
}}
function scanExt(){{ipc({{type:'ext_scan'}});setTimeout(function(){{ipc({{type:'ext_list'}});}},200);}}
function openExtDir(){{ipc({{type:'ext_open_dir'}});}}
function writeSample(){{ipc({{type:'ext_write_sample'}});setTimeout(function(){{ipc({{type:'ext_list'}});}},300);}}
function renderExt(list){{
  var el=$('ext-list');
  if(!list||!list.length){{el.innerHTML='No extensions installed. Use “Install sample extension” or drop a folder with manifest.json into the extensions directory.';return;}}
  el.innerHTML='';
  list.forEach(function(e){{
    var row=document.createElement('div');row.className='ext';
    row.innerHTML='<div><strong>'+e.name+'</strong> <span class="muted">v'+e.version+'</span><div class="muted">'+e.description+'</div><div class="muted">'+e.id+(e.enabled?' · enabled':' · disabled')+'</div></div>';
    var acts=document.createElement('div');acts.className='row';
    var b1=document.createElement('button');b1.className='btn ghost';b1.textContent=e.enabled?'Disable':'Enable';
    b1.onclick=function(){{ipc({{type:e.enabled?'ext_disable':'ext_enable',id:e.id}});setTimeout(scanExt,120);}};
    var b2=document.createElement('button');b2.className='btn danger';b2.textContent='Remove';
    b2.onclick=function(){{if(confirm('Remove '+e.name+'?')){{ipc({{type:'ext_remove',id:e.id}});setTimeout(scanExt,120);}}}};
    acts.appendChild(b1);acts.appendChild(b2);row.appendChild(acts);el.appendChild(row);
  }});
}}
function paintSafety(r){{
  var el=$('sec-out');
  if(!r){{el.textContent='No report';return;}}
  var html='<span class="badge '+r.level+'">'+(r.level||'?').toUpperCase()+'</span> · score '+(r.score||0)+' · <code>'+(r.host||r.url||'')+'</code>';
  if(r.reasons&&r.reasons.length){{html+='<ul class="list">'+r.reasons.map(function(x){{return '<li>'+x+'</li>';}}).join('')+'</ul>';}}
  if(r.tips&&r.tips.length){{html+='<p class="muted" style="margin-top:8px"><strong>Tips:</strong> '+r.tips.join(' ')+'</p>';}}
  el.innerHTML=html;
}}
function checkUrl(){{var u=$('sec-url').value.trim();if(!u)return;ipc({{type:'page_safety',url:u}});}}
function checkCurrent(){{ipc({{type:'page_safety_active'}});}}
function sendBug(){{
  ipc({{type:'bug_report',title:$('bug-title').value,body:$('bug-body').value,include_diag:true}});
}}
function previewBug(){{ipc({{type:'bug_diag_preview'}});}}
window.__amni_receive=function(msg){{
  if(!msg)return;
  if(msg.type==='themes'){{try{{renderThemes(JSON.parse(msg.data));}}catch(e){{}}}}
  if(msg.type==='active_theme'){{try{{var t=typeof msg.data==='string'?JSON.parse(msg.data):msg.data;activeId=t.id||activeId;}}catch(e){{}}}}
  if(msg.type==='theme_export'){{$('th-json').value=msg.data||'';}}
  if(msg.type==='extensions'){{try{{renderExt(JSON.parse(msg.data));}}catch(e){{$('ext-list').textContent='Parse error';}}}}
  if(msg.type==='page_safety'){{paintSafety(typeof msg.data==='string'?JSON.parse(msg.data):msg.data);}}
  if(msg.type==='bug_diag'){{$('bug-diag').textContent=msg.data||'';}}
  if(msg.type==='navigate_to'&&msg.url){{/* host navigates */}}
  if(msg.type==='success'){{var n=document.createElement('div');n.className='ok';n.textContent=msg.message||'OK';document.body.appendChild(n);setTimeout(function(){{n.remove();}},2500);}}
  if(msg.type==='error'){{alert(msg.message||'Error');}}
}};
ipc({{type:'theme_list'}});ipc({{type:'theme_get_active'}});ipc({{type:'ext_list'}});ipc({{type:'bug_diag_preview'}});
</script>
</body></html>"##,
        css = css,
        ver = APP_VERSION,
        ext_dir = ext_dir.replace('\\', "\\\\"),
    )
}
