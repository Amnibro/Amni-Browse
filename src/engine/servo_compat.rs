//! Universal Servo layout/form compat — applied to every content document.
//!
//! Host-agnostic patches for embedder-side gaps. Keep this sheet **conservative**:
//! broad `img`/`svg` sizing and `html { font-stretch }` have already broken
//! logos, icon fonts, and Speedtest controls. Prefer narrow selectors.
//!
//! Re-inject on load-complete and URL change so SPA navigations keep the sheet.

/// Marker attribute on the injected `<style>` element.
pub const STYLE_ATTR: &str = "data-amni-compat";

/// Version bump when the sheet changes so reinject replaces stale rules.
pub const SHEET_REV: &str = "3";

/// Full UA-compat stylesheet for Servo content webviews.
pub fn stylesheet() -> String {
    format!(
        "/* amni servo-compat r{rev} */\
\
/* Forms / editors: normalize stretch only. Never force padding — that shifts\
   caret/text inside fields (x.com). Never set html{{font-stretch}} — that\
   breaks icon fonts and brand webfonts (Cloudflare docs, etc.). */\
input:not([type=checkbox]):not([type=radio]):not([type=range]):not([type=color]):\
not([type=file]):not([type=image]):not([type=hidden]):not([type=submit]):\
not([type=button]):not([type=reset]),\
textarea,select,\
[contenteditable=\"true\"],\
[role=\"textbox\"]{{\
font-stretch:normal;\
line-height:normal;\
box-sizing:border-box}}\
textarea{{resize:vertical}}\
\
/* SVG: only unclip; do not rewrite width/height (collapses icon buttons). */\
svg{{overflow:visible}}\
\
/* Shield leaves empty reserved rails. Collapse known ad iframes only. */\
iframe[src*=\"doubleclick\"],iframe[src*=\"googlesyndication\"],\
iframe[src*=\"googletagservices\"],iframe[src*=\"amazon-adsystem\"],\
iframe[id*=\"google_ads\"],iframe[name*=\"google_ads\"]{{\
display:none!important;width:0!important;height:0!important;min-height:0!important;\
border:0!important;margin:0!important;padding:0!important}}\
[data-ad-slot],[data-ad-client],[data-google-query-id],\
ins.adsbygoogle{{display:none!important;height:0!important;min-height:0!important;\
margin:0!important;padding:0!important;overflow:hidden!important}}",
        rev = SHEET_REV
    )
}

/// JS that installs/replaces the compat sheet (idempotent across SPA loads).
pub fn inject_script() -> String {
    let css = stylesheet()
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("</", "<\\/");
    format!(
        "(function(){{try{{\
var attr='{attr}',rev='{rev}';\
var old=document.querySelector('style['+attr+']');\
if(old&&old.getAttribute(attr)===rev)return;\
if(old)old.remove();\
var s=document.createElement('style');\
s.setAttribute(attr,rev);\
s.setAttribute('data-amni-ext','1');\
s.textContent=`{css}`;\
(document.head||document.documentElement).appendChild(s);\
try{{window.__amniCompatChallenge&&window.__amniCompatChallenge()}}catch(e){{}}\
}}catch(e){{}}}})()",
        attr = STYLE_ATTR,
        rev = SHEET_REV,
        css = css
    )
}

/// Rev for the SVG repair pass; bump to force a re-run over already-marked trees.
pub const SVG_REV: &str = "1";

/// Make every inline `<svg>` self-contained before Servo serialises it standalone.
///
/// Servo rasterises an inline `<svg>` by XML-serialising only that subtree into a
/// `data:` URL (`svgsvgelement.rs::serialize_and_cache_subtree`). Anything the
/// subtree does not carry is lost: `<use href="#id">` into a sprite that lives in
/// another element, `url(#grad)` paint servers, and every author CSS `fill`/`stroke`
/// rule (both are `engine = "gecko"` in Stylo, so Servo never computes them and
/// `fill` falls back to its initial black). `currentColor` collapses to black too.
///
/// This pass runs in the page and rewrites the live DOM so the serialisation is
/// complete: referenced nodes get cloned in under fresh ids, raw author CSS paint
/// declarations get stamped as presentation attributes, and `currentColor` is
/// resolved against the computed `color`, which Servo *does* support.
/// Runs at document start (before any page script) through Servo's user-content manager:
/// polyfills for APIs Servo lacks that sites treat as mandatory. `requestIdleCallback`
/// missing made GitHub's React partials throw and render "Looks like something went wrong".
pub fn document_start_script() -> &'static str {
    r#"(function(){var w=window;if(!w.requestIdleCallback){w.requestIdleCallback=function(cb,o){var start=Date.now();return w.setTimeout(function(){cb({didTimeout:false,timeRemaining:function(){return Math.max(0,50-(Date.now()-start))}})},1)};w.cancelIdleCallback=function(id){w.clearTimeout(id)}}
if(w.Element&&!Element.prototype.scrollIntoViewIfNeeded){Element.prototype.scrollIntoViewIfNeeded=function(c){this.scrollIntoView({block:c===false?'nearest':'center'})}}})()"#
}
pub fn svg_repair_script() -> String {
    format!(
        r##"(function(){{
var REV='{rev}',MARK='data-amni-svg',NS='http://www.w3.org/2000/svg',XL='http://www.w3.org/1999/xlink';
if(window.__amniSvgBusy)return;window.__amniSvgBusy=1;
var S=window.__amniSvg||(window.__amniSvg={{n:0,rules:null,sprites:{{}},pending:0}});
function esc(s){{return String(s).replace(/[^\w-]/g,function(c){{return'\\'+c}})}}
function byId(root,id){{try{{return root.querySelector('#'+esc(id))}}catch(e){{return null}}}}
function parseDecls(body){{
var d={{}},parts=body.split(';');
for(var i=0;i<parts.length;i++){{
var p=parts[i],c=p.indexOf(':');if(c<0)continue;
var k=p.slice(0,c).trim().toLowerCase(),v=p.slice(c+1).replace(/!important\s*$/i,'').trim();
if(!v)continue;
if(/^(fill|stroke|stop-color|stop-opacity|fill-opacity|stroke-opacity|stroke-width|stroke-linecap|stroke-linejoin|stroke-dasharray|fill-rule|color)$/.test(k))d[k]=v;
}}
return d;}}
function scanCss(txt,out){{
if(!txt)return;
var re=/([^{{}}]+)\{{([^{{}}]*)\}}/g,m,guard=0;
while((m=re.exec(txt))&&guard++<4000){{
var sel=m[1].trim();
if(!sel||sel.charAt(0)==='@'||sel.indexOf('\x7b')>=0)continue;
if(!/(^|[;\s])(fill|stroke|stop-color)\s*:/i.test(m[2]))continue;
var d=parseDecls(m[2]);
for(var k in d)if(k!=='color'){{out.push([sel,d]);break}}
}}}}
function paintRules(){{
if(S.rules)return S.rules;
var out=[],st=document.querySelectorAll('style'),i;
for(i=0;i<st.length;i++)if(!st[i].hasAttribute('{attr}'))scanCss(st[i].textContent,out);
var links=document.querySelectorAll('link[rel~=stylesheet][href]');
for(i=0;i<links.length&&i<24;i++)fetchSheet(links[i].href);
S.rules=out;return out;}}
function fetchSheet(href){{
if(S.sprites['css:'+href])return;S.sprites['css:'+href]=1;
try{{
var x=new XMLHttpRequest();x.open('GET',href,true);x.timeout=6000;S.pending++;
x.onload=function(){{try{{if(x.status<400&&/fill\s*:|stroke\s*:/i.test(x.responseText)){{var o=[];scanCss(x.responseText,o);if(o.length){{S.rules=(S.rules||[]).concat(o);S.dirty=1}}}}}}catch(e){{}}S.pending--;later()}};
x.onerror=x.ontimeout=function(){{S.pending--;later()}};
x.send();
}}catch(e){{S.pending--}}}}
function fetchSprite(path){{
if(path in S.sprites)return S.sprites[path];
S.sprites[path]=null;
try{{
var x=new XMLHttpRequest();x.open('GET',path,true);x.timeout=6000;S.pending++;
x.onload=function(){{try{{
var d=document.implementation.createHTMLDocument('s');d.body.innerHTML=x.responseText;
S.sprites[path]=d.body;S.dirty=1;
}}catch(e){{}}S.pending--;later()}};
x.onerror=x.ontimeout=function(){{S.pending--;later()}};
x.send();
}}catch(e){{S.pending--}}
return null;}}
function ensureDefs(svg){{
var d=svg.querySelector('defs');
if(d&&d.parentNode===svg)return d;
d=document.createElementNS(NS,'defs');
svg.insertBefore(d,svg.firstChild);
return d;}}
function hrefOf(el){{
return el.getAttribute('href')||el.getAttributeNS(XL,'href')||el.getAttribute('xlink:href')||'';}}
function adopt(svg,target,depth){{
var clone=target.cloneNode(true),map={{}};
var all=[clone];
var kids=clone.querySelectorAll?clone.querySelectorAll('*'):[];
for(var i=0;i<kids.length;i++)all.push(kids[i]);
for(i=0;i<all.length;i++){{
var old=all[i].getAttribute&&all[i].getAttribute('id');
if(!old)continue;
var nid='amni'+(++S.n);map[old]=nid;all[i].setAttribute('id',nid);
}}
for(i=0;i<all.length;i++)remap(all[i],map);
ensureDefs(svg).appendChild(clone);
pull(svg,clone,depth+1);
return clone;}}
function remap(el,map){{
if(!el.attributes)return;
for(var i=0;i<el.attributes.length;i++){{
var a=el.attributes[i],v=a.value;
if(v.indexOf('#')<0)continue;
var nv=v.replace(/#([\w:.-]+)/g,function(m0,id){{return map[id]?'#'+map[id]:m0}});
if(nv!==v)a.value=nv;
}}}}
function pull(svg,scope,depth){{
if(depth>6)return;
var i,el,h,hash,id,path,src,target;
var uses=scope.querySelectorAll?scope.querySelectorAll('use'):[];
for(i=0;i<uses.length;i++){{
el=uses[i];h=hrefOf(el);hash=h.indexOf('#');
if(hash<0)continue;
id=h.slice(hash+1);path=h.slice(0,hash);
if(!id)continue;
src=path?fetchSprite(new URL(path,location.href).href):document;
if(!src)continue;
target=path?byId(src,id):document.getElementById(id);
if(!target||(svg.contains&&svg.contains(target)&&!path))continue;
var made=adopt(svg,target,depth);
el.setAttribute('href','#'+made.getAttribute('id'));
try{{el.removeAttributeNS(XL,'href')}}catch(e){{}}
el.removeAttribute('xlink:href');
}}
var all=scope.querySelectorAll?scope.querySelectorAll('*'):[];
for(i=0;i<all.length;i++){{
el=all[i];
if(!el.attributes)continue;
for(var j=0;j<el.attributes.length;j++){{
var v=el.attributes[j].value;
var m=/url\x28['"]?#([\w:.-]+)/.exec(v);
if(!m)continue;
target=document.getElementById(m[1]);
if(!target||(svg.contains&&svg.contains(target)))continue;
var c=adopt(svg,target,depth);
el.attributes[j].value=v.replace(/url\x28['"]?#[\w:.-]+/,'url\x28#'+c.getAttribute('id'));
}}}}}}
function stampCss(rules){{
for(var r=0;r<rules.length;r++){{
var els;try{{els=document.querySelectorAll(rules[r][0])}}catch(e){{continue}}
for(var i=0;i<els.length;i++){{
var el=els[i];
if(el.namespaceURI!==NS)continue;
var owned=(el.getAttribute('data-amni-p')||'').split(',');
for(var k in rules[r][1]){{
if(k==='color')continue;
if(el.hasAttribute(k)&&owned.indexOf(k)<0)continue;
el.setAttribute(k,p3(rules[r][1][k]));
if(owned.indexOf(k)<0)owned.push(k);
}}
el.setAttribute('data-amni-p',owned.join(','));
}}}}}}
var CC=['fill','stroke','stop-color','flood-color','lighting-color','style'];
function claim(svg){{
var all=[svg],kids=svg.querySelectorAll('*'),i,j;
for(i=0;i<kids.length;i++)all.push(kids[i]);
for(i=0;i<all.length;i++){{
var el=all[i],own=[];
for(j=0;j<CC.length;j++){{
var v=el.getAttribute(CC[j]);
if(!v||!/currentcolor/i.test(v))continue;
el.setAttribute('data-amni-cc-'+CC[j],v);own.push(CC[j]);
}}
if(own.length)el.setAttribute('data-amni-cc',own.join(','));
}}}}
function p3(col){{
var m=/^color\(display-p3\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)(?:\s*\/\s*([\d.]+))?\)$/.exec(col);
if(!m)return col;
function lin(c){{return c<=0.04045?c/12.92:Math.pow((c+0.055)/1.055,2.4)}}
function gam(c){{c=Math.max(0,Math.min(1,c));return c<=0.0031308?12.92*c:1.055*Math.pow(c,1/2.4)-0.055}}
var r=lin(+m[1]),g=lin(+m[2]),b=lin(+m[3]);
var X=0.4865709*r+0.2656677*g+0.1982173*b,Y=0.2289746*r+0.6917385*g+0.0792869*b,Z=0.0000000*r+0.0451134*g+1.0439444*b;
var R=3.2404542*X-1.5371385*Y-0.4985314*Z,G=-0.9692660*X+1.8760108*Y+0.0415560*Z,B=0.0556434*X-0.2040259*Y+1.0572252*Z;
var rgb=[gam(R),gam(G),gam(B)].map(function(c){{return Math.round(c*255)}});
return m[4]!=null&&+m[4]<1?'rgba('+rgb.join(',')+','+m[4]+')':'rgb('+rgb.join(',')+')';
}}
function resolveColor(svg){{
var els=svg.querySelectorAll('[data-amni-cc]'),i,j;
var all=svg.hasAttribute('data-amni-cc')?[svg]:[];
for(i=0;i<els.length;i++)all.push(els[i]);
for(i=0;i<all.length;i++){{
var el=all[i],col='';
try{{col=p3(getComputedStyle(el).color||'')}}catch(e){{}}
if(!col||/currentcolor/i.test(col))continue;
var own=(el.getAttribute('data-amni-cc')||'').split(',');
for(j=0;j<own.length;j++){{
var k=own[j];if(!k)continue;
var orig=el.getAttribute('data-amni-cc-'+k);
if(orig==null)continue;
var next=k==='style'?orig.replace(/currentcolor/gi,col):orig.replace(/currentcolor/gi,col);
if(el.getAttribute(k)!==next)el.setAttribute(k,next);
}}}}}}
function run(){{
var rules=paintRules();
if(rules.length)stampCss(rules);
var svgs=document.getElementsByTagName('svg'),done=0;
for(var i=0;i<svgs.length&&i<400;i++){{
var svg=svgs[i];
if(svg.parentNode&&svg.parentNode.namespaceURI===NS)continue;
var fresh=svg.getAttribute(MARK)!==REV;
if(!fresh&&!S.dirty){{try{{resolveColor(svg)}}catch(e){{}}continue}}
try{{pull(svg,svg,0);claim(svg);resolveColor(svg);svg.setAttribute(MARK,REV);done++}}catch(e){{}}
}}
S.dirty=0;
return done;}}
function later(){{
if(S.t)return;
S.t=setTimeout(function(){{S.t=0;try{{run()}}catch(e){{}}}},250);}}
try{{run()}}catch(e){{}}
window.__amniSvgBusy=0;
var d=[300,900,2200];
for(var i=0;i<d.length;i++)setTimeout(function(){{try{{run()}}catch(e){{}}}},d[i]);
if(!S.iv){{
var ticks=0;
S.iv=setInterval(function(){{
if(++ticks>20){{clearInterval(S.iv);S.iv=0;return}}
try{{run()}}catch(e){{}}
}},1500);}}
}})()"##,
        rev = SVG_REV,
        attr = STYLE_ATTR
    )
}

/// Detect stripped Cloudflare interstitial (no Turnstile widget) and surface a clear note.
pub fn challenge_notice_script() -> &'static str {
    r#"(function(){try{
if(window.__amniCfNotice)return;
var t=(document.title||'')+' '+(document.body&&document.body.innerText||'').slice(0,800);
var hit=/just a moment|performing security verification|cf-browser-verification|challenge-platform|checking your browser/i.test(t);
if(!hit)return;
var hasWidget=!!(document.querySelector('#challenge-stage,.cf-turnstile,iframe[src*="challenges.cloudflare"],iframe[src*="turnstile"]'));
if(hasWidget)return;
window.__amniCfNotice=1;
var b=document.createElement('div');
b.setAttribute('data-amni-cf-notice','1');
b.style.cssText='position:fixed;left:12px;right:12px;bottom:12px;z-index:2147483646;padding:12px 14px;border-radius:8px;background:#111418;color:#EDEFF2;font:13px/1.4 Segoe UI,system-ui,sans-serif;border:1px solid #20242B;box-shadow:0 8px 24px rgba(0,0,0,.35)';
b.textContent='Amni cannot finish this Cloudflare bot check yet (Turnstile needs engine APIs Servo does not ship). Open the site in Chrome/Edge, or try again if the check is temporary.';
(document.body||document.documentElement).appendChild(b);
}catch(e){}})()"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_is_host_agnostic_and_conservative() {
        let s = stylesheet();
        assert!(!s.to_lowercase().contains("speedtest"));
        assert!(!s.to_lowercase().contains("twitter"));
        assert!(s.contains("contenteditable"));
        assert!(s.contains("font-stretch:normal"));
        assert!(!s.contains("html{font-stretch"));
        assert!(!s.contains("max-width:100%"));
        assert!(!s.contains("height:auto"));
        assert!(s.contains("adsbygoogle"));
    }

    #[test]
    fn inject_script_marks_rev() {
        let js = inject_script();
        assert!(js.contains(STYLE_ATTR));
        assert!(js.contains(SHEET_REV));
        assert!(js.contains("querySelector"));
    }

    #[test]
    fn document_start_script_polyfills_idle_callback() {
        let js = document_start_script();
        assert!(js.contains("requestIdleCallback") && js.contains("cancelIdleCallback") && js.contains("scrollIntoViewIfNeeded"));
        let count = |c: char| js.chars().filter(|x| *x == c).count();
        assert_eq!(count('('), count(')'));
        assert_eq!(count('{'), count('}'));
    }
    #[test]
    fn svg_repair_is_balanced_and_host_agnostic() {
        let js = svg_repair_script();
        let low = js.to_lowercase();
        assert!(!low.contains("speedtest") && !low.contains("cloudflare") && !low.contains("x.com"));
        assert!(js.contains("currentcolor"));
        assert!(js.contains("data-amni-svg"));
        assert!(js.contains(SVG_REV));
        let count = |c: char| js.chars().filter(|x| *x == c).count();
        assert_eq!(count('('), count(')'), "unbalanced parens");
        assert_eq!(count('{'), count('}'), "unbalanced braces");
        assert_eq!(count('['), count(']'), "unbalanced brackets");
        assert!(!js.contains("{rev}"), "format! placeholder leaked into output");
    }

    #[test]
    fn svg_repair_dumps_for_dom_harness() {
        let path = std::path::Path::new("test/svg_repair.generated.js");
        if let Some(dir) = path.parent() { std::fs::create_dir_all(dir).ok(); }
        std::fs::write(path, svg_repair_script()).expect("dump svg repair script");
    }
}
