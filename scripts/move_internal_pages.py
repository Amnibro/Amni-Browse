s=open('src/platform/servo_real.rs',encoding='utf-8').read()
lines=s.split('\n')
start=next(i for i,l in enumerate(lines) if l.startswith('const SETTINGS_TPL: &str'))
end=next(i for i,l in enumerate(lines) if l.startswith('fn esc_html('))
block='\n'.join(lines[start:end+1])
assert 'const NEWTAB_TPL' in block and 'const TUTORIAL_TPL' in block
block=block.replace('const SETTINGS_TPL','pub const SETTINGS_TPL').replace('const NEWTAB_TPL','pub const NEWTAB_TPL').replace('const TUTORIAL_TPL','pub const TUTORIAL_TPL').replace('fn esc_html(','pub fn esc_html(')
block=block.replace('Real Servo &#183; Amni-Scient','__ENGINE__ &#183; Amni-Scient')
header='use crate::ui::theme::Theme;\nuse crate::storage::bookmarks::Bookmark;\n'
tail=r'''
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
'''
open('src/ui/internal_pages.rs','w',encoding='utf-8',newline='\n').write(header+block+'\n'+tail)
new=lines[:start]+['use crate::ui::internal_pages::{NEWTAB_TPL, SETTINGS_TPL, TUTORIAL_TPL, esc_html};']+lines[end+1:]
t='\n'.join(new)
old='NEWTAB_TPL.replace("__THEME__", &self.theme_root_vars()).replace("__TILES__", &tiles).replace("__VER__", env!("CARGO_PKG_VERSION"))'
assert t.count(old)==1
t=t.replace(old,old+'.replace("__ENGINE__", "Real Servo")')
open('src/platform/servo_real.rs','w',encoding='utf-8',newline='').write(t)
m=open('src/ui/mod.rs',encoding='utf-8').read()
open('src/ui/mod.rs','w',encoding='utf-8').write(m.replace('pub mod tokens;','pub mod tokens;\npub mod internal_pages;'))
print('moved',end-start+1,'lines')
