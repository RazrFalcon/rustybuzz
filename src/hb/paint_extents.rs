use crate::hb::face::hb_font_t;
use alloc::vec;
use ttf_parser::colr::{ClipBox, CompositeMode, Paint};
use ttf_parser::{GlyphId, Transform};

type hb_extents_t = ttf_parser::RectF;

#[derive(PartialEq, Eq, Clone, Copy)]
enum status_t {
    EMPTY,
    BOUNDED,
    UNBOUNDED,
}

#[derive(Clone, Copy)]
struct hb_bounds_t {
    status: status_t,
    extents: hb_extents_t,
}

impl hb_bounds_t {
    fn from_extents(extents: &hb_extents_t) -> Self {
        hb_bounds_t {
            extents: *extents,
            status: status_t::BOUNDED,
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

    fn push_clip(&mut self, mut extents: hb_extents_t) {
        if let Some(r) = self.transforms.last_mut() {
            r.transform_extents(&mut extents);
        }

        let b = hb_bounds_t::from_extents(&extents);
        self.clips.push(b);
    }

    fn pop_clip(&mut self) {
        self.clips.pop();
    }

    fn push_group(&mut self, mode: CompositeMode) {
        let src_bounds = hb_bounds_t::default();

        // In harfbuzz, this is in `pop_group` instead, but since we have the composite mode
        // in push group, we need to do it here.
        if let Some(backdrop_bounds) = self.groups.last_mut() {
            match mode {
                CompositeMode::Clear => backdrop_bounds.status = status_t::EMPTY,
                CompositeMode::Source | CompositeMode::SourceOut => *backdrop_bounds = src_bounds,
                CompositeMode::Destination | CompositeMode::DestinationOut => {}
                CompositeMode::SourceIn | CompositeMode::DestinationIn => {
                    if src_bounds.status == status_t::EMPTY {
                        backdrop_bounds.status = status_t::EMPTY;
                    } else if src_bounds.status == status_t::BOUNDED {
                        backdrop_bounds.extents.x_min =
                            backdrop_bounds.extents.x_min.max(src_bounds.extents.x_min);
                        backdrop_bounds.extents.y_min =
                            backdrop_bounds.extents.y_min.max(src_bounds.extents.y_min);
                        backdrop_bounds.extents.x_max =
                            backdrop_bounds.extents.x_max.min(src_bounds.extents.x_max);
                        backdrop_bounds.extents.y_max =
                            backdrop_bounds.extents.y_max.min(src_bounds.extents.y_max);

                        if backdrop_bounds.extents.x_min >= backdrop_bounds.extents.x_max
                            || backdrop_bounds.extents.y_min >= backdrop_bounds.extents.y_max
                        {
                            backdrop_bounds.status = status_t::EMPTY;
                        }
                    }
                }
                _ => {
                    if src_bounds.status == status_t::UNBOUNDED {
                        backdrop_bounds.status = status_t::UNBOUNDED;
                    } else if src_bounds.status == status_t::BOUNDED {
                        if backdrop_bounds.status == status_t::EMPTY {
                            *backdrop_bounds = src_bounds;
                        } else if backdrop_bounds.status == status_t::BOUNDED {
                            backdrop_bounds.extents.x_min =
                                backdrop_bounds.extents.x_min.min(src_bounds.extents.x_min);
                            backdrop_bounds.extents.y_min =
                                backdrop_bounds.extents.y_min.min(src_bounds.extents.y_min);
                            backdrop_bounds.extents.x_max =
                                backdrop_bounds.extents.x_max.max(src_bounds.extents.x_max);
                            backdrop_bounds.extents.y_max =
                                backdrop_bounds.extents.y_max.max(src_bounds.extents.y_max);
                        }
                    }
                }
            }
        }

        self.groups.push(src_bounds);
    }

    fn pop_group(&mut self) {
        self.groups.pop();
    }

    fn paint(&mut self) {
        if let (Some(clip), Some(group)) = (self.clips.last_mut(), self.groups.last_mut()) {
            if clip.status == status_t::EMPTY {
                return; // Shouldn't happen.
            }

            if group.status == status_t::UNBOUNDED {
                return;
            }

            if group.status == status_t::EMPTY {
                *group = *clip;
                return;
            }

            // Group is bounded now.  Clip is not empty.
            if clip.status == status_t::UNBOUNDED {
                group.status = status_t::UNBOUNDED;
                return;
            }

            // Both are bounded. Union.
            group.extents.x_min = group.extents.x_min.min(clip.extents.x_min);
            group.extents.y_min = group.extents.y_min.min(clip.extents.y_min);
            group.extents.x_max = group.extents.x_max.max(clip.extents.x_max);
            group.extents.y_max = group.extents.y_max.max(clip.extents.y_max);
        }
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
        let extents = hb_extents_t {
            x_min: 0.0,
            y_min: 0.0,
            x_max: -1.0,
            y_max: -1.0,
        };

        let mut extent_builder = ExtentBuilder(extents);
        self.face
            .outline_glyph(self.current_glyph, &mut extent_builder);

        let extents = extent_builder.0;

        if extents.x_min < extents.x_max {
            self.push_clip(extents);
        }
    }

    fn push_clip_box(&mut self, clipbox: ClipBox) {
        self.push_clip(clipbox);
    }

    fn pop_clip(&mut self) {
        self.pop_clip();
    }

    fn push_layer(&mut self, mode: CompositeMode) {
        self.push_group(mode);
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

trait TransformExt {
    fn transform_distance(&self, dx: &mut f32, dy: &mut f32);
    fn transform_point(&self, x: &mut f32, y: &mut f32);
    fn transform_extents(&self, extents: &mut hb_extents_t);
}

impl TransformExt for Transform {
    fn transform_distance(&self, dx: &mut f32, dy: &mut f32) {
        let new_x = self.a * *dx + self.c * *dy;
        let new_y = self.b * *dx + self.d * *dy;
        *dx = new_x;
        *dy = new_y;
    }

    fn transform_point(&self, x: &mut f32, y: &mut f32) {
        self.transform_distance(x, y);
        *x += self.e;
        *y += self.f;
    }

    fn transform_extents(&self, extents: &mut hb_extents_t) {
        let mut quad_x = [0.0f32; 4];
        let mut quad_y = [0.0f32; 4];

        quad_x[0] = extents.x_min;
        quad_y[0] = extents.y_min;
        quad_x[1] = extents.x_min;
        quad_y[1] = extents.y_max;
        quad_x[2] = extents.x_max;
        quad_y[2] = extents.y_min;
        quad_x[3] = extents.x_max;
        quad_y[3] = extents.y_max;

        for i in 0..4 {
            self.transform_point(&mut quad_x[i], &mut quad_y[i])
        }

        extents.x_max = quad_x[0];
        extents.x_min = extents.x_max;
        extents.y_max = quad_y[0];
        extents.y_min = extents.y_max;

        for i in 1..4 {
            extents.x_min = extents.x_min.min(quad_x[i]);
            extents.y_min = extents.y_min.min(quad_y[i]);
            extents.x_max = extents.x_max.min(quad_x[i]);
            extents.y_max = extents.y_max.min(quad_y[i]);
        }
    }
}

struct ExtentBuilder(hb_extents_t);

impl ExtentBuilder {
    fn add_point(&mut self, x: f32, y: f32) {
        if self.0.x_max < self.0.x_min {
            self.0.x_max = x;
            self.0.x_min = x;
            self.0.y_max = y;
            self.0.y_min = y;
        } else {
            self.0.x_min = self.0.x_min.min(x);
            self.0.y_min = self.0.y_min.min(y);
            self.0.x_max = self.0.x_max.max(x);
            self.0.y_max = self.0.y_max.max(y);
        }
    }
}

impl ttf_parser::OutlineBuilder for ExtentBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.add_point(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.add_point(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.add_point(x1, y1);
        self.add_point(x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.add_point(x1, y1);
        self.add_point(x2, y2);
        self.add_point(x, y);
    }

    fn close(&mut self) {}
}
