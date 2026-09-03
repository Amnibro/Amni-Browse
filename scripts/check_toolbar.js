const fs=require("fs");
const h=fs.readFileSync("assets/chrome/toolbar.html","utf8");
const rev=(h.match(/chromeRev:'([^']+)'/)||[])[1]||"MISSING";
const rows=[
["#shell is 119px two-row chrome",/#shell\{height:119px/.test(h)],
["#omni row present",h.includes('id="omni"')&&/#omni\{/.test(h)],
["#nav-gap spacer",h.includes('id="nav-gap"')],
["#url-wrap full-width pill",/#url-wrap\{[^}]*flex:1 1 auto;min-width:160px/.test(h)&&/#url-wrap\{[^}]*border-radius:15px/.test(h)],
["#tab-list does not grow",/#tab-list\{[^}]*flex:0 1 auto/.test(h)],
["close is a real button",/\.tab \.close\{/.test(h)&&!/\.tab \.close\{[^}]*pointer-events:none/.test(h)&&h.includes("requestCloseTab")&&h.includes("closest('.close')")],
["bookmark scoped to 26px",/#url-actions \.nav-btn\{[^}]*width:26px/.test(h)],
["lock has all 3 states",["#lock.secure","#lock.insecure","#lock.local"].every(c=>h.includes(c))],
["lock uses SVG glyphs",h.includes("SVG_LOCK")&&h.includes("SVG_WARN")],
["page favicon in omnibox",h.includes('id="page-ico"')&&h.includes("setPageIcon")],
["load pulse shimmer",h.includes("loadPulse")&&/#progress\{height:3px/.test(h)],
["folder tab via CSS seam",/\.tab\.active\{[^}]*box-shadow:0 1px 0 var\(--tab-active\)/.test(h)&&!h.includes('id="folder-outline"')&&!h.includes("updateFolderOutline")],
["chrome-body wraps omni+nav",h.includes('id="chrome-body"')],
["omnibox pointerdown claims focus",h.includes("claimOmni")&&/pointerdown/.test(h)],
["tabs use theme tokens",/\.tab\{[^}]*background:var\(--tab-inactive\)/.test(h)&&/\.tab\.active\{[^}]*background:var\(--tab-active\)/.test(h)],
["no condensed body stretch",!/html,body\{[^}]*font-stretch/.test(h)],
["no chrome font-stretch",!/font-stretch\s*:/.test(h)||!/font-stretch:80%/.test(h)],
["forward stays clickable when dimmed",/\.nav-btn\.disabled\{[^}]*opacity:\.34/.test(h)&&!/\.nav-btn\.disabled\{[^}]*pointer-events:none/.test(h)],
["lock primed before poll",/setLock\(url\.value,null\);poll\(\)/.test(h)],
["zoom uses class not inline",h.includes("classList.toggle('off'")&&!/zoomLevel\.style\.color/.test(h)],
["roving Home/End",h.includes("e.key==='Home'")&&h.includes("e.key==='End'")],
["single tab stop",/tabindex="\$\{t\.active\?0:-1\}"/.test(h)],
["tab drag syncs on uid not url",h.includes("lastBackendUids")&&h.includes("applyMoveIds")&&h.includes("tabUids")&&h.includes("movePending")],
["chrome shell at CSS top",/html,body\{[^}]*justify-content:flex-start/.test(h)&&!/html,body\{[^}]*justify-content:flex-end/.test(h)],
["omnibox claims kbd on pointerdown",h.includes("function claimOmni")&&h.includes("kbd(1)")],
["#shell is not parked with margin-top:auto",!/#shell\{[^}]*margin-top\s*:\s*auto/.test(h)],
["suggest below 119px chrome",/#suggest,#panel\{[^}]*top:119px/.test(h)]];
let bad=0;
for(const[n,ok]of rows){ok||bad++;console.log((ok?"PASS":"FAIL")+"  "+n)}
console.log(`\nchromeRev ${rev} | ${h.length} bytes | ${rows.length-bad}/${rows.length} pass`);
process.exit(bad?1:0);
