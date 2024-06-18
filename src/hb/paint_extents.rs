use ttf_parser::{GlyphId, Transform};
use ttf_parser::colr::{ClipBox, CompositeMode, Paint};
use crate::hb::face::GlyphExtents;

struct hb_bounds_t {
    bounded: bool,
    glyph_extents: GlyphExtents
}


impl hb_bounds_t {
    fn new(extents: &GlyphExtents) -> Self {
        hb_bounds_t {
            glyph_extents: *extents,
            bounded: false
        }
    }
}

impl Default for hb_bounds_t {
    fn default() -> Self {
        Self::new(&GlyphExtents::default())
    }
}

struct hb_paint_extents_context_t {
    clips: alloc::vec::Vec<hb_bounds_t>,
    bounds: alloc::vec::Vec<hb_bounds_t>,
    transforms: alloc::vec::Vec<Transform>,
}

impl hb_paint_extents_context_t {
    fn push_transform(&mut self, trans: &Transform) {
        let r = self.transforms.last().unwrap_or_default();
        let new = Transform::combine(*r, *trans);
        self.transforms.push(new);
    }

    fn pop_transform(&mut self) {
        self.transforms.pop();
    }

    fn push_clip(&mut self, extents: &GlyphExtents) {
        let b = hb_bounds_t::new(extents);
        self.clips.push(b);
    }

    fn pop_clip(&mut self) {
        self.clips.pop();
    }

    fn push_group(&mut self) {
        self.bounds.push(hb_bounds_t::default());
    }

    fn pop_group(&mut self) {
        self.bounds.pop();
    }

    fn add_extents(extents: &GlyphExtents) {
        todo!()
    }
}

impl ttf_parser::colr::Painter for hb_paint_extents_context_t {
    fn outline_glyph(&mut self, glyph_id: GlyphId) {
        todo!()
    }

    fn paint(&mut self, paint: Paint<'_>) {
        todo!()
    }

    fn push_clip(&mut self) {
        todo!()
    }

    fn push_clip_box(&mut self, clipbox: ClipBox) {
        todo!()
    }

    fn pop_clip(&mut self) {
        todo!()
    }

    fn push_layer(&mut self, _: CompositeMode) {
        self.push_group();
    }

    fn pop_layer(&mut self) {
        self.pop_group();
    }

    fn push_transform(&mut self, transform: Transform) {
        self.push_transform(&transform);
    }

    fn pop_transform(&mut self) {
        self.pop_transform();
    }
}