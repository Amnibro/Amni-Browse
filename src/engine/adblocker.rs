use log::debug;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Auth / SSO hosts and script paths — never block (login buttons, GSI, Apple, MSAL).
static AUTH_ALLOW: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        // Google Identity Services (X, etc. Continuue with Google)
        "accounts.google.com",
        "apis.google.com",
        "www.gstatic.com/gsi",
        "gstatic.com/gsi",
        "www.gstatic.com/identity",
        "gstatic.com/identity",
        "www.google.com/recaptcha",
        "www.gstatic.com/recaptcha",
        "recaptcha.net",
        "oauth2.googleapis.com",
        "www.googleapis.com/oauth2",
        "www.googleapis.com/identitytoolkit",
        "identitytoolkit.googleapis.com",
        "securetoken.googleapis.com",
        "appleid.apple.com",
        "appleid.cdn-apple.com",
        "login.microsoftonline.com",
        "login.live.com",
        "login.windows.net",
        "aadcdn.msauth.net",
        "aadcdn.msftauth.net",
        "github.com/login",
        "github.com/sessions",
        "connect.facebook.net/", // FB Login SDK; still blocked if AUTH_PIXEL matches
        "www.facebook.com/login",
        "www.facebook.com/dialog",
        "graph.facebook.com",
        "platform.linkedin.com/in.js",
        "www.linkedin.com/oauth",
        "api.twitter.com/oauth",
        "api.x.com/oauth",
        "twitter.com/i/api",
        "x.com/i/api",
        "auth0.com",
        "okta.com",
        "oktacdn.com",
        "cdn.auth0.com",
        // Cloudflare Turnstile / challenge platform (downdetector, etc.)
        "challenges.cloudflare.com",
        "challenge-platform",
        "cdn-cgi/challenge",
        "turnstile",
        "static.cloudflareinsights.com",
    ]
});

/// Path/query shapes that are always auth, even on hosts that also serve ads.
fn looks_like_auth_url(url_lower: &str) -> bool {
    AUTH_ALLOW.iter().any(|a| url_lower.contains(a))
        || url_lower.contains("/gsi/")
        || url_lower.contains("/o/oauth2")
        || url_lower.contains("/oauth2/")
        || url_lower.contains("ux_mode=popup")
        || url_lower.contains("gsiwebsdk")
        || url_lower.contains("redirect_uri=gis_")
}

/// Pixel / beacon paths on hosts that also serve login SDKs.
static AUTH_PIXEL: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "fbevents.js",
        "facebook.net/signals",
        "facebook.com/tr",
        "pixel.facebook.com",
    ]
});

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
        "fbevents.js",
        "facebook.net/signals",
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

        if AUTH_PIXEL.iter().any(|p| url_lower.contains(p)) {
            self.blocked_count += 1;
            debug!("Blocked (auth-pixel): {}", url);
            return true;
        }

        if looks_like_auth_url(&url_lower) {
            return false;
        }

        for domain in BLOCKED_DOMAINS.iter() {
            if url_lower.contains(domain) {
                self.blocked_count += 1;
                debug!("Blocked (domain): {}", url);
                return true;
            }
        }

        if self.tracker_blocking {
            for pattern in AD_PATTERNS.iter() {
                if pattern.is_match(&url_lower) {
                    self.blocked_count += 1;
                    debug!("Blocked (pattern): {}", url);
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
    fn test_allows_sso_sdks() {
        let mut blocker = AdBlocker::new(true, true);
        assert!(!blocker.should_block("https://connect.facebook.net/en_US/sdk.js"));
        assert!(!blocker.should_block("https://accounts.google.com/gsi/client"));
        assert!(!blocker.should_block("https://accounts.google.com/gsi/select"));
        assert!(!blocker.should_block("https://accounts.google.com/o/oauth2/v2/auth?client_id=x"));
        assert!(!blocker.should_block("https://www.gstatic.com/gsi/style"));
        assert!(!blocker.should_block("https://apis.google.com/js/api.js"));
        assert!(!blocker.should_block("https://appleid.cdn-apple.com/appleauth/static/jsapi/appleid/1/en_US/appleid.auth.js"));
        assert!(!blocker.should_block("https://login.microsoftonline.com/common/oauth2/v2.0/authorize"));
        assert!(!blocker.should_block("https://x.com/i/api/1.1/onboarding/sso_init.json"));
        assert!(blocker.should_block("https://connect.facebook.net/en_US/fbevents.js"));
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
