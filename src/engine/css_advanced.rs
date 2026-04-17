use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct Transform {
    pub matrix: [f32; 6],
}
impl Default for Transform {
    fn default() -> Self { Self { matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0] } }
}
impl Transform {
    pub fn translate(tx: f32, ty: f32) -> Self { Self { matrix: [1.0, 0.0, 0.0, 1.0, tx, ty] } }
    pub fn rotate(deg: f32) -> Self {
        let r = deg * std::f32::consts::PI / 180.0;
        let (s, c) = (r.sin(), r.cos());
        Self { matrix: [c, s, -s, c, 0.0, 0.0] }
    }
    pub fn scale(sx: f32, sy: f32) -> Self { Self { matrix: [sx, 0.0, 0.0, sy, 0.0, 0.0] } }
    pub fn skew(ax: f32, ay: f32) -> Self {
        let rx = ax * std::f32::consts::PI / 180.0;
        let ry = ay * std::f32::consts::PI / 180.0;
        Self { matrix: [1.0, ry.tan(), rx.tan(), 1.0, 0.0, 0.0] }
    }
    pub fn matrix(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self { Self { matrix: [a, b, c, d, e, f] } }
    pub fn apply_to_point(&self, x: f32, y: f32) -> (f32, f32) {
        let m = &self.matrix;
        (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
    }
    pub fn compose(&self, other: &Transform) -> Transform {
        let a = &self.matrix;
        let b = &other.matrix;
        Transform { matrix: [
            a[0]*b[0] + a[2]*b[1], a[1]*b[0] + a[3]*b[1],
            a[0]*b[2] + a[2]*b[3], a[1]*b[2] + a[3]*b[3],
            a[0]*b[4] + a[2]*b[5] + a[4], a[1]*b[4] + a[3]*b[5] + a[5],
        ] }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TimingFunction {
    Ease,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
}
impl TimingFunction {
    pub fn interpolate(&self, t: f32) -> f32 {
        match self {
            TimingFunction::Linear => t,
            TimingFunction::Ease => cubic_bezier_sample(0.25, 0.1, 0.25, 1.0, t),
            TimingFunction::EaseIn => cubic_bezier_sample(0.42, 0.0, 1.0, 1.0, t),
            TimingFunction::EaseOut => cubic_bezier_sample(0.0, 0.0, 0.58, 1.0, t),
            TimingFunction::EaseInOut => cubic_bezier_sample(0.42, 0.0, 0.58, 1.0, t),
            TimingFunction::CubicBezier(x1, y1, x2, y2) => cubic_bezier_sample(*x1, *y1, *x2, *y2, t),
        }
    }
}
fn cubic_bezier_sample(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let mut lo = 0.0_f32;
    let mut hi = 1.0_f32;
    for _ in 0..20 {
        let mid = (lo + hi) * 0.5;
        let x = bezier_component(x1, x2, mid);
        if x < t { lo = mid; } else { hi = mid; }
    }
    let param = (lo + hi) * 0.5;
    bezier_component(y1, y2, param)
}
fn bezier_component(p1: f32, p2: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t
}
#[derive(Debug, Clone)]
pub struct Transition {
    pub property: String,
    pub duration_ms: f32,
    pub delay_ms: f32,
    pub timing: TimingFunction,
}
impl Transition {
    pub fn interpolate(&self, t: f32) -> f32 { self.timing.interpolate(t) }
}
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationDirection { Normal, Reverse, Alternate, AlternateReverse }
#[derive(Debug, Clone, PartialEq)]
pub enum FillMode { None, Forwards, Backwards, Both }
#[derive(Debug, Clone)]
pub struct Keyframe {
    pub percentage: f32,
    pub properties: HashMap<String, String>,
}
#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub keyframes: Vec<Keyframe>,
    pub duration_ms: f32,
    pub iteration_count: f32,
    pub direction: AnimationDirection,
    pub fill_mode: FillMode,
    pub timing: TimingFunction,
}
impl Animation {
    pub fn compute_value_at(&self, property: &str, progress: f32) -> Option<String> {
        let p = match self.direction {
            AnimationDirection::Normal => progress,
            AnimationDirection::Reverse => 1.0 - progress,
            AnimationDirection::Alternate => {
                let cycle = (progress * self.iteration_count) as u32;
                let frac = (progress * self.iteration_count) - cycle as f32;
                if cycle % 2 == 0 { frac } else { 1.0 - frac }
            }
            AnimationDirection::AlternateReverse => {
                let cycle = (progress * self.iteration_count) as u32;
                let frac = (progress * self.iteration_count) - cycle as f32;
                if cycle % 2 == 0 { 1.0 - frac } else { frac }
            }
        };
        let pct = (p * 100.0).clamp(0.0, 100.0);
        let mut before: Option<&Keyframe> = None;
        let mut after: Option<&Keyframe> = None;
        for kf in &self.keyframes {
            if kf.percentage <= pct { before = Some(kf); }
            if kf.percentage >= pct && after.is_none() { after = Some(kf); }
        }
        let b = before?;
        let a = after.unwrap_or(b);
        let bv = b.properties.get(property)?;
        let av = a.properties.get(property)?;
        if (a.percentage - b.percentage).abs() < 0.001 { return Some(bv.clone()); }
        let local_t = (pct - b.percentage) / (a.percentage - b.percentage);
        let eased = self.timing.interpolate(local_t);
        let bnum: Result<f32, _> = bv.trim_end_matches("px").parse();
        let anum: Result<f32, _> = av.trim_end_matches("px").parse();
        if let (Ok(bn), Ok(an)) = (bnum, anum) {
            let v = bn + (an - bn) * eased;
            if bv.ends_with("px") { Some(format!("{}px", v)) }
            else { Some(format!("{}", v)) }
        } else {
            if eased < 0.5 { Some(bv.clone()) } else { Some(av.clone()) }
        }
    }
}
#[derive(Debug, Clone)]
pub enum Gradient {
    Linear { angle_deg: f32, stops: Vec<(f32, [u8; 4])> },
    Radial { cx: f32, cy: f32, radius: f32, stops: Vec<(f32, [u8; 4])> },
}
impl Gradient {
    pub fn sample_color_at(&self, position: f32) -> [u8; 4] {
        let stops = match self {
            Gradient::Linear { stops, .. } => stops,
            Gradient::Radial { stops, .. } => stops,
        };
        if stops.is_empty() { return [0, 0, 0, 255]; }
        if stops.len() == 1 { return stops[0].1; }
        let p = position.clamp(0.0, 1.0);
        let mut before = &stops[0];
        let mut after = &stops[stops.len() - 1];
        for i in 0..stops.len() - 1 {
            if stops[i].0 <= p && stops[i + 1].0 >= p {
                before = &stops[i];
                after = &stops[i + 1];
                break;
            }
        }
        let range = after.0 - before.0;
        if range.abs() < 0.0001 { return before.1; }
        let t = (p - before.0) / range;
        [
            lerp_u8(before.1[0], after.1[0], t),
            lerp_u8(before.1[1], after.1[1], t),
            lerp_u8(before.1[2], after.1[2], t),
            lerp_u8(before.1[3], after.1[3], t),
        ]
    }
}
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).clamp(0.0, 255.0) as u8
}
#[derive(Debug, Clone)]
pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: [u8; 4],
    pub inset: bool,
}
#[derive(Debug, Clone)]
pub struct TextShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub color: [u8; 4],
}
#[derive(Debug, Clone)]
pub enum Filter {
    Blur(f32),
    Brightness(f32),
    Contrast(f32),
    Grayscale(f32),
    Sepia(f32),
    Saturate(f32),
    HueRotate(f32),
    Invert(f32),
    Opacity(f32),
    DropShadow(BoxShadow),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Orientation { Portrait, Landscape }
#[derive(Debug, Clone, PartialEq)]
pub enum ColorScheme { Light, Dark }
#[derive(Debug, Clone, PartialEq)]
pub enum MediaCondition {
    MinWidth(f32),
    MaxWidth(f32),
    MinHeight(f32),
    MaxHeight(f32),
    Orientation(Orientation),
    PrefersColorScheme(ColorScheme),
    PrefersReducedMotion,
}
#[derive(Debug, Clone)]
pub struct MediaQuery {
    pub conditions: Vec<MediaCondition>,
}
impl MediaQuery {
    pub fn evaluate(&self, viewport_w: f32, viewport_h: f32, is_dark: bool) -> bool {
        self.conditions.iter().all(|c| match c {
            MediaCondition::MinWidth(v) => viewport_w >= *v,
            MediaCondition::MaxWidth(v) => viewport_w <= *v,
            MediaCondition::MinHeight(v) => viewport_h >= *v,
            MediaCondition::MaxHeight(v) => viewport_h <= *v,
            MediaCondition::Orientation(o) => match o {
                Orientation::Portrait => viewport_h >= viewport_w,
                Orientation::Landscape => viewport_w > viewport_h,
            },
            MediaCondition::PrefersColorScheme(s) => match s {
                ColorScheme::Dark => is_dark,
                ColorScheme::Light => !is_dark,
            },
            MediaCondition::PrefersReducedMotion => false,
        })
    }
}
pub struct CssVars {
    pub vars: HashMap<String, String>,
}
impl CssVars {
    pub fn new() -> Self { Self { vars: HashMap::new() } }
    pub fn set(&mut self, name: &str, value: &str) { self.vars.insert(name.to_string(), value.to_string()); }
    pub fn resolve_var(&self, name: &str) -> Option<String> {
        let key = if name.starts_with("--") { name.to_string() } else { format!("--{}", name) };
        self.vars.get(&key).cloned()
    }
    pub fn resolve_value(&self, value: &str) -> String {
        let mut result = value.to_string();
        while let Some(start) = result.find("var(") {
            let rest = &result[start + 4..];
            let end = match rest.find(')') { Some(e) => e, None => break };
            let inner = rest[..end].trim();
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            let var_name = parts[0].trim();
            let fallback = parts.get(1).map(|s| s.trim().to_string());
            let resolved = self.resolve_var(var_name).or(fallback).unwrap_or_default();
            result = format!("{}{}{}", &result[..start], resolved, &result[start + 4 + end + 1..]);
        }
        result
    }
}
pub fn parse_calc(expr: &str) -> f32 {
    let s = expr.trim();
    let s = if s.starts_with("calc(") && s.ends_with(')') { &s[5..s.len()-1] } else { s };
    eval_calc_expr(s.trim())
}
fn eval_calc_expr(expr: &str) -> f32 {
    let expr = expr.trim();
    if let Some(inner) = strip_parens(expr) { return eval_calc_expr(inner); }
    let mut depth = 0i32;
    let bytes = expr.as_bytes();
    let mut last_add_sub = None;
    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'+' | b'-' if depth == 0 && i > 0 => {
                let prev = bytes[i - 1];
                if prev != b'*' && prev != b'/' && prev != b'(' {
                    last_add_sub = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    if let Some(pos) = last_add_sub {
        let left = eval_calc_expr(&expr[..pos]);
        let op = bytes[pos];
        let right = eval_calc_expr(&expr[pos+1..]);
        return if op == b'+' { left + right } else { left - right };
    }
    let mut last_mul_div = None;
    depth = 0;
    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'*' | b'/' if depth == 0 => { last_mul_div = Some(i); break; }
            _ => {}
        }
    }
    if let Some(pos) = last_mul_div {
        let left = eval_calc_expr(&expr[..pos]);
        let op = bytes[pos];
        let right = eval_calc_expr(&expr[pos+1..]);
        return if op == b'*' { left * right } else if right != 0.0 { left / right } else { 0.0 };
    }
    parse_calc_value(expr.trim())
}
fn strip_parens(s: &str) -> Option<&str> {
    let s = s.trim();
    if !s.starts_with('(') || !s.ends_with(')') { return None; }
    let mut depth = 0;
    for (i, b) in s.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 && i < s.len() - 1 { return None; }
            }
            _ => {}
        }
    }
    Some(&s[1..s.len()-1])
}
fn parse_calc_value(s: &str) -> f32 {
    let s = s.trim();
    if s.ends_with("px") { return s[..s.len()-2].trim().parse().unwrap_or(0.0); }
    if s.ends_with("em") { return s[..s.len()-2].trim().parse::<f32>().unwrap_or(0.0) * 16.0; }
    if s.ends_with("rem") { return s[..s.len()-3].trim().parse::<f32>().unwrap_or(0.0) * 16.0; }
    if s.ends_with('%') { return s[..s.len()-1].trim().parse::<f32>().unwrap_or(0.0) / 100.0; }
    if s.ends_with("vh") { return s[..s.len()-2].trim().parse::<f32>().unwrap_or(0.0); }
    if s.ends_with("vw") { return s[..s.len()-2].trim().parse::<f32>().unwrap_or(0.0); }
    s.parse().unwrap_or(0.0)
}
pub fn parse_transform(s: &str) -> Transform {
    let mut result = Transform::default();
    let mut rest = s.trim();
    while !rest.is_empty() {
        let paren = match rest.find('(') { Some(p) => p, None => break };
        let func = rest[..paren].trim();
        let close = match rest[paren..].find(')') { Some(c) => paren + c, None => break };
        let args_str = &rest[paren+1..close];
        let args: Vec<f32> = args_str.split(|c: char| c == ',' || c == ' ')
            .filter(|a| !a.is_empty())
            .map(|a| parse_calc_value(a.trim()))
            .collect();
        let t = match func {
            "translate" | "translateX" | "translateY" => {
                let tx = args.first().copied().unwrap_or(0.0);
                let ty = if func == "translateY" { tx } else { args.get(1).copied().unwrap_or(0.0) };
                let tx = if func == "translateY" { 0.0 } else { tx };
                Transform::translate(tx, ty)
            }
            "rotate" => Transform::rotate(args.first().copied().unwrap_or(0.0)),
            "scale" | "scaleX" | "scaleY" => {
                let sx = args.first().copied().unwrap_or(1.0);
                let sy = if func == "scaleX" { 1.0 } else if func == "scaleY" { sx } else { args.get(1).copied().unwrap_or(sx) };
                let sx = if func == "scaleY" { 1.0 } else { sx };
                Transform::scale(sx, sy)
            }
            "skew" | "skewX" | "skewY" => {
                let ax = args.first().copied().unwrap_or(0.0);
                let ay = if func == "skewX" { 0.0 } else if func == "skewY" { ax } else { args.get(1).copied().unwrap_or(0.0) };
                let ax = if func == "skewY" { 0.0 } else { ax };
                Transform::skew(ax, ay)
            }
            "matrix" if args.len() >= 6 => Transform::matrix(args[0], args[1], args[2], args[3], args[4], args[5]),
            _ => Transform::default(),
        };
        result = result.compose(&t);
        rest = rest[close+1..].trim();
    }
    result
}
pub fn parse_gradient(s: &str) -> Option<Gradient> {
    let s = s.trim();
    if s.starts_with("linear-gradient(") && s.ends_with(')') {
        let inner = &s[16..s.len()-1];
        let parts: Vec<&str> = split_gradient_args(inner);
        let mut angle = 180.0_f32;
        let mut start = 0;
        if let Some(first) = parts.first() {
            let ft = first.trim();
            if ft.ends_with("deg") {
                angle = ft[..ft.len()-3].parse().unwrap_or(180.0);
                start = 1;
            } else if ft.starts_with("to ") {
                angle = match ft {
                    "to top" => 0.0, "to right" => 90.0, "to bottom" => 180.0, "to left" => 270.0,
                    "to top right" => 45.0, "to bottom right" => 135.0,
                    "to bottom left" => 225.0, "to top left" => 315.0,
                    _ => 180.0,
                };
                start = 1;
            }
        }
        let stop_count = parts.len() - start;
        let stops: Vec<(f32, [u8; 4])> = parts[start..].iter().enumerate().map(|(i, part)| {
            let (color, pos) = parse_color_stop(part.trim());
            let pos = pos.unwrap_or_else(|| if stop_count <= 1 { 0.0 } else { i as f32 / (stop_count - 1) as f32 });
            (pos, color)
        }).collect();
        Some(Gradient::Linear { angle_deg: angle, stops })
    } else if s.starts_with("radial-gradient(") && s.ends_with(')') {
        let inner = &s[16..s.len()-1];
        let parts: Vec<&str> = split_gradient_args(inner);
        let cx = 0.5_f32;
        let cy = 0.5_f32;
        let radius = 0.5_f32;
        let mut start = 0;
        if let Some(first) = parts.first() {
            if first.contains("at ") || first.contains("circle") || first.contains("ellipse") {
                start = 1;
            }
        }
        let stop_count = parts.len() - start;
        let stops: Vec<(f32, [u8; 4])> = parts[start..].iter().enumerate().map(|(i, part)| {
            let (color, pos) = parse_color_stop(part.trim());
            let pos = pos.unwrap_or_else(|| if stop_count <= 1 { 0.0 } else { i as f32 / (stop_count - 1) as f32 });
            (pos, color)
        }).collect();
        Some(Gradient::Radial { cx, cy, radius, stops })
    } else { None }
}
fn split_gradient_args(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => { result.push(&s[start..i]); start = i + 1; }
            _ => {}
        }
    }
    if start < s.len() { result.push(&s[start..]); }
    result
}
fn parse_color_stop(s: &str) -> ([u8; 4], Option<f32>) {
    let s = s.trim();
    if s.starts_with('#') {
        let parts: Vec<&str> = s.splitn(2, ' ').collect();
        let color = parse_hex_color(parts[0]);
        let pos = parts.get(1).and_then(|p| parse_stop_position(p.trim()));
        (color, pos)
    } else if s.starts_with("rgb") {
        let close = s.find(')').unwrap_or(s.len() - 1);
        let color_str = &s[..=close];
        let color = parse_rgb_color(color_str);
        let rest = s[close+1..].trim();
        let pos = if rest.is_empty() { None } else { parse_stop_position(rest) };
        (color, pos)
    } else {
        let color = match s.split_whitespace().next().unwrap_or("") {
            "red" => [255, 0, 0, 255], "green" => [0, 128, 0, 255], "blue" => [0, 0, 255, 255],
            "white" => [255, 255, 255, 255], "black" => [0, 0, 0, 255], "yellow" => [255, 255, 0, 255],
            "cyan" => [0, 255, 255, 255], "magenta" => [255, 0, 255, 255], "orange" => [255, 165, 0, 255],
            "transparent" => [0, 0, 0, 0],
            _ => [0, 0, 0, 255],
        };
        let parts: Vec<&str> = s.splitn(2, ' ').collect();
        let pos = parts.get(1).and_then(|p| parse_stop_position(p.trim()));
        (color, pos)
    }
}
fn parse_stop_position(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with('%') { s[..s.len()-1].parse::<f32>().ok().map(|v| v / 100.0) }
    else if s.ends_with("px") { s[..s.len()-2].parse::<f32>().ok() }
    else { s.parse::<f32>().ok() }
}
fn parse_hex_color(s: &str) -> [u8; 4] {
    let h = if s.starts_with('#') { &s[1..] } else { s };
    match h.len() {
        3 => [
            u8::from_str_radix(&h[0..1].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&h[1..2].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&h[2..3].repeat(2), 16).unwrap_or(0), 255,
        ],
        6 => [
            u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&h[4..6], 16).unwrap_or(0), 255,
        ],
        8 => [
            u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
            u8::from_str_radix(&h[6..8], 16).unwrap_or(0),
        ],
        _ => [0, 0, 0, 255],
    }
}
fn parse_rgb_color(s: &str) -> [u8; 4] {
    let start = s.find('(').unwrap_or(0) + 1;
    let end = s.find(')').unwrap_or(s.len());
    let nums: Vec<f32> = s[start..end].split(|c: char| c == ',' || c == ' ' || c == '/')
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.trim().trim_end_matches('%').parse().ok())
        .collect();
    let r = nums.first().copied().unwrap_or(0.0) as u8;
    let g = nums.get(1).copied().unwrap_or(0.0) as u8;
    let b = nums.get(2).copied().unwrap_or(0.0) as u8;
    let a = nums.get(3).map(|v| if *v <= 1.0 { (*v * 255.0) as u8 } else { *v as u8 }).unwrap_or(255);
    [r, g, b, a]
}
pub fn parse_box_shadow(s: &str) -> Option<BoxShadow> {
    let s = s.trim();
    if s.is_empty() || s == "none" { return None; }
    let inset = s.contains("inset");
    let s = s.replace("inset", "");
    let mut color = [0u8, 0, 0, 255];
    let mut nums = Vec::new();
    let mut rest = s.trim();
    if let Some(rgb_start) = rest.find("rgb") {
        let rgb_end = rest[rgb_start..].find(')').map(|e| rgb_start + e + 1).unwrap_or(rest.len());
        color = parse_rgb_color(&rest[rgb_start..rgb_end]);
        rest = &rest[..rgb_start];
    }
    for tok in rest.split_whitespace() {
        if tok.starts_with('#') { color = parse_hex_color(tok); }
        else if let Some(v) = tok.strip_suffix("px").and_then(|n| n.parse::<f32>().ok()) { nums.push(v); }
        else if let Ok(v) = tok.parse::<f32>() { nums.push(v); }
        else {
            match tok {
                "red" => color = [255, 0, 0, 255], "green" => color = [0, 128, 0, 255],
                "blue" => color = [0, 0, 255, 255], "white" => color = [255, 255, 255, 255],
                "black" => color = [0, 0, 0, 255], "transparent" => color = [0, 0, 0, 0],
                _ => {}
            }
        }
    }
    Some(BoxShadow {
        offset_x: nums.first().copied().unwrap_or(0.0),
        offset_y: nums.get(1).copied().unwrap_or(0.0),
        blur: nums.get(2).copied().unwrap_or(0.0),
        spread: nums.get(3).copied().unwrap_or(0.0),
        color, inset,
    })
}
pub fn parse_filter(s: &str) -> Vec<Filter> {
    let mut filters = Vec::new();
    let mut rest = s.trim();
    while !rest.is_empty() {
        let paren = match rest.find('(') { Some(p) => p, None => break };
        let func = rest[..paren].trim();
        let close = match rest[paren..].find(')') { Some(c) => paren + c, None => break };
        let arg_str = rest[paren+1..close].trim();
        let val: f32 = if arg_str.ends_with('%') {
            arg_str[..arg_str.len()-1].parse::<f32>().unwrap_or(100.0) / 100.0
        } else if arg_str.ends_with("deg") {
            arg_str[..arg_str.len()-3].parse().unwrap_or(0.0)
        } else if arg_str.ends_with("px") {
            arg_str[..arg_str.len()-2].parse().unwrap_or(0.0)
        } else {
            arg_str.parse().unwrap_or(0.0)
        };
        match func {
            "blur" => filters.push(Filter::Blur(val)),
            "brightness" => filters.push(Filter::Brightness(val)),
            "contrast" => filters.push(Filter::Contrast(val)),
            "grayscale" => filters.push(Filter::Grayscale(val)),
            "sepia" => filters.push(Filter::Sepia(val)),
            "saturate" => filters.push(Filter::Saturate(val)),
            "hue-rotate" => filters.push(Filter::HueRotate(val)),
            "invert" => filters.push(Filter::Invert(val)),
            "opacity" => filters.push(Filter::Opacity(val)),
            "drop-shadow" => {
                if let Some(shadow) = parse_box_shadow(arg_str) {
                    filters.push(Filter::DropShadow(shadow));
                }
            }
            _ => {}
        }
        rest = rest[close+1..].trim();
    }
    filters
}
pub fn parse_transition(s: &str) -> Vec<Transition> {
    s.split(',').filter_map(|part| {
        let tokens: Vec<&str> = part.trim().split_whitespace().collect();
        if tokens.is_empty() { return None; }
        let property = tokens[0].to_string();
        let duration_ms = tokens.get(1).map(|t| parse_time_ms(t)).unwrap_or(0.0);
        let timing = tokens.get(2).map(|t| parse_timing_fn(t)).unwrap_or(TimingFunction::Ease);
        let delay_ms = tokens.get(3).map(|t| parse_time_ms(t)).unwrap_or(0.0);
        Some(Transition { property, duration_ms, delay_ms, timing })
    }).collect()
}
fn parse_time_ms(s: &str) -> f32 {
    if s.ends_with("ms") { s[..s.len()-2].parse().unwrap_or(0.0) }
    else if s.ends_with('s') { s[..s.len()-1].parse::<f32>().unwrap_or(0.0) * 1000.0 }
    else { s.parse().unwrap_or(0.0) }
}
fn parse_timing_fn(s: &str) -> TimingFunction {
    match s {
        "linear" => TimingFunction::Linear,
        "ease" => TimingFunction::Ease,
        "ease-in" => TimingFunction::EaseIn,
        "ease-out" => TimingFunction::EaseOut,
        "ease-in-out" => TimingFunction::EaseInOut,
        _ if s.starts_with("cubic-bezier(") && s.ends_with(')') => {
            let inner = &s[13..s.len()-1];
            let vals: Vec<f32> = inner.split(',').filter_map(|v| v.trim().parse().ok()).collect();
            if vals.len() == 4 { TimingFunction::CubicBezier(vals[0], vals[1], vals[2], vals[3]) }
            else { TimingFunction::Ease }
        }
        _ => TimingFunction::Ease,
    }
}
#[derive(Debug, Clone)]
pub struct MediaQueryList {
    pub queries: Vec<MediaQuery>,
}
impl MediaQueryList {
    pub fn evaluate_all(&self, vw: f32, vh: f32, is_dark: bool) -> bool {
        if self.queries.is_empty() { return true; }
        self.queries.iter().any(|q| q.evaluate(vw, vh, is_dark))
    }
}
fn parse_media_condition(cond: &str) -> Option<MediaCondition> {
    let cond = cond.trim().trim_start_matches('(').trim_end_matches(')').trim();
    let parts: Vec<&str> = cond.splitn(2, ':').collect();
    if parts.len() != 2 {
        if cond == "prefers-reduced-motion" || cond == "prefers-reduced-motion: reduce" {
            return Some(MediaCondition::PrefersReducedMotion);
        }
        return None;
    }
    let prop = parts[0].trim();
    let val = parts[1].trim();
    match prop {
        "min-width" => parse_px_value(val).map(MediaCondition::MinWidth),
        "max-width" => parse_px_value(val).map(MediaCondition::MaxWidth),
        "min-height" => parse_px_value(val).map(MediaCondition::MinHeight),
        "max-height" => parse_px_value(val).map(MediaCondition::MaxHeight),
        "orientation" => match val {
            "portrait" => Some(MediaCondition::Orientation(Orientation::Portrait)),
            "landscape" => Some(MediaCondition::Orientation(Orientation::Landscape)),
            _ => None,
        },
        "prefers-color-scheme" => match val {
            "dark" => Some(MediaCondition::PrefersColorScheme(ColorScheme::Dark)),
            "light" => Some(MediaCondition::PrefersColorScheme(ColorScheme::Light)),
            _ => None,
        },
        "prefers-reduced-motion" => Some(MediaCondition::PrefersReducedMotion),
        _ => None,
    }
}
fn parse_px_value(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with("px") { s[..s.len()-2].trim().parse().ok() }
    else { s.parse().ok() }
}
fn parse_media_query_str(s: &str) -> MediaQuery {
    let mut conditions = Vec::new();
    for part in s.split(" and ") {
        let part = part.trim();
        if part == "screen" || part == "all" || part == "print" { continue; }
        if let Some(c) = parse_media_condition(part) { conditions.push(c); }
    }
    MediaQuery { conditions }
}
pub fn evaluate_media_queries(css: &str, viewport_width: f32, viewport_height: f32, _dpr: f32) -> String {
    let mut result = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if i + 6 < len && &css[i..i+6] == "@media" {
            let media_start = i;
            i += 6;
            while i < len && bytes[i] != b'{' { i += 1; }
            if i >= len { result.push_str(&css[media_start..]); break; }
            let condition_str = css[media_start+6..i].trim();
            let query = parse_media_query_str(condition_str);
            i += 1;
            let mut depth = 1i32;
            let block_start = i;
            while i < len && depth > 0 {
                if bytes[i] == b'{' { depth += 1; }
                else if bytes[i] == b'}' { depth -= 1; }
                if depth > 0 { i += 1; }
            }
            let block_end = i;
            if i < len { i += 1; }
            let matches = query.evaluate(viewport_width, viewport_height, true);
            if matches {
                result.push_str(&css[block_start..block_end]);
                result.push('\n');
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_media_min_width_match() {
        let css = "body { color: red; } @media (min-width: 768px) { .wide { display: block; } }";
        let out = evaluate_media_queries(css, 1024.0, 768.0, 1.0);
        assert!(out.contains(".wide"));
        assert!(out.contains("display: block"));
    }
    #[test]
    fn test_media_min_width_no_match() {
        let css = "@media (min-width: 768px) { .wide { display: block; } }";
        let out = evaluate_media_queries(css, 500.0, 768.0, 1.0);
        assert!(!out.contains(".wide"));
    }
    #[test]
    fn test_media_dark_scheme() {
        let css = "@media (prefers-color-scheme: dark) { body { background: #000; } }";
        let out = evaluate_media_queries(css, 1024.0, 768.0, 1.0);
        assert!(out.contains("background: #000"));
    }
    #[test]
    fn test_media_orientation_landscape() {
        let css = "@media (orientation: landscape) { .land { margin: 0; } }";
        let out = evaluate_media_queries(css, 1024.0, 768.0, 1.0);
        assert!(out.contains(".land"));
        let out2 = evaluate_media_queries(css, 768.0, 1024.0, 1.0);
        assert!(!out2.contains(".land"));
    }
    #[test]
    fn test_passthrough_non_media() {
        let css = "h1 { font-size: 2em; }";
        let out = evaluate_media_queries(css, 800.0, 600.0, 1.0);
        assert!(out.contains("h1 { font-size: 2em; }"));
    }
}
