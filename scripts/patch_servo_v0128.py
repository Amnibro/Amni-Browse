import re
def rw(p,f):
    s=open(p,encoding='utf-8').read(); n=f(s); assert n!=s,p; open(p,'w',encoding='utf-8',newline='\n').write(n)
def sub1(s,a,b):
    assert s.count(a)==1,(a[:60],s.count(a)); return s.replace(a,b)
S='vendor/stylo/style/'; L='vendor/servo/components/layout/'
rw(S+'properties/longhands.toml',lambda s: sub1(s,'[-webkit-text-fill-color]\ntype = "Color"\ninitial = "computed_value::T::currentcolor()"\nstruct = "inherited_text"\nengine = "gecko"\n','[-webkit-text-fill-color]\ntype = "Color"\ninitial = "computed_value::T::currentcolor()"\nstruct = "inherited_text"\n'))
rw(S+'values/specified/background.rs',lambda s: sub1(s,'    // TODO: text and border-area are supposed to combine in backgrounds-4...\n    #[cfg(feature = "gecko")]\n    Text,','    // TODO: text and border-area are supposed to combine in backgrounds-4...\n    Text,'))
rw(S+'values/specified/text.rs',lambda s: sub1(s,'    /// `anywhere`, and `word-break` behave like `normal`.\n    #[cfg(feature = "gecko")]\n    BreakWord,','    /// `anywhere`, and `word-break` behave like `normal`.\n    BreakWord,'))
def box_(s):
    s=sub1(s,'    RubyTextContainer,\n    #[cfg(feature = "gecko")]\n    WebkitBox,\n}','    RubyTextContainer,\n    WebkitBox,\n}')
    s=sub1(s,'    #[cfg(feature = "gecko")]\n    pub const WebkitBox: Self = Self(','    pub const WebkitBox: Self = Self(')
    s=sub1(s,'    #[cfg(feature = "gecko")]\n    pub const WebkitInlineBox: Self = Self(','    pub const WebkitInlineBox: Self = Self(')
    s=sub1(s,'            #[cfg(feature = "gecko")]\n            "-webkit-box" => Full(Display::WebkitBox),\n            #[cfg(feature = "gecko")]\n            "-webkit-inline-box" => Full(Display::WebkitInlineBox),','            "-webkit-box" => Full(Display::WebkitBox),\n            "-webkit-inline-box" => Full(Display::WebkitInlineBox),')
    s=sub1(s,'            #[cfg(feature = "gecko")]\n            Display::WebkitInlineBox => dest.write_str("-webkit-inline-box"),','            Display::WebkitInlineBox => dest.write_str("-webkit-inline-box"),')
    return s
rw(S+'values/specified/box.rs',box_)
rw(L+'style_ext.rs',lambda s: sub1(s,'            stylo::DisplayInside::Flex => DisplayInside::Flex,\n            stylo::DisplayInside::Grid => DisplayInside::Grid,','            stylo::DisplayInside::Flex | stylo::DisplayInside::WebkitBox => DisplayInside::Flex,\n            stylo::DisplayInside::Grid => DisplayInside::Grid,'))
rw(L+'flow/inline/mod.rs',lambda s: sub1(s,'            WordBreak::Normal => LineBreakWordOption::Normal,','            WordBreak::Normal | WordBreak::BreakWord => LineBreakWordOption::Normal,'))
rw(L+'flow/inline/shaping_queue.rs',lambda s: sub1(s,'        let can_break_anywhere = text_style.word_break == WordBreak::BreakAll ||','        let can_break_anywhere = text_style.word_break == WordBreak::BreakAll ||\n            text_style.word_break == WordBreak::BreakWord ||'))
def bg(s):
    s=sub1(s,'use style::values::specified::background::{\n    BackgroundRepeat as RepeatXY, BackgroundRepeatKeyword as Repeat,\n};','use style::values::specified::background::{\n    BackgroundRepeat as RepeatXY, BackgroundRepeatKeyword as Repeat,\n};\nuse style::color::AbsoluteColor;\nuse style::values::computed::Image;\nuse style::values::generics::image::{GenericGradient, GenericGradientItem};')
    s=sub1(s,'            Clip::BorderBox | Clip::BorderArea => fragment_builder.border_rect,','            Clip::BorderBox | Clip::BorderArea | Clip::Text => fragment_builder.border_rect,')
    s=sub1(s,'            Clip::BorderBox => {\n                fragment_builder.border_edge_clip(builder, state, force_clip_creation)\n            },','            Clip::BorderBox | Clip::Text => {\n                fragment_builder.border_edge_clip(builder, state, force_clip_creation)\n            },')
    old_head='''    let painting_area = painter.painting_area(fragment_builder, builder, layer_index);
    let positioning_area = painter.positioning_area(fragment_builder, builder, layer_index);
    let common =
        painter.common_properties(fragment_builder, builder, state, layer_index, painting_area);
'''
    new_head='''    let painting_area = painter.painting_area(fragment_builder, builder, layer_index);
    let positioning_area = painter.positioning_area(fragment_builder, builder, layer_index);
    let common =
        painter.common_properties(fragment_builder, builder, state, layer_index, painting_area);
    let b = painter.style.get_background();
    layout_layer_with(
        painting_area,
        positioning_area,
        common,
        get_cyclic(&b.background_size.0, layer_index),
        *get_cyclic(&b.background_repeat.0, layer_index),
        get_cyclic(&b.background_position_x.0, layer_index),
        get_cyclic(&b.background_position_y.0, layer_index),
        *get_cyclic(&b.background_blend_mode.0, layer_index),
        natural_sizes,
    )
}

/// Lay out the first `mask-image` layer of `style` over the border box, using the mask-*
/// longhands. Repeat is ignored: WebRender image masks are a single rect.
pub(super) fn layout_mask(
    fragment_builder: &super::BuilderForBoxFragment,
    builder: &mut DisplayListBuilder,
    state: &TraversalState,
    style: &ComputedValues,
    natural_sizes: NaturalSizes,
) -> Option<BackgroundLayer> {
    let area = fragment_builder.border_rect;
    let mut common = builder.common_properties(state, area, style);
    if let Some(clip_chain_id) = fragment_builder.border_edge_clip(builder, state, false) {
        common.clip_chain_id = clip_chain_id;
    }
    let svg = style.get_svg();
    layout_layer_with(
        area,
        area,
        common,
        get_cyclic(&svg.mask_size.0, 0),
        RepeatXY(Repeat::NoRepeat, Repeat::NoRepeat),
        get_cyclic(&svg.mask_position_x.0, 0),
        get_cyclic(&svg.mask_position_y.0, 0),
        BackgroundBlendMode::Normal,
        natural_sizes,
    )
}

/// First gradient stop of the first gradient `background-image` layer, else an opaque
/// `background-color`. Used to paint `background-clip: text` glyphs.
pub(super) fn representative_color(style: &ComputedValues) -> Option<AbsoluteColor> {
    let b = style.get_background();
    for image in b.background_image.0.iter() {
        let Image::Gradient(gradient) = image else { continue };
        let items = match &**gradient {
            GenericGradient::Linear { items, .. } |
            GenericGradient::Radial { items, .. } |
            GenericGradient::Conic { items, .. } => items,
        };
        for item in items.iter() {
            match item {
                GenericGradientItem::SimpleColorStop(color) |
                GenericGradientItem::ComplexColorStop { color, .. } => {
                    return Some(style.resolve_color(color));
                },
                GenericGradientItem::InterpolationHint(_) => {},
            }
        }
    }
    let color = style.resolve_color(&b.background_color);
    (color.alpha > 0.0).then_some(color)
}

#[allow(clippy::too_many_arguments)]
fn layout_layer_with(
    painting_area: units::LayoutRect,
    positioning_area: units::LayoutRect,
    common: wr::CommonItemProperties,
    size: &Size,
    repeat_xy: RepeatXY,
    position_x: &LengthPercentage,
    position_y: &LengthPercentage,
    blend_mode: BackgroundBlendMode,
    natural_sizes: NaturalSizes,
) -> Option<BackgroundLayer> {
'''
    s=sub1(s,old_head,new_head)
    s=sub1(s,'''    let b = painter.style.get_background();
    let mut tile_size = match get_cyclic(&b.background_size.0, layer_index) {''','''    let mut tile_size = match size {''')
    s=sub1(s,'''    let RepeatXY(repeat_x, repeat_y) = *get_cyclic(&b.background_repeat.0, layer_index);
    let result_x = layout_1d(
        &mut tile_size.width,
        repeat_x,
        get_cyclic(&b.background_position_x.0, layer_index),''','''    let RepeatXY(repeat_x, repeat_y) = repeat_xy;
    let result_x = layout_1d(
        &mut tile_size.width,
        repeat_x,
        position_x,''')
    s=sub1(s,'''        repeat_y,
        get_cyclic(&b.background_position_y.0, layer_index),''','''        repeat_y,
        position_y,''')
    s=sub1(s,'''    let blend_mode = *get_cyclic(&b.background_blend_mode.0, layer_index);
    Some(BackgroundLayer {''','''    Some(BackgroundLayer {''')
    return s
rw(L+'display_list/background.rs',bg)
def dl(s):
    s=sub1(s,'use style::computed_values::background_blend_mode::SingleComputedValue as BackgroundBlendMode;','use style::computed_values::background_blend_mode::SingleComputedValue as BackgroundBlendMode;\nuse style::computed_values::background_clip::single_value::T as BackgroundClipValue;\nuse background::get_cyclic;')
    s=sub1(s,'''        let parent_style = fragment.style();
        let color = parent_style.clone_color();
        let font_size = parent_style.clone_font_size();''','''        let parent_style = fragment.style();
        let color = parent_style.clone_color();
        let fill_color = parent_style
            .get_inherited_text()
            .clone__webkit_text_fill_color()
            .resolve_to_absolute(&color);
        let clips_to_text = parent_style
            .get_background()
            .background_clip
            .0
            .iter()
            .any(|clip| matches!(clip, BackgroundClipValue::Text));
        let fill_color = match clips_to_text && fill_color.alpha <= 0.0 {
            true => background::representative_color(parent_style).unwrap_or(color),
            false => fill_color,
        };
        let font_size = parent_style.clone_font_size();''')
    s=sub1(s,'''            &glyphs,
            fragment.font_key,
            rgba(color),
            None,
        );''','''            &glyphs,
            fragment.font_key,
            rgba(fill_color),
            None,
        );''')
    s=sub1(s,'''        let b = painter.style.get_background();
        let background_color = painter.style.resolve_color(&b.background_color);
        if background_color.alpha > 0.0 {
            // https://drafts.csswg.org/css-backgrounds/#background-color
            // “The background color is clipped according to the background-clip
            //  value associated with the bottom-most background image layer.”
            let layer_index = b.background_image.0.len() - 1;
            let bounds = painter.painting_area(self, builder, layer_index);
            let common = painter.common_properties(self, builder, state, layer_index, bounds);''','''        let b = painter.style.get_background();
        let background_color = painter.style.resolve_color(&b.background_color);
        let mask_chain = self.build_mask_clip_chain(builder, state, painter.style);
        let layer_index = b.background_image.0.len() - 1;
        let clips_to_text = matches!(get_cyclic(&b.background_clip.0, layer_index), BackgroundClipValue::Text);
        if background_color.alpha > 0.0 && !clips_to_text {
            // https://drafts.csswg.org/css-backgrounds/#background-color
            // “The background color is clipped according to the background-clip
            //  value associated with the bottom-most background image layer.”
            let bounds = painter.painting_area(self, builder, layer_index);
            let mut common = painter.common_properties(self, builder, state, layer_index, bounds);
            if let Some(mask_chain) = mask_chain {
                common.clip_chain_id = mask_chain;
            }''')
    s=sub1(s,'''        self.build_background_image(builder, state, painter);
    }''','''        self.build_background_image(builder, state, painter, mask_chain);
    }

    /// Build a WebRender image-mask clip chain for the first `mask-image` layer of `style`,
    /// parented to the border-edge clip. Only `url()` masks are supported; gradient masks
    /// and repeat are ignored.
    fn build_mask_clip_chain(
        &self,
        builder: &mut DisplayListBuilder,
        state: &TraversalState,
        style: &ComputedValues,
    ) -> Option<wr::ClipChainId> {
        let image = style.get_svg().mask_image.0.first()?;
        let node = self.fragment.base.tag.map(|tag| tag.node);
        let ResolvedImage::Image { image, size } =
            builder.image_resolver.resolve_image(node, image).ok()?
        else {
            return None;
        };
        let intrinsic = NaturalSizes::from_width_and_height(size.width, size.height);
        let layer = background::layout_mask(self, builder, state, style, intrinsic)?;
        let scale = builder.device_pixel_ratio.get();
        let image_key = match image {
            CachedImage::Raster(raster_image) => raster_image.id,
            CachedImage::Vector(vector_image) => node.and_then(|node| {
                let size: DeviceIntSize = Size2D::new(
                    layer.tile_size.width * scale,
                    layer.tile_size.height * scale,
                )
                .to_i32();
                builder.image_resolver.rasterize_vector_image(
                    vector_image.id,
                    size,
                    node,
                    vector_image.svg_id,
                )
            })
            .and_then(|rasterized_image| rasterized_image.id),
        }?;
        let spatial_id = builder.spatial_id(state.spatial_id);
        let rect = LayoutRect::from_origin_and_size(layer.bounds.min, layer.tile_size);
        let clip_id = builder.wr().define_clip_image_mask(
            spatial_id,
            wr::ImageMask { image: image_key, rect },
            &[],
            wr::FillRule::Nonzero,
        );
        Some(
            builder
                .wr()
                .define_clip_chain(Some(layer.common.clip_chain_id), [clip_id]),
        )
    }''')
    s=sub1(s,'''    fn build_background_image(
        &self,
        builder: &mut DisplayListBuilder,
        state: &TraversalState,
        painter: &BackgroundPainter,
    ) {''','''    fn build_background_image(
        &self,
        builder: &mut DisplayListBuilder,
        state: &TraversalState,
        painter: &BackgroundPainter,
        mask_chain: Option<wr::ClipChainId>,
    ) {''')
    s=sub1(s,'''        for (index, image) in b.background_image.0.iter().enumerate().rev() {
            let Ok(resolved_image) = builder.image_resolver.resolve_image(node, image) else {
                continue;
            };''','''        let apply_mask = |mut layer: background::BackgroundLayer| {
            if let Some(mask_chain) = mask_chain {
                layer.common.clip_chain_id = mask_chain;
            }
            layer
        };
        for (index, image) in b.background_image.0.iter().enumerate().rev() {
            if matches!(get_cyclic(&b.background_clip.0, index), BackgroundClipValue::Text) {
                continue;
            }
            let Ok(resolved_image) = builder.image_resolver.resolve_image(node, image) else {
                continue;
            };''')
    s=sub1(s,'''                    let Some(layer) =
                        &background::layout_layer(self, painter, builder, state, index, intrinsic)
                    else {
                        continue;
                    };''','''                    let Some(layer) =
                        background::layout_layer(self, painter, builder, state, index, intrinsic)
                            .map(apply_mask)
                    else {
                        continue;
                    };''')
    s=sub1(s,'''                    let layer =
                        background::layout_layer(self, painter, builder, state, index, intrinsic);
''','''                    let layer =
                        background::layout_layer(self, painter, builder, state, index, intrinsic)
                            .map(apply_mask);
''')
    return s
rw(L+'display_list/mod.rs',dl)
print('servo patched')
