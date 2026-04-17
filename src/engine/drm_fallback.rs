/// DRM Detection and WebView Fallback Routing
///
/// Amni Browse is a custom rendering engine, but some domains require
/// proprietary DRM (Widevine, FairPlay) that cannot be implemented in
/// an open-source engine without licensing agreements. This module detects
/// those domains and routes them to a system WebView or browser as a
/// graceful fallback.

use std::collections::HashMap;

/// A domain known to require DRM-protected content delivery.
pub struct DrmDomain {
    pub domain: &'static str,
    pub requires_widevine: bool,
    pub requires_fairplay: bool,
    pub concession_note: &'static str,
}

/// Static list of known DRM-requiring domains and patterns.
pub static DRM_DOMAINS: &[DrmDomain] = &[
    // --- Video Streaming ---
    DrmDomain {
        domain: "netflix.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Netflix DRM - no open standard alternative exists",
    },
    DrmDomain {
        domain: "disneyplus.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Disney+ DRM - content protection mandated by studios",
    },
    DrmDomain {
        domain: "hulu.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Hulu DRM - streaming license requires EME support",
    },
    DrmDomain {
        domain: "hbomax.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for HBO Max DRM - legacy domain still in use",
    },
    DrmDomain {
        domain: "max.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Max (formerly HBO Max) DRM - Warner Bros Discovery content protection",
    },
    DrmDomain {
        domain: "peacocktv.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Peacock DRM - NBCUniversal content protection",
    },
    DrmDomain {
        domain: "primevideo.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Prime Video DRM - Amazon content protection",
    },
    DrmDomain {
        domain: "amazon.com/gp/video",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Amazon Video DRM - legacy path for Prime Video",
    },
    DrmDomain {
        domain: "paramountplus.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Paramount+ DRM - ViacomCBS content protection",
    },
    DrmDomain {
        domain: "appletv.apple.com",
        requires_widevine: false,
        requires_fairplay: true,
        concession_note: "FairPlay DRM required for Apple TV+ - Apple proprietary content protection, no third-party implementation possible",
    },
    DrmDomain {
        domain: "crunchyroll.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Crunchyroll DRM - anime licensor requirements mandate EME",
    },
    DrmDomain {
        domain: "funimation.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Funimation DRM - anime content protection (merging into Crunchyroll)",
    },
    DrmDomain {
        domain: "discoveryplus.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Discovery+ DRM - Warner Bros Discovery content protection",
    },
    DrmDomain {
        domain: "espnplus.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for ESPN+ DRM - Disney sports streaming content protection",
    },
    // --- Music Streaming ---
    DrmDomain {
        domain: "spotify.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Spotify encrypted audio streams - EME used for web playback DRM",
    },
    DrmDomain {
        domain: "tidal.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Tidal DRM - high-fidelity audio content protection",
    },
    DrmDomain {
        domain: "deezer.com",
        requires_widevine: true,
        requires_fairplay: false,
        concession_note: "Widevine CDM required for Deezer DRM - music streaming content protection",
    },
];

/// Check if a URL matches any known DRM-requiring domain.
///
/// This performs substring matching against the URL to handle both
/// exact domain matches and path-based patterns (e.g., amazon.com/gp/video).
/// Also checks for EME-related Content-Type hints.
pub fn is_drm_required(url: &str) -> bool {
    let lower = url.to_lowercase();
    for entry in DRM_DOMAINS {
        if lower.contains(entry.domain) {
            return true;
        }
    }
    // Detect EME usage hints in URL patterns
    if lower.contains("drm") || lower.contains("widevine") || lower.contains("fairplay") {
        return true;
    }
    // Detect common CDM license server patterns
    if lower.contains("/license") && (lower.contains("playready") || lower.contains("widevine")) {
        return true;
    }
    false
}

/// The fallback method used when DRM is detected.
#[derive(Debug, Clone, PartialEq)]
pub enum FallbackMethod {
    /// Route to a system WebView (e.g., WebView2 on Windows, WKWebView on macOS)
    WebView,
    /// Open in the user's default system browser
    SystemBrowser,
    /// No fallback -- attempt to load in engine anyway
    None,
}

/// A documented concession for a specific DRM domain, explaining
/// why the custom engine cannot handle it.
#[derive(Debug, Clone)]
pub struct DrmConcession {
    pub domain: String,
    pub reason: String,
    pub fallback_method: FallbackMethod,
    pub date_added: String,
    pub notes: String,
}

/// Statistics about DRM fallback routing.
#[derive(Debug, Clone, Default)]
pub struct DrmStats {
    pub webview_loads: u64,
    pub engine_loads: u64,
    pub total_navigations: u64,
}

/// Manages DRM detection, fallback routing, and user overrides.
pub struct DrmFallbackManager {
    pub concessions: Vec<DrmConcession>,
    pub custom_overrides: HashMap<String, bool>,
    pub stats: DrmStats,
}

impl DrmFallbackManager {
    pub fn new() -> Self {
        let concessions = DRM_DOMAINS
            .iter()
            .map(|d| DrmConcession {
                domain: d.domain.to_string(),
                reason: d.concession_note.to_string(),
                fallback_method: if d.requires_fairplay {
                    // FairPlay is Apple-only; system browser is the only viable fallback
                    FallbackMethod::SystemBrowser
                } else {
                    FallbackMethod::WebView
                },
                date_added: "2026-01-01".to_string(),
                notes: format!(
                    "widevine={}, fairplay={}",
                    d.requires_widevine, d.requires_fairplay
                ),
            })
            .collect();

        Self {
            concessions,
            custom_overrides: HashMap::new(),
            stats: DrmStats::default(),
        }
    }

    pub fn is_streaming_path(&self, url: &str) -> bool {
        let lower = url.to_lowercase();
        let non_streaming_patterns = ["/login", "/signin", "/signup", "/register", "/help",
            "/about", "/contact", "/faq", "/terms", "/privacy", "/account", "/settings",
            "/browse", "/search", "/profiles"];
        if non_streaming_patterns.iter().any(|p| lower.contains(p)) { return false; }
        if let Ok(parsed) = url::Url::parse(&lower) {
            let path = parsed.path();
            if path == "/" || path.is_empty() { return false; }
        }
        let streaming_patterns = ["/watch", "/play", "/video", "/stream", "/player",
            "/title/", "/episode/", "/movie/"];
        streaming_patterns.iter().any(|p| lower.contains(p))
    }

    pub fn should_use_webview(&self, url: &str) -> bool {
        let lower = url.to_lowercase();
        for (domain, use_webview) in &self.custom_overrides {
            if lower.contains(domain) { return *use_webview; }
        }
        if is_drm_required(url) {
            return self.is_streaming_path(url);
        }
        false
    }

    /// Add a user override for a specific domain.
    ///
    /// If `use_webview` is true, the domain will always be routed to WebView.
    /// If false, the domain will always use the native engine (even if DRM is detected).
    pub fn add_override(&mut self, domain: &str, use_webview: bool) {
        self.custom_overrides
            .insert(domain.to_lowercase(), use_webview);
    }

    /// Remove a user override for a specific domain, reverting to automatic detection.
    pub fn remove_override(&mut self, domain: &str) {
        self.custom_overrides.remove(&domain.to_lowercase());
    }

    /// Log a navigation event for statistics tracking.
    pub fn log_navigation(&mut self, url: &str, used_webview: bool) {
        self.stats.total_navigations += 1;
        if used_webview {
            self.stats.webview_loads += 1;
        } else {
            self.stats.engine_loads += 1;
        }
        log::info!(
            "DRM navigation: {} [{}]",
            url,
            if used_webview { "WebView" } else { "Engine" }
        );
    }

    /// Generate a formatted report of all DRM concessions with reasons.
    pub fn concessions_report(&self) -> String {
        let mut report = String::from("=== Amni Browse DRM Concessions Report ===\n\n");
        report.push_str(
            "The following domains require proprietary DRM that cannot be implemented\n\
             in an open-source rendering engine without licensing agreements.\n\n",
        );

        for (i, concession) in self.concessions.iter().enumerate() {
            report.push_str(&format!(
                "{}. {}\n   Reason: {}\n   Fallback: {:?}\n   Added: {}\n   Notes: {}\n\n",
                i + 1,
                concession.domain,
                concession.reason,
                concession.fallback_method,
                concession.date_added,
                concession.notes,
            ));
        }

        if !self.custom_overrides.is_empty() {
            report.push_str("--- User Overrides ---\n");
            for (domain, use_webview) in &self.custom_overrides {
                report.push_str(&format!(
                    "  {} -> {}\n",
                    domain,
                    if *use_webview { "WebView" } else { "Engine" }
                ));
            }
            report.push('\n');
        }

        report.push_str(&format!(
            "Total concessions: {}\n",
            self.concessions.len()
        ));

        report
    }

    /// Return statistics as a JSON string.
    pub fn stats_json(&self) -> String {
        format!(
            r#"{{"webview_loads":{},"engine_loads":{},"total_navigations":{},"engine_percentage":{:.1}}}"#,
            self.stats.webview_loads,
            self.stats.engine_loads,
            self.stats.total_navigations,
            if self.stats.total_navigations > 0 {
                (self.stats.engine_loads as f64 / self.stats.total_navigations as f64) * 100.0
            } else {
                100.0
            }
        )
    }
}
