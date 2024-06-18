
use crate::hb::face::{hb_font_t, hb_glyph_extents_t};
use ttf_parser::colr::{ClipBox, CompositeMode, Paint};
use ttf_parser::{GlyphId, Transform};
use crate::hb::paint_extents::status_t::BOUNDED;
use alloc::vec;

type hb_extents_t = ttf_parser::RectF;

enum status_t {
    EMPTY,
    BOUNDED,
    UNBOUNDED
}

struct hb_bounds_t {
    status: status_t,
    extents: hb_extents_t,
}

impl hb_bounds_t {
    fn from_extents(extents: &hb_extents_t) -> Self {
        hb_bounds_t {
            extents: *extents,
            status: BOUNDED
        }
    }

    fn from_status(status: status_t) -> Self {
        hb_bounds_t {
            status,
            ..hb_bounds_t::default()
        }
    }
}

impl Default for hb_bounds_t {
    fn default() -> Self {
        Self::from_extents(&hb_extents_t {
            x_min: 0.0,
            x_max: 0.0,
            y_min: 0.0,
            y_max: 0.0,
        })
    }
}

struct hb_paint_extents_context_t<'a> {
    clips: vec::Vec<hb_bounds_t>,
    groups: vec::Vec<hb_bounds_t>,
    transforms: vec::Vec<Transform>,
    face: &'a hb_font_t<'a>,
    current_glyph: GlyphId,
}

impl<'a> hb_paint_extents_context_t<'a> {
    fn new(face: &'a hb_font_t) -> Self {
        Self {
            clips: vec![hb_bounds_t::from_status(status_t::UNBOUNDED)],
            groups: vec![hb_bounds_t::from_status(status_t::EMPTY)],
            transforms: vec![Transform::default()],
            face,
            current_glyph: Default::default(),
        }
    }

    fn push_transform(&mut self, trans: &Transform) {
        let r = self
            .transforms
            .last()
            .copied()
            .unwrap_or(Transform::default());
        let new = Transform::combine(r, *trans);
        self.transforms.push(new);
    }

    fn pop_transform(&mut self) {
        self.transforms.pop();
    }

    fn push_clip(&mut self, extents: &hb_extents_t) {
        let b = hb_bounds_t::from_extents(extents);
        self.clips.push(b);
    }

    fn pop_clip(&mut self) {
        self.clips.pop();
    }

    fn push_group(&mut self) {
        self.groups.push(hb_bounds_t::default());
    }

    fn pop_group(&mut self) {
        self.groups.pop();
    }

    fn paint(&mut self) {
        todo!()
    }
}

impl ttf_parser::colr::Painter<'_> for hb_paint_extents_context_t<'_> {
    fn outline_glyph(&mut self, glyph_id: GlyphId) {
        self.current_glyph = glyph_id;
    }

    fn paint(&mut self, _: Paint<'_>) {
        self.paint();
    }

    fn push_clip(&mut self) {
        let mut glyph_extents = hb_glyph_extents_t::default();
        self.face
            .glyph_extents(self.current_glyph, &mut glyph_extents);

        let extents = hb_extents_t {
            x_min: glyph_extents.x_bearing as f32,
            y_min: glyph_extents.y_bearing as f32 + glyph_extents.height as f32,
            x_max: glyph_extents.x_bearing as f32 + glyph_extents.width as f32,
            y_max: glyph_extents.y_bearing as f32,
        };
        self.push_clip(&extents);
    }

    fn push_clip_box(&mut self, clipbox: ClipBox) {
        self.push_clip(&clipbox);
    }

    fn pop_clip(&mut self) {
        self.pop_clip();
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
