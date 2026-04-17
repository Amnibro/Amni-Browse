use std::f32::consts::PI;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign { Left, Center, Right }
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextBaseline { Top, Middle, Alphabetic, Bottom }
#[derive(Debug, Clone)]
pub struct CanvasState {
    pub fill_color: [u8; 4],
    pub stroke_color: [u8; 4],
    pub line_width: f32,
    pub global_alpha: f32,
    pub transform: [f32; 6],
    pub clip_rect: Option<(f32, f32, f32, f32)>,
    pub font_size: f32,
    pub text_align: TextAlign,
    pub text_baseline: TextBaseline,
}
impl Default for CanvasState {
    fn default() -> Self {
        Self {
            fill_color: [0, 0, 0, 255],
            stroke_color: [0, 0, 0, 255],
            line_width: 1.0,
            global_alpha: 1.0,
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            clip_rect: None,
            font_size: 10.0,
            text_align: TextAlign::Left,
            text_baseline: TextBaseline::Alphabetic,
        }
    }
}
#[derive(Debug, Clone)]
pub enum PathCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    Arc(f32, f32, f32, f32, f32, bool),
    QuadraticCurveTo(f32, f32, f32, f32),
    BezierCurveTo(f32, f32, f32, f32, f32, f32),
    ClosePath,
}
#[derive(Debug, Clone)]
pub struct Path {
    pub commands: Vec<PathCommand>,
}
impl Path {
    pub fn new() -> Self { Self { commands: Vec::new() } }
    pub fn flatten(&self, tolerance: f32) -> Vec<(f32, f32)> {
        let mut points = Vec::new();
        let mut cx = 0.0f32;
        let mut cy = 0.0f32;
        let mut start_x = 0.0f32;
        let mut start_y = 0.0f32;
        for cmd in &self.commands {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
                    start_x = x; start_y = y; cx = x; cy = y;
                    points.push((x, y));
                }
                PathCommand::LineTo(x, y) => {
                    cx = x; cy = y;
                    points.push((x, y));
                }
                PathCommand::Arc(acx, acy, r, start, end, ccw) => {
                    let (sa, ea) = if ccw && end > start { (start, end - 2.0 * PI) }
                    else if !ccw && end < start { (start, end + 2.0 * PI) }
                    else { (start, end) };
                    let arc_len = (ea - sa).abs() * r;
                    let steps = ((arc_len / tolerance).ceil() as usize).max(8);
                    for i in 0..=steps {
                        let t = i as f32 / steps as f32;
                        let angle = sa + (ea - sa) * t;
                        let px = acx + r * angle.cos();
                        let py = acy + r * angle.sin();
                        points.push((px, py));
                    }
                    let final_angle = ea;
                    cx = acx + r * final_angle.cos();
                    cy = acy + r * final_angle.sin();
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
                    let steps = 16usize;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let it = 1.0 - t;
                        let px = it * it * cx + 2.0 * it * t * cpx + t * t * x;
                        let py = it * it * cy + 2.0 * it * t * cpy + t * t * y;
                        points.push((px, py));
                    }
                    cx = x; cy = y;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
                    let steps = 24usize;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let it = 1.0 - t;
                        let px = it*it*it*cx + 3.0*it*it*t*cp1x + 3.0*it*t*t*cp2x + t*t*t*x;
                        let py = it*it*it*cy + 3.0*it*it*t*cp1y + 3.0*it*t*t*cp2y + t*t*t*y;
                        points.push((px, py));
                    }
                    cx = x; cy = y;
                }
                PathCommand::ClosePath => {
                    cx = start_x; cy = start_y;
                    points.push((start_x, start_y));
                }
            }
        }
        points
    }
}
pub struct Canvas2D {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    state: CanvasState,
    state_stack: Vec<CanvasState>,
    current_path: Path,
}
impl Canvas2D {
    pub fn new(w: u32, h: u32) -> Self {
        let pixels = vec![255u8; (w * h * 4) as usize];
        Self { width: w, height: h, pixels, state: CanvasState::default(), state_stack: Vec::new(), current_path: Path::new() }
    }
    pub fn save(&mut self) { self.state_stack.push(self.state.clone()); }
    pub fn restore(&mut self) { if let Some(s) = self.state_stack.pop() { self.state = s; } }
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let color = self.apply_alpha(self.state.fill_color);
        let (tx, ty) = self.transform_point(x, y);
        let (tx2, ty2) = self.transform_point(x + w, y + h);
        let x0 = tx.min(tx2).max(0.0) as i32;
        let y0 = ty.min(ty2).max(0.0) as i32;
        let x1 = tx.max(tx2).min(self.width as f32) as i32;
        let y1 = ty.max(ty2).min(self.height as f32) as i32;
        for py in y0..y1 { for px in x0..x1 { self.set_pixel(px, py, &color); } }
    }
    pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let color = self.apply_alpha(self.state.stroke_color);
        let lw = self.state.line_width;
        self.draw_line_internal(x, y, x + w, y, &color, lw);
        self.draw_line_internal(x + w, y, x + w, y + h, &color, lw);
        self.draw_line_internal(x + w, y + h, x, y + h, &color, lw);
        self.draw_line_internal(x, y + h, x, y, &color, lw);
    }
    pub fn clear_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let (tx, ty) = self.transform_point(x, y);
        let (tx2, ty2) = self.transform_point(x + w, y + h);
        let x0 = tx.min(tx2).max(0.0) as i32;
        let y0 = ty.min(ty2).max(0.0) as i32;
        let x1 = tx.max(tx2).min(self.width as f32) as i32;
        let y1 = ty.max(ty2).min(self.height as f32) as i32;
        let clear = [0u8; 4];
        for py in y0..y1 { for px in x0..x1 { self.set_pixel_direct(px, py, &clear); } }
    }
    pub fn begin_path(&mut self) { self.current_path = Path::new(); }
    pub fn move_to(&mut self, x: f32, y: f32) { self.current_path.commands.push(PathCommand::MoveTo(x, y)); }
    pub fn line_to(&mut self, x: f32, y: f32) { self.current_path.commands.push(PathCommand::LineTo(x, y)); }
    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, start_angle: f32, end_angle: f32, ccw: bool) {
        self.current_path.commands.push(PathCommand::Arc(cx, cy, r, start_angle, end_angle, ccw));
    }
    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        self.current_path.commands.push(PathCommand::QuadraticCurveTo(cpx, cpy, x, y));
    }
    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        self.current_path.commands.push(PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y));
    }
    pub fn close_path(&mut self) { self.current_path.commands.push(PathCommand::ClosePath); }
    pub fn fill(&mut self) {
        let color = self.apply_alpha(self.state.fill_color);
        let points = self.current_path.flatten(2.0);
        let transformed: Vec<(f32, f32)> = points.iter().map(|&(x, y)| self.transform_point(x, y)).collect();
        if transformed.len() < 3 { return; }
        let min_y = transformed.iter().map(|p| p.1).fold(f32::MAX, f32::min).max(0.0) as i32;
        let max_y = transformed.iter().map(|p| p.1).fold(f32::MIN, f32::max).min(self.height as f32) as i32;
        for y in min_y..max_y {
            let yf = y as f32 + 0.5;
            let mut intersections = Vec::new();
            let n = transformed.len();
            for i in 0..n {
                let (x0, y0) = transformed[i];
                let (x1, y1) = transformed[(i + 1) % n];
                if (y0 <= yf && y1 > yf) || (y1 <= yf && y0 > yf) {
                    let t = (yf - y0) / (y1 - y0);
                    intersections.push(x0 + t * (x1 - x0));
                }
            }
            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut i = 0;
            while i + 1 < intersections.len() {
                let x_start = intersections[i].max(0.0) as i32;
                let x_end = intersections[i + 1].min(self.width as f32) as i32;
                for x in x_start..x_end { self.set_pixel(x, y, &color); }
                i += 2;
            }
        }
    }
    pub fn stroke(&mut self) {
        let color = self.apply_alpha(self.state.stroke_color);
        let lw = self.state.line_width;
        let points = self.current_path.flatten(2.0);
        let transformed: Vec<(f32, f32)> = points.iter().map(|&(x, y)| self.transform_point(x, y)).collect();
        for i in 0..transformed.len().saturating_sub(1) {
            let (x0, y0) = transformed[i];
            let (x1, y1) = transformed[i + 1];
            self.draw_line_raw(x0, y0, x1, y1, &color, lw);
        }
    }
    pub fn fill_text(&mut self, text: &str, x: f32, y: f32) {
        let color = self.apply_alpha(self.state.fill_color);
        let fs = self.state.font_size;
        let char_w = (fs * 0.6).max(1.0);
        let char_h = fs;
        let total_w = text.len() as f32 * char_w;
        let start_x = match self.state.text_align {
            TextAlign::Left => x,
            TextAlign::Center => x - total_w * 0.5,
            TextAlign::Right => x - total_w,
        };
        let start_y = match self.state.text_baseline {
            TextBaseline::Top => y,
            TextBaseline::Middle => y - char_h * 0.5,
            TextBaseline::Alphabetic => y - char_h * 0.8,
            TextBaseline::Bottom => y - char_h,
        };
        for (i, _ch) in text.chars().enumerate() {
            let cx = start_x + i as f32 * char_w;
            let (tx, ty) = self.transform_point(cx + 1.0, start_y + 1.0);
            let (tx2, ty2) = self.transform_point(cx + char_w - 1.0, start_y + char_h - 1.0);
            let px0 = tx.min(tx2).max(0.0) as i32;
            let py0 = ty.min(ty2).max(0.0) as i32;
            let px1 = tx.max(tx2).min(self.width as f32) as i32;
            let py1 = ty.max(ty2).min(self.height as f32) as i32;
            for py in py0..py1 { for px in px0..px1 { self.set_pixel(px, py, &color); } }
        }
    }
    pub fn draw_image(&mut self, src_pixels: &[u8], src_w: u32, src_h: u32, dx: f32, dy: f32, dw: f32, dh: f32) {
        if src_w == 0 || src_h == 0 || dw <= 0.0 || dh <= 0.0 { return; }
        let (tdx, tdy) = self.transform_point(dx, dy);
        let (tdx2, tdy2) = self.transform_point(dx + dw, dy + dh);
        let x0 = tdx.min(tdx2).max(0.0) as i32;
        let y0 = tdy.min(tdy2).max(0.0) as i32;
        let x1 = tdx.max(tdx2).min(self.width as f32) as i32;
        let y1 = tdy.max(tdy2).min(self.height as f32) as i32;
        let actual_dw = (tdx2 - tdx).abs();
        let actual_dh = (tdy2 - tdy).abs();
        let scale_x = src_w as f32 / actual_dw;
        let scale_y = src_h as f32 / actual_dh;
        let base_x = tdx.min(tdx2);
        let base_y = tdy.min(tdy2);
        for py in y0..y1 {
            for px in x0..x1 {
                let sx = ((px as f32 - base_x) * scale_x) as u32;
                let sy = ((py as f32 - base_y) * scale_y) as u32;
                if sx < src_w && sy < src_h {
                    let si = ((sy * src_w + sx) * 4) as usize;
                    if si + 3 < src_pixels.len() {
                        let c = [src_pixels[si], src_pixels[si+1], src_pixels[si+2], src_pixels[si+3]];
                        self.set_pixel(px, py, &c);
                    }
                }
            }
        }
    }
    pub fn translate(&mut self, tx: f32, ty: f32) {
        let [a, b, c, d, e, f] = self.state.transform;
        self.state.transform = [a, b, c, d, a * tx + c * ty + e, b * tx + d * ty + f];
    }
    pub fn rotate(&mut self, angle: f32) {
        let cos = angle.cos();
        let sin = angle.sin();
        let [a, b, c, d, e, f] = self.state.transform;
        self.state.transform = [
            a * cos + c * sin, b * cos + d * sin,
            c * cos - a * sin, d * cos - b * sin,
            e, f,
        ];
    }
    pub fn scale(&mut self, sx: f32, sy: f32) {
        let [a, b, c, d, e, f] = self.state.transform;
        self.state.transform = [a * sx, b * sx, c * sy, d * sy, e, f];
    }
    pub fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.state.transform = [a, b, c, d, e, f];
    }
    pub fn set_fill_style(&mut self, color: [u8; 4]) { self.state.fill_color = color; }
    pub fn set_stroke_style(&mut self, color: [u8; 4]) { self.state.stroke_color = color; }
    pub fn set_line_width(&mut self, w: f32) { self.state.line_width = w.max(0.0); }
    pub fn set_global_alpha(&mut self, a: f32) { self.state.global_alpha = a.clamp(0.0, 1.0); }
    pub fn set_font_size(&mut self, size: f32) { self.state.font_size = size.max(1.0); }
    pub fn set_text_align(&mut self, align: TextAlign) { self.state.text_align = align; }
    pub fn set_text_baseline(&mut self, baseline: TextBaseline) { self.state.text_baseline = baseline; }
    pub fn set_clip_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.state.clip_rect = Some((x, y, w, h));
    }
    pub fn get_image_data(&self, x: u32, y: u32, w: u32, h: u32) -> Vec<u8> {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for row in y..(y + h).min(self.height) {
            for col in x..(x + w).min(self.width) {
                let idx = ((row * self.width + col) * 4) as usize;
                if idx + 3 < self.pixels.len() {
                    data.extend_from_slice(&self.pixels[idx..idx + 4]);
                } else { data.extend_from_slice(&[0, 0, 0, 0]); }
            }
        }
        data
    }
    pub fn put_image_data(&mut self, data: &[u8], x: u32, y: u32) {
        let w = if data.len() >= 4 { (data.len() / 4) as u32 } else { return };
        let stride = ((w as f32).sqrt() as u32).max(1);
        let mut i = 0;
        let mut row = y;
        let mut col = x;
        while i + 3 < data.len() {
            if col < self.width && row < self.height {
                let idx = ((row * self.width + col) * 4) as usize;
                if idx + 3 < self.pixels.len() {
                    self.pixels[idx..idx + 4].copy_from_slice(&data[i..i + 4]);
                }
            }
            col += 1;
            if col >= x + stride { col = x; row += 1; }
            i += 4;
        }
    }
    pub fn to_png_bytes(&self) -> Vec<u8> {
        let img = image::RgbaImage::from_raw(self.width, self.height, self.pixels.clone())
            .unwrap_or_else(|| image::RgbaImage::new(self.width, self.height));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap_or(());
        buf.into_inner()
    }
    fn set_pixel(&mut self, x: i32, y: i32, color: &[u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { return; }
        if let Some((cx, cy, cw, ch)) = self.state.clip_rect {
            let (tx, ty) = (x as f32, y as f32);
            if tx < cx || tx >= cx + cw || ty < cy || ty >= cy + ch { return; }
        }
        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        if idx + 3 >= self.pixels.len() { return; }
        let a = color[3] as f32 / 255.0;
        if a >= 1.0 {
            self.pixels[idx..idx + 4].copy_from_slice(color);
        } else if a > 0.0 {
            let inv = 1.0 - a;
            self.pixels[idx] = (self.pixels[idx] as f32 * inv + color[0] as f32 * a) as u8;
            self.pixels[idx+1] = (self.pixels[idx+1] as f32 * inv + color[1] as f32 * a) as u8;
            self.pixels[idx+2] = (self.pixels[idx+2] as f32 * inv + color[2] as f32 * a) as u8;
            self.pixels[idx+3] = (self.pixels[idx+3] as f32 * inv + 255.0 * a) as u8;
        }
    }
    fn set_pixel_direct(&mut self, x: i32, y: i32, color: &[u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { return; }
        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        if idx + 3 < self.pixels.len() { self.pixels[idx..idx + 4].copy_from_slice(color); }
    }
    fn apply_alpha(&self, mut color: [u8; 4]) -> [u8; 4] {
        color[3] = (color[3] as f32 * self.state.global_alpha) as u8;
        color
    }
    fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        let [a, b, c, d, e, f] = self.state.transform;
        (a * x + c * y + e, b * x + d * y + f)
    }
    fn draw_line_internal(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: &[u8; 4], width: f32) {
        let (tx0, ty0) = self.transform_point(x0, y0);
        let (tx1, ty1) = self.transform_point(x1, y1);
        self.draw_line_raw(tx0, ty0, tx1, ty1, color, width);
    }
    fn draw_line_raw(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: &[u8; 4], width: f32) {
        let hw = (width / 2.0).max(0.5);
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 { return; }
        let nx = -dy / len * hw;
        let ny = dx / len * hw;
        let corners = [
            (x0 + nx, y0 + ny), (x0 - nx, y0 - ny),
            (x1 - nx, y1 - ny), (x1 + nx, y1 + ny),
        ];
        let min_y = corners.iter().map(|c| c.1).fold(f32::MAX, f32::min).max(0.0) as i32;
        let max_y = corners.iter().map(|c| c.1).fold(f32::MIN, f32::max).min(self.height as f32) as i32;
        for y in min_y..max_y {
            let yf = y as f32 + 0.5;
            let mut intersections = Vec::new();
            for i in 0..4 {
                let (ax, ay) = corners[i];
                let (bx, by) = corners[(i + 1) % 4];
                if (ay <= yf && by > yf) || (by <= yf && ay > yf) {
                    let t = (yf - ay) / (by - ay);
                    intersections.push(ax + t * (bx - ax));
                }
            }
            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut i = 0;
            while i + 1 < intersections.len() {
                let xs = intersections[i].max(0.0) as i32;
                let xe = intersections[i + 1].min(self.width as f32) as i32;
                for x in xs..xe { self.set_pixel(x, y, color); }
                i += 2;
            }
        }
    }
}
