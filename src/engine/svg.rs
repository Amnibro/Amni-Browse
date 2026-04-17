use crate::engine::paint::{PaintCommand, PaintRect};
use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq)]
pub enum SvgElement {
    Rect, Circle, Ellipse, Line, Polyline, Polygon, Path, Text,
    Group, Use, Defs, ClipPath, LinearGradient, RadialGradient, Image,
}
#[derive(Debug, Clone)]
pub struct SvgNode {
    pub element: SvgElement,
    pub attrs: HashMap<String, String>,
    pub children: Vec<SvgNode>,
    pub transform: Option<String>,
    pub text_content: String,
}
impl SvgNode {
    pub fn new(element: SvgElement) -> Self {
        Self { element, attrs: HashMap::new(), children: Vec::new(), transform: None, text_content: String::new() }
    }
    fn attr_f32(&self, name: &str, default: f32) -> f32 {
        self.attrs.get(name).and_then(|v| v.parse::<f32>().ok()).unwrap_or(default)
    }
    fn attr_str(&self, name: &str) -> &str {
        self.attrs.get(name).map(|s| s.as_str()).unwrap_or("")
    }
}
#[derive(Debug, Clone)]
pub struct SvgDocument {
    pub root: SvgNode,
    pub width: f32,
    pub height: f32,
    pub view_box: (f32, f32, f32, f32),
    pub defs: HashMap<String, SvgNode>,
}
#[derive(Debug, Clone)]
pub enum PathSegment {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    HLineTo(f32),
    VLineTo(f32),
    CubicBezier(f32, f32, f32, f32, f32, f32),
    SmoothCubic(f32, f32, f32, f32),
    QuadBezier(f32, f32, f32, f32),
    SmoothQuad(f32, f32),
    Arc(f32, f32, f32, bool, bool, f32, f32),
    Close,
}
fn tag_to_element(tag: &str) -> Option<SvgElement> {
    match tag {
        "rect" => Some(SvgElement::Rect),
        "circle" => Some(SvgElement::Circle),
        "ellipse" => Some(SvgElement::Ellipse),
        "line" => Some(SvgElement::Line),
        "polyline" => Some(SvgElement::Polyline),
        "polygon" => Some(SvgElement::Polygon),
        "path" => Some(SvgElement::Path),
        "text" => Some(SvgElement::Text),
        "g" => Some(SvgElement::Group),
        "use" => Some(SvgElement::Use),
        "defs" => Some(SvgElement::Defs),
        "clipPath" | "clippath" => Some(SvgElement::ClipPath),
        "linearGradient" | "lineargradient" => Some(SvgElement::LinearGradient),
        "radialGradient" | "radialgradient" => Some(SvgElement::RadialGradient),
        "image" => Some(SvgElement::Image),
        "svg" => Some(SvgElement::Group),
        _ => None,
    }
}
struct XmlToken {
    tag: String,
    attrs: HashMap<String, String>,
    self_closing: bool,
    is_close: bool,
    text: String,
}
fn tokenize_xml(xml: &str) -> Vec<XmlToken> {
    let mut tokens = Vec::new();
    let mut chars = xml.char_indices().peekable();
    let mut text_start = 0;
    while let Some(&(i, ch)) = chars.peek() {
        if ch == '<' {
            let text_slice = xml[text_start..i].trim();
            if !text_slice.is_empty() {
                tokens.push(XmlToken { tag: String::new(), attrs: HashMap::new(), self_closing: false, is_close: false, text: text_slice.to_string() });
            }
            chars.next();
            if chars.peek().map(|&(_, c)| c) == Some('!') || chars.peek().map(|&(_, c)| c) == Some('?') {
                while let Some(&(_, c)) = chars.peek() {
                    chars.next();
                    if c == '>' { break; }
                }
                text_start = chars.peek().map(|&(i, _)| i).unwrap_or(xml.len());
                continue;
            }
            let is_close = chars.peek().map(|&(_, c)| c) == Some('/');
            if is_close { chars.next(); }
            let mut tag = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_whitespace() || c == '>' || c == '/' { break; }
                tag.push(c);
                chars.next();
            }
            let mut attrs = HashMap::new();
            loop {
                while chars.peek().map(|&(_, c)| c.is_whitespace()).unwrap_or(false) { chars.next(); }
                if chars.peek().map(|&(_, c)| c == '>' || c == '/').unwrap_or(true) { break; }
                let mut attr_name = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c == '=' || c.is_whitespace() || c == '>' || c == '/' { break; }
                    attr_name.push(c);
                    chars.next();
                }
                while chars.peek().map(|&(_, c)| c.is_whitespace()).unwrap_or(false) { chars.next(); }
                if chars.peek().map(|&(_, c)| c) == Some('=') {
                    chars.next();
                    while chars.peek().map(|&(_, c)| c.is_whitespace()).unwrap_or(false) { chars.next(); }
                    let quote = chars.peek().map(|&(_, c)| c).unwrap_or('"');
                    let has_quote = quote == '"' || quote == '\'';
                    if has_quote { chars.next(); }
                    let mut val = String::new();
                    while let Some(&(_, c)) = chars.peek() {
                        if has_quote && c == quote { chars.next(); break; }
                        if !has_quote && (c.is_whitespace() || c == '>' || c == '/') { break; }
                        val.push(c);
                        chars.next();
                    }
                    attrs.insert(attr_name, val);
                } else if !attr_name.is_empty() {
                    attrs.insert(attr_name, String::new());
                }
            }
            let mut self_closing = false;
            if chars.peek().map(|&(_, c)| c) == Some('/') { self_closing = true; chars.next(); }
            if chars.peek().map(|&(_, c)| c) == Some('>') { chars.next(); }
            text_start = chars.peek().map(|&(i, _)| i).unwrap_or(xml.len());
            tokens.push(XmlToken { tag, attrs, self_closing, is_close, text: String::new() });
        } else {
            chars.next();
        }
    }
    let trailing = xml[text_start..].trim();
    if !trailing.is_empty() {
        tokens.push(XmlToken { tag: String::new(), attrs: HashMap::new(), self_closing: false, is_close: false, text: trailing.to_string() });
    }
    tokens
}
pub fn parse_svg(xml: &str) -> Result<SvgDocument, String> {
    let tokens = tokenize_xml(xml);
    let mut stack: Vec<SvgNode> = Vec::new();
    let mut root: Option<SvgNode> = None;
    let mut svg_width = 300.0f32;
    let mut svg_height = 150.0f32;
    let mut view_box = (0.0f32, 0.0f32, 300.0f32, 150.0f32);
    for token in &tokens {
        if !token.text.is_empty() {
            if let Some(parent) = stack.last_mut() {
                parent.text_content.push_str(&token.text);
            }
            continue;
        }
        if token.is_close {
            if let Some(node) = stack.pop() {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else { root = Some(node); }
            }
            continue;
        }
        let tag = token.tag.as_str();
        if tag == "svg" {
            if let Some(w) = token.attrs.get("width") { svg_width = parse_dimension(w); }
            if let Some(h) = token.attrs.get("height") { svg_height = parse_dimension(h); }
            if let Some(vb) = token.attrs.get("viewBox").or_else(|| token.attrs.get("viewbox")) {
                let parts: Vec<f32> = vb.split(|c: char| c == ',' || c.is_whitespace())
                    .filter_map(|s| s.trim().parse().ok()).collect();
                if parts.len() >= 4 { view_box = (parts[0], parts[1], parts[2], parts[3]); }
            }
        }
        let element = tag_to_element(tag).unwrap_or(SvgElement::Group);
        let mut node = SvgNode::new(element);
        node.attrs = token.attrs.clone();
        node.transform = token.attrs.get("transform").cloned();
        if token.self_closing {
            if let Some(parent) = stack.last_mut() {
                parent.children.push(node);
            } else { root = Some(node); }
        } else { stack.push(node); }
    }
    while let Some(node) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        } else { root = Some(node); }
    }
    let root = root.unwrap_or_else(|| SvgNode::new(SvgElement::Group));
    let mut defs = HashMap::new();
    collect_defs(&root, &mut defs);
    Ok(SvgDocument { root, width: svg_width, height: svg_height, view_box, defs })
}
fn parse_dimension(s: &str) -> f32 {
    let s = s.trim().trim_end_matches("px").trim_end_matches("pt").trim_end_matches("em").trim_end_matches('%');
    s.parse::<f32>().unwrap_or(0.0)
}
fn collect_defs(node: &SvgNode, defs: &mut HashMap<String, SvgNode>) {
    if let Some(id) = node.attrs.get("id") { defs.insert(id.clone(), node.clone()); }
    for child in &node.children { collect_defs(child, defs); }
}
pub fn render_to_commands(doc: &SvgDocument) -> Vec<PaintCommand> {
    let mut commands = Vec::new();
    let sx = doc.width / doc.view_box.2;
    let sy = doc.height / doc.view_box.3;
    let ctx = RenderCtx { scale_x: sx, scale_y: sy, offset_x: -doc.view_box.0 * sx, offset_y: -doc.view_box.1 * sy, defs: &doc.defs };
    render_node(&doc.root, &ctx, &mut commands);
    commands
}
struct RenderCtx<'a> {
    scale_x: f32,
    scale_y: f32,
    offset_x: f32,
    offset_y: f32,
    defs: &'a HashMap<String, SvgNode>,
}
impl<'a> RenderCtx<'a> {
    fn tx(&self, x: f32) -> f32 { x * self.scale_x + self.offset_x }
    fn ty(&self, y: f32) -> f32 { y * self.scale_y + self.offset_y }
    fn tw(&self, w: f32) -> f32 { w * self.scale_x }
    fn th(&self, h: f32) -> f32 { h * self.scale_y }
}
fn render_node(node: &SvgNode, ctx: &RenderCtx, commands: &mut Vec<PaintCommand>) {
    match node.element {
        SvgElement::Defs | SvgElement::ClipPath | SvgElement::LinearGradient | SvgElement::RadialGradient => return,
        _ => {}
    }
    let fill_str = node.attr_str("fill");
    let fill_color = if fill_str == "none" { None } else if fill_str.is_empty() { Some([0, 0, 0, 255]) } else { Some(parse_svg_color(fill_str)) };
    let stroke_str = node.attr_str("stroke");
    let stroke_color = if stroke_str.is_empty() || stroke_str == "none" { None } else { Some(parse_svg_color(stroke_str)) };
    let stroke_width = node.attr_f32("stroke-width", 1.0);
    let opacity = node.attr_f32("opacity", 1.0);
    let apply_opacity = |mut c: [u8; 4]| -> [u8; 4] { c[3] = (c[3] as f32 * opacity) as u8; c };
    match node.element {
        SvgElement::Rect => {
            let x = node.attr_f32("x", 0.0);
            let y = node.attr_f32("y", 0.0);
            let w = node.attr_f32("width", 0.0);
            let h = node.attr_f32("height", 0.0);
            if let Some(fc) = fill_color {
                commands.push(PaintCommand::FillRect {
                    rect: PaintRect { x: ctx.tx(x), y: ctx.ty(y), w: ctx.tw(w), h: ctx.th(h) },
                    color: apply_opacity(fc),
                });
            }
            if let Some(sc) = stroke_color {
                let sw = stroke_width * ctx.scale_x;
                let rx = ctx.tx(x); let ry = ctx.ty(y);
                let rw = ctx.tw(w); let rh = ctx.th(h);
                let c = apply_opacity(sc);
                commands.push(PaintCommand::FillRect { rect: PaintRect { x: rx, y: ry, w: rw, h: sw }, color: c });
                commands.push(PaintCommand::FillRect { rect: PaintRect { x: rx, y: ry + rh - sw, w: rw, h: sw }, color: c });
                commands.push(PaintCommand::FillRect { rect: PaintRect { x: rx, y: ry, w: sw, h: rh }, color: c });
                commands.push(PaintCommand::FillRect { rect: PaintRect { x: rx + rw - sw, y: ry, w: sw, h: rh }, color: c });
            }
        }
        SvgElement::Circle => {
            let cx = node.attr_f32("cx", 0.0);
            let cy = node.attr_f32("cy", 0.0);
            let r = node.attr_f32("r", 0.0);
            if let Some(fc) = fill_color {
                render_ellipse_fill(ctx.tx(cx), ctx.ty(cy), ctx.tw(r), ctx.th(r), apply_opacity(fc), commands);
            }
        }
        SvgElement::Ellipse => {
            let cx = node.attr_f32("cx", 0.0);
            let cy = node.attr_f32("cy", 0.0);
            let rx = node.attr_f32("rx", 0.0);
            let ry = node.attr_f32("ry", 0.0);
            if let Some(fc) = fill_color {
                render_ellipse_fill(ctx.tx(cx), ctx.ty(cy), ctx.tw(rx), ctx.th(ry), apply_opacity(fc), commands);
            }
        }
        SvgElement::Line => {
            let x1 = node.attr_f32("x1", 0.0);
            let y1 = node.attr_f32("y1", 0.0);
            let x2 = node.attr_f32("x2", 0.0);
            let y2 = node.attr_f32("y2", 0.0);
            if let Some(sc) = stroke_color {
                let sw = (stroke_width * ctx.scale_x).max(1.0);
                render_line(ctx.tx(x1), ctx.ty(y1), ctx.tx(x2), ctx.ty(y2), sw, apply_opacity(sc), commands);
            }
        }
        SvgElement::Polyline | SvgElement::Polygon => {
            let points_str = node.attr_str("points");
            let pts = parse_point_list(points_str);
            if pts.len() >= 2 {
                if node.element == SvgElement::Polygon {
                    if let Some(fc) = fill_color {
                        render_polygon_fill(&pts, ctx, apply_opacity(fc), commands);
                    }
                }
                if let Some(sc) = stroke_color {
                    let sw = (stroke_width * ctx.scale_x).max(1.0);
                    let c = apply_opacity(sc);
                    for i in 0..pts.len() - 1 {
                        render_line(ctx.tx(pts[i].0), ctx.ty(pts[i].1), ctx.tx(pts[i+1].0), ctx.ty(pts[i+1].1), sw, c, commands);
                    }
                    if node.element == SvgElement::Polygon && pts.len() > 2 {
                        let last = pts.len() - 1;
                        render_line(ctx.tx(pts[last].0), ctx.ty(pts[last].1), ctx.tx(pts[0].0), ctx.ty(pts[0].1), sw, c, commands);
                    }
                }
            }
        }
        SvgElement::Path => {
            let d = node.attr_str("d");
            if !d.is_empty() {
                let segments = parse_path_data(d);
                render_path_segments(&segments, ctx, fill_color.map(apply_opacity), stroke_color.map(apply_opacity), stroke_width, commands);
            }
        }
        SvgElement::Text => {
            let x = node.attr_f32("x", 0.0);
            let y = node.attr_f32("y", 0.0);
            let fs = node.attr_f32("font-size", 16.0);
            let color = fill_color.map(apply_opacity).unwrap_or([0, 0, 0, 255]);
            let text = node.text_content.trim();
            if !text.is_empty() {
                commands.push(PaintCommand::DrawText {
                    x: ctx.tx(x), y: ctx.ty(y), text: text.to_string(),
                    font_size: fs * ctx.scale_y, color, max_width: 10000.0,
                });
            }
        }
        SvgElement::Use => {
            let href = node.attrs.get("href").or_else(|| node.attrs.get("xlink:href")).cloned().unwrap_or_default();
            let id = href.strip_prefix('#').unwrap_or(&href);
            if let Some(referenced) = ctx.defs.get(id) {
                render_node(referenced, ctx, commands);
            }
        }
        SvgElement::Image => {
            let x = node.attr_f32("x", 0.0);
            let y = node.attr_f32("y", 0.0);
            let w = node.attr_f32("width", 0.0);
            let h = node.attr_f32("height", 0.0);
            commands.push(PaintCommand::DrawImage {
                rect: PaintRect { x: ctx.tx(x), y: ctx.ty(y), w: ctx.tw(w), h: ctx.th(h) },
                image_data: None, img_w: 0, img_h: 0,
            });
        }
        SvgElement::Group => {}
        _ => {}
    }
    for child in &node.children { render_node(child, ctx, commands); }
}
fn render_ellipse_fill(cx: f32, cy: f32, rx: f32, ry: f32, color: [u8; 4], commands: &mut Vec<PaintCommand>) {
    let steps = ((rx.max(ry) * 4.0) as i32).clamp(8, 128);
    let y_start = (cy - ry).max(0.0) as i32;
    let y_end = (cy + ry) as i32 + 1;
    for y in y_start..y_end {
        let dy = y as f32 + 0.5 - cy;
        if ry.abs() < 0.001 { continue; }
        let ratio = dy / ry;
        if ratio.abs() > 1.0 { continue; }
        let half_w = rx * (1.0 - ratio * ratio).sqrt();
        let x0 = cx - half_w;
        let _ = steps;
        commands.push(PaintCommand::FillRect {
            rect: PaintRect { x: x0, y: y as f32, w: half_w * 2.0, h: 1.0 },
            color,
        });
    }
}
fn render_line(x0: f32, y0: f32, x1: f32, y1: f32, width: f32, color: [u8; 4], commands: &mut Vec<PaintCommand>) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 { return; }
    if dy.abs() < 0.5 {
        let min_x = x0.min(x1);
        let min_y = y0.min(y1) - width * 0.5;
        commands.push(PaintCommand::FillRect {
            rect: PaintRect { x: min_x, y: min_y, w: len, h: width },
            color,
        });
        return;
    }
    if dx.abs() < 0.5 {
        let min_x = x0.min(x1) - width * 0.5;
        let min_y = y0.min(y1);
        commands.push(PaintCommand::FillRect {
            rect: PaintRect { x: min_x, y: min_y, w: width, h: len },
            color,
        });
        return;
    }
    let steps = len.ceil() as i32;
    let step_w = width.max(1.0);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let px = x0 + dx * t;
        let py = y0 + dy * t;
        commands.push(PaintCommand::FillRect {
            rect: PaintRect { x: px - step_w * 0.5, y: py - step_w * 0.5, w: step_w, h: step_w },
            color,
        });
    }
}
fn parse_point_list(s: &str) -> Vec<(f32, f32)> {
    let nums: Vec<f32> = s.split(|c: char| c == ',' || c.is_whitespace())
        .filter_map(|p| p.trim().parse().ok()).collect();
    nums.chunks(2).filter_map(|c| if c.len() == 2 { Some((c[0], c[1])) } else { None }).collect()
}
fn render_polygon_fill(pts: &[(f32, f32)], ctx: &RenderCtx, color: [u8; 4], commands: &mut Vec<PaintCommand>) {
    if pts.len() < 3 { return; }
    let transformed: Vec<(f32, f32)> = pts.iter().map(|&(x, y)| (ctx.tx(x), ctx.ty(y))).collect();
    let min_y = transformed.iter().map(|p| p.1).fold(f32::MAX, f32::min);
    let max_y = transformed.iter().map(|p| p.1).fold(f32::MIN, f32::max);
    let min_x = transformed.iter().map(|p| p.0).fold(f32::MAX, f32::min);
    let yi = min_y as i32;
    let ye = max_y as i32 + 1;
    for y in yi..ye {
        let yf = y as f32 + 0.5;
        let mut xs = Vec::new();
        let n = transformed.len();
        for i in 0..n {
            let (x0, y0) = transformed[i];
            let (x1, y1) = transformed[(i + 1) % n];
            if (y0 <= yf && y1 > yf) || (y1 <= yf && y0 > yf) {
                let t = (yf - y0) / (y1 - y0);
                xs.push(x0 + t * (x1 - x0));
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut i = 0;
        while i + 1 < xs.len() {
            let _ = min_x;
            commands.push(PaintCommand::FillRect {
                rect: PaintRect { x: xs[i], y: y as f32, w: xs[i+1] - xs[i], h: 1.0 },
                color,
            });
            i += 2;
        }
    }
}
pub fn parse_path_data(d: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut chars = d.chars().peekable();
    let mut current_cmd = ' ';
    let parse_num = |chars: &mut std::iter::Peekable<std::str::Chars>| -> Option<f32> {
        while chars.peek().map(|c| *c == ',' || c.is_whitespace()).unwrap_or(false) { chars.next(); }
        let mut s = String::new();
        if chars.peek() == Some(&'-') || chars.peek() == Some(&'+') { s.push(chars.next().unwrap()); }
        let mut has_dot = false;
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() { s.push(c); chars.next(); }
            else if c == '.' && !has_dot { has_dot = true; s.push(c); chars.next(); }
            else if c == 'e' || c == 'E' {
                s.push(c); chars.next();
                if chars.peek() == Some(&'-') || chars.peek() == Some(&'+') { s.push(chars.next().unwrap()); }
            }
            else { break; }
        }
        if s.is_empty() || s == "-" || s == "+" { None } else { s.parse().ok() }
    };
    let parse_flag = |chars: &mut std::iter::Peekable<std::str::Chars>| -> Option<bool> {
        while chars.peek().map(|c| *c == ',' || c.is_whitespace()).unwrap_or(false) { chars.next(); }
        match chars.next() {
            Some('0') => Some(false),
            Some('1') => Some(true),
            _ => None,
        }
    };
    while chars.peek().is_some() {
        while chars.peek().map(|c| c.is_whitespace() || *c == ',').unwrap_or(false) { chars.next(); }
        if let Some(&c) = chars.peek() {
            if c.is_ascii_alphabetic() { current_cmd = c; chars.next(); }
        } else { break; }
        match current_cmd {
            'M' => { if let (Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::MoveTo(x, y)); current_cmd = 'L'; } }
            'm' => { if let (Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::MoveTo(x, y)); current_cmd = 'l'; } }
            'L' => { if let (Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::LineTo(x, y)); } }
            'l' => { if let (Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::LineTo(x, y)); } }
            'H' => { if let Some(x) = parse_num(&mut chars) { segments.push(PathSegment::HLineTo(x)); } }
            'h' => { if let Some(x) = parse_num(&mut chars) { segments.push(PathSegment::HLineTo(x)); } }
            'V' => { if let Some(y) = parse_num(&mut chars) { segments.push(PathSegment::VLineTo(y)); } }
            'v' => { if let Some(y) = parse_num(&mut chars) { segments.push(PathSegment::VLineTo(y)); } }
            'C' => { if let (Some(x1), Some(y1), Some(x2), Some(y2), Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::CubicBezier(x1, y1, x2, y2, x, y)); } }
            'c' => { if let (Some(x1), Some(y1), Some(x2), Some(y2), Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::CubicBezier(x1, y1, x2, y2, x, y)); } }
            'S' => { if let (Some(x2), Some(y2), Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::SmoothCubic(x2, y2, x, y)); } }
            's' => { if let (Some(x2), Some(y2), Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::SmoothCubic(x2, y2, x, y)); } }
            'Q' => { if let (Some(x1), Some(y1), Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::QuadBezier(x1, y1, x, y)); } }
            'q' => { if let (Some(x1), Some(y1), Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::QuadBezier(x1, y1, x, y)); } }
            'T' => { if let (Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::SmoothQuad(x, y)); } }
            't' => { if let (Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars)) { segments.push(PathSegment::SmoothQuad(x, y)); } }
            'A' => {
                if let (Some(rx), Some(ry), Some(rot)) = (parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars)) {
                    if let (Some(large), Some(sweep)) = (parse_flag(&mut chars), parse_flag(&mut chars)) {
                        if let (Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars)) {
                            segments.push(PathSegment::Arc(rx, ry, rot, large, sweep, x, y));
                        }
                    }
                }
            }
            'a' => {
                if let (Some(rx), Some(ry), Some(rot)) = (parse_num(&mut chars), parse_num(&mut chars), parse_num(&mut chars)) {
                    if let (Some(large), Some(sweep)) = (parse_flag(&mut chars), parse_flag(&mut chars)) {
                        if let (Some(x), Some(y)) = (parse_num(&mut chars), parse_num(&mut chars)) {
                            segments.push(PathSegment::Arc(rx, ry, rot, large, sweep, x, y));
                        }
                    }
                }
            }
            'Z' | 'z' => { segments.push(PathSegment::Close); }
            _ => { chars.next(); }
        }
    }
    segments
}
fn render_path_segments(segments: &[PathSegment], ctx: &RenderCtx, fill: Option<[u8; 4]>, stroke: Option<[u8; 4]>, stroke_width: f32, commands: &mut Vec<PaintCommand>) {
    let mut points: Vec<(f32, f32)> = Vec::new();
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut start_x = 0.0f32;
    let mut start_y = 0.0f32;
    for seg in segments {
        match *seg {
            PathSegment::MoveTo(x, y) => { cx = x; cy = y; start_x = x; start_y = y; points.push((ctx.tx(x), ctx.ty(y))); }
            PathSegment::LineTo(x, y) => { cx = x; cy = y; points.push((ctx.tx(x), ctx.ty(y))); }
            PathSegment::HLineTo(x) => { cx = x; points.push((ctx.tx(x), ctx.ty(cy))); }
            PathSegment::VLineTo(y) => { cy = y; points.push((ctx.tx(cx), ctx.ty(y))); }
            PathSegment::CubicBezier(x1, y1, x2, y2, x, y) => {
                let steps = 24;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let it = 1.0 - t;
                    let px = it*it*it*cx + 3.0*it*it*t*x1 + 3.0*it*t*t*x2 + t*t*t*x;
                    let py = it*it*it*cy + 3.0*it*it*t*y1 + 3.0*it*t*t*y2 + t*t*t*y;
                    points.push((ctx.tx(px), ctx.ty(py)));
                }
                cx = x; cy = y;
            }
            PathSegment::SmoothCubic(x2, y2, x, y) => {
                let steps = 24;
                let x1 = cx; let y1 = cy;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let it = 1.0 - t;
                    let px = it*it*it*cx + 3.0*it*it*t*x1 + 3.0*it*t*t*x2 + t*t*t*x;
                    let py = it*it*it*cy + 3.0*it*it*t*y1 + 3.0*it*t*t*y2 + t*t*t*y;
                    points.push((ctx.tx(px), ctx.ty(py)));
                }
                cx = x; cy = y;
            }
            PathSegment::QuadBezier(x1, y1, x, y) => {
                let steps = 16;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let it = 1.0 - t;
                    let px = it*it*cx + 2.0*it*t*x1 + t*t*x;
                    let py = it*it*cy + 2.0*it*t*y1 + t*t*y;
                    points.push((ctx.tx(px), ctx.ty(py)));
                }
                cx = x; cy = y;
            }
            PathSegment::SmoothQuad(x, y) => {
                let steps = 16;
                let x1 = cx; let y1 = cy;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let it = 1.0 - t;
                    let px = it*it*cx + 2.0*it*t*x1 + t*t*x;
                    let py = it*it*cy + 2.0*it*t*y1 + t*t*y;
                    points.push((ctx.tx(px), ctx.ty(py)));
                }
                cx = x; cy = y;
            }
            PathSegment::Arc(rx, ry, _rot, _large, _sweep, x, y) => {
                let steps = 24;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let px = cx + (x - cx) * t;
                    let py = cy + (y - cy) * t;
                    let bulge = (std::f32::consts::PI * t).sin() * rx.min(ry) * 0.3;
                    let dx = x - cx; let dy = y - cy;
                    let len = (dx*dx + dy*dy).sqrt().max(0.001);
                    let nx = -dy / len; let ny = dx / len;
                    points.push((ctx.tx(px + nx * bulge), ctx.ty(py + ny * bulge)));
                }
                cx = x; cy = y;
            }
            PathSegment::Close => {
                cx = start_x; cy = start_y;
                points.push((ctx.tx(start_x), ctx.ty(start_y)));
            }
        }
    }
    if let Some(fc) = fill {
        if points.len() >= 3 {
            let min_y = points.iter().map(|p| p.1).fold(f32::MAX, f32::min);
            let max_y = points.iter().map(|p| p.1).fold(f32::MIN, f32::max);
            let yi = min_y as i32;
            let ye = max_y as i32 + 1;
            for y in yi..ye {
                let yf = y as f32 + 0.5;
                let mut xs = Vec::new();
                let n = points.len();
                for i in 0..n {
                    let (x0, y0) = points[i];
                    let (x1, y1) = points[(i + 1) % n];
                    if (y0 <= yf && y1 > yf) || (y1 <= yf && y0 > yf) {
                        let t = (yf - y0) / (y1 - y0);
                        xs.push(x0 + t * (x1 - x0));
                    }
                }
                xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mut i = 0;
                while i + 1 < xs.len() {
                    commands.push(PaintCommand::FillRect {
                        rect: PaintRect { x: xs[i], y: y as f32, w: xs[i+1] - xs[i], h: 1.0 },
                        color: fc,
                    });
                    i += 2;
                }
            }
        }
    }
    if let Some(sc) = stroke {
        let sw = (stroke_width * ctx.scale_x).max(1.0);
        for i in 0..points.len().saturating_sub(1) {
            render_line(points[i].0, points[i].1, points[i+1].0, points[i+1].1, sw, sc, commands);
        }
    }
}
pub fn parse_svg_color(s: &str) -> [u8; 4] {
    let s = s.trim();
    if s.starts_with('#') {
        let hex = &s[1..];
        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0);
            return [r * 17, g * 17, b * 17, 255];
        } else if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            return [r, g, b, 255];
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
            return [r, g, b, a];
        }
    }
    if s.starts_with("rgba(") && s.ends_with(')') {
        let inner = &s[5..s.len()-1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4 {
            let r = parts[0].trim().parse::<u8>().unwrap_or(0);
            let g = parts[1].trim().parse::<u8>().unwrap_or(0);
            let b = parts[2].trim().parse::<u8>().unwrap_or(0);
            let a = parts[3].trim().parse::<f32>().unwrap_or(1.0);
            return [r, g, b, (a * 255.0) as u8];
        }
    }
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len()-1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<u8>().unwrap_or(0);
            let g = parts[1].trim().parse::<u8>().unwrap_or(0);
            let b = parts[2].trim().parse::<u8>().unwrap_or(0);
            return [r, g, b, 255];
        }
    }
    match s.to_lowercase().as_str() {
        "black" => [0, 0, 0, 255],
        "silver" => [192, 192, 192, 255],
        "gray" | "grey" => [128, 128, 128, 255],
        "white" => [255, 255, 255, 255],
        "maroon" => [128, 0, 0, 255],
        "red" => [255, 0, 0, 255],
        "purple" => [128, 0, 128, 255],
        "fuchsia" | "magenta" => [255, 0, 255, 255],
        "green" => [0, 128, 0, 255],
        "lime" => [0, 255, 0, 255],
        "olive" => [128, 128, 0, 255],
        "yellow" => [255, 255, 0, 255],
        "navy" => [0, 0, 128, 255],
        "blue" => [0, 0, 255, 255],
        "teal" => [0, 128, 128, 255],
        "aqua" | "cyan" => [0, 255, 255, 255],
        "orange" => [255, 165, 0, 255],
        "transparent" => [0, 0, 0, 0],
        _ => [0, 0, 0, 255],
    }
}
pub fn parse_svg_transform(s: &str) -> [f32; 6] {
    let mut result = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let s = s.trim();
    let mut pos = 0;
    while pos < s.len() {
        let rest = &s[pos..];
        let (func, args_start) = if let Some(idx) = rest.find('(') {
            (&rest[..idx].trim(), idx + 1)
        } else { break };
        let args_end = rest[args_start..].find(')').map(|i| args_start + i).unwrap_or(rest.len());
        let args_str = &rest[args_start..args_end];
        let args: Vec<f32> = args_str.split(|c: char| c == ',' || c.is_whitespace())
            .filter_map(|p| p.trim().parse().ok()).collect();
        pos += args_end + 1;
        while pos < s.len() && (s.as_bytes()[pos] == b',' || s.as_bytes()[pos] == b' ') { pos += 1; }
        let mat = match *func {
            "translate" => {
                let tx = args.first().copied().unwrap_or(0.0);
                let ty = args.get(1).copied().unwrap_or(0.0);
                [1.0, 0.0, 0.0, 1.0, tx, ty]
            }
            "scale" => {
                let sx = args.first().copied().unwrap_or(1.0);
                let sy = args.get(1).copied().unwrap_or(sx);
                [sx, 0.0, 0.0, sy, 0.0, 0.0]
            }
            "rotate" => {
                let angle = args.first().copied().unwrap_or(0.0) * std::f32::consts::PI / 180.0;
                let cos = angle.cos();
                let sin = angle.sin();
                if args.len() >= 3 {
                    let cx = args[1]; let cy = args[2];
                    [cos, sin, -sin, cos, cx - cos * cx + sin * cy, cy - sin * cx - cos * cy + cy * 2.0 - cy]
                } else { [cos, sin, -sin, cos, 0.0, 0.0] }
            }
            "skewX" => {
                let angle = args.first().copied().unwrap_or(0.0) * std::f32::consts::PI / 180.0;
                [1.0, 0.0, angle.tan(), 1.0, 0.0, 0.0]
            }
            "skewY" => {
                let angle = args.first().copied().unwrap_or(0.0) * std::f32::consts::PI / 180.0;
                [1.0, angle.tan(), 0.0, 1.0, 0.0, 0.0]
            }
            "matrix" if args.len() >= 6 => { [args[0], args[1], args[2], args[3], args[4], args[5]] }
            _ => [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        };
        result = multiply_matrices(result, mat);
    }
    result
}
fn multiply_matrices(a: [f32; 6], b: [f32; 6]) -> [f32; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}
