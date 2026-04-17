use super::layout::LayoutRect;
use super::style::{ComputedStyle, Color};
use std::collections::HashMap;
use markup5ever_rcdom::{Handle, NodeData};

#[derive(Debug, Clone)]
pub enum PaintCommand {
    FillRect { rect: PaintRect, color: [u8; 4] },
    DrawText { x: f32, y: f32, text: String, font_size: f32, color: [u8; 4], max_width: f32 },
    DrawImage { rect: PaintRect, image_data: Option<Vec<u8>>, img_w: u32, img_h: u32 },
    PushClip { rect: PaintRect },
    PopClip,
    DrawCanvas { rect: PaintRect, canvas_id: usize },
    DrawSvg { rect: PaintRect, svg_content: String },
    DrawVideo { rect: PaintRect, video_id: usize },
    DrawAudio { rect: PaintRect, audio_id: usize },
    DrawIframe { rect: PaintRect, src: String },
}

#[derive(Debug, Clone, Default)]
pub struct PaintRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<&LayoutRect> for PaintRect {
    fn from(lr: &LayoutRect) -> Self {
        PaintRect { x: lr.x, y: lr.y, w: lr.w, h: lr.h }
    }
}

#[derive(Debug, Clone)]
pub struct DisplayList {
    pub commands: Vec<PaintCommand>,
    pub width: u32,
    pub height: u32,
}

impl DisplayList {
    pub fn new(w: u32, h: u32) -> Self { Self { commands: Vec::new(), width: w, height: h } }
    pub fn push(&mut self, cmd: PaintCommand) { self.commands.push(cmd); }
}

pub struct RenderNode {
    pub id: usize,
    pub tag: String,
    pub text: String,
    pub image_src: String,
    pub iframe_src: String,
    pub style: ComputedStyle,
    pub children: Vec<usize>,
}

pub struct RenderTree {
    pub nodes: HashMap<usize, RenderNode>,
    pub root_id: usize,
}

impl RenderTree {
    pub fn new() -> Self { Self { nodes: HashMap::new(), root_id: 0 } }

    pub fn build_from_dom(handle: &Handle, sheets: &[super::style::StyleSheet], counter: &mut usize) -> Self {
        let mut tree = Self::new();
        tree.root_id = *counter;
        Self::walk(handle, sheets, counter, &mut tree);
        tree
    }

    fn walk(handle: &Handle, sheets: &[super::style::StyleSheet], counter: &mut usize, tree: &mut RenderTree) {
        let my_id = *counter;
        *counter += 1;
        let mut cs = ComputedStyle::default();
        cs.font_size = 16.0;
        cs.line_height = 1.2;
        cs.opacity = 1.0;
        cs.flex_shrink = 1.0;
        let mut tag = String::new();
        let mut text = String::new();
        let mut image_src = String::new();
        let mut iframe_src = String::new();
        match &handle.data {
            NodeData::Element { name, attrs, .. } => {
                tag = name.local.to_string();
                let attrs_map: HashMap<String, String> = attrs.borrow().iter()
                    .map(|a| (a.name.local.to_string(), a.value.to_string())).collect();
                let id_attr = attrs_map.get("id").cloned().unwrap_or_default();
                let class_attr = attrs_map.get("class").cloned().unwrap_or_default();
                let classes: Vec<&str> = class_attr.split_whitespace().collect();
                for sheet in sheets {
                    for rule in &sheet.rules {
                        if selector_matches(&rule.selectors, &tag, &id_attr, &classes) {
                            cs.apply_declarations(&rule.declarations);
                        }
                    }
                }
                if let Some(style_attr) = attrs_map.get("style") {
                    let inline = super::style::StyleSheet::parse(&format!("_i {{ {} }}", style_attr));
                    for rule in &inline.rules { cs.apply_declarations(&rule.declarations); }
                }
                if tag == "img" { image_src = attrs_map.get("src").cloned().unwrap_or_default(); }
                if tag == "iframe" { iframe_src = attrs_map.get("src").cloned().unwrap_or_default(); }
                apply_tag_defaults(&tag, &mut cs);
            }
            NodeData::Text { contents } => { text = contents.borrow().trim().to_string(); }
            _ => {}
        }
        let mut child_ids = Vec::new();
        for child in handle.children.borrow().iter() {
            let child_id = *counter;
            Self::walk(child, sheets, counter, tree);
            child_ids.push(child_id);
        }
        tree.nodes.insert(my_id, RenderNode { id: my_id, tag, text, image_src, iframe_src, style: cs, children: child_ids });
    }
}

fn apply_tag_defaults(tag: &str, cs: &mut ComputedStyle) {
    match tag {
        "h1" => { cs.font_size = 32.0; cs.font_weight = 700; }
        "h2" => { cs.font_size = 24.0; cs.font_weight = 700; }
        "h3" => { cs.font_size = 20.0; cs.font_weight = 700; }
        "h4" => { cs.font_size = 16.0; cs.font_weight = 700; }
        "h5" => { cs.font_size = 14.0; cs.font_weight = 700; }
        "h6" => { cs.font_size = 12.0; cs.font_weight = 700; }
        "strong" | "b" => { cs.font_weight = 700; }
        "em" | "i" => {}
        "code" | "pre" => { cs.font_family = "monospace".into(); }
        _ => {}
    }
}

fn selector_matches(selectors: &[String], tag: &str, id: &str, classes: &[&str]) -> bool {
    selectors.iter().any(|sel| {
        let sel = sel.trim();
        if sel == tag || sel == "*" { return true; }
        if sel.starts_with('#') && &sel[1..] == id { return true; }
        if sel.starts_with('.') && classes.contains(&&sel[1..]) { return true; }
        false
    })
}

pub fn build_display_list(
    tree: &RenderTree,
    layouts: &HashMap<usize, LayoutRect>,
    images: &super::image_decode::ImageCache,
    node_id: usize,
    parent_x: f32,
    parent_y: f32,
    dl: &mut DisplayList,
) {
    let node = match tree.nodes.get(&node_id) { Some(n) => n, None => return };
    let lr = layouts.get(&node_id);
    let (abs_x, abs_y, w, h) = if let Some(r) = lr {
        (parent_x + r.x, parent_y + r.y, r.w, r.h)
    } else {
        (parent_x, parent_y, 0.0, 0.0)
    };
    if node.style.background_color.a > 0.0 && w > 0.0 && h > 0.0 {
        dl.push(PaintCommand::FillRect {
            rect: PaintRect { x: abs_x, y: abs_y, w, h },
            color: color_to_rgba(&node.style.background_color),
        });
    }
    let bw = &node.style.border_width;
    if bw.top > 0.0 {
        let bc = color_to_rgba(&node.style.color);
        dl.push(PaintCommand::FillRect { rect: PaintRect { x: abs_x, y: abs_y, w, h: bw.top }, color: bc });
        dl.push(PaintCommand::FillRect { rect: PaintRect { x: abs_x, y: abs_y + h - bw.bottom, w, h: bw.bottom }, color: bc });
        dl.push(PaintCommand::FillRect { rect: PaintRect { x: abs_x, y: abs_y, w: bw.left, h }, color: bc });
        dl.push(PaintCommand::FillRect { rect: PaintRect { x: abs_x + w - bw.right, y: abs_y, w: bw.right, h }, color: bc });
    }
    if !node.text.is_empty() {
        dl.push(PaintCommand::DrawText {
            x: abs_x + node.style.padding.left,
            y: abs_y + node.style.padding.top,
            text: node.text.clone(),
            font_size: node.style.font_size,
            color: color_to_rgba(&node.style.color),
            max_width: w - node.style.padding.left - node.style.padding.right,
        });
    }
    if !node.image_src.is_empty() {
        let img = images.get(&node.image_src);
        dl.push(PaintCommand::DrawImage {
            rect: PaintRect { x: abs_x, y: abs_y, w, h },
            image_data: img.map(|i| i.rgba_data.clone()),
            img_w: img.map(|i| i.width).unwrap_or(0),
            img_h: img.map(|i| i.height).unwrap_or(0),
        });
    }
    match node.tag.as_str() {
        "video" => { dl.push(PaintCommand::DrawVideo { rect: PaintRect { x: abs_x, y: abs_y, w: w.max(320.0), h: h.max(180.0) }, video_id: node.id }); }
        "audio" => { dl.push(PaintCommand::DrawAudio { rect: PaintRect { x: abs_x, y: abs_y, w: w.max(300.0), h: h.max(40.0) }, audio_id: node.id }); }
        "canvas" => { dl.push(PaintCommand::DrawCanvas { rect: PaintRect { x: abs_x, y: abs_y, w: w.max(150.0), h: h.max(150.0) }, canvas_id: node.id }); }
        "svg" => { dl.push(PaintCommand::DrawSvg { rect: PaintRect { x: abs_x, y: abs_y, w, h }, svg_content: node.text.clone() }); }
        "iframe" => { dl.push(PaintCommand::DrawIframe { rect: PaintRect { x: abs_x, y: abs_y, w: w.max(300.0), h: h.max(150.0) }, src: node.iframe_src.clone() }); }
        _ => {}
    }
    for &child_id in &node.children {
        build_display_list(tree, layouts, images, child_id, abs_x, abs_y, dl);
    }
}

pub fn color_to_rgba(c: &Color) -> [u8; 4] {
    [c.r, c.g, c.b, (c.a * 255.0) as u8]
}

pub struct SoftwareRenderer {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    clip_stack: Vec<PaintRect>,
    #[cfg(feature = "servo-engine")]
    font: Option<fontdue::Font>,
}

impl SoftwareRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let n = (width * height * 4) as usize;
        let pixels = vec![255u8; n];
        Self {
            pixels, width, height,
            clip_stack: Vec::new(),
            #[cfg(feature = "servo-engine")]
            font: Self::load_font(),
        }
    }

    #[cfg(feature = "servo-engine")]
    fn load_font() -> Option<fontdue::Font> {
        let paths = if cfg!(target_os = "windows") {
            vec!["C:/Windows/Fonts/segoeui.ttf", "C:/Windows/Fonts/arial.ttf", "C:/Windows/Fonts/tahoma.ttf"]
        } else if cfg!(target_os = "macos") {
            vec!["/System/Library/Fonts/Helvetica.ttc", "/Library/Fonts/Arial.ttf"]
        } else {
            vec!["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", "/usr/share/fonts/TTF/DejaVuSans.ttf"]
        };
        for p in paths {
            if let Ok(data) = std::fs::read(p) {
                if let Ok(f) = fontdue::Font::from_bytes(data, fontdue::FontSettings::default()) {
                    log::info!("Loaded font: {}", p);
                    return Some(f);
                }
            }
        }
        log::warn!("No system font found");
        None
    }

    pub fn render(&mut self, dl: &DisplayList) {
        for cmd in &dl.commands {
            match cmd {
                PaintCommand::FillRect { rect, color } => self.fill_rect(rect, color),
                PaintCommand::DrawText { x, y, text, font_size, color, max_width } => {
                    self.draw_text(*x, *y, text, *font_size, color, *max_width);
                }
                PaintCommand::DrawImage { rect, image_data, img_w, img_h } => {
                    if let Some(data) = image_data {
                        self.draw_image(rect, data, *img_w, *img_h);
                    } else {
                        self.fill_rect(rect, &[220, 220, 220, 255]);
                    }
                }
                PaintCommand::PushClip { rect } => self.clip_stack.push(rect.clone()),
                PaintCommand::PopClip => { self.clip_stack.pop(); }
                PaintCommand::DrawCanvas { rect, .. } => {
                    self.fill_rect(rect, &[240, 240, 240, 255]);
                    self.draw_border(rect, &[180, 180, 180, 255]);
                }
                PaintCommand::DrawSvg { rect, .. } => {
                    self.fill_rect(rect, &[248, 248, 248, 255]);
                    self.draw_border(rect, &[200, 200, 200, 255]);
                }
                PaintCommand::DrawVideo { rect, .. } => {
                    self.fill_rect(rect, &[20, 20, 20, 255]);
                    self.draw_play_icon(rect);
                }
                PaintCommand::DrawAudio { rect, .. } => {
                    self.fill_rect(rect, &[45, 45, 50, 255]);
                    let btn_rect = PaintRect { x: rect.x + 8.0, y: rect.y + (rect.h - 20.0) / 2.0, w: 20.0, h: 20.0 };
                    self.draw_play_icon(&btn_rect);
                    let bar_rect = PaintRect { x: rect.x + 36.0, y: rect.y + rect.h / 2.0 - 2.0, w: rect.w - 44.0, h: 4.0 };
                    self.fill_rect(&bar_rect, &[80, 80, 90, 255]);
                }
                PaintCommand::DrawIframe { rect, .. } => {
                    self.fill_rect(rect, &[210, 230, 250, 255]);
                    self.draw_border(rect, &[100, 140, 200, 255]);
                }
            }
        }
    }

    fn is_clipped(&self, x: f32, y: f32) -> bool {
        self.clip_stack.iter().any(|c| x < c.x || x >= c.x + c.w || y < c.y || y >= c.y + c.h)
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: &[u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { return; }
        if self.is_clipped(x as f32, y as f32) { return; }
        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        if idx + 3 >= self.pixels.len() { return; }
        let a = color[3] as f32 / 255.0;
        if a >= 1.0 {
            self.pixels[idx..idx + 4].copy_from_slice(color);
        } else {
            let inv = 1.0 - a;
            self.pixels[idx] = (self.pixels[idx] as f32 * inv + color[0] as f32 * a) as u8;
            self.pixels[idx + 1] = (self.pixels[idx + 1] as f32 * inv + color[1] as f32 * a) as u8;
            self.pixels[idx + 2] = (self.pixels[idx + 2] as f32 * inv + color[2] as f32 * a) as u8;
            self.pixels[idx + 3] = 255;
        }
    }

    fn fill_rect(&mut self, rect: &PaintRect, color: &[u8; 4]) {
        let x0 = rect.x.max(0.0) as i32;
        let y0 = rect.y.max(0.0) as i32;
        let x1 = (rect.x + rect.w).min(self.width as f32) as i32;
        let y1 = (rect.y + rect.h).min(self.height as f32) as i32;
        for y in y0..y1 { for x in x0..x1 { self.set_pixel(x, y, color); } }
    }

    fn draw_text(&mut self, x: f32, y: f32, text: &str, font_size: f32, color: &[u8; 4], max_width: f32) {
        #[cfg(feature = "servo-engine")]
        {
            let font = match self.font.take() { Some(f) => f, None => return };
            let sz = font_size.clamp(6.0, 200.0);
            let mut cx = x;
            let max_x = x + max_width;
            for ch in text.chars() {
                if cx >= max_x { break; }
                let (metrics, bitmap) = font.rasterize(ch, sz);
                if metrics.width > 0 && metrics.height > 0 {
                    let gy0 = y + sz * 0.8 - metrics.height as f32 - metrics.ymin as f32;
                    for row in 0..metrics.height {
                        for col in 0..metrics.width {
                            let cov = bitmap[row * metrics.width + col];
                            if cov > 0 {
                                let blended = [color[0], color[1], color[2], ((cov as f32 / 255.0) * color[3] as f32) as u8];
                                self.set_pixel(cx as i32 + col as i32, gy0 as i32 + row as i32, &blended);
                            }
                        }
                    }
                }
                cx += metrics.advance_width;
            }
            self.font = Some(font);
        }
        #[cfg(not(feature = "servo-engine"))]
        { let _ = (x, y, text, font_size, color, max_width); }
    }

    fn draw_image(&mut self, rect: &PaintRect, rgba: &[u8], img_w: u32, img_h: u32) {
        if img_w == 0 || img_h == 0 { return; }
        let scale_x = img_w as f32 / rect.w;
        let scale_y = img_h as f32 / rect.h;
        let x0 = rect.x.max(0.0) as i32;
        let y0 = rect.y.max(0.0) as i32;
        let x1 = (rect.x + rect.w).min(self.width as f32) as i32;
        let y1 = (rect.y + rect.h).min(self.height as f32) as i32;
        for py in y0..y1 {
            for px in x0..x1 {
                let sx = ((px as f32 - rect.x) * scale_x) as u32;
                let sy = ((py as f32 - rect.y) * scale_y) as u32;
                if sx < img_w && sy < img_h {
                    let si = ((sy * img_w + sx) * 4) as usize;
                    if si + 3 < rgba.len() {
                        self.set_pixel(px, py, &[rgba[si], rgba[si+1], rgba[si+2], rgba[si+3]]);
                    }
                }
            }
        }
    }

    fn draw_border(&mut self, rect: &PaintRect, color: &[u8; 4]) {
        let x0 = rect.x.max(0.0) as i32;
        let y0 = rect.y.max(0.0) as i32;
        let x1 = (rect.x + rect.w).min(self.width as f32) as i32 - 1;
        let y1 = (rect.y + rect.h).min(self.height as f32) as i32 - 1;
        for x in x0..=x1 { self.set_pixel(x, y0, color); self.set_pixel(x, y1, color); }
        for y in y0..=y1 { self.set_pixel(x0, y, color); self.set_pixel(x1, y, color); }
    }
    fn draw_play_icon(&mut self, rect: &PaintRect) {
        let cx = rect.x + rect.w / 2.0;
        let cy = rect.y + rect.h / 2.0;
        let sz = rect.w.min(rect.h) * 0.3;
        let color = [255u8, 255, 255, 200];
        for row in 0..(sz as i32 * 2) {
            let ry = cy - sz + row as f32;
            let frac = (row as f32) / (sz * 2.0);
            let half_w = sz * (0.5 - (frac - 0.5).abs()) * 2.0;
            for col in 0..(half_w as i32) {
                self.set_pixel((cx - sz * 0.3 + col as f32) as i32, ry as i32, &color);
            }
        }
    }
    pub fn clear(&mut self, color: [u8; 4]) {
        for i in (0..self.pixels.len()).step_by(4) {
            self.pixels[i] = color[0]; self.pixels[i+1] = color[1];
            self.pixels[i+2] = color[2]; self.pixels[i+3] = color[3];
        }
    }

    pub fn render_with_resources(&mut self, dl: &DisplayList, canvases: &HashMap<usize, crate::engine::canvas::Canvas2D>) {
        for cmd in &dl.commands {
            match cmd {
                PaintCommand::FillRect { rect, color } => self.fill_rect(rect, color),
                PaintCommand::DrawText { x, y, text, font_size, color, max_width } => {
                    self.draw_text(*x, *y, text, *font_size, color, *max_width);
                }
                PaintCommand::DrawImage { rect, image_data, img_w, img_h } => {
                    if let Some(data) = image_data {
                        self.draw_image(rect, data, *img_w, *img_h);
                    } else {
                        self.fill_rect(rect, &[220, 220, 220, 255]);
                    }
                }
                PaintCommand::PushClip { rect } => self.clip_stack.push(rect.clone()),
                PaintCommand::PopClip => { self.clip_stack.pop(); }
                PaintCommand::DrawCanvas { rect, canvas_id } => {
                    if let Some(canvas) = canvases.get(canvas_id) {
                        self.render_canvas(rect, canvas);
                    } else {
                        self.fill_rect(rect, &[240, 240, 240, 255]);
                        self.draw_border(rect, &[180, 180, 180, 255]);
                    }
                }
                PaintCommand::DrawSvg { rect, svg_content } => {
                    if !svg_content.is_empty() {
                        self.render_svg(rect, svg_content);
                    } else {
                        self.fill_rect(rect, &[248, 248, 248, 255]);
                        self.draw_border(rect, &[200, 200, 200, 255]);
                    }
                }
                PaintCommand::DrawVideo { rect, .. } => {
                    self.fill_rect(rect, &[20, 20, 20, 255]);
                    self.draw_play_icon(rect);
                }
                PaintCommand::DrawAudio { rect, .. } => {
                    self.fill_rect(rect, &[45, 45, 50, 255]);
                    let btn_rect = PaintRect { x: rect.x + 8.0, y: rect.y + (rect.h - 20.0) / 2.0, w: 20.0, h: 20.0 };
                    self.draw_play_icon(&btn_rect);
                    let bar_rect = PaintRect { x: rect.x + 36.0, y: rect.y + rect.h / 2.0 - 2.0, w: rect.w - 44.0, h: 4.0 };
                    self.fill_rect(&bar_rect, &[80, 80, 90, 255]);
                }
                PaintCommand::DrawIframe { rect, .. } => {
                    self.fill_rect(rect, &[210, 230, 250, 255]);
                    self.draw_border(rect, &[100, 140, 200, 255]);
                }
            }
        }
    }

    fn render_canvas(&mut self, rect: &PaintRect, canvas: &crate::engine::canvas::Canvas2D) {
        if canvas.width == 0 || canvas.height == 0 { return; }
        let scale_x = canvas.width as f32 / rect.w;
        let scale_y = canvas.height as f32 / rect.h;
        let x0 = rect.x.max(0.0) as i32;
        let y0 = rect.y.max(0.0) as i32;
        let x1 = (rect.x + rect.w).min(self.width as f32) as i32;
        let y1 = (rect.y + rect.h).min(self.height as f32) as i32;
        for py in y0..y1 {
            for px in x0..x1 {
                let sx = ((px as f32 - rect.x) * scale_x) as u32;
                let sy = ((py as f32 - rect.y) * scale_y) as u32;
                if sx < canvas.width && sy < canvas.height {
                    let si = ((sy * canvas.width + sx) * 4) as usize;
                    if si + 3 < canvas.pixels.len() {
                        self.set_pixel(px, py, &[canvas.pixels[si], canvas.pixels[si+1], canvas.pixels[si+2], canvas.pixels[si+3]]);
                    }
                }
            }
        }
    }

    fn render_svg(&mut self, rect: &PaintRect, svg_content: &str) {
        let doc = match crate::engine::svg::parse_svg(svg_content) {
            Ok(d) => d,
            Err(_) => {
                self.fill_rect(rect, &[248, 248, 248, 255]);
                self.draw_border(rect, &[200, 200, 200, 255]);
                return;
            }
        };
        let svg_commands = crate::engine::svg::render_to_commands(&doc);
        let sx = if doc.width > 0.0 { rect.w / doc.width } else { 1.0 };
        let sy = if doc.height > 0.0 { rect.h / doc.height } else { 1.0 };
        for cmd in &svg_commands {
            match cmd {
                PaintCommand::FillRect { rect: sr, color } => {
                    let mapped = PaintRect {
                        x: sr.x * sx + rect.x,
                        y: sr.y * sy + rect.y,
                        w: sr.w * sx,
                        h: sr.h * sy,
                    };
                    self.fill_rect(&mapped, color);
                }
                PaintCommand::DrawText { x, y, text, font_size, color, max_width } => {
                    self.draw_text(
                        x * sx + rect.x,
                        y * sy + rect.y,
                        text,
                        font_size * sy,
                        color,
                        max_width * sx,
                    );
                }
                PaintCommand::DrawImage { rect: sr, image_data, img_w, img_h } => {
                    let mapped = PaintRect {
                        x: sr.x * sx + rect.x,
                        y: sr.y * sy + rect.y,
                        w: sr.w * sx,
                        h: sr.h * sy,
                    };
                    if let Some(data) = image_data {
                        self.draw_image(&mapped, data, *img_w, *img_h);
                    } else {
                        self.fill_rect(&mapped, &[220, 220, 220, 255]);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn pixels(&self) -> &[u8] { &self.pixels }
}
