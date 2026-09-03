(function(){
function report(){
  try{
    var out={};
    var all=Array.prototype.slice.call(document.querySelectorAll('*'));
    var masks=all.filter(function(e){var cs=getComputedStyle(e);var m=cs.maskImage||cs.webkitMaskImage||'';return m&&m!=='none'});
    out.masks=masks.length;
    out.maskSample=masks.slice(0,5).map(function(e){var cs=getComputedStyle(e);var r=e.getBoundingClientRect();return{tag:e.tagName,cls:(e.className||'').toString().slice(0,60),mask:(cs.maskImage||cs.webkitMaskImage).slice(0,120),bg:cs.backgroundColor,w:Math.round(r.width),h:Math.round(r.height)}});
    var svgs=all.filter(function(e){return e.tagName&&e.tagName.toLowerCase()==='svg'});
    out.svgs=svgs.length;
    out.svgZero=svgs.filter(function(s){var r=s.getBoundingClientRect();return r.width===0||r.height===0}).length;
    var uses=Array.prototype.slice.call(document.querySelectorAll('use'));
    out.uses=uses.length;
    out.useExternal=uses.map(function(u){return u.getAttribute('href')||u.getAttribute('xlink:href')||''}).filter(function(h){return h&&h[0]!=='#'}).slice(0,5);
    var imgs=Array.prototype.slice.call(document.images);
    out.imgs=imgs.length;
    out.imgBroken=imgs.filter(function(i){return i.complete&&i.naturalWidth===0}).slice(0,5).map(function(i){return (i.currentSrc||i.src).slice(0,120)});
    var iconFont=all.filter(function(e){var f=getComputedStyle(e).fontFamily||'';return /icon|awesome|material|glyph/i.test(f)&&e.childNodes.length===1&&e.firstChild.nodeType===3});
    out.iconFontEls=iconFont.length;
    out.iconFontSample=iconFont.slice(0,4).map(function(e){return{font:getComputedStyle(e).fontFamily.slice(0,50),text:Array.prototype.map.call(e.textContent.slice(0,3),function(c){return c.charCodeAt(0).toString(16)}).join(',')}});
    function at(x,y){var e=document.elementFromPoint(x,y);if(!e)return null;var r=e.getBoundingClientRect();var cs=getComputedStyle(e);return{tag:e.tagName,id:e.id,cls:(e.className||'').toString().slice(0,80),pos:cs.position,bg:cs.backgroundColor,rect:[Math.round(r.left),Math.round(r.top),Math.round(r.width),Math.round(r.height)],html:(e.outerHTML||'').slice(0,240)}}
    out.bottomRight=at(innerWidth-80,innerHeight-12);
    out.bottomRight2=at(innerWidth-300,innerHeight-12);
    var logo=document.querySelector('header a[href="/"], a[aria-label*="ogo"], [class*="logo"], [class*="Logo"], a[href="/"] svg');
    if(logo){var r=logo.getBoundingClientRect();var cs=getComputedStyle(logo);out.logo={tag:logo.tagName,cls:(logo.className||'').toString().slice(0,80),rect:[Math.round(r.left),Math.round(r.top),Math.round(r.width),Math.round(r.height)],display:cs.display,vis:cs.visibility,op:cs.opacity,color:cs.color,fill:cs.fill,html:(logo.outerHTML||'').slice(0,400)}}
    var circles=Array.prototype.slice.call(document.querySelectorAll('svg circle')).slice(0,3).map(function(c){var cs=getComputedStyle(c);return{fill:cs.fill,stroke:cs.stroke,parent:(c.parentNode.getAttribute('class')||'').slice(0,40)}});
    out.circles=circles;
    document.title='AMNIPROBE '+JSON.stringify(out);
  }catch(e){document.title='AMNIPROBE-ERR '+e}
}
setTimeout(report,7000);
return 'scheduled';
})()
