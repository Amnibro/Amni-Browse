use serde::{Deserialize, Serialize};
use std::fs;
use crate::storage::config::BrowserConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub bg_primary: String,
    pub bg_secondary: String,
    pub bg_tertiary: String,
    pub bg_hover: String,
    pub border: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub accent: String,
    pub accent_hover: String,
    pub accent_glow: String,
    pub danger: String,
    pub success: String,
    pub warning: String,
    pub gradient_start: String,
    pub gradient_mid: String,
    pub gradient_end: String,
    pub tab_active: String,
    pub tab_inactive: String,
    pub background_image: Option<String>,
    pub background_opacity: f32,
    pub font_family: String,
    pub border_radius: String,
    pub is_custom: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self::amni_dark()
    }
}

impl Theme {
    pub fn amni_dark() -> Self {
        Self {
            id: "amni-dark".into(),
            name: "Amni Dark".into(),
            bg_primary: "#0a0e14".into(),
            bg_secondary: "#0f1419".into(),
            bg_tertiary: "#1a1f2e".into(),
            bg_hover: "#1e2636".into(),
            border: "#1a2332".into(),
            text_primary: "#e0e6f0".into(),
            text_secondary: "#6b7d99".into(),
            accent: "#00d4ff".into(),
            accent_hover: "#33ddff".into(),
            accent_glow: "rgba(0, 212, 255, 0.15)".into(),
            danger: "#ff4757".into(),
            success: "#2ed573".into(),
            warning: "#ffa502".into(),
            gradient_start: "#00d4ff".into(),
            gradient_mid: "#7c5cfc".into(),
            gradient_end: "#2ed573".into(),
            tab_active: "#0a0e14".into(),
            tab_inactive: "#0f1419".into(),
            background_image: None,
            background_opacity: 1.0,
            font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', Helvetica, Arial, sans-serif".into(),
            border_radius: "8px".into(),
            is_custom: false,
        }
    }

    pub fn amni_cosmos() -> Self {
        Self {
            id: "amni-cosmos".into(),
            name: "Amni Cosmos".into(),
            bg_primary: "#070b1a".into(),
            bg_secondary: "#0c1228".into(),
            bg_tertiary: "#141d3a".into(),
            bg_hover: "#1a264d".into(),
            border: "#1a2652".into(),
            text_primary: "#e0e8ff".into(),
            text_secondary: "#6878a8".into(),
            accent: "#7c5cfc".into(),
            accent_hover: "#9b82fd".into(),
            accent_glow: "rgba(124, 92, 252, 0.15)".into(),
            danger: "#ff6b81".into(),
            success: "#7bed9f".into(),
            warning: "#ffda79".into(),
            gradient_start: "#7c5cfc".into(),
            gradient_mid: "#00d4ff".into(),
            gradient_end: "#ff6b81".into(),
            tab_active: "#070b1a".into(),
            tab_inactive: "#0c1228".into(),
            background_image: None,
            background_opacity: 1.0,
            font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', Helvetica, Arial, sans-serif".into(),
            border_radius: "10px".into(),
            is_custom: false,
        }
    }

    pub fn amni_emerald() -> Self {
        Self {
            id: "amni-emerald".into(),
            name: "Amni Emerald".into(),
            bg_primary: "#0a140e".into(),
            bg_secondary: "#0f1a13".into(),
            bg_tertiary: "#162e1d".into(),
            bg_hover: "#1e3d27".into(),
            border: "#1a3322".into(),
            text_primary: "#e0f0e6".into(),
            text_secondary: "#6b9978".into(),
            accent: "#2ed573".into(),
            accent_hover: "#55e08e".into(),
            accent_glow: "rgba(46, 213, 115, 0.15)".into(),
            danger: "#ff4757".into(),
            success: "#2ed573".into(),
            warning: "#ffa502".into(),
            gradient_start: "#2ed573".into(),
            gradient_mid: "#00d4ff".into(),
            gradient_end: "#7c5cfc".into(),
            tab_active: "#0a140e".into(),
            tab_inactive: "#0f1a13".into(),
            background_image: None,
            background_opacity: 1.0,
            font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', Helvetica, Arial, sans-serif".into(),
            border_radius: "6px".into(),
            is_custom: false,
        }
    }

    pub fn amni_light() -> Self {
        Self {
            id: "amni-light".into(),
            name: "Amni Light".into(),
            bg_primary: "#f5f7fa".into(),
            bg_secondary: "#ffffff".into(),
            bg_tertiary: "#ebeef2".into(),
            bg_hover: "#e1e5eb".into(),
            border: "#d0d5dd".into(),
            text_primary: "#1a1f2e".into(),
            text_secondary: "#5a6577".into(),
            accent: "#0077cc".into(),
            accent_hover: "#0099ee".into(),
            accent_glow: "rgba(0, 119, 204, 0.1)".into(),
            danger: "#d63031".into(),
            success: "#00b894".into(),
            warning: "#e17055".into(),
            gradient_start: "#0077cc".into(),
            gradient_mid: "#6c5ce7".into(),
            gradient_end: "#00b894".into(),
            tab_active: "#ffffff".into(),
            tab_inactive: "#f0f2f5".into(),
            background_image: None,
            background_opacity: 1.0,
            font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', Helvetica, Arial, sans-serif".into(),
            border_radius: "8px".into(),
            is_custom: false,
        }
    }

    pub fn amni_crimson() -> Self {
        Self {
            id: "amni-crimson".into(),
            name: "Amni Crimson".into(),
            bg_primary: "#140a0e".into(),
            bg_secondary: "#1a0f14".into(),
            bg_tertiary: "#2e161e".into(),
            bg_hover: "#3d1e2a".into(),
            border: "#331a24".into(),
            text_primary: "#f0e0e6".into(),
            text_secondary: "#996b7d".into(),
            accent: "#ff4757".into(),
            accent_hover: "#ff6b7e".into(),
            accent_glow: "rgba(255, 71, 87, 0.15)".into(),
            danger: "#ff4757".into(),
            success: "#2ed573".into(),
            warning: "#ffa502".into(),
            gradient_start: "#ff4757".into(),
            gradient_mid: "#ffa502".into(),
            gradient_end: "#7c5cfc".into(),
            tab_active: "#140a0e".into(),
            tab_inactive: "#1a0f14".into(),
            background_image: None,
            background_opacity: 1.0,
            font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', Helvetica, Arial, sans-serif".into(),
            border_radius: "8px".into(),
            is_custom: false,
        }
    }

    pub fn amni_solarflare() -> Self {
        Self {
            id: "amni-solarflare".into(),
            name: "Amni Solarflare".into(),
            bg_primary: "#1a1208".into(),
            bg_secondary: "#23180b".into(),
            bg_tertiary: "#36210f".into(),
            bg_hover: "#4b2a13".into(),
            border: "#5f3213".into(),
            text_primary: "#ffe8cc".into(),
            text_secondary: "#d9b48a".into(),
            accent: "#ff9f1c".into(),
            accent_hover: "#ffbf69".into(),
            accent_glow: "rgba(255, 159, 28, 0.2)".into(),
            danger: "#ff4d6d".into(),
            success: "#70e000".into(),
            warning: "#ffd60a".into(),
            gradient_start: "#ff9f1c".into(),
            gradient_mid: "#ff4d6d".into(),
            gradient_end: "#ffd60a".into(),
            tab_active: "#1a1208".into(),
            tab_inactive: "#23180b".into(),
            background_image: None,
            background_opacity: 1.0,
            font_family: "'Trebuchet MS', 'Segoe UI', sans-serif".into(),
            border_radius: "12px".into(),
            is_custom: false,
        }
    }

    pub fn amni_mint_matrix() -> Self {
        Self {
            id: "amni-mint-matrix".into(),
            name: "Amni Mint Matrix".into(),
            bg_primary: "#071210".into(),
            bg_secondary: "#0c1b18".into(),
            bg_tertiary: "#102723".into(),
            bg_hover: "#18352f".into(),
            border: "#1f4a42".into(),
            text_primary: "#d8fff3".into(),
            text_secondary: "#7ec8b2".into(),
            accent: "#00e5a8".into(),
            accent_hover: "#5dffcf".into(),
            accent_glow: "rgba(0, 229, 168, 0.18)".into(),
            danger: "#ef476f".into(),
            success: "#06d6a0".into(),
            warning: "#ffd166".into(),
            gradient_start: "#00e5a8".into(),
            gradient_mid: "#00b4d8".into(),
            gradient_end: "#90e0ef".into(),
            tab_active: "#071210".into(),
            tab_inactive: "#0c1b18".into(),
            background_image: None,
            background_opacity: 1.0,
            font_family: "'Consolas', 'Segoe UI', monospace".into(),
            border_radius: "6px".into(),
            is_custom: false,
        }
    }

    pub fn amni_paper_sunset() -> Self {
        Self {
            id: "amni-paper-sunset".into(),
            name: "Amni Paper Sunset".into(),
            bg_primary: "#fdf6ec".into(),
            bg_secondary: "#fffaf2".into(),
            bg_tertiary: "#f7ecde".into(),
            bg_hover: "#f1dfca".into(),
            border: "#e8ceb3".into(),
            text_primary: "#3a2b20".into(),
            text_secondary: "#6f5644".into(),
            accent: "#c85a3d".into(),
            accent_hover: "#e07a5f".into(),
            accent_glow: "rgba(200, 90, 61, 0.15)".into(),
            danger: "#b00020".into(),
            success: "#2a9d8f".into(),
            warning: "#e9c46a".into(),
            gradient_start: "#c85a3d".into(),
            gradient_mid: "#f4a261".into(),
            gradient_end: "#e9c46a".into(),
            tab_active: "#fffaf2".into(),
            tab_inactive: "#f7ecde".into(),
            background_image: None,
            background_opacity: 1.0,
            font_family: "'Georgia', 'Palatino Linotype', serif".into(),
            border_radius: "14px".into(),
            is_custom: false,
        }
    }

    pub fn amni_deep_space() -> Self {
        Self {
            id: "amni-deep-space".into(),
            name: "Amni Deep Space".into(),
            bg_primary: "#050812".into(),
            bg_secondary: "#0b1020".into(),
            bg_tertiary: "#141b32".into(),
            bg_hover: "#1b2645".into(),
            border: "#26365f".into(),
            text_primary: "#dbe7ff".into(),
            text_secondary: "#8aa0d6".into(),
            accent: "#56cfe1".into(),
            accent_hover: "#72efdd".into(),
            accent_glow: "rgba(86, 207, 225, 0.2)".into(),
            danger: "#ff5d8f".into(),
            success: "#80ed99".into(),
            warning: "#ffd166".into(),
            gradient_start: "#7f5af0".into(),
            gradient_mid: "#56cfe1".into(),
            gradient_end: "#72efdd".into(),
            tab_active: "#050812".into(),
            tab_inactive: "#0b1020".into(),
            background_image: None,
            background_opacity: 1.0,
            font_family: "'Segoe UI', 'Tahoma', sans-serif".into(),
            border_radius: "4px".into(),
            is_custom: false,
        }
    }

    pub fn all_builtin() -> Vec<Theme> {
        vec![
            Self::amni_dark(),
            Self::amni_cosmos(),
            Self::amni_emerald(),
            Self::amni_light(),
            Self::amni_crimson(),
            Self::amni_solarflare(),
            Self::amni_mint_matrix(),
            Self::amni_paper_sunset(),
            Self::amni_deep_space(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub active_theme_id: String,
    pub custom_themes: Vec<Theme>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            active_theme_id: "amni-dark".into(),
            custom_themes: Vec::new(),
        }
    }
}

impl ThemeConfig {
    fn file_path() -> std::path::PathBuf {
        BrowserConfig::config_dir().join("theme.json")
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            let cfg = Self::default();
            cfg.save();
            cfg
        }
    }

    pub fn save(&self) {
        let path = Self::file_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(&path, data).ok();
        }
    }

    pub fn active_theme(&self) -> Theme {
        if let Some(custom) = self.custom_themes.iter().find(|t| t.id == self.active_theme_id) {
            return custom.clone();
        }
        Theme::all_builtin()
            .into_iter()
            .find(|t| t.id == self.active_theme_id)
            .unwrap_or_else(Theme::amni_dark)
    }

    pub fn set_theme(&mut self, theme_id: &str) {
        self.active_theme_id = theme_id.to_string();
        self.save();
    }

    pub fn add_custom_theme(&mut self, theme: Theme) {
        self.custom_themes.retain(|t| t.id != theme.id);
        self.custom_themes.push(theme);
        self.save();
    }

    pub fn remove_custom_theme(&mut self, theme_id: &str) {
        self.custom_themes.retain(|t| t.id != theme_id);
        if self.active_theme_id == theme_id {
            self.active_theme_id = "amni-dark".into();
        }
        self.save();
    }

    pub fn all_themes(&self) -> Vec<Theme> {
        let mut themes = Theme::all_builtin();
        themes.extend(self.custom_themes.clone());
        themes
    }

    pub fn theme_to_css_vars(theme: &Theme) -> String {
        format!(
            r#"
            --bg-primary: {};
            --bg-secondary: {};
            --bg-tertiary: {};
            --bg-hover: {};
            --border: {};
            --text-primary: {};
            --text-secondary: {};
            --accent: {};
            --accent-hover: {};
            --accent-glow: {};
            --danger: {};
            --success: {};
            --warning: {};
            --gradient-start: {};
            --gradient-mid: {};
            --gradient-end: {};
            --tab-active: {};
            --tab-inactive: {};
            --bg-opacity: {};
            --font-family: {};
            --radius: {};
            "#,
            theme.bg_primary,
            theme.bg_secondary,
            theme.bg_tertiary,
            theme.bg_hover,
            theme.border,
            theme.text_primary,
            theme.text_secondary,
            theme.accent,
            theme.accent_hover,
            theme.accent_glow,
            theme.danger,
            theme.success,
            theme.warning,
            theme.gradient_start,
            theme.gradient_mid,
            theme.gradient_end,
            theme.tab_active,
            theme.tab_inactive,
            theme.background_opacity,
            theme.font_family,
            theme.border_radius,
        )
    }

    pub fn all_themes_json(&self) -> String {
        serde_json::to_string(&self.all_themes()).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn active_theme_json(&self) -> String {
        serde_json::to_string(&self.active_theme()).unwrap_or_else(|_| "{}".to_string())
    }
}
