import re
p='src/engine/adblocker.rs'; s=open(p,encoding='utf-8').read()
i=s.index('pub fn content_rules'); j=s.index('pub fn blocked_count')
f=s[i:j]
f=re.sub(r'\\+([./?])', lambda m: '\\\\\\\\'+m.group(1), f)
s=s[:i]+f+s[j:]
if 'fn content_rules_are_json' not in s:
    s=s.rstrip('\n')+r'''
#[cfg(test)]
mod content_rules_tests {
    use super::AdBlocker;
    #[test]
    fn content_rules_are_json() {
        let v: serde_json::Value = serde_json::from_str(&AdBlocker::content_rules()).expect("valid content-blocker json");
        let rules = v.as_array().unwrap();
        assert!(rules.len() > 100);
        assert!(rules.iter().any(|r| r["action"]["type"] == "block" && r["trigger"]["url-filter"].as_str().unwrap().contains("doubleclick\\.net")));
        assert!(rules.iter().any(|r| r["action"]["type"] == "ignore-previous-rules"));
        assert_eq!(rules.last().unwrap()["action"]["type"], "css-display-none");
    }
}
'''
open(p,'w',encoding='utf-8',newline='\n').write(s)
print([l.strip() for l in f.split('\n') if 'let esc' in l][0][:100])
