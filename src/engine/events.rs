use std::collections::HashMap;
use super::layout::LayoutRect;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventPhase { None, Capture, AtTarget, Bubble }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Click, DblClick, MouseDown, MouseUp, MouseMove, MouseEnter, MouseLeave,
    MouseOver, MouseOut, KeyDown, KeyUp, KeyPress, Focus, Blur, FocusIn, FocusOut,
    Input, Change, Submit, Reset, Scroll, Resize, Load, Unload, Error,
    DOMContentLoaded, Wheel, ContextMenu, TouchStart, TouchEnd, TouchMove,
    DragStart, Drag, DragEnd, Drop, Paste, Copy, Cut,
}
impl EventType {
    pub fn bubbles(&self) -> bool {
        !matches!(self, EventType::Focus | EventType::Blur | EventType::Load |
            EventType::Unload | EventType::MouseEnter | EventType::MouseLeave |
            EventType::Resize | EventType::Error)
    }
    pub fn cancelable(&self) -> bool {
        matches!(self, EventType::Click | EventType::DblClick | EventType::MouseDown |
            EventType::MouseUp | EventType::KeyDown | EventType::KeyUp | EventType::KeyPress |
            EventType::ContextMenu | EventType::Submit | EventType::Wheel |
            EventType::TouchStart | EventType::TouchEnd | EventType::TouchMove |
            EventType::DragStart | EventType::Drag | EventType::Drop)
    }
}
#[derive(Debug, Clone)]
pub struct DomEvent {
    pub event_type: EventType,
    pub target: usize,
    pub current_target: usize,
    pub phase: EventPhase,
    pub bubbles: bool,
    pub cancelable: bool,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub immediate_propagation_stopped: bool,
    pub client_x: f32,
    pub client_y: f32,
    pub page_x: f32,
    pub page_y: f32,
    pub key: String,
    pub key_code: u32,
    pub shift_key: bool,
    pub ctrl_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub button: u8,
    pub detail: u32,
    pub timestamp: u64,
}
impl DomEvent {
    pub fn new(event_type: EventType, target: usize) -> Self {
        Self {
            bubbles: event_type.bubbles(), cancelable: event_type.cancelable(),
            event_type, target, current_target: target, phase: EventPhase::None,
            default_prevented: false, propagation_stopped: false,
            immediate_propagation_stopped: false, client_x: 0.0, client_y: 0.0,
            page_x: 0.0, page_y: 0.0, key: String::new(), key_code: 0,
            shift_key: false, ctrl_key: false, alt_key: false, meta_key: false,
            button: 0, detail: 0, timestamp: 0,
        }
    }
    pub fn mouse(event_type: EventType, target: usize, x: f32, y: f32, button: u8) -> Self {
        let mut e = Self::new(event_type, target);
        e.client_x = x; e.client_y = y; e.page_x = x; e.page_y = y; e.button = button;
        e
    }
    pub fn keyboard(event_type: EventType, target: usize, key: &str, code: u32, shift: bool, ctrl: bool, alt: bool, meta: bool) -> Self {
        let mut e = Self::new(event_type, target);
        e.key = key.to_string(); e.key_code = code;
        e.shift_key = shift; e.ctrl_key = ctrl; e.alt_key = alt; e.meta_key = meta;
        e
    }
    pub fn prevent_default(&mut self) { if self.cancelable { self.default_prevented = true; } }
    pub fn stop_propagation(&mut self) { self.propagation_stopped = true; }
    pub fn stop_immediate_propagation(&mut self) {
        self.propagation_stopped = true;
        self.immediate_propagation_stopped = true;
    }
}
#[derive(Debug, Clone)]
pub struct EventHandler {
    pub callback_id: u32,
    pub phase: EventPhase,
    pub once: bool,
}
pub struct EventDispatcher {
    handlers: HashMap<(usize, EventType), Vec<EventHandler>>,
    node_ancestors: HashMap<usize, Vec<usize>>,
    next_callback_id: u32,
}
impl EventDispatcher {
    pub fn new() -> Self {
        Self { handlers: HashMap::new(), node_ancestors: HashMap::new(), next_callback_id: 1 }
    }
    pub fn add_listener(&mut self, node_id: usize, event_type: EventType, use_capture: bool, once: bool) -> u32 {
        let id = self.next_callback_id;
        self.next_callback_id += 1;
        let phase = if use_capture { EventPhase::Capture } else { EventPhase::Bubble };
        let handler = EventHandler { callback_id: id, phase, once };
        self.handlers.entry((node_id, event_type)).or_insert_with(Vec::new).push(handler);
        id
    }
    pub fn remove_listener(&mut self, node_id: usize, event_type: EventType, callback_id: u32) {
        if let Some(handlers) = self.handlers.get_mut(&(node_id, event_type)) {
            handlers.retain(|h| h.callback_id != callback_id);
        }
    }
    pub fn dispatch(&mut self, event: &mut DomEvent) -> Vec<u32> {
        let mut fired = Vec::new();
        let path = self.build_path(event.target);
        let mut once_removals: Vec<(usize, EventType, u32)> = Vec::new();
        event.phase = EventPhase::Capture;
        for &node_id in &path[..path.len().saturating_sub(1)] {
            if event.propagation_stopped { break; }
            event.current_target = node_id;
            self.fire_handlers(node_id, event, EventPhase::Capture, &mut fired, &mut once_removals);
        }
        if !event.propagation_stopped {
            event.phase = EventPhase::AtTarget;
            event.current_target = event.target;
            self.fire_handlers(event.target, event, EventPhase::Capture, &mut fired, &mut once_removals);
            if !event.immediate_propagation_stopped {
                self.fire_handlers(event.target, event, EventPhase::Bubble, &mut fired, &mut once_removals);
            }
        }
        if event.bubbles && !event.propagation_stopped {
            event.phase = EventPhase::Bubble;
            for &node_id in path[..path.len().saturating_sub(1)].iter().rev() {
                if event.propagation_stopped { break; }
                event.current_target = node_id;
                self.fire_handlers(node_id, event, EventPhase::Bubble, &mut fired, &mut once_removals);
            }
        }
        for (node_id, event_type, callback_id) in once_removals {
            self.remove_listener(node_id, event_type, callback_id);
        }
        fired
    }
    fn fire_handlers(&self, node_id: usize, event: &mut DomEvent, phase: EventPhase, fired: &mut Vec<u32>, once_removals: &mut Vec<(usize, EventType, u32)>) {
        let handlers = match self.handlers.get(&(node_id, event.event_type)) { Some(h) => h.clone(), None => return };
        for handler in &handlers {
            if event.immediate_propagation_stopped { break; }
            if handler.phase == phase || (event.phase == EventPhase::AtTarget) {
                if handler.phase != phase && event.phase != EventPhase::AtTarget { continue; }
                fired.push(handler.callback_id);
                if handler.once { once_removals.push((node_id, event.event_type, handler.callback_id)); }
            }
        }
    }
    fn build_path(&self, target: usize) -> Vec<usize> {
        if let Some(ancestors) = self.node_ancestors.get(&target) {
            let mut path = ancestors.clone();
            path.push(target);
            path
        } else {
            vec![target]
        }
    }
    pub fn set_ancestors(&mut self, node_id: usize, ancestors: Vec<usize>) {
        self.node_ancestors.insert(node_id, ancestors);
    }
    pub fn clear_node(&mut self, node_id: usize) {
        let keys: Vec<(usize, EventType)> = self.handlers.keys()
            .filter(|(nid, _)| *nid == node_id).cloned().collect();
        for k in keys { self.handlers.remove(&k); }
        self.node_ancestors.remove(&node_id);
    }
}
pub struct FocusManager {
    pub current_focus: Option<usize>,
    pub tab_order: Vec<usize>,
}
impl FocusManager {
    pub fn new() -> Self { Self { current_focus: None, tab_order: Vec::new() } }
    pub fn focus(&mut self, node_id: usize, dispatcher: &mut EventDispatcher) -> Vec<DomEvent> {
        let mut events = Vec::new();
        if let Some(old) = self.current_focus {
            if old == node_id { return events; }
            let mut blur = DomEvent::new(EventType::Blur, old);
            dispatcher.dispatch(&mut blur);
            events.push(blur);
            let mut focus_out = DomEvent::new(EventType::FocusOut, old);
            dispatcher.dispatch(&mut focus_out);
            events.push(focus_out);
        }
        self.current_focus = Some(node_id);
        let mut focus = DomEvent::new(EventType::Focus, node_id);
        dispatcher.dispatch(&mut focus);
        events.push(focus);
        let mut focus_in = DomEvent::new(EventType::FocusIn, node_id);
        dispatcher.dispatch(&mut focus_in);
        events.push(focus_in);
        events
    }
    pub fn blur(&mut self, dispatcher: &mut EventDispatcher) -> Vec<DomEvent> {
        let mut events = Vec::new();
        if let Some(old) = self.current_focus.take() {
            let mut blur = DomEvent::new(EventType::Blur, old);
            dispatcher.dispatch(&mut blur);
            events.push(blur);
            let mut focus_out = DomEvent::new(EventType::FocusOut, old);
            dispatcher.dispatch(&mut focus_out);
            events.push(focus_out);
        }
        events
    }
    pub fn tab_next(&mut self, dispatcher: &mut EventDispatcher) -> Vec<DomEvent> {
        if self.tab_order.is_empty() { return Vec::new(); }
        let idx = self.current_focus.and_then(|id| self.tab_order.iter().position(|&t| t == id))
            .map(|i| (i + 1) % self.tab_order.len()).unwrap_or(0);
        self.focus(self.tab_order[idx], dispatcher)
    }
    pub fn tab_prev(&mut self, dispatcher: &mut EventDispatcher) -> Vec<DomEvent> {
        if self.tab_order.is_empty() { return Vec::new(); }
        let idx = self.current_focus.and_then(|id| self.tab_order.iter().position(|&t| t == id))
            .map(|i| if i == 0 { self.tab_order.len() - 1 } else { i - 1 }).unwrap_or(self.tab_order.len() - 1);
        self.focus(self.tab_order[idx], dispatcher)
    }
    pub fn set_tab_order(&mut self, ids: Vec<usize>) { self.tab_order = ids; }
}
pub struct HitTester;
impl HitTester {
    pub fn hit_test(x: f32, y: f32, layout_rects: &HashMap<usize, LayoutRect>) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (&node_id, rect) in layout_rects {
            if x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h {
                let area = rect.w * rect.h;
                match best {
                    Some((_, best_area)) if area < best_area => { best = Some((node_id, area)); }
                    None => { best = Some((node_id, area)); }
                    _ => {}
                }
            }
        }
        best.map(|(id, _)| id)
    }
}
