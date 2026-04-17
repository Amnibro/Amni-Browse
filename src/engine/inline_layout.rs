/// Inline formatting context — line breaking, text flow, inline element layout.
/// Implements CSS inline layout model: inline boxes packed into line boxes.

use super::style::{ComputedStyle, Display, TextAlign};
use super::text::TextShaper;

#[derive(Debug, Clone)]
pub enum InlineItem {
    Text { text: String, style: InlineStyle },
    InlineBox { width: f32, height: f32, node_id: usize },
    LineBreak,
}

#[derive(Debug, Clone, Default)]
pub struct InlineStyle {
    pub font_size: f32,
    pub font_weight: u16,
    pub line_height: f32,
    pub color: [u8; 4],
    pub is_bold: bool,
    pub is_italic: bool,
}

impl InlineStyle {
    pub fn from_computed(cs: &ComputedStyle) -> Self {
        Self {
            font_size: cs.font_size.max(1.0),
            font_weight: cs.font_weight,
            line_height: if cs.line_height > 0.0 { cs.line_height } else { cs.font_size * 1.2 },
            color: [cs.color.r, cs.color.g, cs.color.b, (cs.color.a * 255.0) as u8],
            is_bold: cs.font_weight >= 700,
            is_italic: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PositionedFragment {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
    pub style: InlineStyle,
    pub node_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct LineBox {
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub fragments: Vec<PositionedFragment>,
}

pub struct InlineFormattingContext {
    pub lines: Vec<LineBox>,
    pub total_height: f32,
    pub max_width: f32,
}

impl InlineFormattingContext {
    pub fn layout(items: &[InlineItem], max_width: f32, text_align: TextAlign, shaper: &TextShaper) -> Self {
        let mut ctx = Self { lines: Vec::new(), total_height: 0.0, max_width };
        if items.is_empty() { return ctx; }

        let mut current_fragments: Vec<PositionedFragment> = Vec::new();
        let mut cursor_x = 0.0f32;
        let mut line_height = 0.0f32;
        let mut line_baseline = 0.0f32;

        for item in items {
            match item {
                InlineItem::LineBreak => {
                    ctx.commit_line(&mut current_fragments, &mut cursor_x, &mut line_height, &mut line_baseline, text_align);
                }
                InlineItem::InlineBox { width, height, node_id } => {
                    if cursor_x + width > max_width && cursor_x > 0.0 {
                        ctx.commit_line(&mut current_fragments, &mut cursor_x, &mut line_height, &mut line_baseline, text_align);
                    }
                    current_fragments.push(PositionedFragment {
                        x: cursor_x, y: 0.0, width: *width, height: *height,
                        text: String::new(), style: InlineStyle::default(), node_id: Some(*node_id),
                    });
                    cursor_x += width;
                    line_height = line_height.max(*height);
                    line_baseline = line_baseline.max(*height * 0.8);
                }
                InlineItem::Text { text, style } => {
                    let words = split_words(text);
                    let space_w = shaper.measure(" ", style.font_size).0;

                    for (i, word) in words.iter().enumerate() {
                        if word.is_empty() { continue; }
                        let (word_w, _) = shaper.measure(word, style.font_size);

                        // Does this word fit on the current line?
                        let needed = if cursor_x > 0.0 && i > 0 { space_w + word_w } else { word_w };
                        if cursor_x + needed > max_width && cursor_x > 0.0 {
                            ctx.commit_line(&mut current_fragments, &mut cursor_x, &mut line_height, &mut line_baseline, text_align);
                        }

                        // Add space before word if not at line start
                        if cursor_x > 0.0 && i > 0 {
                            cursor_x += space_w;
                        }

                        let frag_height = style.line_height.max(style.font_size * 1.2);
                        current_fragments.push(PositionedFragment {
                            x: cursor_x, y: 0.0, width: word_w, height: frag_height,
                            text: word.clone(), style: style.clone(), node_id: None,
                        });
                        cursor_x += word_w;
                        line_height = line_height.max(frag_height);
                        line_baseline = line_baseline.max(style.font_size * 0.8);
                    }
                }
            }
        }

        // Commit remaining fragments
        if !current_fragments.is_empty() {
            ctx.commit_line(&mut current_fragments, &mut cursor_x, &mut line_height, &mut line_baseline, text_align);
        }

        ctx
    }

    fn commit_line(
        &mut self, fragments: &mut Vec<PositionedFragment>,
        cursor_x: &mut f32, line_height: &mut f32, line_baseline: &mut f32,
        text_align: TextAlign,
    ) {
        if fragments.is_empty() {
            *line_height = 0.0; *line_baseline = 0.0; *cursor_x = 0.0;
            return;
        }

        let lh = line_height.max(16.0);
        let content_width = *cursor_x;

        // Apply text alignment
        let offset = match text_align {
            TextAlign::Center => ((self.max_width - content_width) / 2.0).max(0.0),
            TextAlign::Right => (self.max_width - content_width).max(0.0),
            _ => 0.0,
        };

        // Position fragments vertically within the line
        for frag in fragments.iter_mut() {
            frag.x += offset;
            frag.y = self.total_height + (*line_baseline - frag.height * 0.8).max(0.0);
        }

        self.lines.push(LineBox {
            y: self.total_height,
            width: content_width,
            height: lh,
            baseline: *line_baseline,
            fragments: std::mem::take(fragments),
        });

        self.total_height += lh;
        *cursor_x = 0.0;
        *line_height = 0.0;
        *line_baseline = 0.0;
    }

    pub fn fragment_count(&self) -> usize {
        self.lines.iter().map(|l| l.fragments.len()).sum()
    }

    pub fn line_count(&self) -> usize { self.lines.len() }

    /// Collects all positioned fragments with absolute positions
    pub fn all_fragments(&self) -> Vec<&PositionedFragment> {
        self.lines.iter().flat_map(|l| l.fragments.iter()).collect()
    }
}

fn split_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() { words.push(current); }
    words
}

/// Extract inline items from a DOM subtree for inline formatting
pub fn extract_inline_items(
    handle: &markup5ever_rcdom::Handle,
    parent_style: &ComputedStyle,
) -> Vec<InlineItem> {
    use markup5ever_rcdom::NodeData;
    let mut items = Vec::new();
    extract_inline_recursive(handle, parent_style, &mut items);
    items
}

fn extract_inline_recursive(
    handle: &markup5ever_rcdom::Handle,
    style: &ComputedStyle,
    items: &mut Vec<InlineItem>,
) {
    use markup5ever_rcdom::NodeData;
    match &handle.data {
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            let collapsed = collapse_whitespace(&text);
            if !collapsed.is_empty() {
                items.push(InlineItem::Text {
                    text: collapsed,
                    style: InlineStyle::from_computed(style),
                });
            }
        }
        NodeData::Element { name, .. } => {
            let tag = name.local.to_string();
            match tag.as_str() {
                "br" => items.push(InlineItem::LineBreak),
                "img" => {
                    items.push(InlineItem::InlineBox { width: 100.0, height: 100.0, node_id: 0 });
                }
                // Block-level elements start a new line in inline context
                "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                | "section" | "article" | "header" | "footer" | "nav" | "main"
                | "blockquote" | "pre" | "ul" | "ol" | "li" | "table" | "form" | "hr" => {
                    items.push(InlineItem::LineBreak);
                    for child in handle.children.borrow().iter() {
                        extract_inline_recursive(child, style, items);
                    }
                    items.push(InlineItem::LineBreak);
                }
                // Inline elements: inherit style and recurse
                _ => {
                    let mut child_style = style.clone();
                    match tag.as_str() {
                        "strong" | "b" => child_style.font_weight = 700,
                        "em" | "i" => {}
                        "code" => child_style.font_family = "monospace".into(),
                        "small" => child_style.font_size *= 0.85,
                        "big" => child_style.font_size *= 1.2,
                        "h1" => { child_style.font_size = 32.0; child_style.font_weight = 700; }
                        "h2" => { child_style.font_size = 24.0; child_style.font_weight = 700; }
                        "h3" => { child_style.font_size = 20.0; child_style.font_weight = 700; }
                        _ => {}
                    }
                    for child in handle.children.borrow().iter() {
                        extract_inline_recursive(child, &child_style, items);
                    }
                }
            }
        }
        _ => {
            for child in handle.children.borrow().iter() {
                extract_inline_recursive(child, style, items);
            }
        }
    }
}

fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space && !result.is_empty() {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = false;
        }
    }
    if result.ends_with(' ') { result.pop(); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_splitting() {
        assert_eq!(split_words("hello world"), vec!["hello", "world"]);
        assert_eq!(split_words("  a  b  "), vec!["a", "b"]);
        assert_eq!(split_words(""), Vec::<String>::new());
    }

    #[test]
    fn whitespace_collapse() {
        assert_eq!(collapse_whitespace("  hello   world  "), "hello world");
        assert_eq!(collapse_whitespace("\n  foo\n  bar  \n"), "foo bar");
    }

    #[test]
    fn basic_inline_layout() {
        let shaper = TextShaper::new();
        let items = vec![
            InlineItem::Text {
                text: "Hello world this is a test of line breaking in the inline layout engine".into(),
                style: InlineStyle { font_size: 16.0, line_height: 19.2, ..Default::default() },
            },
        ];
        let ctx = InlineFormattingContext::layout(&items, 200.0, TextAlign::Left, &shaper);
        assert!(ctx.line_count() > 1, "should wrap to multiple lines");
        assert!(ctx.total_height > 0.0);
        assert!(ctx.fragment_count() > 0);
    }
}
