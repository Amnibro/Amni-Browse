def rw(p,f,nl='\n'):
    s=open(p,encoding='utf-8').read(); n=f(s); assert n!=s,p; open(p,'w',encoding='utf-8',newline=nl).write(n)
def sub1(s,a,b):
    assert s.count(a)==1,(a[:70],s.count(a)); return s.replace(a,b)
def sle(s):
    s=sub1(s,'use crate::dom::element::Element;','use crate::dom::element::Element;\nuse crate::dom::html::htmltextareaelement::HTMLTextAreaElement;')
    s=sub1(s,'''pub struct ServoLayoutElement<'dom> {
    /// The wrapped private DOM Element.
    pub(super) element: LayoutDom<'dom, Element>,
    /// The possibly nested [`PseudoElementChain`] for this element.
    pub(super) pseudo_element_chain: PseudoElementChain,
}
''','''pub struct ServoLayoutElement<'dom> {
    /// The wrapped private DOM Element.
    pub(super) element: LayoutDom<'dom, Element>,
    /// The possibly nested [`PseudoElementChain`] for this element.
    pub(super) pseudo_element_chain: PseudoElementChain,
}

impl ServoLayoutElement<'_> {
    /// The `rows` attribute of a `<textarea>`, which sizes its automatic block size.
    pub fn text_area_rows(&self) -> Option<u32> {
        self.element
            .downcast::<HTMLTextAreaElement>()
            .map(|textarea| textarea.get_rows())
    }
}
''')
    return s
pass
def el(s):
    s=sub1(s,'''        let rows = self
            .downcast::<HTMLTextAreaElement>()
            .map(LayoutDom::get_rows);
        if let Some(rows) = rows {
            let rows = rows as i32;
            if rows > 0 {
                // TODO(mttr) This should take scrollbar size into consideration.
                //
                // https://html.spec.whatwg.org/multipage/#textarea-effective-height
                let value = specified::NoCalcLength::from_em(rows as CSSFloat * 1.35);
                push(PropertyDeclaration::Height(
                    specified::Size::LengthPercentage(NonNegative(
                        specified::LengthPercentage::Length(value),
                    )),
                ));
            }
        }
''','''        // `rows` sizes the automatic block size of a `<textarea>` in layout
        // (`IndependentFormattingContext::text_area_rows`), like other engines, instead of
        // a presentational `height` that would fight `box-sizing` and author padding.
''')
    return s
pass
def fc(s):
    s=sub1(s,'''    /// If this [`IndependentFormattingContext`] was a layout root, this stores the data
    /// necessary to lay it out again.
    pub layout_root_layout_inputs: AtomicRefCell<Option<Box<LayoutRootLayoutInputs>>>,
}
''','''    /// If this [`IndependentFormattingContext`] was a layout root, this stores the data
    /// necessary to lay it out again.
    pub layout_root_layout_inputs: AtomicRefCell<Option<Box<LayoutRootLayoutInputs>>>,
    /// For a `<textarea>`, its `rows` attribute: the automatic block size is at least
    /// `rows` line heights. <https://html.spec.whatwg.org/multipage/#textarea-effective-height>
    text_area_rows: Option<u32>,
}
''')
    s=sub1(s,'''        base.set_subtree_size(contents.subtree_size() + 1);
        Self {
            base,
            contents,
            propagated_data,
            layout_root_layout_inputs: None.into(),
        }
    }

    pub(crate) fn rebuild(''','''        base.set_subtree_size(contents.subtree_size() + 1);
        Self {
            base,
            contents,
            propagated_data,
            layout_root_layout_inputs: None.into(),
            text_area_rows: None,
        }
    }

    pub(crate) fn rebuild(''')
    s=sub1(s,'''        let base = LayoutBoxBase::new(base_fragment_info, node_and_style_info.style.clone());
        base.set_subtree_size(contents.subtree_size() + 1);

        Self {
            base,
            contents,
            propagated_data,
            layout_root_layout_inputs: None.into(),
        }''','''        let base = LayoutBoxBase::new(base_fragment_info, node_and_style_info.style.clone());
        base.set_subtree_size(contents.subtree_size() + 1);
        let text_area_rows = node_and_style_info
            .node
            .as_element()
            .and_then(|element| element.text_area_rows())
            .filter(|rows| *rows > 0);

        Self {
            base,
            contents,
            propagated_data,
            layout_root_layout_inputs: None.into(),
            text_area_rows,
        }''')
    s=sub1(s,'''        let mut child_positioning_context = PositioningContext::default();
        let result = self.layout_without_caching(
            layout_context,
            &mut child_positioning_context,
            containing_block_for_children,
            containing_block,
            preferred_aspect_ratio,
            lazy_block_size,
        );
        self.base.cache_independent_formatting_context_layout(''','''        let mut child_positioning_context = PositioningContext::default();
        let mut result = self.layout_without_caching(
            layout_context,
            &mut child_positioning_context,
            containing_block_for_children,
            containing_block,
            preferred_aspect_ratio,
            lazy_block_size,
        );
        if let Some(rows) = self.text_area_rows {
            let font = self.style().get_font();
            let font_size = font.font_size.computed_size();
            let line_height: Au = match font.line_height {
                LineHeight::Normal => (font_size * 1.3).into(),
                LineHeight::Number(number) => (font_size * number.0).into(),
                LineHeight::Length(length) => length.0.into(),
            };
            result.content_block_size = result
                .content_block_size
                .max(line_height * rows as i32);
        }
        self.base.cache_independent_formatting_context_layout(''')
    if 'use style::values::computed::LineHeight;' not in s and 'LineHeight' not in s.split('impl IndependentFormattingContext')[0]:
        s=sub1(s,'use style::dom::TNode;','use style::dom::TNode;\nuse style::values::computed::LineHeight;') if 'use style::dom::TNode;' in s else s.replace('use app_units::Au;','use app_units::Au;\nuse style::dom::TNode;\nuse style::values::computed::LineHeight;',1)
    return s
rw('vendor/servo/components/layout/formatting_contexts.rs',fc)
def tests(s):
    s=sub1(s,'''        assert!(!js.contains("{{") && !js.contains("}}"), "format! escapes leaked into output");''','''        assert!(!js.contains("{rev}"), "format! placeholder leaked into output");''')
    return s
rw('src/engine/servo_compat.rs',tests,'')
print('textarea rows patched')
