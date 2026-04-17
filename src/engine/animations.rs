use std::collections::HashMap;
use super::css_advanced::{TimingFunction, AnimationDirection, FillMode};

#[derive(Debug, Clone, PartialEq)]
pub enum PlayState { Running, Paused }

#[derive(Debug, Clone)]
pub struct AnimationUpdate {
    pub node_id: usize,
    pub property: String,
    pub value: f32,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct ActiveTransition {
    pub node_id: usize,
    pub property: String,
    pub from_value: f32,
    pub to_value: f32,
    pub duration_ms: f64,
    pub delay_ms: f64,
    pub elapsed_ms: f64,
    pub timing: TimingFunction,
}

impl ActiveTransition {
    pub fn interpolate(&self) -> f32 {
        if self.elapsed_ms < self.delay_ms { return self.from_value; }
        let active = self.elapsed_ms - self.delay_ms;
        if self.duration_ms <= 0.0 { return self.to_value; }
        let t = (active / self.duration_ms).clamp(0.0, 1.0) as f32;
        let eased = self.timing.interpolate(t);
        self.from_value + (self.to_value - self.from_value) * eased
    }

    fn is_complete(&self) -> bool {
        self.elapsed_ms >= self.delay_ms + self.duration_ms
    }
}

#[derive(Debug, Clone)]
pub struct ActiveAnimation {
    pub node_id: usize,
    pub name: String,
    pub duration_ms: f64,
    pub delay_ms: f64,
    pub elapsed_ms: f64,
    pub iteration_count: f32,
    pub current_iteration: u32,
    pub direction: AnimationDirection,
    pub timing: TimingFunction,
    pub fill_mode: FillMode,
    pub play_state: PlayState,
}

impl ActiveAnimation {
    pub fn progress(&self) -> f32 {
        if self.elapsed_ms < self.delay_ms {
            return match self.fill_mode {
                FillMode::Backwards | FillMode::Both => 0.0,
                _ => 0.0,
            };
        }
        let active = self.elapsed_ms - self.delay_ms;
        if self.duration_ms <= 0.0 { return 1.0; }
        let total_duration = self.duration_ms * self.iteration_count as f64;
        let clamped = if self.iteration_count.is_infinite() { active } else { active.min(total_duration) };
        let iter_progress = (clamped % self.duration_ms) / self.duration_ms;
        let iteration = (clamped / self.duration_ms) as u32;
        let t = iter_progress.clamp(0.0, 1.0) as f32;
        let directed = match self.direction {
            AnimationDirection::Normal => t,
            AnimationDirection::Reverse => 1.0 - t,
            AnimationDirection::Alternate => {
                if iteration % 2 == 0 { t } else { 1.0 - t }
            }
            AnimationDirection::AlternateReverse => {
                if iteration % 2 == 0 { 1.0 - t } else { t }
            }
        };
        self.timing.interpolate(directed)
    }

    fn is_complete(&self) -> bool {
        if self.iteration_count.is_infinite() { return false; }
        let active = self.elapsed_ms - self.delay_ms;
        if active < 0.0 { return false; }
        active >= self.duration_ms * self.iteration_count as f64
    }
}

#[derive(Debug, Clone)]
pub struct KeyframeStop {
    pub offset: f32,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Keyframes {
    pub name: String,
    pub stops: Vec<KeyframeStop>,
}

impl Keyframes {
    pub fn interpolate_property(&self, property: &str, progress: f32) -> Option<f32> {
        let mut before: Option<&KeyframeStop> = None;
        let mut after: Option<&KeyframeStop> = None;
        for stop in &self.stops {
            if stop.offset <= progress { before = Some(stop); }
            if stop.offset >= progress && after.is_none() { after = Some(stop); }
        }
        let b = before?;
        let a = after.unwrap_or(b);
        let bv = b.properties.get(property)?;
        let av = a.properties.get(property)?;
        let bn: f32 = bv.trim_end_matches("px").parse().ok()?;
        let an: f32 = av.trim_end_matches("px").parse().ok()?;
        if (a.offset - b.offset).abs() < 0.0001 { return Some(bn); }
        let local_t = (progress - b.offset) / (a.offset - b.offset);
        Some(bn + (an - bn) * local_t)
    }

    pub fn properties(&self) -> Vec<String> {
        let mut props = std::collections::HashSet::new();
        for stop in &self.stops {
            for key in stop.properties.keys() { props.insert(key.clone()); }
        }
        props.into_iter().collect()
    }
}

pub struct AnimationController {
    transitions: HashMap<usize, Vec<ActiveTransition>>,
    animations: HashMap<usize, Vec<ActiveAnimation>>,
    keyframes: HashMap<String, Keyframes>,
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            animations: HashMap::new(),
            keyframes: HashMap::new(),
        }
    }

    pub fn register_keyframes(&mut self, name: &str, css: &str) {
        let stops = parse_keyframe_block(css);
        self.keyframes.insert(name.to_string(), Keyframes { name: name.to_string(), stops });
    }

    pub fn start_transition(
        &mut self, node_id: usize, property: &str, from: f32, to: f32,
        duration_ms: f64, delay_ms: f64, timing: TimingFunction,
    ) {
        let list = self.transitions.entry(node_id).or_default();
        list.retain(|t| t.property != property);
        list.push(ActiveTransition {
            node_id, property: property.to_string(), from_value: from, to_value: to,
            duration_ms, delay_ms, elapsed_ms: 0.0, timing,
        });
    }

    pub fn start_animation(
        &mut self, node_id: usize, name: &str, duration_ms: f64, delay_ms: f64,
        iteration_count: f32, direction: AnimationDirection, timing: TimingFunction,
        fill_mode: FillMode,
    ) {
        let list = self.animations.entry(node_id).or_default();
        list.retain(|a| a.name != name);
        list.push(ActiveAnimation {
            node_id, name: name.to_string(), duration_ms, delay_ms, elapsed_ms: 0.0,
            iteration_count, current_iteration: 0, direction, timing, fill_mode,
            play_state: PlayState::Running,
        });
    }

    pub fn tick(&mut self, delta_ms: f64) -> Vec<AnimationUpdate> {
        let mut updates = Vec::new();
        for (_node_id, trans) in &mut self.transitions {
            for t in trans.iter_mut() {
                t.elapsed_ms += delta_ms;
                updates.push(AnimationUpdate {
                    node_id: t.node_id,
                    property: t.property.clone(),
                    value: t.interpolate(),
                    completed: t.is_complete(),
                });
            }
        }
        for (node_id, anims) in &mut self.animations {
            for anim in anims.iter_mut() {
                if anim.play_state == PlayState::Paused { continue; }
                anim.elapsed_ms += delta_ms;
                let progress = anim.progress();
                if anim.elapsed_ms > anim.delay_ms {
                    let active = anim.elapsed_ms - anim.delay_ms;
                    anim.current_iteration = (active / anim.duration_ms) as u32;
                }
                let completed = anim.is_complete();
                if let Some(kf) = self.keyframes.get(&anim.name) {
                    for prop in kf.properties() {
                        if let Some(val) = kf.interpolate_property(&prop, progress) {
                            updates.push(AnimationUpdate {
                                node_id: *node_id, property: prop, value: val, completed,
                            });
                        }
                    }
                } else {
                    updates.push(AnimationUpdate {
                        node_id: *node_id, property: anim.name.clone(),
                        value: progress, completed,
                    });
                }
            }
        }
        for trans in self.transitions.values_mut() {
            trans.retain(|t| !t.is_complete());
        }
        self.transitions.retain(|_, v| !v.is_empty());
        for anims in self.animations.values_mut() {
            anims.retain(|a| !a.is_complete());
        }
        self.animations.retain(|_, v| !v.is_empty());
        updates
    }

    pub fn has_active_animations(&self) -> bool {
        !self.transitions.is_empty() || !self.animations.is_empty()
    }

    pub fn cancel_animations(&mut self, node_id: usize) {
        self.animations.remove(&node_id);
    }

    pub fn cancel_transitions(&mut self, node_id: usize) {
        self.transitions.remove(&node_id);
    }

    pub fn active_count(&self) -> usize {
        self.transitions.values().map(|v| v.len()).sum::<usize>()
            + self.animations.values().map(|v| v.len()).sum::<usize>()
    }
}

pub fn evaluate(easing: &TimingFunction, t: f32) -> f32 {
    easing.interpolate(t)
}

fn parse_keyframe_block(css: &str) -> Vec<KeyframeStop> {
    let mut stops = Vec::new();
    let css = css.trim();
    let mut rest = css;
    while !rest.is_empty() {
        rest = rest.trim();
        if rest.is_empty() { break; }
        let brace = match rest.find('{') { Some(i) => i, None => break };
        let selector = rest[..brace].trim();
        let close = match rest[brace..].find('}') { Some(i) => brace + i, None => break };
        let body = rest[brace + 1..close].trim();
        let offsets = parse_keyframe_selector(selector);
        let properties = parse_declarations(body);
        for offset in offsets {
            stops.push(KeyframeStop { offset, properties: properties.clone() });
        }
        rest = &rest[close + 1..];
    }
    stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap_or(std::cmp::Ordering::Equal));
    stops
}

fn parse_keyframe_selector(s: &str) -> Vec<f32> {
    s.split(',').filter_map(|part| {
        let part = part.trim();
        match part {
            "from" => Some(0.0),
            "to" => Some(1.0),
            _ if part.ends_with('%') => {
                part[..part.len() - 1].trim().parse::<f32>().ok().map(|v| v / 100.0)
            }
            _ => part.parse::<f32>().ok().map(|v| v / 100.0),
        }
    }).collect()
}

fn parse_declarations(body: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for decl in body.split(';') {
        let decl = decl.trim();
        if decl.is_empty() { continue; }
        if let Some(colon) = decl.find(':') {
            let prop = decl[..colon].trim().to_string();
            let val = decl[colon + 1..].trim().to_string();
            map.insert(prop, val);
        }
    }
    map
}

pub fn parse_keyframes_from_css(css: &str) -> Vec<Keyframes> {
    let mut result = Vec::new();
    let mut rest = css;
    while let Some(at_pos) = rest.find("@keyframes") {
        rest = &rest[at_pos + 10..];
        let trimmed = rest.trim_start();
        let name_end = trimmed.find(|c: char| c == '{' || c.is_whitespace())
            .unwrap_or(trimmed.len());
        let name = trimmed[..name_end].trim().to_string();
        let brace_start = match trimmed.find('{') { Some(i) => i, None => break };
        let after_brace = &trimmed[brace_start + 1..];
        let mut depth = 1i32;
        let mut block_end = 0;
        for (i, ch) in after_brace.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 { block_end = i; break; }
                }
                _ => {}
            }
        }
        let block = &after_brace[..block_end];
        let stops = parse_keyframe_block(block);
        if !name.is_empty() {
            result.push(Keyframes { name, stops });
        }
        rest = &after_brace[block_end..];
        if rest.starts_with('}') { rest = &rest[1..]; }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interpolation() {
        let t = ActiveTransition {
            node_id: 1, property: "opacity".into(), from_value: 0.0, to_value: 1.0,
            duration_ms: 1000.0, delay_ms: 0.0, elapsed_ms: 500.0,
            timing: TimingFunction::Linear,
        };
        let val = t.interpolate();
        assert!((val - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_ease_timing_nonlinear() {
        let t = ActiveTransition {
            node_id: 1, property: "opacity".into(), from_value: 0.0, to_value: 1.0,
            duration_ms: 1000.0, delay_ms: 0.0, elapsed_ms: 500.0,
            timing: TimingFunction::Ease,
        };
        let val = t.interpolate();
        assert!(val != 0.5);
        assert!(val > 0.0 && val < 1.0);
    }

    #[test]
    fn test_transition_tick_completion() {
        let mut ctrl = AnimationController::new();
        ctrl.start_transition(1, "width", 0.0, 100.0, 200.0, 0.0, TimingFunction::Linear);
        assert_eq!(ctrl.active_count(), 1);
        let updates = ctrl.tick(100.0);
        assert_eq!(updates.len(), 1);
        assert!((updates[0].value - 50.0).abs() < 1.0);
        assert!(!updates[0].completed);
        let updates = ctrl.tick(100.0);
        assert_eq!(updates.len(), 1);
        assert!((updates[0].value - 100.0).abs() < 1.0);
        assert!(updates[0].completed);
        assert_eq!(ctrl.active_count(), 0);
    }

    #[test]
    fn test_animation_iteration() {
        let mut ctrl = AnimationController::new();
        ctrl.register_keyframes("slide", "from { left: 0px; } to { left: 100px; }");
        ctrl.start_animation(
            1, "slide", 100.0, 0.0, 3.0,
            AnimationDirection::Normal, TimingFunction::Linear, FillMode::None,
        );
        let updates = ctrl.tick(50.0);
        assert!(!updates.is_empty());
        assert!(!updates[0].completed);
        ctrl.tick(50.0);
        let updates = ctrl.tick(50.0);
        assert!(!updates.is_empty());
        let _ = ctrl.tick(200.0);
        assert_eq!(ctrl.active_count(), 0);
    }

    #[test]
    fn test_keyframe_parsing() {
        let css = r#"
            @keyframes fadeIn {
                0% { opacity: 0; }
                50% { opacity: 0.5; }
                100% { opacity: 1; }
            }
            @keyframes slide {
                from { left: 0px; }
                to { left: 200px; }
            }
        "#;
        let kfs = parse_keyframes_from_css(css);
        assert_eq!(kfs.len(), 2);
        assert_eq!(kfs[0].name, "fadeIn");
        assert_eq!(kfs[0].stops.len(), 3);
        assert!((kfs[0].stops[0].offset - 0.0).abs() < 0.001);
        assert!((kfs[0].stops[1].offset - 0.5).abs() < 0.001);
        assert!((kfs[0].stops[2].offset - 1.0).abs() < 0.001);
        assert_eq!(kfs[1].name, "slide");
        assert_eq!(kfs[1].stops.len(), 2);
        assert_eq!(kfs[1].stops[0].properties.get("left").unwrap(), "0px");
        assert_eq!(kfs[1].stops[1].properties.get("left").unwrap(), "200px");
    }

    #[test]
    fn test_delay_holds_from_value() {
        let t = ActiveTransition {
            node_id: 1, property: "left".into(), from_value: 10.0, to_value: 50.0,
            duration_ms: 100.0, delay_ms: 200.0, elapsed_ms: 100.0,
            timing: TimingFunction::Linear,
        };
        assert!((t.interpolate() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_cancel_transitions() {
        let mut ctrl = AnimationController::new();
        ctrl.start_transition(5, "opacity", 0.0, 1.0, 500.0, 0.0, TimingFunction::Linear);
        assert_eq!(ctrl.active_count(), 1);
        ctrl.cancel_transitions(5);
        assert_eq!(ctrl.active_count(), 0);
    }

    #[test]
    fn test_alternate_direction() {
        let anim = ActiveAnimation {
            node_id: 1, name: "bounce".into(), duration_ms: 100.0, delay_ms: 0.0,
            elapsed_ms: 150.0, iteration_count: 4.0, current_iteration: 1,
            direction: AnimationDirection::Alternate, timing: TimingFunction::Linear,
            fill_mode: FillMode::None, play_state: PlayState::Running,
        };
        let p = anim.progress();
        assert!(p >= 0.0 && p <= 1.0);
    }
}
