use std::collections::HashMap;
use super::paint::{PaintCommand, PaintRect};
#[derive(Debug, Clone, PartialEq)]
pub enum FormElementType {
    TextInput, Password, Email, Number, Search, Tel, Url,
    Textarea, Checkbox, Radio, Select, Button, Submit, Reset,
    File, Range, Color, Date, Hidden,
}
#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
}
#[derive(Debug, Clone)]
pub struct FormElement {
    pub id: String,
    pub element_type: FormElementType,
    pub name: String,
    pub value: String,
    pub placeholder: String,
    pub checked: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub options: Vec<SelectOption>,
    pub selected_index: usize,
    pub focus: bool,
    pub cursor_pos: usize,
    pub selection_start: usize,
    pub selection_end: usize,
    pub form_id: String,
    pub scroll_offset: f32,
}
impl FormElement {
    pub fn new(id: &str, element_type: FormElementType) -> Self {
        Self {
            id: id.to_string(), element_type, name: String::new(), value: String::new(),
            placeholder: String::new(), checked: false, disabled: false, readonly: false,
            min: 0.0, max: 100.0, step: 1.0, options: Vec::new(), selected_index: 0,
            focus: false, cursor_pos: 0, selection_start: 0, selection_end: 0,
            form_id: String::new(), scroll_offset: 0.0,
        }
    }
}
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: String,
}
pub struct FormState {
    pub elements: HashMap<String, FormElement>,
    pub active_form_id: Option<String>,
    pub focused_element_id: Option<String>,
}
impl FormState {
    pub fn new() -> Self {
        Self { elements: HashMap::new(), active_form_id: None, focused_element_id: None }
    }
    pub fn add_element(&mut self, elem: FormElement) {
        self.elements.insert(elem.id.clone(), elem);
    }
    pub fn handle_key_input(&mut self, element_id: &str, key: &str, shift: bool, ctrl: bool) {
        let elem = match self.elements.get_mut(element_id) { Some(e) => e, None => return };
        if elem.disabled || elem.readonly { return; }
        match elem.element_type {
            FormElementType::TextInput | FormElementType::Password | FormElementType::Email |
            FormElementType::Number | FormElementType::Search | FormElementType::Tel |
            FormElementType::Url | FormElementType::Textarea => {
                match key {
                    "Backspace" => {
                        if elem.selection_start != elem.selection_end {
                            let lo = elem.selection_start.min(elem.selection_end);
                            let hi = elem.selection_start.max(elem.selection_end);
                            elem.value = format!("{}{}", &elem.value[..lo], &elem.value[hi..]);
                            elem.cursor_pos = lo;
                            elem.selection_start = lo;
                            elem.selection_end = lo;
                        } else if elem.cursor_pos > 0 {
                            let mut chars: Vec<char> = elem.value.chars().collect();
                            if elem.cursor_pos <= chars.len() {
                                chars.remove(elem.cursor_pos - 1);
                                elem.value = chars.into_iter().collect();
                                elem.cursor_pos -= 1;
                            }
                        }
                    }
                    "Delete" => {
                        if elem.selection_start != elem.selection_end {
                            let lo = elem.selection_start.min(elem.selection_end);
                            let hi = elem.selection_start.max(elem.selection_end);
                            elem.value = format!("{}{}", &elem.value[..lo], &elem.value[hi..]);
                            elem.cursor_pos = lo;
                            elem.selection_start = lo;
                            elem.selection_end = lo;
                        } else {
                            let chars: Vec<char> = elem.value.chars().collect();
                            if elem.cursor_pos < chars.len() {
                                let mut chars = chars;
                                chars.remove(elem.cursor_pos);
                                elem.value = chars.into_iter().collect();
                            }
                        }
                    }
                    "ArrowLeft" => {
                        if elem.cursor_pos > 0 { elem.cursor_pos -= 1; }
                        if !shift { elem.selection_start = elem.cursor_pos; elem.selection_end = elem.cursor_pos; }
                        else { elem.selection_end = elem.cursor_pos; }
                    }
                    "ArrowRight" => {
                        let len = elem.value.chars().count();
                        if elem.cursor_pos < len { elem.cursor_pos += 1; }
                        if !shift { elem.selection_start = elem.cursor_pos; elem.selection_end = elem.cursor_pos; }
                        else { elem.selection_end = elem.cursor_pos; }
                    }
                    "Home" => {
                        elem.cursor_pos = 0;
                        if !shift { elem.selection_start = 0; elem.selection_end = 0; }
                        else { elem.selection_end = 0; }
                    }
                    "End" => {
                        elem.cursor_pos = elem.value.chars().count();
                        if !shift { elem.selection_start = elem.cursor_pos; elem.selection_end = elem.cursor_pos; }
                        else { elem.selection_end = elem.cursor_pos; }
                    }
                    "Enter" if elem.element_type == FormElementType::Textarea => {
                        let chars: Vec<char> = elem.value.chars().collect();
                        let (before, after) = chars.split_at(elem.cursor_pos.min(chars.len()));
                        elem.value = format!("{}\n{}", before.iter().collect::<String>(), after.iter().collect::<String>());
                        elem.cursor_pos += 1;
                    }
                    _ if ctrl && (key == "a" || key == "A") => {
                        elem.selection_start = 0;
                        elem.selection_end = elem.value.chars().count();
                        elem.cursor_pos = elem.selection_end;
                    }
                    _ if key.len() == 1 && !ctrl => {
                        let ch = key.chars().next().unwrap();
                        if elem.selection_start != elem.selection_end {
                            let lo = elem.selection_start.min(elem.selection_end);
                            let hi = elem.selection_start.max(elem.selection_end);
                            elem.value = format!("{}{}{}", &elem.value[..lo], ch, &elem.value[hi..]);
                            elem.cursor_pos = lo + 1;
                        } else {
                            let chars: Vec<char> = elem.value.chars().collect();
                            let (before, after) = chars.split_at(elem.cursor_pos.min(chars.len()));
                            elem.value = format!("{}{}{}", before.iter().collect::<String>(), ch, after.iter().collect::<String>());
                            elem.cursor_pos += 1;
                        }
                        elem.selection_start = elem.cursor_pos;
                        elem.selection_end = elem.cursor_pos;
                    }
                    _ => {}
                }
            }
            FormElementType::Checkbox => {
                if key == " " || key == "Enter" { elem.checked = !elem.checked; }
            }
            FormElementType::Radio => {
                if key == " " || key == "Enter" { elem.checked = true; }
            }
            FormElementType::Select => {
                match key {
                    "ArrowDown" => {
                        if elem.selected_index + 1 < elem.options.len() {
                            elem.selected_index += 1;
                            elem.value = elem.options[elem.selected_index].value.clone();
                        }
                    }
                    "ArrowUp" => {
                        if elem.selected_index > 0 {
                            elem.selected_index -= 1;
                            elem.value = elem.options[elem.selected_index].value.clone();
                        }
                    }
                    _ => {}
                }
            }
            FormElementType::Range => {
                match key {
                    "ArrowRight" | "ArrowUp" => {
                        let v: f32 = elem.value.parse().unwrap_or(elem.min);
                        let nv = (v + elem.step).min(elem.max);
                        elem.value = nv.to_string();
                    }
                    "ArrowLeft" | "ArrowDown" => {
                        let v: f32 = elem.value.parse().unwrap_or(elem.min);
                        let nv = (v - elem.step).max(elem.min);
                        elem.value = nv.to_string();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    pub fn handle_click(&mut self, element_id: &str, _x: f32, _y: f32) {
        if let Some(old_id) = self.focused_element_id.take() {
            if let Some(old) = self.elements.get_mut(&old_id) { old.focus = false; }
        }
        let elem = match self.elements.get_mut(element_id) { Some(e) => e, None => return };
        if elem.disabled { return; }
        elem.focus = true;
        self.focused_element_id = Some(element_id.to_string());
        match elem.element_type {
            FormElementType::Checkbox => { elem.checked = !elem.checked; }
            FormElementType::Radio => { elem.checked = true; }
            _ => {}
        }
    }
    pub fn handle_mouse_down(&mut self, element_id: &str, x: f32, _y: f32) {
        let elem = match self.elements.get_mut(element_id) { Some(e) => e, None => return };
        if elem.element_type == FormElementType::Range {
            let ratio = x.clamp(0.0, 1.0);
            let nv = elem.min + ratio * (elem.max - elem.min);
            let stepped = (((nv - elem.min) / elem.step).round() * elem.step + elem.min).clamp(elem.min, elem.max);
            elem.value = stepped.to_string();
        }
    }
    pub fn handle_mouse_up(&mut self, _element_id: &str, _x: f32, _y: f32) {}
    pub fn submit_form(&self, form_id: &str) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for elem in self.elements.values() {
            if elem.form_id != form_id { continue; }
            if elem.element_type == FormElementType::Hidden { result.insert(elem.name.clone(), elem.value.clone()); continue; }
            if elem.disabled { continue; }
            match elem.element_type {
                FormElementType::Checkbox => {
                    if elem.checked { result.insert(elem.name.clone(), if elem.value.is_empty() { "on".to_string() } else { elem.value.clone() }); }
                }
                FormElementType::Radio => {
                    if elem.checked { result.insert(elem.name.clone(), elem.value.clone()); }
                }
                FormElementType::Select => {
                    if let Some(opt) = elem.options.get(elem.selected_index) {
                        result.insert(elem.name.clone(), opt.value.clone());
                    }
                }
                FormElementType::Button | FormElementType::Reset => {}
                _ => { result.insert(elem.name.clone(), elem.value.clone()); }
            }
        }
        result
    }
    pub fn validate_element(&self, id: &str) -> ValidationResult {
        let elem = match self.elements.get(id) {
            Some(e) => e,
            None => return ValidationResult { valid: false, message: "Element not found".to_string() },
        };
        match elem.element_type {
            FormElementType::Email => {
                if elem.value.is_empty() { return ValidationResult { valid: true, message: String::new() }; }
                let valid = elem.value.contains('@') && elem.value.contains('.');
                ValidationResult { valid, message: if valid { String::new() } else { "Invalid email address".to_string() } }
            }
            FormElementType::Number => {
                if elem.value.is_empty() { return ValidationResult { valid: true, message: String::new() }; }
                match elem.value.parse::<f32>() {
                    Ok(v) => {
                        if v < elem.min { ValidationResult { valid: false, message: format!("Value must be at least {}", elem.min) } }
                        else if v > elem.max { ValidationResult { valid: false, message: format!("Value must be at most {}", elem.max) } }
                        else { ValidationResult { valid: true, message: String::new() } }
                    }
                    Err(_) => ValidationResult { valid: false, message: "Not a valid number".to_string() },
                }
            }
            FormElementType::Url => {
                if elem.value.is_empty() { return ValidationResult { valid: true, message: String::new() }; }
                let valid = elem.value.starts_with("http://") || elem.value.starts_with("https://");
                ValidationResult { valid, message: if valid { String::new() } else { "Invalid URL".to_string() } }
            }
            FormElementType::Tel => {
                if elem.value.is_empty() { return ValidationResult { valid: true, message: String::new() }; }
                let valid = elem.value.chars().all(|c| c.is_ascii_digit() || c == '+' || c == '-' || c == '(' || c == ')' || c == ' ');
                ValidationResult { valid, message: if valid { String::new() } else { "Invalid phone number".to_string() } }
            }
            _ => ValidationResult { valid: true, message: String::new() },
        }
    }
}
pub fn render_text_input(elem: &FormElement, x: f32, y: f32, w: f32, h: f32) -> Vec<PaintCommand> {
    let mut cmds = Vec::new();
    let bg = if elem.disabled { [220, 220, 220, 255] } else { [255, 255, 255, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h }, color: bg });
    let border_color = if elem.focus { [66, 133, 244, 255] } else { [180, 180, 180, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h: 1.0 }, color: border_color });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y: y + h - 1.0, w, h: 1.0 }, color: border_color });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: 1.0, h }, color: border_color });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: x + w - 1.0, y, w: 1.0, h }, color: border_color });
    let padding = 4.0;
    let font_size = (h - 8.0).max(10.0);
    if elem.value.is_empty() && !elem.placeholder.is_empty() {
        cmds.push(PaintCommand::DrawText {
            x: x + padding, y: y + padding, text: elem.placeholder.clone(),
            font_size, color: [160, 160, 160, 255], max_width: w - padding * 2.0,
        });
    } else {
        let display = if elem.element_type == FormElementType::Password {
            "\u{2022}".repeat(elem.value.chars().count())
        } else { elem.value.clone() };
        cmds.push(PaintCommand::DrawText {
            x: x + padding, y: y + padding, text: display,
            font_size, color: [0, 0, 0, 255], max_width: w - padding * 2.0,
        });
    }
    if elem.focus {
        let cursor_x = x + padding + (elem.cursor_pos as f32 * font_size * 0.6);
        cmds.push(PaintCommand::FillRect {
            rect: PaintRect { x: cursor_x, y: y + 2.0, w: 1.0, h: h - 4.0 },
            color: [0, 0, 0, 255],
        });
    }
    cmds
}
pub fn render_button(elem: &FormElement, x: f32, y: f32, w: f32, h: f32) -> Vec<PaintCommand> {
    let mut cmds = Vec::new();
    let bg = if elem.disabled { [200, 200, 200, 255] } else { [240, 240, 240, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h }, color: bg });
    let border = [160, 160, 160, 255];
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h: 1.0 }, color: [255, 255, 255, 200] });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y: y + h - 1.0, w, h: 1.0 }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: 1.0, h }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: x + w - 1.0, y, w: 1.0, h }, color: border });
    let label = if elem.value.is_empty() {
        match elem.element_type {
            FormElementType::Submit => "Submit".to_string(),
            FormElementType::Reset => "Reset".to_string(),
            _ => "Button".to_string(),
        }
    } else { elem.value.clone() };
    let font_size = (h - 8.0).max(10.0);
    let text_w = label.len() as f32 * font_size * 0.6;
    let text_x = x + (w - text_w) * 0.5;
    let text_y = y + (h - font_size) * 0.5;
    let color = if elem.disabled { [120, 120, 120, 255] } else { [0, 0, 0, 255] };
    cmds.push(PaintCommand::DrawText { x: text_x, y: text_y, text: label, font_size, color, max_width: w });
    cmds
}
pub fn render_checkbox(elem: &FormElement, x: f32, y: f32, size: f32) -> Vec<PaintCommand> {
    let mut cmds = Vec::new();
    let bg = if elem.disabled { [220, 220, 220, 255] } else { [255, 255, 255, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: size, h: size }, color: bg });
    let border = if elem.focus { [66, 133, 244, 255] } else { [120, 120, 120, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: size, h: 1.0 }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y: y + size - 1.0, w: size, h: 1.0 }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: 1.0, h: size }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: x + size - 1.0, y, w: 1.0, h: size }, color: border });
    if elem.checked {
        let inset = size * 0.2;
        let check_color = if elem.disabled { [120, 120, 120, 255] } else { [66, 133, 244, 255] };
        let cw = size - inset * 2.0;
        let ch_h = 2.0_f32;
        let cx = x + inset;
        let cy = y + size * 0.35;
        cmds.push(PaintCommand::FillRect { rect: PaintRect { x: cx, y: cy + cw * 0.4, w: cw * 0.4, h: ch_h }, color: check_color });
        cmds.push(PaintCommand::FillRect { rect: PaintRect { x: cx + cw * 0.25, y: cy, w: ch_h, h: cw * 0.65 }, color: check_color });
    }
    cmds
}
pub fn render_radio(elem: &FormElement, x: f32, y: f32, size: f32) -> Vec<PaintCommand> {
    let mut cmds = Vec::new();
    let bg = if elem.disabled { [220, 220, 220, 255] } else { [255, 255, 255, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: size, h: size }, color: bg });
    let border = if elem.focus { [66, 133, 244, 255] } else { [120, 120, 120, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: size, h: 1.0 }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y: y + size - 1.0, w: size, h: 1.0 }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: 1.0, h: size }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: x + size - 1.0, y, w: 1.0, h: size }, color: border });
    if elem.checked {
        let dot_size = size * 0.4;
        let dot_x = x + (size - dot_size) * 0.5;
        let dot_y = y + (size - dot_size) * 0.5;
        let dot_color = if elem.disabled { [120, 120, 120, 255] } else { [66, 133, 244, 255] };
        cmds.push(PaintCommand::FillRect { rect: PaintRect { x: dot_x, y: dot_y, w: dot_size, h: dot_size }, color: dot_color });
    }
    cmds
}
pub fn render_select(elem: &FormElement, x: f32, y: f32, w: f32, h: f32) -> Vec<PaintCommand> {
    let mut cmds = Vec::new();
    let bg = if elem.disabled { [220, 220, 220, 255] } else { [255, 255, 255, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h }, color: bg });
    let border = if elem.focus { [66, 133, 244, 255] } else { [180, 180, 180, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h: 1.0 }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y: y + h - 1.0, w, h: 1.0 }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: 1.0, h }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: x + w - 1.0, y, w: 1.0, h }, color: border });
    let display_text = elem.options.get(elem.selected_index)
        .map(|o| o.label.clone()).unwrap_or_else(|| elem.value.clone());
    let font_size = (h - 8.0).max(10.0);
    cmds.push(PaintCommand::DrawText {
        x: x + 4.0, y: y + (h - font_size) * 0.5, text: display_text,
        font_size, color: [0, 0, 0, 255], max_width: w - 24.0,
    });
    let arrow_x = x + w - 16.0;
    let arrow_y = y + h * 0.5 - 2.0;
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: arrow_x, y: arrow_y, w: 8.0, h: 2.0 }, color: [100, 100, 100, 255] });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: arrow_x + 2.0, y: arrow_y + 2.0, w: 4.0, h: 2.0 }, color: [100, 100, 100, 255] });
    cmds
}
pub fn render_textarea(elem: &FormElement, x: f32, y: f32, w: f32, h: f32) -> Vec<PaintCommand> {
    let mut cmds = Vec::new();
    let bg = if elem.disabled { [220, 220, 220, 255] } else { [255, 255, 255, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h }, color: bg });
    let border = if elem.focus { [66, 133, 244, 255] } else { [180, 180, 180, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h: 1.0 }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y: y + h - 1.0, w, h: 1.0 }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: 1.0, h }, color: border });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: x + w - 1.0, y, w: 1.0, h }, color: border });
    cmds.push(PaintCommand::PushClip { rect: PaintRect { x: x + 1.0, y: y + 1.0, w: w - 2.0, h: h - 2.0 } });
    let font_size = 14.0_f32;
    let line_height = font_size * 1.4;
    let padding = 4.0;
    let lines: Vec<&str> = elem.value.split('\n').collect();
    let text_color = [0, 0, 0, 255];
    for (i, line) in lines.iter().enumerate() {
        let ly = y + padding + (i as f32 * line_height) - elem.scroll_offset;
        if ly + line_height < y || ly > y + h { continue; }
        cmds.push(PaintCommand::DrawText {
            x: x + padding, y: ly, text: line.to_string(),
            font_size, color: text_color, max_width: w - padding * 2.0,
        });
    }
    cmds.push(PaintCommand::PopClip);
    if elem.focus {
        let mut char_count = 0;
        let mut cursor_line = 0;
        let mut cursor_col = 0;
        for (i, line) in lines.iter().enumerate() {
            let line_len = line.chars().count();
            if char_count + line_len >= elem.cursor_pos && (i == lines.len() - 1 || char_count + line_len >= elem.cursor_pos) {
                cursor_line = i;
                cursor_col = elem.cursor_pos - char_count;
                break;
            }
            char_count += line_len + 1;
        }
        let cx = x + padding + cursor_col as f32 * font_size * 0.6;
        let cy = y + padding + cursor_line as f32 * line_height - elem.scroll_offset;
        cmds.push(PaintCommand::FillRect {
            rect: PaintRect { x: cx, y: cy, w: 1.0, h: font_size },
            color: [0, 0, 0, 255],
        });
    }
    cmds
}
pub fn render_range(elem: &FormElement, x: f32, y: f32, w: f32, h: f32) -> Vec<PaintCommand> {
    let mut cmds = Vec::new();
    let track_h = 4.0;
    let track_y = y + (h - track_h) * 0.5;
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y: track_y, w, h: track_h }, color: [200, 200, 200, 255] });
    let val: f32 = elem.value.parse().unwrap_or(elem.min);
    let ratio = if (elem.max - elem.min).abs() > 0.0001 { (val - elem.min) / (elem.max - elem.min) } else { 0.0 };
    let filled_w = w * ratio.clamp(0.0, 1.0);
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y: track_y, w: filled_w, h: track_h }, color: [66, 133, 244, 255] });
    let thumb_w = 16.0;
    let thumb_h = 16.0;
    let thumb_x = x + filled_w - thumb_w * 0.5;
    let thumb_y = y + (h - thumb_h) * 0.5;
    let thumb_color = if elem.focus { [50, 100, 200, 255] } else { [66, 133, 244, 255] };
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: thumb_x, y: thumb_y, w: thumb_w, h: thumb_h }, color: thumb_color });
    cmds
}
pub fn render_progress(value: f32, max: f32, x: f32, y: f32, w: f32, h: f32) -> Vec<PaintCommand> {
    let mut cmds = Vec::new();
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h }, color: [230, 230, 230, 255] });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w, h: 1.0 }, color: [180, 180, 180, 255] });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y: y + h - 1.0, w, h: 1.0 }, color: [180, 180, 180, 255] });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x, y, w: 1.0, h }, color: [180, 180, 180, 255] });
    cmds.push(PaintCommand::FillRect { rect: PaintRect { x: x + w - 1.0, y, w: 1.0, h }, color: [180, 180, 180, 255] });
    let ratio = if max > 0.0 { (value / max).clamp(0.0, 1.0) } else { 0.0 };
    let fill_w = (w - 2.0) * ratio;
    if fill_w > 0.0 {
        cmds.push(PaintCommand::FillRect { rect: PaintRect { x: x + 1.0, y: y + 1.0, w: fill_w, h: h - 2.0 }, color: [66, 133, 244, 255] });
    }
    cmds
}
