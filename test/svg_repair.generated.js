(function(){
var REV='1',MARK='data-amni-svg',NS='http://www.w3.org/2000/svg',XL='http://www.w3.org/1999/xlink';
if(window.__amniSvgBusy)return;window.__amniSvgBusy=1;
var S=window.__amniSvg||(window.__amniSvg={n:0,rules:null,sprites:{},pending:0});
function esc(s){return String(s).replace(/[^\w-]/g,function(c){return'\\'+c})}
function byId(root,id){try{return root.querySelector('#'+esc(id))}catch(e){return null}}
function parseDecls(body){
var d={},parts=body.split(';');
for(var i=0;i<parts.length;i++){
var p=parts[i],c=p.indexOf(':');if(c<0)continue;
var k=p.slice(0,c).trim().toLowerCase(),v=p.slice(c+1).replace(/!important\s*$/i,'').trim();
if(!v)continue;
if(/^(fill|stroke|stop-color|stop-opacity|fill-opacity|stroke-opacity|stroke-width|stroke-linecap|stroke-linejoin|stroke-dasharray|fill-rule|color)$/.test(k))d[k]=v;
}
return d;}
function scanCss(txt,out){
if(!txt)return;
var re=/([^{}]+)\{([^{}]*)\}/g,m,guard=0;
while((m=re.exec(txt))&&guard++<4000){
var sel=m[1].trim();
if(!sel||sel.charAt(0)==='@'||sel.indexOf('\x7b')>=0)continue;
if(!/(^|[;\s])(fill|stroke|stop-color)\s*:/i.test(m[2]))continue;
var d=parseDecls(m[2]);
for(var k in d)if(k!=='color'){out.push([sel,d]);break}
}}
function paintRules(){
if(S.rules)return S.rules;
var out=[],st=document.querySelectorAll('style'),i;
for(i=0;i<st.length;i++)if(!st[i].hasAttribute('data-amni-compat'))scanCss(st[i].textContent,out);
var links=document.querySelectorAll('link[rel~=stylesheet][href]');
for(i=0;i<links.length&&i<24;i++)fetchSheet(links[i].href);
S.rules=out;return out;}
function fetchSheet(href){
if(S.sprites['css:'+href])return;S.sprites['css:'+href]=1;
try{
var x=new XMLHttpRequest();x.open('GET',href,true);x.timeout=6000;S.pending++;
x.onload=function(){try{if(x.status<400&&/fill\s*:|stroke\s*:/i.test(x.responseText)){var o=[];scanCss(x.responseText,o);if(o.length){S.rules=(S.rules||[]).concat(o);S.dirty=1}}}catch(e){}S.pending--;later()};
x.onerror=x.ontimeout=function(){S.pending--;later()};
x.send();
}catch(e){S.pending--}}
function fetchSprite(path){
if(path in S.sprites)return S.sprites[path];
S.sprites[path]=null;
try{
var x=new XMLHttpRequest();x.open('GET',path,true);x.timeout=6000;S.pending++;
x.onload=function(){try{
var d=document.implementation.createHTMLDocument('s');d.body.innerHTML=x.responseText;
S.sprites[path]=d.body;S.dirty=1;
}catch(e){}S.pending--;later()};
x.onerror=x.ontimeout=function(){S.pending--;later()};
x.send();
}catch(e){S.pending--}
return null;}
function ensureDefs(svg){
var d=svg.querySelector('defs');
if(d&&d.parentNode===svg)return d;
d=document.createElementNS(NS,'defs');
svg.insertBefore(d,svg.firstChild);
return d;}
function hrefOf(el){
return el.getAttribute('href')||el.getAttributeNS(XL,'href')||el.getAttribute('xlink:href')||'';}
function adopt(svg,target,depth){
var clone=target.cloneNode(true),map={};
var all=[clone];
var kids=clone.querySelectorAll?clone.querySelectorAll('*'):[];
for(var i=0;i<kids.length;i++)all.push(kids[i]);
for(i=0;i<all.length;i++){
var old=all[i].getAttribute&&all[i].getAttribute('id');
if(!old)continue;
var nid='amni'+(++S.n);map[old]=nid;all[i].setAttribute('id',nid);
}
for(i=0;i<all.length;i++)remap(all[i],map);
ensureDefs(svg).appendChild(clone);
pull(svg,clone,depth+1);
return clone;}
function remap(el,map){
if(!el.attributes)return;
for(var i=0;i<el.attributes.length;i++){
var a=el.attributes[i],v=a.value;
if(v.indexOf('#')<0)continue;
var nv=v.replace(/#([\w:.-]+)/g,function(m0,id){return map[id]?'#'+map[id]:m0});
if(nv!==v)a.value=nv;
}}
function pull(svg,scope,depth){
if(depth>6)return;
var i,el,h,hash,id,path,src,target;
var uses=scope.querySelectorAll?scope.querySelectorAll('use'):[];
for(i=0;i<uses.length;i++){
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
try{el.removeAttributeNS(XL,'href')}catch(e){}
el.removeAttribute('xlink:href');
}
var all=scope.querySelectorAll?scope.querySelectorAll('*'):[];
for(i=0;i<all.length;i++){
el=all[i];
if(!el.attributes)continue;
for(var j=0;j<el.attributes.length;j++){
var v=el.attributes[j].value;
var m=/url\x28['"]?#([\w:.-]+)/.exec(v);
if(!m)continue;
target=document.getElementById(m[1]);
if(!target||(svg.contains&&svg.contains(target)))continue;
var c=adopt(svg,target,depth);
el.attributes[j].value=v.replace(/url\x28['"]?#[\w:.-]+/,'url\x28#'+c.getAttribute('id'));
}}}
function stampCss(rules){
for(var r=0;r<rules.length;r++){
var els;try{els=document.querySelectorAll(rules[r][0])}catch(e){continue}
for(var i=0;i<els.length;i++){
var el=els[i];
if(el.namespaceURI!==NS)continue;
var owned=(el.getAttribute('data-amni-p')||'').split(',');
for(var k in rules[r][1]){
if(k==='color')continue;
if(el.hasAttribute(k)&&owned.indexOf(k)<0)continue;
el.setAttribute(k,p3(rules[r][1][k]));
if(owned.indexOf(k)<0)owned.push(k);
}
el.setAttribute('data-amni-p',owned.join(','));
}}}
var CC=['fill','stroke','stop-color','flood-color','lighting-color','style'];
function claim(svg){
var all=[svg],kids=svg.querySelectorAll('*'),i,j;
for(i=0;i<kids.length;i++)all.push(kids[i]);
for(i=0;i<all.length;i++){
var el=all[i],own=[];
for(j=0;j<CC.length;j++){
var v=el.getAttribute(CC[j]);
if(!v||!/currentcolor/i.test(v))continue;
el.setAttribute('data-amni-cc-'+CC[j],v);own.push(CC[j]);
}
if(own.length)el.setAttribute('data-amni-cc',own.join(','));
}}
function p3(col){
var m=/^color\(display-p3\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)(?:\s*\/\s*([\d.]+))?\)$/.exec(col);
if(!m)return col;
function lin(c){return c<=0.04045?c/12.92:Math.pow((c+0.055)/1.055,2.4)}
function gam(c){c=Math.max(0,Math.min(1,c));return c<=0.0031308?12.92*c:1.055*Math.pow(c,1/2.4)-0.055}
var r=lin(+m[1]),g=lin(+m[2]),b=lin(+m[3]);
var X=0.4865709*r+0.2656677*g+0.1982173*b,Y=0.2289746*r+0.6917385*g+0.0792869*b,Z=0.0000000*r+0.0451134*g+1.0439444*b;
var R=3.2404542*X-1.5371385*Y-0.4985314*Z,G=-0.9692660*X+1.8760108*Y+0.0415560*Z,B=0.0556434*X-0.2040259*Y+1.0572252*Z;
var rgb=[gam(R),gam(G),gam(B)].map(function(c){return Math.round(c*255)});
return m[4]!=null&&+m[4]<1?'rgba('+rgb.join(',')+','+m[4]+')':'rgb('+rgb.join(',')+')';
}
function resolveColor(svg){
var els=svg.querySelectorAll('[data-amni-cc]'),i,j;
var all=svg.hasAttribute('data-amni-cc')?[svg]:[];
for(i=0;i<els.length;i++)all.push(els[i]);
for(i=0;i<all.length;i++){
var el=all[i],col='';
try{col=p3(getComputedStyle(el).color||'')}catch(e){}
if(!col||/currentcolor/i.test(col))continue;
var own=(el.getAttribute('data-amni-cc')||'').split(',');
for(j=0;j<own.length;j++){
var k=own[j];if(!k)continue;
var orig=el.getAttribute('data-amni-cc-'+k);
if(orig==null)continue;
var next=k==='style'?orig.replace(/currentcolor/gi,col):orig.replace(/currentcolor/gi,col);
if(el.getAttribute(k)!==next)el.setAttribute(k,next);
}}}
function run(){
var rules=paintRules();
if(rules.length)stampCss(rules);
var svgs=document.getElementsByTagName('svg'),done=0;
for(var i=0;i<svgs.length&&i<400;i++){
var svg=svgs[i];
if(svg.parentNode&&svg.parentNode.namespaceURI===NS)continue;
var fresh=svg.getAttribute(MARK)!==REV;
if(!fresh&&!S.dirty){try{resolveColor(svg)}catch(e){}continue}
try{pull(svg,svg,0);claim(svg);resolveColor(svg);svg.setAttribute(MARK,REV);done++}catch(e){}
}
S.dirty=0;
return done;}
function later(){
if(S.t)return;
S.t=setTimeout(function(){S.t=0;try{run()}catch(e){}},250);}
try{run()}catch(e){}
window.__amniSvgBusy=0;
var d=[300,900,2200];
for(var i=0;i<d.length;i++)setTimeout(function(){try{run()}catch(e){}},d[i]);
if(!S.iv){
var ticks=0;
S.iv=setInterval(function(){
if(++ticks>20){clearInterval(S.iv);S.iv=0;return}
try{run()}catch(e){}
},1500);}
})()