#[cfg(feature = "servo-engine")]
use fontdue::{Font, FontSettings};

pub struct TextShaper {
    #[cfg(feature = "servo-engine")]
    font: Option<Font>,
}

pub struct ShapedLine {
    pub text: String,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
}

pub struct ShapedText {
    pub lines: Vec<ShapedLine>,
    pub total_width: f32,
    pub total_height: f32,
}

impl TextShaper {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "servo-engine")]
            font: Self::load_font(),
        }
    }

    #[cfg(feature = "servo-engine")]
    fn load_font() -> Option<Font> {
        let paths = if cfg!(target_os = "windows") {
            vec!["C:/Windows/Fonts/segoeui.ttf", "C:/Windows/Fonts/arial.ttf"]
        } else if cfg!(target_os = "macos") {
            vec!["/System/Library/Fonts/Helvetica.ttc", "/Library/Fonts/Arial.ttf"]
        } else {
            vec!["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]
        };
        for p in paths {
            if let Ok(data) = std::fs::read(p) {
                if let Ok(f) = Font::from_bytes(data, FontSettings::default()) { return Some(f); }
            }
        }
        None
    }

    pub fn measure(&self, text: &str, font_size: f32) -> (f32, f32) {
        #[cfg(feature = "servo-engine")]
        if let Some(font) = &self.font {
            let sz = font_size.clamp(6.0, 200.0);
            let mut w = 0.0f32;
            let h = sz * 1.2;
            for ch in text.chars() {
                let m = font.metrics(ch, sz);
                w += m.advance_width;
            }
            return (w, h);
        }
        (text.len() as f32 * font_size * 0.6, font_size * 1.2)
    }

    pub fn layout_text(&self, text: &str, font_size: f32, max_width: f32) -> ShapedText {
        let line_height = font_size * 1.2;
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return ShapedText { lines: vec![], total_width: 0.0, total_height: 0.0 };
        }
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0f32;
        let mut max_line_width = 0.0f32;
        let space_width = self.measure(" ", font_size).0;

        for word in &words {
            let (word_w, _) = self.measure(word, font_size);
            if !current_line.is_empty() && current_width + space_width + word_w > max_width {
                max_line_width = max_line_width.max(current_width);
                lines.push(ShapedLine {
                    text: current_line.clone(), width: current_width,
                    height: line_height, baseline: font_size * 0.8,
                });
                current_line.clear();
                current_width = 0.0;
            }
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += space_width;
            }
            current_line.push_str(word);
            current_width += word_w;
        }
        if !current_line.is_empty() {
            max_line_width = max_line_width.max(current_width);
            lines.push(ShapedLine {
                text: current_line, width: current_width,
                height: line_height, baseline: font_size * 0.8,
            });
        }
        let total_height = lines.len() as f32 * line_height;
        ShapedText { lines, total_width: max_line_width, total_height }
    }

    pub fn char_width(&self, ch: char, font_size: f32) -> f32 {
        #[cfg(feature = "servo-engine")]
        if let Some(font) = &self.font {
            return font.metrics(ch, font_size.clamp(6.0, 200.0)).advance_width;
        }
        font_size * 0.6
    }
}
