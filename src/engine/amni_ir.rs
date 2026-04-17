/// AmniIR — 77-opcode intermediate representation for the AmniShunt rendering engine.
/// All instructions serialize to/from base-17 wire format.

use super::base17::{StreamEncoder, StreamDecoder, CodecError};
use std::collections::HashMap;

// --- Opcode definitions ---

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
    // Category 0: Document Structure
    DocStart = 0x00, DocEnd = 0x01,
    NodeCreate = 0x02, NodeDestroy = 0x03,
    NodeAppend = 0x04, NodeInsert = 0x05, NodeRemove = 0x06,
    AttrSet = 0x07, AttrDel = 0x08,
    TextSet = 0x09, TextAppend = 0x0A,
    // Category 1: Layout
    BoxCreate = 0x10, BoxUpdate = 0x11,
    BoxMargin = 0x12, BoxPadding = 0x13, BoxBorder = 0x14,
    BoxDisplay = 0x15, BoxOverflow = 0x16,
    FlexConfig = 0x17, TextLayout = 0x18, TextRun = 0x19,
    // Category 2: Paint
    PaintRect = 0x20, PaintBorder = 0x21,
    PaintText = 0x22, PaintImage = 0x23,
    PaintShadow = 0x24, PaintClip = 0x25, PaintUnclip = 0x26,
    PaintOpacity = 0x27, PaintTransform = 0x28, PaintGradient = 0x29,
    LayerPush = 0x2A, LayerPop = 0x2B,
    // Category 3: Event & Control
    ScrollTo = 0x30, ScrollBy = 0x31,
    Navigate = 0x32, ResourceReq = 0x33, ResourceResp = 0x34,
    FrameBegin = 0x35, FrameEnd = 0x36,
    HitTest = 0x37, HitResult = 0x38,
}

impl Opcode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(Self::DocStart), 0x01 => Some(Self::DocEnd),
            0x02 => Some(Self::NodeCreate), 0x03 => Some(Self::NodeDestroy),
            0x04 => Some(Self::NodeAppend), 0x05 => Some(Self::NodeInsert),
            0x06 => Some(Self::NodeRemove), 0x07 => Some(Self::AttrSet),
            0x08 => Some(Self::AttrDel), 0x09 => Some(Self::TextSet),
            0x0A => Some(Self::TextAppend),
            0x10 => Some(Self::BoxCreate), 0x11 => Some(Self::BoxUpdate),
            0x12 => Some(Self::BoxMargin), 0x13 => Some(Self::BoxPadding),
            0x14 => Some(Self::BoxBorder), 0x15 => Some(Self::BoxDisplay),
            0x16 => Some(Self::BoxOverflow), 0x17 => Some(Self::FlexConfig),
            0x18 => Some(Self::TextLayout), 0x19 => Some(Self::TextRun),
            0x20 => Some(Self::PaintRect), 0x21 => Some(Self::PaintBorder),
            0x22 => Some(Self::PaintText), 0x23 => Some(Self::PaintImage),
            0x24 => Some(Self::PaintShadow), 0x25 => Some(Self::PaintClip),
            0x26 => Some(Self::PaintUnclip), 0x27 => Some(Self::PaintOpacity),
            0x28 => Some(Self::PaintTransform), 0x29 => Some(Self::PaintGradient),
            0x2A => Some(Self::LayerPush), 0x2B => Some(Self::LayerPop),
            0x30 => Some(Self::ScrollTo), 0x31 => Some(Self::ScrollBy),
            0x32 => Some(Self::Navigate), 0x33 => Some(Self::ResourceReq),
            0x34 => Some(Self::ResourceResp), 0x35 => Some(Self::FrameBegin),
            0x36 => Some(Self::FrameEnd), 0x37 => Some(Self::HitTest),
            0x38 => Some(Self::HitResult),
            _ => None,
        }
    }

    pub fn category(&self) -> &'static str {
        match (*self as u8) >> 4 {
            0 => "document", 1 => "layout", 2 => "paint", 3 => "event", _ => "unknown",
        }
    }
}

// --- Instruction with typed operands ---

#[derive(Debug, Clone)]
pub struct Instruction {
    pub op: Opcode,
    pub operands: Operands,
}

#[derive(Debug, Clone)]
pub enum Operands {
    None,
    DocStart { width: f32, height: f32 },
    Node { node_id: u32 },
    NodeCreate { node_id: u32, tag_hash: u32 },
    NodeTree { parent_id: u32, child_id: u32 },
    NodeInsert { parent_id: u32, child_id: u32, index: u16 },
    AttrSet { node_id: u32, key: String, value: String },
    AttrDel { node_id: u32, key: String },
    TextContent { node_id: u32, text: String },
    BoxCreate { node_id: u32, x: f32, y: f32, w: f32, h: f32 },
    BoxEdges { node_id: u32, top: f32, right: f32, bottom: f32, left: f32 },
    BoxMode { node_id: u32, mode: u8 },
    FlexConfig { node_id: u32, dir: u8, wrap: u8, justify: u8, align: u8 },
    TextLayout { node_id: u32, font_id: u32, size: f32, line_h: f32 },
    TextRun { node_id: u32, text: String, glyph_count: u16 },
    PaintRect { x: f32, y: f32, w: f32, h: f32, color: u32 },
    PaintBorder { x: f32, y: f32, w: f32, h: f32, widths: [f32; 4], color: u32, radius: f32 },
    PaintText { x: f32, y: f32, text: String, color: u32 },
    PaintImage { x: f32, y: f32, w: f32, h: f32, texture_id: u32 },
    PaintShadow { x: f32, y: f32, w: f32, h: f32, blur: f32, spread: f32, color: u32 },
    PaintClip { x: f32, y: f32, w: f32, h: f32 },
    PaintOpacity { value: f32 },
    PaintTransform { matrix: [f32; 6] },
    PaintGradient { x: f32, y: f32, w: f32, h: f32, grad_type: u8, stops: Vec<(f32, u32)> },
    LayerPush { layer_id: u32 },
    ScrollCoord { x: f32, y: f32 },
    NavigateUrl { url: String },
    ResourceReq { url: String, res_type: u8 },
    ResourceResp { req_id: u32, data_len: u32 },
    FrameBegin { frame_num: u32, timestamp: u32 },
    HitCoord { x: f32, y: f32 },
    HitResult { node_id: u32, x: f32, y: f32 },
}

// --- Serialization ---

impl Instruction {
    pub fn encode(&self, enc: &mut StreamEncoder) {
        enc.write_byte(self.op as u8);
        match &self.operands {
            Operands::None => {}
            Operands::DocStart { width, height } => { enc.write_f32(*width); enc.write_f32(*height); }
            Operands::Node { node_id } => { enc.write_u32(*node_id); }
            Operands::NodeCreate { node_id, tag_hash } => { enc.write_u32(*node_id); enc.write_u32(*tag_hash); }
            Operands::NodeTree { parent_id, child_id } => { enc.write_u32(*parent_id); enc.write_u32(*child_id); }
            Operands::NodeInsert { parent_id, child_id, index } => { enc.write_u32(*parent_id); enc.write_u32(*child_id); enc.write_u16(*index); }
            Operands::AttrSet { node_id, key, value } => { enc.write_u32(*node_id); enc.write_string(key); enc.write_string(value); }
            Operands::AttrDel { node_id, key } => { enc.write_u32(*node_id); enc.write_string(key); }
            Operands::TextContent { node_id, text } => { enc.write_u32(*node_id); enc.write_string(text); }
            Operands::BoxCreate { node_id, x, y, w, h } => { enc.write_u32(*node_id); enc.write_f32(*x); enc.write_f32(*y); enc.write_f32(*w); enc.write_f32(*h); }
            Operands::BoxEdges { node_id, top, right, bottom, left } => { enc.write_u32(*node_id); enc.write_f32(*top); enc.write_f32(*right); enc.write_f32(*bottom); enc.write_f32(*left); }
            Operands::BoxMode { node_id, mode } => { enc.write_u32(*node_id); enc.write_byte(*mode); }
            Operands::FlexConfig { node_id, dir, wrap, justify, align } => { enc.write_u32(*node_id); enc.write_byte(*dir); enc.write_byte(*wrap); enc.write_byte(*justify); enc.write_byte(*align); }
            Operands::TextLayout { node_id, font_id, size, line_h } => { enc.write_u32(*node_id); enc.write_u32(*font_id); enc.write_f32(*size); enc.write_f32(*line_h); }
            Operands::TextRun { node_id, text, glyph_count } => { enc.write_u32(*node_id); enc.write_string(text); enc.write_u16(*glyph_count); }
            Operands::PaintRect { x, y, w, h, color } => { enc.write_f32(*x); enc.write_f32(*y); enc.write_f32(*w); enc.write_f32(*h); enc.write_u32(*color); }
            Operands::PaintBorder { x, y, w, h, widths, color, radius } => {
                enc.write_f32(*x); enc.write_f32(*y); enc.write_f32(*w); enc.write_f32(*h);
                for ww in widths { enc.write_f32(*ww); }
                enc.write_u32(*color); enc.write_f32(*radius);
            }
            Operands::PaintText { x, y, text, color } => { enc.write_f32(*x); enc.write_f32(*y); enc.write_string(text); enc.write_u32(*color); }
            Operands::PaintImage { x, y, w, h, texture_id } => { enc.write_f32(*x); enc.write_f32(*y); enc.write_f32(*w); enc.write_f32(*h); enc.write_u32(*texture_id); }
            Operands::PaintShadow { x, y, w, h, blur, spread, color } => { enc.write_f32(*x); enc.write_f32(*y); enc.write_f32(*w); enc.write_f32(*h); enc.write_f32(*blur); enc.write_f32(*spread); enc.write_u32(*color); }
            Operands::PaintClip { x, y, w, h } => { enc.write_f32(*x); enc.write_f32(*y); enc.write_f32(*w); enc.write_f32(*h); }
            Operands::PaintOpacity { value } => { enc.write_f32(*value); }
            Operands::PaintTransform { matrix } => { for m in matrix { enc.write_f32(*m); } }
            Operands::PaintGradient { x, y, w, h, grad_type, stops } => {
                enc.write_f32(*x); enc.write_f32(*y); enc.write_f32(*w); enc.write_f32(*h);
                enc.write_byte(*grad_type); enc.write_u16(stops.len() as u16);
                for (pos, col) in stops { enc.write_f32(*pos); enc.write_u32(*col); }
            }
            Operands::LayerPush { layer_id } => { enc.write_u32(*layer_id); }
            Operands::ScrollCoord { x, y } => { enc.write_f32(*x); enc.write_f32(*y); }
            Operands::NavigateUrl { url } => { enc.write_string(url); }
            Operands::ResourceReq { url, res_type } => { enc.write_string(url); enc.write_byte(*res_type); }
            Operands::ResourceResp { req_id, data_len } => { enc.write_u32(*req_id); enc.write_u32(*data_len); }
            Operands::FrameBegin { frame_num, timestamp } => { enc.write_u32(*frame_num); enc.write_u32(*timestamp); }
            Operands::HitCoord { x, y } => { enc.write_f32(*x); enc.write_f32(*y); }
            Operands::HitResult { node_id, x, y } => { enc.write_u32(*node_id); enc.write_f32(*x); enc.write_f32(*y); }
        }
    }

    pub fn decode(dec: &mut StreamDecoder) -> Result<Self, CodecError> {
        let op_byte = dec.read_byte()?;
        let op = Opcode::from_byte(op_byte).ok_or(CodecError::InvalidDigit(op_byte))?;
        let operands = match op {
            Opcode::DocStart => Operands::DocStart { width: dec.read_f32()?, height: dec.read_f32()? },
            Opcode::DocEnd | Opcode::PaintUnclip | Opcode::LayerPop | Opcode::FrameEnd => Operands::None,
            Opcode::NodeCreate => Operands::NodeCreate { node_id: dec.read_u32()?, tag_hash: dec.read_u32()? },
            Opcode::NodeDestroy => Operands::Node { node_id: dec.read_u32()? },
            Opcode::NodeAppend | Opcode::NodeRemove => Operands::NodeTree { parent_id: dec.read_u32()?, child_id: dec.read_u32()? },
            Opcode::NodeInsert => Operands::NodeInsert { parent_id: dec.read_u32()?, child_id: dec.read_u32()?, index: dec.read_u16()? },
            Opcode::AttrSet => Operands::AttrSet { node_id: dec.read_u32()?, key: dec.read_string()?, value: dec.read_string()? },
            Opcode::AttrDel => Operands::AttrDel { node_id: dec.read_u32()?, key: dec.read_string()? },
            Opcode::TextSet | Opcode::TextAppend => Operands::TextContent { node_id: dec.read_u32()?, text: dec.read_string()? },
            Opcode::BoxCreate | Opcode::BoxUpdate => Operands::BoxCreate { node_id: dec.read_u32()?, x: dec.read_f32()?, y: dec.read_f32()?, w: dec.read_f32()?, h: dec.read_f32()? },
            Opcode::BoxMargin | Opcode::BoxPadding | Opcode::BoxBorder => Operands::BoxEdges { node_id: dec.read_u32()?, top: dec.read_f32()?, right: dec.read_f32()?, bottom: dec.read_f32()?, left: dec.read_f32()? },
            Opcode::BoxDisplay | Opcode::BoxOverflow => Operands::BoxMode { node_id: dec.read_u32()?, mode: dec.read_byte()? },
            Opcode::FlexConfig => Operands::FlexConfig { node_id: dec.read_u32()?, dir: dec.read_byte()?, wrap: dec.read_byte()?, justify: dec.read_byte()?, align: dec.read_byte()? },
            Opcode::TextLayout => Operands::TextLayout { node_id: dec.read_u32()?, font_id: dec.read_u32()?, size: dec.read_f32()?, line_h: dec.read_f32()? },
            Opcode::TextRun => Operands::TextRun { node_id: dec.read_u32()?, text: dec.read_string()?, glyph_count: dec.read_u16()? },
            Opcode::PaintRect => Operands::PaintRect { x: dec.read_f32()?, y: dec.read_f32()?, w: dec.read_f32()?, h: dec.read_f32()?, color: dec.read_u32()? },
            Opcode::PaintBorder => {
                let (x, y, w, h) = (dec.read_f32()?, dec.read_f32()?, dec.read_f32()?, dec.read_f32()?);
                let widths = [dec.read_f32()?, dec.read_f32()?, dec.read_f32()?, dec.read_f32()?];
                Operands::PaintBorder { x, y, w, h, widths, color: dec.read_u32()?, radius: dec.read_f32()? }
            }
            Opcode::PaintText => Operands::PaintText { x: dec.read_f32()?, y: dec.read_f32()?, text: dec.read_string()?, color: dec.read_u32()? },
            Opcode::PaintImage => Operands::PaintImage { x: dec.read_f32()?, y: dec.read_f32()?, w: dec.read_f32()?, h: dec.read_f32()?, texture_id: dec.read_u32()? },
            Opcode::PaintShadow => Operands::PaintShadow { x: dec.read_f32()?, y: dec.read_f32()?, w: dec.read_f32()?, h: dec.read_f32()?, blur: dec.read_f32()?, spread: dec.read_f32()?, color: dec.read_u32()? },
            Opcode::PaintClip => Operands::PaintClip { x: dec.read_f32()?, y: dec.read_f32()?, w: dec.read_f32()?, h: dec.read_f32()? },
            Opcode::PaintOpacity => Operands::PaintOpacity { value: dec.read_f32()? },
            Opcode::PaintTransform => { let mut m = [0.0f32; 6]; for i in 0..6 { m[i] = dec.read_f32()?; } Operands::PaintTransform { matrix: m } }
            Opcode::PaintGradient => {
                let (x, y, w, h) = (dec.read_f32()?, dec.read_f32()?, dec.read_f32()?, dec.read_f32()?);
                let gt = dec.read_byte()?;
                let n = dec.read_u16()? as usize;
                let mut stops = Vec::with_capacity(n);
                for _ in 0..n { stops.push((dec.read_f32()?, dec.read_u32()?)); }
                Operands::PaintGradient { x, y, w, h, grad_type: gt, stops }
            }
            Opcode::LayerPush => Operands::LayerPush { layer_id: dec.read_u32()? },
            Opcode::ScrollTo | Opcode::ScrollBy => Operands::ScrollCoord { x: dec.read_f32()?, y: dec.read_f32()? },
            Opcode::Navigate => Operands::NavigateUrl { url: dec.read_string()? },
            Opcode::ResourceReq => Operands::ResourceReq { url: dec.read_string()?, res_type: dec.read_byte()? },
            Opcode::ResourceResp => Operands::ResourceResp { req_id: dec.read_u32()?, data_len: dec.read_u32()? },
            Opcode::FrameBegin => Operands::FrameBegin { frame_num: dec.read_u32()?, timestamp: dec.read_u32()? },
            Opcode::HitTest => Operands::HitCoord { x: dec.read_f32()?, y: dec.read_f32()? },
            Opcode::HitResult => Operands::HitResult { node_id: dec.read_u32()?, x: dec.read_f32()?, y: dec.read_f32()? },
        };
        Ok(Instruction { op, operands })
    }
}

// --- IR Program (sequence of instructions) ---

pub struct IrProgram {
    pub instructions: Vec<Instruction>,
}

impl IrProgram {
    pub fn new() -> Self { Self { instructions: Vec::new() } }

    pub fn push(&mut self, inst: Instruction) { self.instructions.push(inst); }
    pub fn len(&self) -> usize { self.instructions.len() }
    pub fn is_empty(&self) -> bool { self.instructions.is_empty() }

    pub fn encode_to_wire(&self) -> Vec<u8> {
        let mut enc = StreamEncoder::new();
        for inst in &self.instructions { inst.encode(&mut enc); }
        enc.finish()
    }

    pub fn decode_from_wire(wire: &[u8]) -> Result<Self, CodecError> {
        let mut dec = StreamDecoder::new(wire)?;
        let mut prog = Self::new();
        while !dec.is_at_end() {
            if dec.peek_meta().is_some() { dec.skip_meta(); continue; }
            prog.push(Instruction::decode(&mut dec)?);
        }
        Ok(prog)
    }
}

// --- IR Builder (convenience API) ---

pub struct IrBuilder {
    prog: IrProgram,
    next_node_id: u32,
    next_layer_id: u32,
    frame_count: u32,
}

impl IrBuilder {
    pub fn new() -> Self { Self { prog: IrProgram::new(), next_node_id: 1, next_layer_id: 1, frame_count: 0 } }

    pub fn doc_start(&mut self, w: f32, h: f32) {
        self.prog.push(Instruction { op: Opcode::DocStart, operands: Operands::DocStart { width: w, height: h } });
    }
    pub fn doc_end(&mut self) {
        self.prog.push(Instruction { op: Opcode::DocEnd, operands: Operands::None });
    }
    pub fn create_node(&mut self, tag: &str) -> u32 {
        let id = self.next_node_id; self.next_node_id += 1;
        self.prog.push(Instruction { op: Opcode::NodeCreate, operands: Operands::NodeCreate { node_id: id, tag_hash: hash_tag(tag) } });
        id
    }
    pub fn append_child(&mut self, parent: u32, child: u32) {
        self.prog.push(Instruction { op: Opcode::NodeAppend, operands: Operands::NodeTree { parent_id: parent, child_id: child } });
    }
    pub fn set_attr(&mut self, node: u32, key: &str, value: &str) {
        self.prog.push(Instruction { op: Opcode::AttrSet, operands: Operands::AttrSet { node_id: node, key: key.into(), value: value.into() } });
    }
    pub fn set_text(&mut self, node: u32, text: &str) {
        self.prog.push(Instruction { op: Opcode::TextSet, operands: Operands::TextContent { node_id: node, text: text.into() } });
    }
    pub fn create_box(&mut self, node: u32, x: f32, y: f32, w: f32, h: f32) {
        self.prog.push(Instruction { op: Opcode::BoxCreate, operands: Operands::BoxCreate { node_id: node, x, y, w, h } });
    }
    pub fn set_margin(&mut self, node: u32, t: f32, r: f32, b: f32, l: f32) {
        self.prog.push(Instruction { op: Opcode::BoxMargin, operands: Operands::BoxEdges { node_id: node, top: t, right: r, bottom: b, left: l } });
    }
    pub fn set_padding(&mut self, node: u32, t: f32, r: f32, b: f32, l: f32) {
        self.prog.push(Instruction { op: Opcode::BoxPadding, operands: Operands::BoxEdges { node_id: node, top: t, right: r, bottom: b, left: l } });
    }
    pub fn paint_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        self.prog.push(Instruction { op: Opcode::PaintRect, operands: Operands::PaintRect { x, y, w, h, color } });
    }
    pub fn paint_text(&mut self, x: f32, y: f32, text: &str, color: u32) {
        self.prog.push(Instruction { op: Opcode::PaintText, operands: Operands::PaintText { x, y, text: text.into(), color } });
    }
    pub fn paint_image(&mut self, x: f32, y: f32, w: f32, h: f32, tex: u32) {
        self.prog.push(Instruction { op: Opcode::PaintImage, operands: Operands::PaintImage { x, y, w, h, texture_id: tex } });
    }
    pub fn paint_clip(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.prog.push(Instruction { op: Opcode::PaintClip, operands: Operands::PaintClip { x, y, w, h } });
    }
    pub fn paint_unclip(&mut self) {
        self.prog.push(Instruction { op: Opcode::PaintUnclip, operands: Operands::None });
    }
    pub fn push_layer(&mut self) -> u32 {
        let id = self.next_layer_id; self.next_layer_id += 1;
        self.prog.push(Instruction { op: Opcode::LayerPush, operands: Operands::LayerPush { layer_id: id } });
        id
    }
    pub fn pop_layer(&mut self) {
        self.prog.push(Instruction { op: Opcode::LayerPop, operands: Operands::None });
    }
    pub fn frame_begin(&mut self) -> u32 {
        let n = self.frame_count; self.frame_count += 1;
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u32;
        self.prog.push(Instruction { op: Opcode::FrameBegin, operands: Operands::FrameBegin { frame_num: n, timestamp: ts } });
        n
    }
    pub fn frame_end(&mut self) {
        self.prog.push(Instruction { op: Opcode::FrameEnd, operands: Operands::None });
    }

    pub fn finish(self) -> IrProgram { self.prog }
}

pub fn hash_tag(tag: &str) -> u32 {
    let mut h = 0x811c9dc5u32;
    for b in tag.as_bytes() { h ^= *b as u32; h = h.wrapping_mul(0x01000193); }
    h
}

// --- DOM-to-IR emitter: converts RenderTree + layout → IrProgram ---

use super::paint::RenderTree;
use super::layout::LayoutRect;

pub fn emit_ir_from_render_tree(
    tree: &RenderTree,
    layouts: &HashMap<usize, LayoutRect>,
    vw: f32, vh: f32,
) -> IrProgram {
    let mut builder = IrBuilder::new();
    builder.doc_start(vw, vh);
    builder.frame_begin();
    emit_node(&mut builder, tree, layouts, tree.root_id, 0.0, 0.0);
    builder.frame_end();
    builder.doc_end();
    builder.finish()
}

fn emit_node(
    b: &mut IrBuilder, tree: &RenderTree, layouts: &HashMap<usize, LayoutRect>,
    node_id: usize, px: f32, py: f32,
) {
    let node = match tree.nodes.get(&node_id) { Some(n) => n, None => return };
    let lr = layouts.get(&node_id);
    let (ax, ay, w, h) = if let Some(r) = lr {
        (px + r.x, py + r.y, r.w, r.h)
    } else { (px, py, 0.0, 0.0) };

    let ir_id = b.create_node(&node.tag);
    b.create_box(ir_id, ax, ay, w, h);

    let cs = &node.style;
    if cs.background_color.a > 0.0 && w > 0.0 && h > 0.0 {
        let c = rgba_to_u32(cs.background_color.r, cs.background_color.g, cs.background_color.b, (cs.background_color.a * 255.0) as u8);
        b.paint_rect(ax, ay, w, h, c);
    }
    if !node.text.is_empty() {
        let c = rgba_to_u32(cs.color.r, cs.color.g, cs.color.b, (cs.color.a * 255.0) as u8);
        b.paint_text(ax + cs.padding.left, ay + cs.padding.top, &node.text, c);
    }
    if !node.image_src.is_empty() {
        b.paint_image(ax, ay, w, h, hash_tag(&node.image_src));
    }
    for &child_id in &node.children {
        emit_node(b, tree, layouts, child_id, ax, ay);
    }
}

fn rgba_to_u32(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | a as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_roundtrip() {
        for b in 0..=0x38u8 {
            if let Some(op) = Opcode::from_byte(b) {
                assert_eq!(op as u8, b);
            }
        }
    }

    #[test]
    fn builder_encode_decode() {
        let mut b = IrBuilder::new();
        b.doc_start(1280.0, 720.0);
        let n = b.create_node("div");
        b.create_box(n, 0.0, 0.0, 100.0, 50.0);
        b.paint_rect(0.0, 0.0, 100.0, 50.0, 0xFF0000FF);
        b.paint_text(10.0, 10.0, "Hello", 0x000000FF);
        b.doc_end();
        let prog = b.finish();

        let wire = prog.encode_to_wire();
        let decoded = IrProgram::decode_from_wire(&wire).unwrap();
        assert_eq!(decoded.len(), prog.len());
    }

    #[test]
    fn tag_hash_stable() {
        assert_eq!(hash_tag("div"), hash_tag("div"));
        assert_ne!(hash_tag("div"), hash_tag("span"));
    }
}
