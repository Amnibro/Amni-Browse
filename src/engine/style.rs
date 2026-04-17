use cssparser::{Parser, ParserInput, Token};
use std::collections::HashMap;
#[derive(Debug, Clone, Default)]
pub struct StyleSheet {
    pub rules: Vec<StyleRule>,
}
#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selectors: Vec<String>,
    pub declarations: Vec<Declaration>,
}
#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: String,
    pub important: bool,
}
#[derive(Debug, Clone, Default)]
pub struct ComputedStyle {
    pub display: Display,
    pub position: Position,
    pub width: Dimension,
    pub height: Dimension,
    pub margin: Edges,
    pub padding: Edges,
    pub border_width: Edges,
    pub color: Color,
    pub background_color: Color,
    pub font_size: f32,
    pub font_weight: u16,
    pub font_family: String,
    pub line_height: f32,
    pub text_align: TextAlign,
    pub overflow: Overflow,
    pub flex_direction: FlexDir,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub gap: f32,
    pub opacity: f32,
    pub z_index: i32,
    pub border_radius: f32,
    pub border_color: Color,
    pub border_style: BorderStyle,
    pub text_shadow: String,
    pub text_decoration: TextDecoration,
    pub text_transform: TextTransform,
    pub white_space: WhiteSpace,
    pub word_break: WordBreak,
    pub cursor: String,
    pub visibility: Visibility,
    pub min_width: Dimension,
    pub max_width: Dimension,
    pub min_height: Dimension,
    pub max_height: Dimension,
    pub vertical_align: VerticalAlign,
    pub list_style_type: String,
    pub outline_width: f32,
    pub outline_color: Color,
    pub outline_style: BorderStyle,
    pub transition_raw: String,
    pub transform_raw: String,
    pub animation_raw: String,
    pub filter_raw: String,
    pub background_image: String,
    pub background_size: String,
    pub background_position: String,
    pub background_repeat: String,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub text_indent: f32,
    pub flex_wrap: FlexWrap,
    pub flex_basis: Dimension,
    pub align_items: AlignItems,
    pub align_self: String,
    pub justify_content: JustifyContent,
    pub align_content: AlignContent,
    pub order: i32,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub top: Dimension,
    pub right: Dimension,
    pub bottom: Dimension,
    pub left: Dimension,
    pub props: HashMap<String, String>,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Display { #[default] Block, Inline, Flex, Grid, InlineBlock, InlineFlex, None, Contents }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Position { #[default] Static, Relative, Absolute, Fixed, Sticky }
#[derive(Debug, Clone, Default)]
pub struct Dimension { pub value: f32, pub unit: DimUnit }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum DimUnit { #[default] Auto, Px, Pct, Em, Rem, Vh, Vw }
#[derive(Debug, Clone, Default)]
pub struct Edges { pub top: f32, pub right: f32, pub bottom: f32, pub left: f32 }
#[derive(Debug, Clone, Default)]
pub struct Color { pub r: u8, pub g: u8, pub b: u8, pub a: f32 }
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TextAlign { #[default] Left, Center, Right, Justify }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Overflow { #[default] Visible, Hidden, Scroll, Auto }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum FlexDir { #[default] Row, Column, RowReverse, ColumnReverse }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum BorderStyle { #[default] None, Solid, Dashed, Dotted, Double, Groove, Ridge, Inset, Outset }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum TextDecoration { #[default] None, Underline, Overline, LineThrough }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum TextTransform { #[default] None, Uppercase, Lowercase, Capitalize }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum WhiteSpace { #[default] Normal, NoWrap, Pre, PreWrap, PreLine }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum WordBreak { #[default] Normal, BreakAll, KeepAll }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Visibility { #[default] Visible, Hidden, Collapse }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum VerticalAlign { #[default] Baseline, Top, Middle, Bottom, TextTop, TextBottom }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum FlexWrap { #[default] NoWrap, Wrap, WrapReverse }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AlignItems { #[default] Stretch, FlexStart, FlexEnd, Center, Baseline }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum JustifyContent { #[default] FlexStart, FlexEnd, Center, SpaceBetween, SpaceAround, SpaceEvenly }
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AlignContent { #[default] Stretch, FlexStart, FlexEnd, Center, SpaceBetween, SpaceAround }
impl StyleSheet {
    pub fn parse(css: &str) -> Self {
        let mut sheet = Self::default();
        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);
        while !parser.is_exhausted() {
            if let Ok(rule) = Self::parse_rule(&mut parser) { sheet.rules.push(rule); }
            else { let _ = parser.next(); }
        }
        sheet
    }
    fn parse_rule(parser: &mut Parser) -> Result<StyleRule, ()> {
        let mut selectors = Vec::new();
        let mut sel_buf = String::new();
        loop {
            let tok = parser.next().map_err(|_| ())?;
            match tok {
                Token::CurlyBracketBlock => break,
                Token::Comma => { let s = sel_buf.trim().to_string(); if !s.is_empty() { selectors.push(s); } sel_buf.clear(); }
                Token::Ident(i) => sel_buf.push_str(&i),
                Token::Delim(c) => sel_buf.push(*c),
                Token::Hash(h) | Token::IDHash(h) => { sel_buf.push('#'); sel_buf.push_str(&h); }
                Token::WhiteSpace(_) => sel_buf.push(' '),
                Token::Colon => sel_buf.push(':'),
                _ => { sel_buf.push_str(&format!("{:?}", tok).chars().take(20).collect::<String>()); }
            }
        }
        let s = sel_buf.trim().to_string();
        if !s.is_empty() { selectors.push(s); }
        let declarations = parser.parse_nested_block(|p| {
            let mut decls = Vec::new();
            while !p.is_exhausted() {
                if let Ok(d) = Self::parse_declaration(p) { decls.push(d); }
                else { let _ = p.next(); }
            }
            Ok::<_, cssparser::ParseError<'_, ()>>(decls)
        }).unwrap_or_default();
        Ok(StyleRule { selectors, declarations })
    }
    fn parse_declaration(parser: &mut Parser) -> Result<Declaration, ()> {
        let prop = match parser.next().map_err(|_| ())? {
            Token::Ident(i) => i.to_string(),
            _ => return Err(()),
        };
        match parser.next().map_err(|_| ())? {
            Token::Colon => {}
            _ => return Err(()),
        }
        let mut value = String::new();
        let mut important = false;
        loop {
            match parser.next() {
                Ok(Token::Semicolon) | Err(_) => break,
                Ok(Token::Delim('!')) => { important = true; }
                Ok(Token::Ident(ref i)) if i.eq_ignore_ascii_case("important") && important => {}
                Ok(tok) => {
                    if !value.is_empty() { value.push(' '); }
                    match &tok {
                        Token::Ident(i) => value.push_str(i),
                        Token::Number { value: n, .. } => value.push_str(&n.to_string()),
                        Token::Percentage { unit_value, .. } => { value.push_str(&(unit_value * 100.0).to_string()); value.push('%'); }
                        Token::Dimension { value: n, unit, .. } => { value.push_str(&n.to_string()); value.push_str(unit); }
                        Token::Hash(h) => { value.push('#'); value.push_str(h); }
                        Token::IDHash(h) => { value.push('#'); value.push_str(h); }
                        Token::QuotedString(s) => { value.push('"'); value.push_str(s); value.push('"'); }
                        Token::Function(f) => { value.push_str(f); value.push('('); }
                        Token::Comma => value.push(','),
                        Token::Delim(c) => value.push(*c),
                        Token::CloseParenthesis => value.push(')'),
                        _ => {}
                    }
                }
            }
        }
        Ok(Declaration { property: prop, value: value.trim().to_string(), important })
    }
}
impl ComputedStyle {
    pub fn apply_declarations(&mut self, decls: &[Declaration]) {
        for d in decls {
            self.props.insert(d.property.clone(), d.value.clone());
            match d.property.as_str() {
                "display" => self.display = match d.value.as_str() {
                    "flex" => Display::Flex, "grid" => Display::Grid, "inline" => Display::Inline,
                    "inline-block" => Display::InlineBlock, "inline-flex" => Display::InlineFlex,
                    "none" => Display::None, "contents" => Display::Contents, _ => Display::Block,
                },
                "position" => self.position = match d.value.as_str() {
                    "relative" => Position::Relative, "absolute" => Position::Absolute,
                    "fixed" => Position::Fixed, "sticky" => Position::Sticky, _ => Position::Static,
                },
                "color" => self.color = Self::parse_color(&d.value),
                "background-color" | "background" => self.background_color = Self::parse_color(&d.value),
                "font-size" => self.font_size = Self::parse_px(&d.value, 16.0),
                "font-weight" => self.font_weight = match d.value.as_str() {
                    "bold" => 700, "normal" => 400, "lighter" => 300, "bolder" => 800,
                    _ => d.value.parse().unwrap_or(400),
                },
                "font-family" => self.font_family = d.value.clone(),
                "line-height" => self.line_height = Self::parse_px(&d.value, 1.2),
                "text-align" => self.text_align = match d.value.as_str() {
                    "center" => TextAlign::Center, "right" => TextAlign::Right, "justify" => TextAlign::Justify, _ => TextAlign::Left,
                },
                "opacity" => self.opacity = d.value.parse().unwrap_or(1.0),
                "z-index" => self.z_index = d.value.parse().unwrap_or(0),
                "flex-direction" => self.flex_direction = match d.value.as_str() {
                    "column" => FlexDir::Column, "row-reverse" => FlexDir::RowReverse, "column-reverse" => FlexDir::ColumnReverse, _ => FlexDir::Row,
                },
                "flex-grow" => self.flex_grow = d.value.parse().unwrap_or(0.0),
                "flex-shrink" => self.flex_shrink = d.value.parse().unwrap_or(1.0),
                "gap" => self.gap = Self::parse_px(&d.value, 0.0),
                "width" => self.width = Self::parse_dim(&d.value),
                "height" => self.height = Self::parse_dim(&d.value),
                "border-radius" => self.border_radius = Self::parse_px(&d.value, 0.0),
                // ── margin shorthand & longhands ──
                "margin" => {
                    let (t, r, b, l) = Self::parse_edges(&d.value);
                    self.margin = Edges { top: t, right: r, bottom: b, left: l };
                }
                "margin-top" => self.margin.top = Self::parse_px(&d.value, 0.0),
                "margin-right" => self.margin.right = Self::parse_px(&d.value, 0.0),
                "margin-bottom" => self.margin.bottom = Self::parse_px(&d.value, 0.0),
                "margin-left" => self.margin.left = Self::parse_px(&d.value, 0.0),
                // ── padding shorthand & longhands ──
                "padding" => {
                    let (t, r, b, l) = Self::parse_edges(&d.value);
                    self.padding = Edges { top: t, right: r, bottom: b, left: l };
                }
                "padding-top" => self.padding.top = Self::parse_px(&d.value, 0.0),
                "padding-right" => self.padding.right = Self::parse_px(&d.value, 0.0),
                "padding-bottom" => self.padding.bottom = Self::parse_px(&d.value, 0.0),
                "padding-left" => self.padding.left = Self::parse_px(&d.value, 0.0),
                // ── border shorthand & longhands ──
                "border" => {
                    // shorthand: "1px solid #000"
                    let parts: Vec<&str> = d.value.split_whitespace().collect();
                    for part in &parts {
                        if part.starts_with('#') || part.starts_with("rgb") {
                            self.border_color = Self::parse_color(part);
                        } else if part.ends_with("px") || part.parse::<f32>().is_ok() {
                            let w = Self::parse_px(part, 0.0);
                            self.border_width = Edges { top: w, right: w, bottom: w, left: w };
                        } else {
                            self.border_style = Self::parse_border_style(part);
                        }
                    }
                }
                "border-width" => {
                    let (t, r, b, l) = Self::parse_edges(&d.value);
                    self.border_width = Edges { top: t, right: r, bottom: b, left: l };
                }
                "border-top-width" => self.border_width.top = Self::parse_px(&d.value, 0.0),
                "border-right-width" => self.border_width.right = Self::parse_px(&d.value, 0.0),
                "border-bottom-width" => self.border_width.bottom = Self::parse_px(&d.value, 0.0),
                "border-left-width" => self.border_width.left = Self::parse_px(&d.value, 0.0),
                "border-style" => self.border_style = Self::parse_border_style(&d.value),
                "border-color" => self.border_color = Self::parse_color(&d.value),
                "border-top" | "border-right" | "border-bottom" | "border-left" => {
                    let parts: Vec<&str> = d.value.split_whitespace().collect();
                    let side_w = parts.first().map(|p| Self::parse_px(p, 0.0)).unwrap_or(0.0);
                    match d.property.as_str() {
                        "border-top" => self.border_width.top = side_w,
                        "border-right" => self.border_width.right = side_w,
                        "border-bottom" => self.border_width.bottom = side_w,
                        "border-left" => self.border_width.left = side_w,
                        _ => {}
                    }
                    if let Some(style_part) = parts.get(1) {
                        self.border_style = Self::parse_border_style(style_part);
                    }
                    if let Some(color_part) = parts.get(2) {
                        self.border_color = Self::parse_color(color_part);
                    }
                }
                // ── sizing constraints ──
                "min-width" => self.min_width = Self::parse_dim(&d.value),
                "max-width" => self.max_width = Self::parse_dim(&d.value),
                "min-height" => self.min_height = Self::parse_dim(&d.value),
                "max-height" => self.max_height = Self::parse_dim(&d.value),
                // ── text properties ──
                "text-decoration" | "text-decoration-line" => self.text_decoration = match d.value.as_str() {
                    "underline" => TextDecoration::Underline,
                    "overline" => TextDecoration::Overline,
                    "line-through" => TextDecoration::LineThrough,
                    _ => TextDecoration::None,
                },
                "text-transform" => self.text_transform = match d.value.as_str() {
                    "uppercase" => TextTransform::Uppercase,
                    "lowercase" => TextTransform::Lowercase,
                    "capitalize" => TextTransform::Capitalize,
                    _ => TextTransform::None,
                },
                "white-space" => self.white_space = match d.value.as_str() {
                    "nowrap" => WhiteSpace::NoWrap,
                    "pre" => WhiteSpace::Pre,
                    "pre-wrap" => WhiteSpace::PreWrap,
                    "pre-line" => WhiteSpace::PreLine,
                    _ => WhiteSpace::Normal,
                },
                "word-break" => self.word_break = match d.value.as_str() {
                    "break-all" => WordBreak::BreakAll,
                    "keep-all" => WordBreak::KeepAll,
                    _ => WordBreak::Normal,
                },
                "letter-spacing" => self.letter_spacing = Self::parse_px(&d.value, 0.0),
                "word-spacing" => self.word_spacing = Self::parse_px(&d.value, 0.0),
                "text-indent" => self.text_indent = Self::parse_px(&d.value, 0.0),
                // ── visibility & cursor ──
                "visibility" => self.visibility = match d.value.as_str() {
                    "hidden" => Visibility::Hidden,
                    "collapse" => Visibility::Collapse,
                    _ => Visibility::Visible,
                },
                "cursor" => self.cursor = d.value.clone(),
                // ── vertical-align ──
                "vertical-align" => self.vertical_align = match d.value.as_str() {
                    "top" => VerticalAlign::Top,
                    "middle" => VerticalAlign::Middle,
                    "bottom" => VerticalAlign::Bottom,
                    "text-top" => VerticalAlign::TextTop,
                    "text-bottom" => VerticalAlign::TextBottom,
                    _ => VerticalAlign::Baseline,
                },
                // ── list styles ──
                "list-style-type" => self.list_style_type = d.value.clone(),
                "list-style" => {
                    // shorthand — store the type portion (first word)
                    self.list_style_type = d.value.split_whitespace().next().unwrap_or("disc").to_string();
                }
                // ── outline ──
                "outline" => {
                    let parts: Vec<&str> = d.value.split_whitespace().collect();
                    for part in &parts {
                        if part.starts_with('#') || part.starts_with("rgb") {
                            self.outline_color = Self::parse_color(part);
                        } else if part.ends_with("px") || part.parse::<f32>().is_ok() {
                            self.outline_width = Self::parse_px(part, 0.0);
                        } else {
                            self.outline_style = Self::parse_border_style(part);
                        }
                    }
                }
                "outline-width" => self.outline_width = Self::parse_px(&d.value, 0.0),
                "outline-color" => self.outline_color = Self::parse_color(&d.value),
                "outline-style" => self.outline_style = Self::parse_border_style(&d.value),
                // ── box-shadow / text-shadow (store raw) ──
                "box-shadow" => { self.props.insert("box-shadow".into(), d.value.clone()); }
                "text-shadow" => self.text_shadow = d.value.clone(),
                // ── transform / transition / animation / filter (store raw) ──
                "transform" => self.transform_raw = d.value.clone(),
                "transition" => self.transition_raw = d.value.clone(),
                "animation" => self.animation_raw = d.value.clone(),
                "filter" => self.filter_raw = d.value.clone(),
                // ── background longhands ──
                "background-image" => self.background_image = d.value.clone(),
                "background-size" => self.background_size = d.value.clone(),
                "background-position" => self.background_position = d.value.clone(),
                "background-repeat" => self.background_repeat = d.value.clone(),
                // ── overflow longhands ──
                "overflow" => {
                    let ov = match d.value.as_str() {
                        "hidden" => Overflow::Hidden, "scroll" => Overflow::Scroll, "auto" => Overflow::Auto, _ => Overflow::Visible,
                    };
                    self.overflow = ov.clone();
                    self.overflow_x = ov.clone();
                    self.overflow_y = ov;
                }
                "overflow-x" => self.overflow_x = match d.value.as_str() {
                    "hidden" => Overflow::Hidden, "scroll" => Overflow::Scroll, "auto" => Overflow::Auto, _ => Overflow::Visible,
                },
                "overflow-y" => self.overflow_y = match d.value.as_str() {
                    "hidden" => Overflow::Hidden, "scroll" => Overflow::Scroll, "auto" => Overflow::Auto, _ => Overflow::Visible,
                },
                // ── flexbox extended ──
                "flex-wrap" => self.flex_wrap = match d.value.as_str() {
                    "wrap" => FlexWrap::Wrap, "wrap-reverse" => FlexWrap::WrapReverse, _ => FlexWrap::NoWrap,
                },
                "flex-basis" => self.flex_basis = Self::parse_dim(&d.value),
                "flex" => {
                    // shorthand: flex: <grow> [<shrink>] [<basis>]
                    let parts: Vec<&str> = d.value.split_whitespace().collect();
                    if let Some(g) = parts.first() { self.flex_grow = g.parse().unwrap_or(0.0); }
                    if let Some(s) = parts.get(1) { self.flex_shrink = s.parse().unwrap_or(1.0); }
                    if let Some(b) = parts.get(2) { self.flex_basis = Self::parse_dim(b); }
                }
                "align-items" => self.align_items = match d.value.as_str() {
                    "flex-start" | "start" => AlignItems::FlexStart,
                    "flex-end" | "end" => AlignItems::FlexEnd,
                    "center" => AlignItems::Center,
                    "baseline" => AlignItems::Baseline,
                    _ => AlignItems::Stretch,
                },
                "align-self" => self.align_self = d.value.clone(),
                "justify-content" => self.justify_content = match d.value.as_str() {
                    "flex-end" | "end" => JustifyContent::FlexEnd,
                    "center" => JustifyContent::Center,
                    "space-between" => JustifyContent::SpaceBetween,
                    "space-around" => JustifyContent::SpaceAround,
                    "space-evenly" => JustifyContent::SpaceEvenly,
                    _ => JustifyContent::FlexStart,
                },
                "align-content" => self.align_content = match d.value.as_str() {
                    "flex-start" | "start" => AlignContent::FlexStart,
                    "flex-end" | "end" => AlignContent::FlexEnd,
                    "center" => AlignContent::Center,
                    "space-between" => AlignContent::SpaceBetween,
                    "space-around" => AlignContent::SpaceAround,
                    _ => AlignContent::Stretch,
                },
                "order" => self.order = d.value.parse().unwrap_or(0),
                // ── grid (store raw in props) ──
                "grid-template-columns" | "grid-template-rows" | "grid-column" | "grid-row"
                | "grid-gap" | "grid-row-gap" | "grid-column-gap" => {
                    self.props.insert(d.property.clone(), d.value.clone());
                }
                // ── position offsets ──
                "top" => self.top = Self::parse_dim(&d.value),
                "right" => self.right = Self::parse_dim(&d.value),
                "bottom" => self.bottom = Self::parse_dim(&d.value),
                "left" => self.left = Self::parse_dim(&d.value),
                // ── pointer-events, user-select, content ──
                "pointer-events" | "user-select" | "content" => {
                    self.props.insert(d.property.clone(), d.value.clone());
                }
                // ── intrinsic sizing keywords (store raw) ──
                "max-content" | "min-content" | "fit-content" => {
                    self.props.insert(d.property.clone(), d.value.clone());
                }
                _ => {}
            }
        }
    }
    fn parse_color(s: &str) -> Color {
        let s = s.trim();
        if s.starts_with('#') {
            let hex = &s[1..];
            let (r, g, b) = match hex.len() {
                3 => (u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0), u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0), u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0)),
                6 | 8 => (u8::from_str_radix(&hex[0..2], 16).unwrap_or(0), u8::from_str_radix(&hex[2..4], 16).unwrap_or(0), u8::from_str_radix(&hex[4..6], 16).unwrap_or(0)),
                _ => (0, 0, 0),
            };
            let a = if hex.len() == 8 { u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0 } else { 1.0 };
            Color { r, g, b, a }
        } else { Color { r: 0, g: 0, b: 0, a: 1.0 } }
    }
    fn parse_px(s: &str, default: f32) -> f32 {
        let s = s.trim().trim_end_matches("px").trim_end_matches("em").trim_end_matches("rem");
        s.parse().unwrap_or(default)
    }
    fn parse_dim(s: &str) -> Dimension {
        let s = s.trim();
        if s == "auto" { return Dimension { value: 0.0, unit: DimUnit::Auto }; }
        if s.ends_with('%') { return Dimension { value: s.trim_end_matches('%').parse().unwrap_or(0.0), unit: DimUnit::Pct }; }
        if s.ends_with("px") { return Dimension { value: s.trim_end_matches("px").parse().unwrap_or(0.0), unit: DimUnit::Px }; }
        if s.ends_with("em") { return Dimension { value: s.trim_end_matches("em").parse().unwrap_or(0.0), unit: DimUnit::Em }; }
        if s.ends_with("rem") { return Dimension { value: s.trim_end_matches("rem").parse().unwrap_or(0.0), unit: DimUnit::Rem }; }
        if s.ends_with("vh") { return Dimension { value: s.trim_end_matches("vh").parse().unwrap_or(0.0), unit: DimUnit::Vh }; }
        if s.ends_with("vw") { return Dimension { value: s.trim_end_matches("vw").parse().unwrap_or(0.0), unit: DimUnit::Vw }; }
        Dimension { value: s.parse().unwrap_or(0.0), unit: DimUnit::Px }
    }
    fn parse_edges(s: &str) -> (f32, f32, f32, f32) {
        let parts: Vec<f32> = s.split_whitespace()
            .map(|p| Self::parse_px(p, 0.0))
            .collect();
        match parts.len() {
            1 => (parts[0], parts[0], parts[0], parts[0]),
            2 => (parts[0], parts[1], parts[0], parts[1]),
            3 => (parts[0], parts[1], parts[2], parts[1]),
            4 => (parts[0], parts[1], parts[2], parts[3]),
            _ => (0.0, 0.0, 0.0, 0.0),
        }
    }
    fn parse_border_style(s: &str) -> BorderStyle {
        match s.trim() {
            "solid" => BorderStyle::Solid,
            "dashed" => BorderStyle::Dashed,
            "dotted" => BorderStyle::Dotted,
            "double" => BorderStyle::Double,
            "groove" => BorderStyle::Groove,
            "ridge" => BorderStyle::Ridge,
            "inset" => BorderStyle::Inset,
            "outset" => BorderStyle::Outset,
            _ => BorderStyle::None,
        }
    }
}
