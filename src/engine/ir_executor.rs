/// IR Executor — Processes AmniIR instruction streams and produces paint output.
/// Translates AmniIR opcodes into PaintCommands for the software renderer,
/// or directly into wgpu draw calls when GPU-accelerated.

use super::amni_ir::{Instruction, Opcode, Operands, IrProgram};
use super::paint::{PaintCommand, PaintRect, DisplayList};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct IrNode {
    id: u32,
    tag_hash: u32,
    attrs: HashMap<String, String>,
    text: String,
    children: Vec<u32>,
    parent: Option<u32>,
}

#[derive(Debug, Clone)]
struct IrBox {
    x: f32, y: f32, w: f32, h: f32,
    margin: [f32; 4],
    padding: [f32; 4],
    border: [f32; 4],
    display: u8,
    overflow: u8,
}

impl Default for IrBox {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, w: 0.0, h: 0.0,
            margin: [0.0; 4], padding: [0.0; 4], border: [0.0; 4],
            display: 1, overflow: 0 }
    }
}

pub struct IrExecutor {
    nodes: HashMap<u32, IrNode>,
    boxes: HashMap<u32, IrBox>,
    textures: HashMap<u32, TextureRef>,
    layer_stack: Vec<u32>,
    clip_stack: Vec<PaintRect>,
    current_opacity: f32,
    viewport_w: f32,
    viewport_h: f32,
    scroll_x: f32,
    scroll_y: f32,
    frame_count: u32,
}

#[derive(Debug, Clone)]
pub struct TextureRef {
    pub id: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub display_list: DisplayList,
    pub node_count: usize,
    pub box_count: usize,
    pub paint_count: usize,
    pub frame_num: u32,
    pub navigate_requests: Vec<String>,
    pub resource_requests: Vec<(String, u8)>,
}

impl IrExecutor {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(), boxes: HashMap::new(),
            textures: HashMap::new(), layer_stack: Vec::new(),
            clip_stack: Vec::new(), current_opacity: 1.0,
            viewport_w: 1280.0, viewport_h: 720.0,
            scroll_x: 0.0, scroll_y: 0.0, frame_count: 0,
        }
    }

    pub fn execute(&mut self, program: &IrProgram) -> ExecutionResult {
        let mut dl = DisplayList::new(self.viewport_w as u32, self.viewport_h as u32);
        let mut navigate_requests = Vec::new();
        let mut resource_requests = Vec::new();
        let mut paint_count = 0usize;

        for inst in &program.instructions {
            match inst.op {
                Opcode::DocStart => {
                    if let Operands::DocStart { width, height } = &inst.operands {
                        self.viewport_w = *width;
                        self.viewport_h = *height;
                        dl = DisplayList::new(*width as u32, *height as u32);
                    }
                }
                Opcode::DocEnd => {}
                Opcode::NodeCreate => {
                    if let Operands::NodeCreate { node_id, tag_hash } = &inst.operands {
                        self.nodes.insert(*node_id, IrNode {
                            id: *node_id, tag_hash: *tag_hash,
                            attrs: HashMap::new(), text: String::new(),
                            children: Vec::new(), parent: None,
                        });
                    }
                }
                Opcode::NodeDestroy => {
                    if let Operands::Node { node_id } = &inst.operands {
                        self.nodes.remove(node_id);
                        self.boxes.remove(node_id);
                    }
                }
                Opcode::NodeAppend => {
                    if let Operands::NodeTree { parent_id, child_id } = &inst.operands {
                        if let Some(parent) = self.nodes.get_mut(parent_id) {
                            parent.children.push(*child_id);
                        }
                        if let Some(child) = self.nodes.get_mut(child_id) {
                            child.parent = Some(*parent_id);
                        }
                    }
                }
                Opcode::NodeInsert => {
                    if let Operands::NodeInsert { parent_id, child_id, index } = &inst.operands {
                        if let Some(parent) = self.nodes.get_mut(parent_id) {
                            let idx = (*index as usize).min(parent.children.len());
                            parent.children.insert(idx, *child_id);
                        }
                        if let Some(child) = self.nodes.get_mut(child_id) {
                            child.parent = Some(*parent_id);
                        }
                    }
                }
                Opcode::NodeRemove => {
                    if let Operands::NodeTree { parent_id, child_id } = &inst.operands {
                        if let Some(parent) = self.nodes.get_mut(parent_id) {
                            parent.children.retain(|c| c != child_id);
                        }
                    }
                }
                Opcode::AttrSet => {
                    if let Operands::AttrSet { node_id, key, value } = &inst.operands {
                        if let Some(node) = self.nodes.get_mut(node_id) {
                            node.attrs.insert(key.clone(), value.clone());
                        }
                    }
                }
                Opcode::AttrDel => {
                    if let Operands::AttrDel { node_id, key } = &inst.operands {
                        if let Some(node) = self.nodes.get_mut(node_id) { node.attrs.remove(key); }
                    }
                }
                Opcode::TextSet => {
                    if let Operands::TextContent { node_id, text } = &inst.operands {
                        if let Some(node) = self.nodes.get_mut(node_id) { node.text = text.clone(); }
                    }
                }
                Opcode::TextAppend => {
                    if let Operands::TextContent { node_id, text } = &inst.operands {
                        if let Some(node) = self.nodes.get_mut(node_id) { node.text.push_str(text); }
                    }
                }
                Opcode::BoxCreate | Opcode::BoxUpdate => {
                    if let Operands::BoxCreate { node_id, x, y, w, h } = &inst.operands {
                        let bx = self.boxes.entry(*node_id).or_insert_with(IrBox::default);
                        bx.x = *x; bx.y = *y; bx.w = *w; bx.h = *h;
                    }
                }
                Opcode::BoxMargin => {
                    if let Operands::BoxEdges { node_id, top, right, bottom, left } = &inst.operands {
                        let bx = self.boxes.entry(*node_id).or_insert_with(IrBox::default);
                        bx.margin = [*top, *right, *bottom, *left];
                    }
                }
                Opcode::BoxPadding => {
                    if let Operands::BoxEdges { node_id, top, right, bottom, left } = &inst.operands {
                        let bx = self.boxes.entry(*node_id).or_insert_with(IrBox::default);
                        bx.padding = [*top, *right, *bottom, *left];
                    }
                }
                Opcode::BoxBorder => {
                    if let Operands::BoxEdges { node_id, top, right, bottom, left } = &inst.operands {
                        let bx = self.boxes.entry(*node_id).or_insert_with(IrBox::default);
                        bx.border = [*top, *right, *bottom, *left];
                    }
                }
                Opcode::BoxDisplay => {
                    if let Operands::BoxMode { node_id, mode } = &inst.operands {
                        let bx = self.boxes.entry(*node_id).or_insert_with(IrBox::default);
                        bx.display = *mode;
                    }
                }
                Opcode::BoxOverflow => {
                    if let Operands::BoxMode { node_id, mode } = &inst.operands {
                        let bx = self.boxes.entry(*node_id).or_insert_with(IrBox::default);
                        bx.overflow = *mode;
                    }
                }
                // Paint instructions → display list
                Opcode::PaintRect => {
                    if let Operands::PaintRect { x, y, w, h, color } = &inst.operands {
                        dl.push(PaintCommand::FillRect {
                            rect: PaintRect { x: *x - self.scroll_x, y: *y - self.scroll_y, w: *w, h: *h },
                            color: u32_to_rgba(*color),
                        });
                        paint_count += 1;
                    }
                }
                Opcode::PaintBorder => {
                    if let Operands::PaintBorder { x, y, w, h, widths, color, .. } = &inst.operands {
                        let rgba = u32_to_rgba(*color);
                        let sx = *x - self.scroll_x;
                        let sy = *y - self.scroll_y;
                        if widths[0] > 0.0 { dl.push(PaintCommand::FillRect { rect: PaintRect { x: sx, y: sy, w: *w, h: widths[0] }, color: rgba }); }
                        if widths[1] > 0.0 { dl.push(PaintCommand::FillRect { rect: PaintRect { x: sx + w - widths[1], y: sy, w: widths[1], h: *h }, color: rgba }); }
                        if widths[2] > 0.0 { dl.push(PaintCommand::FillRect { rect: PaintRect { x: sx, y: sy + h - widths[2], w: *w, h: widths[2] }, color: rgba }); }
                        if widths[3] > 0.0 { dl.push(PaintCommand::FillRect { rect: PaintRect { x: sx, y: sy, w: widths[3], h: *h }, color: rgba }); }
                        paint_count += 4;
                    }
                }
                Opcode::PaintText => {
                    if let Operands::PaintText { x, y, text, color } = &inst.operands {
                        dl.push(PaintCommand::DrawText {
                            x: *x - self.scroll_x, y: *y - self.scroll_y,
                            text: text.clone(), font_size: 16.0,
                            color: u32_to_rgba(*color), max_width: self.viewport_w,
                        });
                        paint_count += 1;
                    }
                }
                Opcode::PaintImage => {
                    if let Operands::PaintImage { x, y, w, h, texture_id } = &inst.operands {
                        dl.push(PaintCommand::DrawImage {
                            rect: PaintRect { x: *x - self.scroll_x, y: *y - self.scroll_y, w: *w, h: *h },
                            image_data: None, img_w: 0, img_h: 0,
                        });
                        paint_count += 1;
                    }
                }
                Opcode::PaintClip => {
                    if let Operands::PaintClip { x, y, w, h } = &inst.operands {
                        let clip = PaintRect { x: *x - self.scroll_x, y: *y - self.scroll_y, w: *w, h: *h };
                        dl.push(PaintCommand::PushClip { rect: clip.clone() });
                        self.clip_stack.push(clip);
                    }
                }
                Opcode::PaintUnclip => {
                    dl.push(PaintCommand::PopClip);
                    self.clip_stack.pop();
                }
                Opcode::PaintOpacity => {
                    if let Operands::PaintOpacity { value } = &inst.operands {
                        self.current_opacity = *value;
                    }
                }
                Opcode::LayerPush => {
                    if let Operands::LayerPush { layer_id } = &inst.operands {
                        self.layer_stack.push(*layer_id);
                    }
                }
                Opcode::LayerPop => { self.layer_stack.pop(); }
                Opcode::ScrollTo => {
                    if let Operands::ScrollCoord { x, y } = &inst.operands {
                        self.scroll_x = *x; self.scroll_y = *y;
                    }
                }
                Opcode::ScrollBy => {
                    if let Operands::ScrollCoord { x, y } = &inst.operands {
                        self.scroll_x += *x; self.scroll_y += *y;
                    }
                }
                Opcode::Navigate => {
                    if let Operands::NavigateUrl { url } = &inst.operands {
                        navigate_requests.push(url.clone());
                    }
                }
                Opcode::ResourceReq => {
                    if let Operands::ResourceReq { url, res_type } = &inst.operands {
                        resource_requests.push((url.clone(), *res_type));
                    }
                }
                Opcode::FrameBegin => {
                    if let Operands::FrameBegin { frame_num, .. } = &inst.operands {
                        self.frame_count = *frame_num;
                    }
                }
                _ => {} // FrameEnd, HitTest, HitResult, ResourceResp, etc.
            }
        }

        ExecutionResult {
            display_list: dl,
            node_count: self.nodes.len(),
            box_count: self.boxes.len(),
            paint_count,
            frame_num: self.frame_count,
            navigate_requests,
            resource_requests,
        }
    }

    pub fn scroll_to(&mut self, x: f32, y: f32) { self.scroll_x = x; self.scroll_y = y; }
    pub fn scroll_by(&mut self, dx: f32, dy: f32) { self.scroll_x += dx; self.scroll_y += dy; }
    pub fn viewport(&self) -> (f32, f32) { (self.viewport_w, self.viewport_h) }
    pub fn scroll_pos(&self) -> (f32, f32) { (self.scroll_x, self.scroll_y) }

    pub fn register_texture(&mut self, id: u32, width: u32, height: u32) {
        self.textures.insert(id, TextureRef { id, width, height });
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<u32> {
        let sx = x + self.scroll_x;
        let sy = y + self.scroll_y;
        let mut hit: Option<(u32, f32)> = None; // (node_id, area) — smallest area wins
        for (id, bx) in &self.boxes {
            if sx >= bx.x && sx < bx.x + bx.w && sy >= bx.y && sy < bx.y + bx.h {
                let area = bx.w * bx.h;
                if hit.is_none() || area < hit.unwrap().1 {
                    hit = Some((*id, area));
                }
            }
        }
        hit.map(|(id, _)| id)
    }

    pub fn get_node_text(&self, id: u32) -> Option<&str> {
        self.nodes.get(&id).map(|n| n.text.as_str())
    }

    pub fn get_node_attr(&self, id: u32, key: &str) -> Option<&str> {
        self.nodes.get(&id).and_then(|n| n.attrs.get(key).map(|s| s.as_str()))
    }

    pub fn clear(&mut self) {
        self.nodes.clear(); self.boxes.clear();
        self.layer_stack.clear(); self.clip_stack.clear();
        self.current_opacity = 1.0;
        self.scroll_x = 0.0; self.scroll_y = 0.0;
    }
}

fn u32_to_rgba(c: u32) -> [u8; 4] {
    [(c >> 24) as u8, (c >> 16) as u8, (c >> 8) as u8, c as u8]
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::amni_ir::IrBuilder;

    #[test]
    fn execute_basic_program() {
        let mut b = IrBuilder::new();
        b.doc_start(800.0, 600.0);
        b.frame_begin();
        let n = b.create_node("div");
        b.create_box(n, 10.0, 20.0, 200.0, 100.0);
        b.paint_rect(10.0, 20.0, 200.0, 100.0, 0xFF0000FF);
        b.paint_text(15.0, 25.0, "Hello", 0x000000FF);
        b.frame_end();
        b.doc_end();
        let prog = b.finish();

        let mut exec = IrExecutor::new();
        let result = exec.execute(&prog);
        assert_eq!(result.node_count, 1);
        assert_eq!(result.box_count, 1);
        assert_eq!(result.paint_count, 2);
    }

    #[test]
    fn hit_test_works() {
        let mut b = IrBuilder::new();
        b.doc_start(800.0, 600.0);
        let n = b.create_node("div");
        b.create_box(n, 100.0, 100.0, 200.0, 200.0);
        b.doc_end();
        let prog = b.finish();

        let mut exec = IrExecutor::new();
        exec.execute(&prog);
        assert!(exec.hit_test(150.0, 150.0).is_some());
        assert!(exec.hit_test(50.0, 50.0).is_none());
    }

    #[test]
    fn wire_roundtrip_execution() {
        let mut b = IrBuilder::new();
        b.doc_start(640.0, 480.0);
        b.paint_rect(0.0, 0.0, 640.0, 480.0, 0xFFFFFFFF);
        b.doc_end();
        let prog = b.finish();

        let wire = prog.encode_to_wire();
        let decoded = IrProgram::decode_from_wire(&wire).unwrap();

        let mut exec = IrExecutor::new();
        let result = exec.execute(&decoded);
        assert_eq!(result.paint_count, 1);
    }
}
