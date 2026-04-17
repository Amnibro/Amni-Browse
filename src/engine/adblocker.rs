use log::debug;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static BLOCKED_DOMAINS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let domains: HashSet<&str> = [
        "doubleclick.net",
        "googlesyndication.com",
        "googleadservices.com",
        "google-analytics.com",
        "googletagmanager.com",
        "googletagservices.com",
        "adservice.google.com",
        "pagead2.googlesyndication.com",
        "facebook.com/tr",
        "connect.facebook.net",
        "pixel.facebook.com",
        "facebook-hardware.com",
        "adnxs.com",
        "adsrvr.org",
        "advertising.com",
        "rubiconproject.com",
        "pubmatic.com",
        "openx.net",
        "casalemedia.com",
        "criteo.com",
        "criteo.net",
        "outbrain.com",
        "taboola.com",
        "mgid.com",
        "hotjar.com",
        "mixpanel.com",
        "segment.io",
        "amplitude.com",
        "fullstory.com",
        "mouseflow.com",
        "crazyegg.com",
        "clicktale.com",
        "newrelic.com",
        "nr-data.net",
        "optimizely.com",
        "scorecardresearch.com",
        "quantserve.com",
        "demdex.net",
        "omtrdc.net",
        "rlcdn.com",
        "bluekai.com",
        "krxd.net",
        "exelator.com",
        "agkn.com",
        "adsymptotic.com",
        "adform.net",
        "serving-sys.com",
        "eyeota.net",
        "mathtag.com",
        "tapad.com",
        "cdn.jsdelivr.net/npm/fingerprintjs",
        "platform.twitter.com/widgets",
        "platform.linkedin.com",
        "bat.bing.com",
        "tr.snapchat.com",
        "analytics.tiktok.com",
        "sc-static.net",
    ]
    .into_iter()
    .collect();
    domains
});

static AD_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    let patterns = vec![
        r"[?&]utm_[a-z]+=",       // UTM tracking params
        r"/ads?/",                 // Ad paths
        r"/adserv",                // Ad serving
        r"/pixel[./]",            // Tracking pixels
        r"/beacon[./]",           // Beacons
        r"[?&]fbclid=",           // Facebook click ID
        r"[?&]gclid=",            // Google click ID
        r"[?&]mc_[a-z]+=",        // Mailchimp tracking
        r"/track(ing)?[./]",      // Generic tracking
        r"\.gif\?.*&t=",          // Tracking GIFs
        r"/collect\?",            // Data collection endpoints
        r"/__utm\.gif",           // UTM tracking pixel
        r"/piwik\.",              // Piwik/Matomo
        r"/matomo\.",             // Matomo analytics
    ];

    patterns
        .into_iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
});

#[derive(Debug, Clone)]
pub struct AdBlocker {
    pub enabled: bool,
    pub tracker_blocking: bool,
    blocked_count: u64,
}

impl AdBlocker {
    pub fn new(block_ads: bool, block_trackers: bool) -> Self {
        Self {
            enabled: block_ads,
            tracker_blocking: block_trackers,
            blocked_count: 0,
        }
    }

    pub fn should_block(&mut self, url: &str) -> bool {
        if !self.enabled {
            return false;
        }

        let url_lower = url.to_lowercase();

        for domain in BLOCKED_DOMAINS.iter() {
            if url_lower.contains(domain) {
                self.blocked_count += 1;
                debug!("ðŸ›¡ï¸ Blocked (domain): {}", url);
                return true;
            }
        }

        if self.tracker_blocking {
            for pattern in AD_PATTERNS.iter() {
                if pattern.is_match(&url_lower) {
                    self.blocked_count += 1;
                    debug!("ðŸ›¡ï¸ Blocked (pattern): {}", url);
                    return true;
                }
            }
        }

        false
    }

    pub fn clean_url(url: &str) -> String {
        if let Ok(mut parsed) = url::Url::parse(url) {
            let tracking_params: HashSet<&str> = [
                "utm_source",
                "utm_medium",
                "utm_campaign",
                "utm_term",
                "utm_content",
                "fbclid",
                "gclid",
                "gclsrc",
                "mc_cid",
                "mc_eid",
                "msclkid",
                "yclid",
                "dclid",
                "_ga",
                "_gl",
                "ref",
                "igshid",
                "si",
            ]
            .into_iter()
            .collect();

            let cleaned_pairs: Vec<(String, String)> = parsed
                .query_pairs()
                .filter(|(key, _)| !tracking_params.contains(key.as_ref()))
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            if cleaned_pairs.is_empty() {
                parsed.set_query(None);
            } else {
                let query_string: String = cleaned_pairs
                    .iter()
                    .map(|(k, v)| {
                        if v.is_empty() {
                            k.clone()
                        } else {
                            format!("{}={}", k, v)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("&");
                parsed.set_query(Some(&query_string));
            }

            parsed.to_string()
        } else {
            url.to_string()
        }
    }

    pub fn blocked_count(&self) -> u64 {
        self.blocked_count
    }
    pub fn is_blocked_url(url: &str) -> bool {
        let u = url.to_lowercase();
        BLOCKED_DOMAINS.iter().any(|d| u.contains(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_known_ad_domains() {
        let mut blocker = AdBlocker::new(true, true);
        assert!(blocker.should_block("https://doubleclick.net/ad.js"));
        assert!(blocker.should_block("https://google-analytics.com/collect"));
        assert!(blocker.should_block("https://pixel.facebook.com/tr"));
    }

    #[test]
    fn test_allows_normal_urls() {
        let mut blocker = AdBlocker::new(true, true);
        assert!(!blocker.should_block("https://www.rust-lang.org/"));
        assert!(!blocker.should_block("https://github.com/"));
    }

    #[test]
    fn test_cleans_tracking_params() {
        let url = "https://example.com/page?id=123&utm_source=twitter&utm_medium=social&real=yes";
        let cleaned = AdBlocker::clean_url(url);
        assert!(cleaned.contains("id=123"));
        assert!(cleaned.contains("real=yes"));
        assert!(!cleaned.contains("utm_source"));
        assert!(!cleaned.contains("utm_medium"));
    }

    #[test]
    fn test_blocks_tracking_patterns() {
        let mut blocker = AdBlocker::new(true, true);
        assert!(blocker.should_block("https://example.com/track/pixel.gif"));
        assert!(blocker.should_block("https://example.com?utm_source=test"));
    }
}
