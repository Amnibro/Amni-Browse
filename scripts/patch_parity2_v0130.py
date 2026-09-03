def rw(p,f,nl=None):
    s=open(p,encoding='utf-8').read(); n=f(s); assert n!=s,p; open(p,'w',encoding='utf-8',newline=nl).write(n)
def sub1(s,a,b):
    assert s.count(a)==1,(a[:70],s.count(a)); return s.replace(a,b)
FIND_JS=r"""const FIND_SCRIPT: &str = "(function(){var H=window.CSS&&CSS.highlights;var st={q:'',ranges:[],i:-1};function clear(){if(H){CSS.highlights.delete('amni-find');CSS.highlights.delete('amni-find-cur')}st={q:'',ranges:[],i:-1}}function collect(q){var out=[],w=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT,{acceptNode:function(n){var p=n.parentElement;if(!p)return NodeFilter.FILTER_REJECT;var t=p.tagName;if(t==='SCRIPT'||t==='STYLE'||t==='NOSCRIPT')return NodeFilter.FILTER_REJECT;return n.nodeValue.toLowerCase().indexOf(q)>=0?NodeFilter.FILTER_ACCEPT:NodeFilter.FILTER_SKIP}}),n;while((n=w.nextNode())){var s=n.nodeValue.toLowerCase(),k=0;while((k=s.indexOf(q,k))>=0){var r=document.createRange();r.setStart(n,k);r.setEnd(n,k+q.length);out.push(r);k+=q.length;if(out.length>5000)return out}}return out}function paint(){if(!H)return;var h=new Highlight();st.ranges.forEach(function(r){h.add(r)});CSS.highlights.set('amni-find',h);if(st.i>=0)CSS.highlights.set('amni-find-cur',new Highlight(st.ranges[st.i]))}function ensureCss(){if(document.getElementById('amni-find-css'))return;var s=document.createElement('style');s.id='amni-find-css';s.textContent='::highlight(amni-find){background:#ffd54a;color:#111}::highlight(amni-find-cur){background:#ff8a00;color:#111}';(document.head||document.documentElement).appendChild(s)}window.__amniFind=function(q,dir){q=(q||'').toLowerCase();if(!q){clear();return 0}ensureCss();if(q!==st.q){st.q=q;st.ranges=collect(q);st.i=st.ranges.length?0:-1}else if(st.ranges.length){st.i=(st.i+(dir<0?-1:1)+st.ranges.length)%st.ranges.length}if(!st.ranges.length){paint();return 0}var r=st.ranges[st.i];try{var sel=window.getSelection();sel.removeAllRanges();if(!H)sel.addRange(r)}catch(e){}try{var el=r.startContainer.parentElement;el&&el.scrollIntoView({block:'center',inline:'nearest'})}catch(e){}paint();return st.ranges.length};window.__amniFindClear=clear})()";"""
def chromium(s):
    s=sub1(s,'enum Ev { Cmd(String, HashMap<String, String>),',FIND_JS+'\nenum Ev { Cmd(String, HashMap<String, String>),')
    s=sub1(s,'struct Tab { uid: u64, view: WebView, core: Option<ICoreWebView2>, url: String, title: String, private: bool, loading: bool, zoom: f64, can_back: bool, can_forward: bool, icon: Option<String>, audio: bool }',
             'struct Tab { uid: u64, view: WebView, core: Option<ICoreWebView2>, url: String, title: String, private: bool, loading: bool, zoom: f64, can_back: bool, can_forward: bool, icon: Option<String>, audio: bool, pinned: bool, group: Option<String> }')
    s=sub1(s,'    blocker: Rc<RefCell<AdBlocker>>,\n    shield: Rc<Cell<bool>>,\n}\ntype Push','    blocker: Rc<RefCell<AdBlocker>>,\n    shield: Rc<Cell<bool>>,\n    collapsed: Vec<String>,\n    ephemeral: bool,\n}\ntype Push')
    s=sub1(s,'.with_initialization_script(&format!("{};{}", FETCH_SHIM, KEY_SCRIPT))\n            .with_custom_protocol','.with_initialization_script(&format!("{};{};{}", FETCH_SHIM, KEY_SCRIPT, FIND_SCRIPT))\n            .with_custom_protocol')
    s=sub1(s,'let tab = Tab { uid, view, core, url: url.to_string(), title: String::new(), private, loading: true, zoom: self.state.config.default_zoom, can_back: false, can_forward: false, icon: None, audio: false };',
             'let tab = Tab { uid, view, core, url: url.to_string(), title: String::new(), private, loading: true, zoom: self.state.config.default_zoom, can_back: false, can_forward: false, icon: None, audio: false, pinned: false, group: None };')
    s=sub1(s,'''    fn persist(&mut self) {
        if !self.state.config.restore_session { return; }''','''    fn persist(&mut self) {
        if !self.state.config.restore_session || self.ephemeral { return; }''')
    s=sub1(s,'''SessionTab { url: display_url(&t.url), title: t.title.clone(), is_active: self.tabs.get(self.active).map(|a| a.uid == t.uid).unwrap_or(false), history: vec![display_url(&t.url)], history_index: 0, engine: "chromium".into() }''',
             '''SessionTab { url: display_url(&t.url), title: t.title.clone(), is_active: self.tabs.get(self.active).map(|a| a.uid == t.uid).unwrap_or(false), history: vec![display_url(&t.url)], history_index: 0, engine: "chromium".into(), pinned: t.pinned, group: t.group.clone() }''')
    s=sub1(s,'''"url": display_url(&t.url), "active": i == self.active, "loading": t.loading, "engine": "chromium", "icon": t.icon, "is_private": t.private, "audio": t.audio,''',
             '''"url": display_url(&t.url), "active": i == self.active, "loading": t.loading, "engine": "chromium", "icon": t.icon, "is_private": t.private, "audio": t.audio, "pinned": t.pinned, "group": t.group, "collapsed": t.group.as_ref().map(|g| self.collapsed.contains(g)).unwrap_or(false),''')
    s=sub1(s,'''            "find" => { let q = a.get("q").cloned().unwrap_or_default(); self.find_query = q.clone(); if !q.is_empty() { self.active_js(&format!("window.find({},false,false,true)", serde_json::to_string(&q).unwrap_or_default())); } }
            "find_next" => { let q = self.find_query.clone(); self.active_js(&format!("window.find({},false,false,true)", serde_json::to_string(&q).unwrap_or_default())); }
            "find_prev" => { let q = self.find_query.clone(); self.active_js(&format!("window.find({},false,true,true)", serde_json::to_string(&q).unwrap_or_default())); }
            "find_close" => { self.find_query.clear(); self.active_js("try{window.getSelection().removeAllRanges()}catch(e){}"); self.focus_content(); }''',
             '''            "find" | "find_next" | "find_prev" => {
                let q = a.get("q").cloned().unwrap_or_else(|| self.find_query.clone());
                let dir = match name { "find_prev" => -1, _ => a.get("dir").and_then(|d| d.parse::<i32>().ok()).unwrap_or(1) };
                self.find_query = q.clone();
                self.active_js(&format!("window.__amniFind&&window.__amniFind({},{})", serde_json::to_string(&q).unwrap_or_default(), dir));
            }
            "find_close" => { self.find_query.clear(); self.active_js("window.__amniFindClear&&window.__amniFindClear()"); self.focus_content(); }
            "pin_tab" => {
                if let Some(i) = a.get("id").and_then(|s| idx_of(s)).filter(|i| *i < self.tabs.len()) {
                    let was_active = self.active == i;
                    let mut t = self.tabs.remove(i);
                    t.pinned = !t.pinned;
                    let dest = match t.pinned { true => self.tabs.iter().filter(|x| x.pinned).count(), false => self.tabs.iter().filter(|x| x.pinned).count().min(self.tabs.len()) };
                    self.tabs.insert(dest, t);
                    self.active = match was_active { true => dest, false => self.tabs.iter().position(|x| self.tabs.get(self.active).map(|a| a.uid == x.uid).unwrap_or(false)).unwrap_or(self.active.min(self.tabs.len() - 1)) };
                    self.layout(); self.persist();
                }
            }
            "tab_set_group" => {
                if let Some(i) = a.get("id").and_then(|s| idx_of(s)).filter(|i| *i < self.tabs.len()) {
                    let g = a.get("group").map(|g| g.trim().to_string()).filter(|g| !g.is_empty());
                    self.tabs[i].group = g.clone();
                    if let Some(g) = g { let uid = self.tabs[i].uid; let t = self.tabs.remove(i); let last = self.tabs.iter().rposition(|x| x.group.as_deref() == Some(g.as_str())).map(|p| p + 1).unwrap_or(i.min(self.tabs.len())); self.tabs.insert(last, t); self.active = self.tabs.iter().position(|x| x.uid == uid).filter(|_| self.active == i).unwrap_or_else(|| self.tabs.iter().position(|x| self.tabs.get(self.active).map(|a| a.uid == x.uid).unwrap_or(false)).unwrap_or(0)); }
                    self.layout(); self.persist();
                }
            }
            "group_toggle" => {
                if let Some(g) = a.get("group").cloned() {
                    match self.collapsed.iter().position(|x| x == &g) { Some(p) => { self.collapsed.remove(p); } None => { self.collapsed.push(g.clone()); if self.active_tab().and_then(|t| t.group.clone()).as_deref() == Some(g.as_str()) { if let Some(n) = self.tabs.iter().position(|t| t.group.as_deref() != Some(g.as_str())) { self.switch_tab(n); } } } }
                }
            }
            "new_window" => { let _ = std::process::Command::new(std::env::current_exe().unwrap_or_default()).arg("--new-window").spawn(); }''')
    s=sub1(s,'''            ("n", true, false) => self.open_tab(None, true),''','''            ("n", true, false) => self.open_tab(None, true),
            ("n", false, false) => self.command("new_window", &HashMap::new()),''')
    s=sub1(s,'''var hot={t:1,w:1,l:1,d:1,tab:1,h:1,j:1,u:1,f:1,p:1,r:1,'1':1,'2':1,'3':1,'4':1,'5':1,'6':1,'7':1,'8':1,'9':1,'=':1,'+':1,'-':1,'0':1,n:e.shiftKey?1:0,k:e.shiftKey?1:0,i:e.shiftKey?1:0};''','''var hot={t:1,w:1,l:1,d:1,tab:1,h:1,j:1,u:1,f:1,p:1,r:1,n:1,'1':1,'2':1,'3':1,'4':1,'5':1,'6':1,'7':1,'8':1,'9':1,'=':1,'+':1,'-':1,'0':1,k:e.shiftKey?1:0,i:e.shiftKey?1:0};''')
    s=sub1(s,'''                    if !started && !private && !is_internal(&u) && u.starts_with("http") { self.state.history.record_visit(&u, &title); self.state.history.save(); }''',
             '''                    if !started && !private && !is_internal(&u) && u.starts_with("http") { self.state.history.record_visit(&u, &title); self.state.history.save(); }
                    if !started && !is_internal(&u) {
                        let scripts = self.state.extensions.get_content_scripts(&u);
                        if let Some(t) = self.tabs.get(i) { for (_id, js, css) in scripts { for sheet in css { let _ = t.view.evaluate_script(&crate::engine::daily_driver::inject_css_script(&sheet)); } for code in js { let _ = t.view.evaluate_script(&code); } } }
                    }''')
    s=sub1(s,'''    let saved = SessionManager::load().filter(|_| state.config.restore_session);''','''    let ephemeral = std::env::args().any(|a| a == "--new-window");
    let saved = SessionManager::load().filter(|_| state.config.restore_session && !ephemeral);''')
    s=sub1(s,'''find_query: String::new(), protocol: protocol.clone(), events: events.clone(), proxy: proxy.clone(), blocker, shield };''','''find_query: String::new(), protocol: protocol.clone(), events: events.clone(), proxy: proxy.clone(), blocker, shield, collapsed: Vec::new(), ephemeral };''')
    s=sub1(s,'''        a.spawn_tab(&u, false, None);
        if t.is_active { active = a.tabs.len().saturating_sub(1); }''','''        let i = a.spawn_tab(&u, false, None);
        if let Some(tab) = a.tabs.get_mut(i) { tab.pinned = t.pinned; tab.group = t.group.clone(); }
        if t.is_active { active = a.tabs.len().saturating_sub(1); }''')
    return s
rw('src/platform/chromium.rs',chromium,'\n')
def session(s):
    s=sub1(s,'    #[serde(default)]\n    pub engine: String,\n}','    #[serde(default)]\n    pub engine: String,\n    #[serde(default)]\n    pub pinned: bool,\n    #[serde(default)]\n    pub group: Option<String>,\n}')
    return s
rw('src/storage/session.rs',session)
def servo(s):
    return s.replace('history: vec![url], history_index: 0, engine: if media { "media".into() } else { "servo".into() } })','history: vec![url], history_index: 0, engine: if media { "media".into() } else { "servo".into() }, pinned: false, group: None })')
rw('src/platform/servo_real.rs',servo,'')
def modrs(s):
    return sub1(s,'#[cfg(feature = "webview")]\npub mod chromium;','#[cfg(all(feature = "webview", target_os = "windows"))]\npub mod chromium;')
rw('src/platform/mod.rs',modrs)
def mainrs(s):
    return sub1(s,'''    #[cfg(all(feature = "webview", not(feature = "servo-real")))]
    { info!("  Backend: Chromium (WebView2 via wry/tao)"); platform::chromium::run(state); }''','''    #[cfg(all(feature = "webview", not(feature = "servo-real"), target_os = "windows"))]
    { info!("  Backend: Chromium (WebView2 via wry/tao)"); platform::chromium::run(state); }
    #[cfg(all(feature = "webview", not(feature = "servo-real"), not(target_os = "windows")))]
    { let _ = state; info!("  Backend: WebView (wry/tao, WebKitGTK)"); platform::webview::Browser::new().run(); }''')
rw('src/main.rs',mainrs)
def toolbar(s):
    s=sub1(s,'.tab .favicon.ico{background:transparent;padding:0}','.tab .favicon.ico{background:transparent;padding:0}\n.tab.pinned{min-width:34px;max-width:34px;padding:0 9px}\n.tab.pinned .title,.tab.pinned .close{display:none}\n.tab.grouped{border-top:2px solid var(--gc,var(--accent))}\n.grp{display:inline-flex;align-items:center;gap:6px;height:22px;margin:0 2px 4px 6px;padding:0 9px;border-radius:11px;background:var(--gc,var(--accent));color:#111;font-size:11px;font-weight:700;cursor:pointer;align-self:flex-end;white-space:nowrap}\n.grp .n{opacity:.7;font-weight:500}\n#tabmenu{position:absolute;z-index:40;display:none;min-width:190px;padding:6px;border-radius:8px;background:var(--bg-elev);border:1px solid var(--stroke);box-shadow:0 8px 24px rgba(0,0,0,.45)}\n#tabmenu.on{display:block}\n#tabmenu button{display:block;width:100%;text-align:left;padding:7px 10px;border:0;border-radius:6px;background:transparent;color:var(--text);font:inherit;cursor:pointer}\n#tabmenu button:hover{background:var(--bg-hover)}')
    old_render="const html=(list||[]).map(t=>{const m=mono(t.title&&t.title!=='New Tab'?t.title:t.url);const ico=t.icon?`<span class=\"favicon ico\"><img src=\"${esc(t.icon)}\" alt=\"\"></span>`:`<span class=\"favicon\">${esc(m.ch)}</span>`;return`<div class=\"tab ${t.active?'active':''} ${t.engine==='media'?'media':''} ${t.loading?'loading':''}\" data-id=\"${esc(t.id)}\" tabindex=\"${t.active?0:-1}\" role=\"tab\" aria-selected=\"${t.active?'true':'false'}\" title=\"${esc(t.title||'New Tab')}\">${ico}<span class=\"title\">${esc(t.title||'New Tab')}</span><button type=\"button\" class=\"close\" title=\"Close tab\" aria-label=\"Close tab\">&#215;</button><span class=\"engine-badge\"></span></div>`}).join('');"
    new_render="let lastGroup=null;const gcol=g=>'hsl('+(Array.from(g).reduce((a,c)=>(a*31+c.charCodeAt(0))>>>0,7)%360)+',60%,55%)';const html=(list||[]).map(t=>{const m=mono(t.title&&t.title!=='New Tab'?t.title:t.url);const ico=t.icon?`<span class=\"favicon ico\"><img src=\"${esc(t.icon)}\" alt=\"\"></span>`:`<span class=\"favicon\">${esc(m.ch)}</span>`;let head='';if(t.group&&t.group!==lastGroup){const n=(list||[]).filter(x=>x.group===t.group).length;head=`<span class=\"grp\" data-group=\"${esc(t.group)}\" style=\"--gc:${gcol(t.group)}\" title=\"${t.collapsed?'Expand':'Collapse'} group\">${esc(t.group)}<span class=\"n\">${t.collapsed?'('+n+')':''}</span></span>`}lastGroup=t.group||null;if(t.collapsed&&!t.active)return head;return head+`<div class=\"tab ${t.active?'active':''} ${t.engine==='media'?'media':''} ${t.loading?'loading':''} ${t.pinned?'pinned':''} ${t.group?'grouped':''}\" data-id=\"${esc(t.id)}\" data-group=\"${esc(t.group||'')}\" ${t.group?'style=\"--gc:'+gcol(t.group)+'\"':''} tabindex=\"${t.active?0:-1}\" role=\"tab\" aria-selected=\"${t.active?'true':'false'}\" title=\"${esc(t.title||'New Tab')}\">${ico}<span class=\"title\">${esc(t.title||'New Tab')}</span><button type=\"button\" class=\"close\" title=\"Close tab\" aria-label=\"Close tab\">&#215;</button><span class=\"engine-badge\"></span></div>`}).join('');"
    s=sub1(s,old_render,new_render)
    s=sub1(s,"  $('#tabs').addEventListener('dblclick',e=>{if(!e.target.closest('.tab')&&!e.target.closest('#new-tab'))cmd('new_tab')});",
             "  $('#tabs').addEventListener('dblclick',e=>{if(!e.target.closest('.tab')&&!e.target.closest('#new-tab')&&!e.target.closest('.grp'))cmd('new_tab')});\n  $('#tabs').addEventListener('click',e=>{const g=e.target.closest('.grp');if(g){e.stopPropagation();cmd('group_toggle',{group:g.dataset.group})}});\n  const tabMenu=document.createElement('div');tabMenu.id='tabmenu';document.body.appendChild(tabMenu);\n  function hideTabMenu(){tabMenu.classList.remove('on');tabMenu.innerHTML='';if(!suggest.classList.contains('on')&&!panel.classList.contains('on'))overlay(0)}\n  $('#tabs').addEventListener('contextmenu',e=>{const t=e.target.closest('.tab');if(!t)return;e.preventDefault();const id=t.dataset.id,pinned=t.classList.contains('pinned'),group=t.dataset.group||'';tabMenu.innerHTML=`<button data-a=\"pin\">${pinned?'Unpin tab':'Pin tab'}</button><button data-a=\"group\">${group?'Move to group\\u2026':'Add to group\\u2026'}</button>${group?'<button data-a=\"ungroup\">Remove from group</button>':''}<button data-a=\"dup\">Duplicate</button><button data-a=\"close\">Close tab</button>`;tabMenu.style.left=Math.min(e.clientX,window.innerWidth-200)+'px';tabMenu.style.top=(e.clientY+4)+'px';tabMenu.classList.add('on');overlay(window.innerHeight);tabMenu.onclick=ev=>{const b=ev.target.closest('button');if(!b)return;const a=b.dataset.a;hideTabMenu();if(a==='pin')cmd('pin_tab',{id});else if(a==='group'){const name=window.prompt('Group name',group||'');if(name!=null)cmd('tab_set_group',{id,group:name})}else if(a==='ungroup')cmd('tab_set_group',{id,group:''});else if(a==='dup')cmd('duplicate_tab');else if(a==='close')cmd('close_tab',{id})}});\n  document.addEventListener('mousedown',e=>{if(tabMenu.classList.contains('on')&&!e.target.closest('#tabmenu'))hideTabMenu()});\n  document.addEventListener('keydown',e=>{if(e.key==='Escape'&&tabMenu.classList.contains('on'))hideTabMenu()});")
    return s
rw('assets/chrome/toolbar.html',toolbar)
print('parity2 patched')
