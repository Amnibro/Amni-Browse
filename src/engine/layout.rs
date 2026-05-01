use taffy::prelude::*;
use taffy::Point;
use super::style::{ComputedStyle, Display as CssDisplay, Position as CssPos, FlexDir, DimUnit, Overflow as CssOverflow};
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct TextInfo {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
}
pub struct LayoutEngine {
    tree: TaffyTree,
    nodes: HashMap<usize, NodeId>,
    results: HashMap<usize, LayoutRect>,
    texts: HashMap<NodeId, TextInfo>,
}
#[derive(Debug, Clone, Default)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl LayoutEngine {
    pub fn new() -> Self {
        Self { tree: TaffyTree::new(), nodes: HashMap::new(), results: HashMap::new(), texts: HashMap::new() }
    }
    pub fn add_node(&mut self, id: usize, style: &ComputedStyle, children: &[usize]) -> Option<NodeId> {
        let child_ids: Vec<NodeId> = children.iter().filter_map(|c| self.nodes.get(c).copied()).collect();
        let ts = Self::to_taffy_style(style);
        let node = self.tree.new_with_children(ts, &child_ids).ok()?;
        self.nodes.insert(id, node);
        Some(node)
    }
    pub fn add_leaf(&mut self, id: usize, style: &ComputedStyle) -> Option<NodeId> {
        let ts = Self::to_taffy_style(style);
        let node = self.tree.new_leaf(ts).ok()?;
        self.nodes.insert(id, node);
        Some(node)
    }
    pub fn add_leaf_with_text(&mut self, id: usize, style: &ComputedStyle, text: TextInfo) -> Option<NodeId> {
        let ts = Self::to_taffy_style(style);
        let node = self.tree.new_leaf(ts).ok()?;
        self.nodes.insert(id, node);
        self.texts.insert(node, text);
        Some(node)
    }
    pub fn compute(&mut self, root_id: usize, viewport_w: f32, viewport_h: f32) {
        if let Some(&root) = self.nodes.get(&root_id) {
            let avail = Size { width: AvailableSpace::Definite(viewport_w), height: AvailableSpace::Definite(viewport_h) };
            let texts = &self.texts;
            let _ = self.tree.compute_layout_with_measure(root, avail, |_known, avail, node_id, _ctx, _style| {
                if let Some(ti) = texts.get(&node_id) {
                    let chars = ti.text.chars().count() as f32;
                    let char_w = ti.font_size * 0.55;
                    let single = chars * char_w;
                    let avail_w = match avail.width {
                        AvailableSpace::Definite(v) => v.max(1.0),
                        _ => 1_000_000.0,
                    };
                    let line_h = ti.font_size * ti.line_height;
                    if single <= avail_w {
                        return Size { width: single, height: line_h };
                    }
                    let lines = (single / avail_w).ceil().max(1.0);
                    return Size { width: avail_w, height: lines * line_h };
                }
                Size::ZERO
            });
            self.collect_results(root_id);
        }
    }
    fn collect_results(&mut self, id: usize) {
        if let Some(&node) = self.nodes.get(&id) {
            if let Ok(lo) = self.tree.layout(node) {
                self.results.insert(id, LayoutRect { x: lo.location.x, y: lo.location.y, w: lo.size.width, h: lo.size.height });
            }
        }
    }
    pub fn collect_all(&mut self, ids: &[usize]) {
        for &id in ids { self.collect_results(id); }
    }
    pub fn get_layout(&self, id: usize) -> Option<&LayoutRect> { self.results.get(&id) }
    pub fn clear(&mut self) { self.tree.clear(); self.nodes.clear(); self.results.clear(); }
    fn to_taffy_style(cs: &ComputedStyle) -> Style {
        Style {
            display: match cs.display {
                CssDisplay::Flex | CssDisplay::InlineFlex => taffy::Display::Flex,
                CssDisplay::Grid => taffy::Display::Grid,
                CssDisplay::None => taffy::Display::None,
                _ => taffy::Display::Block,
            },
            position: match cs.position {
                CssPos::Absolute => taffy::Position::Absolute,
                _ => taffy::Position::Relative,
            },
            size: Size {
                width: Self::dim_to_taffy(&cs.width),
                height: Self::dim_to_taffy(&cs.height),
            },
            margin: Rect {
                top: length(cs.margin.top),
                right: length(cs.margin.right),
                bottom: length(cs.margin.bottom),
                left: length(cs.margin.left),
            },
            padding: Rect {
                top: length(cs.padding.top),
                right: length(cs.padding.right),
                bottom: length(cs.padding.bottom),
                left: length(cs.padding.left),
            },
            border: Rect {
                top: length(cs.border_width.top),
                right: length(cs.border_width.right),
                bottom: length(cs.border_width.bottom),
                left: length(cs.border_width.left),
            },
            flex_direction: match cs.flex_direction {
                FlexDir::Row => taffy::FlexDirection::Row,
                FlexDir::Column => taffy::FlexDirection::Column,
                FlexDir::RowReverse => taffy::FlexDirection::RowReverse,
                FlexDir::ColumnReverse => taffy::FlexDirection::ColumnReverse,
            },
            flex_grow: cs.flex_grow,
            flex_shrink: cs.flex_shrink,
            gap: Size { width: length(cs.gap), height: length(cs.gap) },
            overflow: Point {
                x: match cs.overflow { CssOverflow::Hidden => taffy::Overflow::Hidden, CssOverflow::Scroll => taffy::Overflow::Scroll, _ => taffy::Overflow::Visible },
                y: match cs.overflow { CssOverflow::Hidden => taffy::Overflow::Hidden, CssOverflow::Scroll => taffy::Overflow::Scroll, _ => taffy::Overflow::Visible },
            },
            ..Default::default()
        }
    }
    fn dim_to_taffy(d: &super::style::Dimension) -> taffy::Dimension {
        match d.unit {
            DimUnit::Auto => taffy::Dimension::Auto,
            DimUnit::Px => taffy::Dimension::Length(d.value),
            DimUnit::Pct => taffy::Dimension::Percent(d.value / 100.0),
            _ => taffy::Dimension::Auto,
        }
    }
}
