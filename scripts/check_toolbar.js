const fs=require("fs");
const h=fs.readFileSync("assets/chrome/toolbar.html","utf8");
const rev=(h.match(/chromeRev:'([^']+)'/)||[])[1]||"MISSING";
const rows=[
["#nav-end not edge-pinned",!/#nav-end\{[^}]*margin-left\s*:\s*auto/.test(h)],
["#url-wrap does not grow",/#url-wrap\{flex:0 1 960px/.test(h)],
["#tab-list does not grow",/#tab-list\{[^}]*flex:0 1 auto/.test(h)],
["close inert until hover",/\.tab \.close\{[^}]*pointer-events:none/.test(h)&&/\.tab:hover \.close[^}]*pointer-events:auto/.test(h)],
["bookmark scoped to 26px",/#url-actions \.nav-btn\{[^}]*width:26px/.test(h)],
["lock has all 3 states",["#lock.secure","#lock.insecure","#lock.local"].every(c=>h.includes(c))],
["lock primed before poll",/setLock\(url\.value\);poll\(\)/.test(h)],
["zoom uses class not inline",h.includes("classList.toggle('off'")&&!/zoomLevel\.style\.color/.test(h)],
["roving Home/End",h.includes("e.key==='Home'")&&h.includes("e.key==='End'")],
["single tab stop",/tabindex="\$\{t\.active\?0:-1\}"/.test(h)]];
let bad=0;
for(const[n,ok]of rows){ok||bad++;console.log((ok?"PASS":"FAIL")+"  "+n)}
console.log(`\nchromeRev ${rev} | ${h.length} bytes | ${rows.length-bad}/${rows.length} pass`);
process.exit(bad?1:0);
