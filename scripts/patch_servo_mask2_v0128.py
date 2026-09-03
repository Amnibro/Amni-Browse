def rw(p,f):
    s=open(p,encoding='utf-8').read(); n=f(s); assert n!=s,p; open(p,'w',encoding='utf-8',newline='\n').write(n)
def sub1(s,a,b):
    assert s.count(a)==1,(a[:70],s.count(a)); return s.replace(a,b)
p='vendor/servo/components/layout/display_list/mod.rs'
def dl(s):
    s=sub1(s,r'''        let mask_chain = self.build_mask_clip_chain(builder, state, painter.style);
        if let Some(mask_chain) = mask_chain {
            let spatial_id = builder.spatial_id(state.spatial_id);
            builder.wr().push_stacking_context(
                spatial_id,
                PrimitiveFlags::empty(),
                Some(mask_chain),
                TransformStyle::Flat,
                wr::MixBlendMode::Normal,
                &[],
                &[],
                RasterSpace::Screen,
                StackingContextFlags::empty(),
                None,
            );
        }
        let layer_index = b.background_image.0.len() - 1;''',r'''        if let Some((image_key, layer)) = self.mask_image_layer(builder, state, painter.style) {
            // `mask-image` over a solid `background-color`: paint the mask image itself and
            // tint its alpha with the background colour through a colour matrix, which
            // WebRender applies to any primitive (image-mask clips only work on pictures).
            let tint = rgba(background_color);
            let matrix = [
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, tint.a,
                tint.r, tint.g, tint.b, 0.0,
            ];
            let spatial_id = builder.spatial_id(state.spatial_id);
            builder.wr().push_stacking_context(
                spatial_id,
                PrimitiveFlags::empty(),
                None,
                TransformStyle::Flat,
                wr::MixBlendMode::Normal,
                &[wr::FilterOp::ColorMatrix(matrix)],
                &[],
                RasterSpace::Screen,
                StackingContextFlags::empty(),
                None,
            );
            builder.wr().push_image(
                &layer.common,
                layer.bounds,
                wr::ImageRendering::Auto,
                wr::AlphaType::PremultipliedAlpha,
                image_key,
                wr::ColorF::WHITE,
            );
            builder.wr().pop_stacking_context();
            builder.mark_is_paintable();
            return;
        }
        let layer_index = b.background_image.0.len() - 1;''')
    s=sub1(s,r'''        self.build_background_image(builder, state, painter);
        if mask_chain.is_some() {
            builder.wr().pop_stacking_context();
        }
    }

    /// Build a WebRender image-mask clip chain for the first `mask-image` layer of `style`,
    /// parented to the border-edge clip. Only `url()` masks are supported; gradient masks
    /// and repeat are ignored.
    fn build_mask_clip_chain(
        &self,
        builder: &mut DisplayListBuilder,
        state: &TraversalState,
        style: &ComputedValues,
    ) -> Option<wr::ClipChainId> {''',r'''        self.build_background_image(builder, state, painter);
    }

    /// Resolve the first `mask-image: url()` layer of `style` to a WebRender image key and
    /// its laid-out rectangle over the border box. Gradient masks and repeat are ignored.
    fn mask_image_layer(
        &self,
        builder: &mut DisplayListBuilder,
        state: &TraversalState,
        style: &ComputedValues,
    ) -> Option<(wr::ImageKey, background::BackgroundLayer)> {''')
    s=sub1(s,r'''        let spatial_id = builder.spatial_id(state.spatial_id);
        let rect = LayoutRect::from_origin_and_size(layer.bounds.min, layer.tile_size);
        log::info!("mask-image clip: image {:?} natural {:?} rect {:?} border {:?}", image_key, size, rect, self.border_rect);
        let clip_id = builder.wr().define_clip_image_mask(
            spatial_id,
            wr::ImageMask { image: image_key, rect },
            &[],
            wr::FillRule::Nonzero,
        );
        let parent = match layer.common.clip_chain_id {
            ClipChainId::INVALID => None,
            id => Some(id),
        };
        Some(builder.wr().define_clip_chain(parent, [clip_id]))
    }''',r'''        Some((image_key, layer))
    }''')
    return s
rw(p,dl)
print('mask2 patched')
