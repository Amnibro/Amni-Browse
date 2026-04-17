use std::collections::HashMap;

/// Supported web font formats.
#[derive(Debug, Clone, PartialEq)]
pub enum FontFormat {
    Ttf,
    Otf,
    Woff,
    Woff2, // flagged as unsupported for now (needs decompression)
    Unknown,
}

impl FontFormat {
    pub fn from_url(url: &str) -> Self {
        let lower = url.to_ascii_lowercase();
        if lower.ends_with(".ttf") {
            FontFormat::Ttf
        } else if lower.ends_with(".otf") {
            FontFormat::Otf
        } else if lower.ends_with(".woff2") {
            FontFormat::Woff2
        } else if lower.ends_with(".woff") {
            FontFormat::Woff
        } else {
            FontFormat::Unknown
        }
    }

    pub fn is_supported(&self) -> bool {
        matches!(self, FontFormat::Ttf | FontFormat::Otf | FontFormat::Woff)
    }
}

/// Represents a single @font-face declaration.
#[derive(Debug, Clone)]
pub struct FontFace {
    pub family: String,
    pub src_url: String,
    pub weight: u16,
    pub style: FontStyle,
    pub format: FontFormat,
    pub loaded: bool,
    pub data: Option<Vec<u8>>,
}

/// Font style variants.
#[derive(Debug, Clone, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle::Normal
    }
}

/// Cache holding parsed @font-face entries and loaded font data.
#[derive(Debug, Clone)]
pub struct FontCache {
    pub faces: Vec<FontFace>,
    pub loaded_fonts: HashMap<String, Vec<u8>>,
}

impl FontCache {
    pub fn new() -> Self {
        FontCache {
            faces: Vec::new(),
            loaded_fonts: HashMap::new(),
        }
    }

    /// Return the URL that should be fetched to load the given font face.
    /// Returns None if the format is unsupported (e.g., woff2).
    pub fn request_load(&self, face: &FontFace) -> Option<String> {
        if !face.format.is_supported() {
            log::warn!(
                "Font format {:?} is not supported for '{}' ({})",
                face.format,
                face.family,
                face.src_url
            );
            return None;
        }
        if face.loaded || self.loaded_fonts.contains_key(&face.family) {
            return None;
        }
        Some(face.src_url.clone())
    }

    /// Store raw font bytes for the given family name.
    pub fn store_font(&mut self, family: &str, data: Vec<u8>) {
        self.loaded_fonts.insert(family.to_string(), data.clone());
        // Mark all matching faces as loaded.
        for face in &mut self.faces {
            if face.family.eq_ignore_ascii_case(family) {
                face.loaded = true;
                face.data = Some(data.clone());
            }
        }
    }

    /// Retrieve raw font data for a given family and weight.
    /// Exact weight match is preferred; if not found, returns the family default.
    pub fn get_font_data(&self, family: &str, weight: u16) -> Option<&[u8]> {
        // First try an exact weight match among loaded faces.
        for face in &self.faces {
            if face.family.eq_ignore_ascii_case(family) && face.weight == weight {
                if let Some(ref data) = face.data {
                    return Some(data.as_slice());
                }
            }
        }
        // Fall back to any loaded data for this family.
        self.loaded_fonts.get(family).map(|d| d.as_slice())
    }

    /// Check whether a font family has been loaded.
    pub fn has_font(&self, family: &str) -> bool {
        self.loaded_fonts.contains_key(family)
    }

    /// Register parsed font faces into the cache.
    pub fn add_faces(&mut self, faces: Vec<FontFace>) {
        self.faces.extend(faces);
    }

    /// Return the list of URLs that need to be fetched.
    pub fn pending_urls(&self) -> Vec<(String, String)> {
        let mut urls = Vec::new();
        for face in &self.faces {
            if let Some(url) = self.request_load(face) {
                urls.push((face.family.clone(), url));
            }
        }
        urls
    }
}

/// Parse @font-face rules from raw CSS text.
///
/// Extracts blocks matching:
/// ```css
/// @font-face {
///     font-family: "MyFont";
///     src: url("https://example.com/font.woff2");
///     font-weight: 700;
///     font-style: italic;
/// }
/// ```
pub fn parse_font_face_rules(css: &str) -> Vec<FontFace> {
    let mut faces = Vec::new();
    let lower = css.to_ascii_lowercase();
    let mut search_start = 0;

    while let Some(at_pos) = lower[search_start..].find("@font-face") {
        let abs_pos = search_start + at_pos;
        // Find the opening brace.
        let brace_start = match css[abs_pos..].find('{') {
            Some(p) => abs_pos + p,
            None => break,
        };
        // Find the matching closing brace.
        let brace_end = match css[brace_start..].find('}') {
            Some(p) => brace_start + p,
            None => break,
        };

        let block = &css[brace_start + 1..brace_end];
        let face = parse_font_face_block(block);
        if !face.family.is_empty() && !face.src_url.is_empty() {
            faces.push(face);
        }

        search_start = brace_end + 1;
    }

    faces
}

/// Parse the content inside a single @font-face { ... } block.
fn parse_font_face_block(block: &str) -> FontFace {
    let mut family = String::new();
    let mut src_url = String::new();
    let mut weight: u16 = 400;
    let mut style = FontStyle::Normal;

    for declaration in block.split(';') {
        let declaration = declaration.trim();
        if declaration.is_empty() {
            continue;
        }
        let (prop, val) = match declaration.split_once(':') {
            Some((p, v)) => (p.trim().to_ascii_lowercase(), v.trim().to_string()),
            None => continue,
        };

        match prop.as_str() {
            "font-family" => {
                family = val
                    .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                    .to_string();
            }
            "src" => {
                // Extract the URL from url("...") or url('...') or url(...)
                src_url = extract_url(&val);
            }
            "font-weight" => {
                weight = match val.trim() {
                    "normal" => 400,
                    "bold" => 700,
                    "lighter" => 300,
                    "bolder" => 800,
                    other => other.parse().unwrap_or(400),
                };
            }
            "font-style" => {
                style = match val.trim() {
                    "italic" => FontStyle::Italic,
                    "oblique" => FontStyle::Oblique,
                    _ => FontStyle::Normal,
                };
            }
            _ => {}
        }
    }

    let format = FontFormat::from_url(&src_url);

    FontFace {
        family,
        src_url,
        weight,
        style,
        format,
        loaded: false,
        data: None,
    }
}

/// Extract URL from a CSS url() function value.
/// Handles: url("..."), url('...'), url(...)
/// Also handles comma-separated src lists, taking the first url().
fn extract_url(val: &str) -> String {
    let lower = val.to_ascii_lowercase();
    if let Some(start) = lower.find("url(") {
        let after_url = &val[start + 4..];
        // Find the closing paren.
        if let Some(end) = after_url.find(')') {
            let inner = after_url[..end].trim();
            // Strip surrounding quotes.
            let inner = inner
                .trim_matches(|c: char| c == '"' || c == '\'')
                .to_string();
            return inner;
        }
    }
    String::new()
}

/// Extract @font-face rules from CSS, returning the cleaned CSS (without @font-face blocks)
/// and the parsed font face list. This is the integration point: call this before passing
/// CSS to the style parser.
pub fn extract_font_faces_from_css(css: &str) -> (String, Vec<FontFace>) {
    let faces = parse_font_face_rules(css);

    // Remove @font-face blocks from the CSS so the style parser does not choke on them.
    let mut cleaned = String::with_capacity(css.len());
    let lower = css.to_ascii_lowercase();
    let mut pos = 0;

    while pos < css.len() {
        if let Some(at_pos) = lower[pos..].find("@font-face") {
            let abs_at = pos + at_pos;
            // Copy everything before this @font-face.
            cleaned.push_str(&css[pos..abs_at]);

            // Find the closing brace.
            if let Some(brace_end) = css[abs_at..].find('}') {
                pos = abs_at + brace_end + 1;
            } else {
                // Malformed; skip the rest.
                pos = css.len();
            }
        } else {
            cleaned.push_str(&css[pos..]);
            break;
        }
    }

    (cleaned, faces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_face_basic() {
        let css = r#"
            @font-face {
                font-family: "Roboto";
                src: url("https://fonts.example.com/roboto.woff2");
                font-weight: 400;
                font-style: normal;
            }
        "#;
        let faces = parse_font_face_rules(css);
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].family, "Roboto");
        assert_eq!(faces[0].src_url, "https://fonts.example.com/roboto.woff2");
        assert_eq!(faces[0].weight, 400);
        assert_eq!(faces[0].style, FontStyle::Normal);
        assert_eq!(faces[0].format, FontFormat::Woff2);
        assert!(!faces[0].format.is_supported());
    }

    #[test]
    fn test_parse_font_face_ttf() {
        let css = r#"
            @font-face {
                font-family: 'OpenSans';
                src: url(https://example.com/opensans.ttf);
                font-weight: bold;
                font-style: italic;
            }
        "#;
        let faces = parse_font_face_rules(css);
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].family, "OpenSans");
        assert_eq!(faces[0].weight, 700);
        assert_eq!(faces[0].style, FontStyle::Italic);
        assert_eq!(faces[0].format, FontFormat::Ttf);
        assert!(faces[0].format.is_supported());
    }

    #[test]
    fn test_multiple_font_faces() {
        let css = r#"
            @font-face {
                font-family: "A";
                src: url("a.otf");
            }
            body { color: red; }
            @font-face {
                font-family: "B";
                src: url("b.woff");
                font-weight: 300;
            }
        "#;
        let faces = parse_font_face_rules(css);
        assert_eq!(faces.len(), 2);
        assert_eq!(faces[0].family, "A");
        assert_eq!(faces[0].format, FontFormat::Otf);
        assert_eq!(faces[1].family, "B");
        assert_eq!(faces[1].weight, 300);
        assert_eq!(faces[1].format, FontFormat::Woff);
    }

    #[test]
    fn test_extract_removes_font_face_blocks() {
        let css = r#"@font-face { font-family: "X"; src: url("x.ttf"); }
body { color: black; }"#;
        let (cleaned, faces) = extract_font_faces_from_css(css);
        assert_eq!(faces.len(), 1);
        assert!(cleaned.contains("body"));
        assert!(!cleaned.contains("@font-face"));
    }

    #[test]
    fn test_font_cache_operations() {
        let mut cache = FontCache::new();
        assert!(!cache.has_font("Roboto"));

        let face = FontFace {
            family: "Roboto".to_string(),
            src_url: "https://example.com/roboto.ttf".to_string(),
            weight: 400,
            style: FontStyle::Normal,
            format: FontFormat::Ttf,
            loaded: false,
            data: None,
        };

        // Should want to load it.
        assert!(cache.request_load(&face).is_some());

        cache.add_faces(vec![face]);
        assert!(!cache.has_font("Roboto"));

        // Simulate storing the font.
        cache.store_font("Roboto", vec![0x00, 0x01, 0x02]);
        assert!(cache.has_font("Roboto"));
        assert_eq!(cache.get_font_data("Roboto", 400).unwrap(), &[0x00, 0x01, 0x02]);

        // Already loaded, so request_load returns None.
        assert!(cache.pending_urls().is_empty());
    }

    #[test]
    fn test_woff2_unsupported() {
        let face = FontFace {
            family: "Noto".to_string(),
            src_url: "https://example.com/noto.woff2".to_string(),
            weight: 400,
            style: FontStyle::Normal,
            format: FontFormat::Woff2,
            loaded: false,
            data: None,
        };
        let cache = FontCache::new();
        // woff2 is unsupported, so request_load should return None.
        assert!(cache.request_load(&face).is_none());
    }
}
