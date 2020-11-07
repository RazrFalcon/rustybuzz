use crate::{ffi, Direction, Font};
use crate::buffer::{Buffer, GlyphPosition};
use crate::unicode::{modified_combining_class, CanonicalCombiningClass, GeneralCategory, Space};
use crate::ot::*;

fn recategorize_combining_class(u: u32, mut class: u8) -> u8 {
    use CanonicalCombiningClass as Class;
    use modified_combining_class as mcc;

    if class >= 200 {
        return class;
    }

    // Thai / Lao need some per-character work.
    if u & !0xFF == 0x0E00 {
        if class == 0 {
            match u {
                0x0E31 |
                0x0E34 |
                0x0E35 |
                0x0E36 |
                0x0E37 |
                0x0E47 |
                0x0E4C |
                0x0E4D |
                0x0E4E => class = Class::AboveRight as u8,

                0x0EB1 |
                0x0EB4 |
                0x0EB5 |
                0x0EB6 |
                0x0EB7 |
                0x0EBB |
                0x0ECC |
                0x0ECD => class = Class::Above as u8,

                0x0EBC => class = Class::Below as u8,

                _ => {}
            }
        } else {
            // Thai virama is below-right
            if u == 0x0E3A {
                class = Class::BelowRight as u8;
            }
        }
    }

    match class {
        // Hebrew
        mcc::CCC10 => Class::Below as u8,         // sheva
        mcc::CCC11 => Class::Below as u8,         // hataf segol
        mcc::CCC12 => Class::Below as u8,         // hataf patah
        mcc::CCC13 => Class::Below as u8,         // hataf qamats
        mcc::CCC14 => Class::Below as u8,         // hiriq
        mcc::CCC15 => Class::Below as u8,         // tsere
        mcc::CCC16 => Class::Below as u8,         // segol
        mcc::CCC17 => Class::Below as u8,         // patah
        mcc::CCC18 => Class::Below as u8,         // qamats
        mcc::CCC20 => Class::Below as u8,         // qubuts
        mcc::CCC22 => Class::Below as u8,         // meteg
        mcc::CCC23 => Class::AttachedAbove as u8, // rafe
        mcc::CCC24 => Class::AboveRight as u8,    // shin dot
        mcc::CCC25 => Class::AboveLeft as u8,     // sin dot
        mcc::CCC19 => Class::AboveLeft as u8,     // holam
        mcc::CCC26 => Class::Above as u8,         // point varika
        mcc::CCC21 => class,                      // dagesh

        // Arabic and Syriac
        mcc::CCC27 => Class::Above as u8, // fathatan
        mcc::CCC28 => Class::Above as u8, // dammatan
        mcc::CCC30 => Class::Above as u8, // fatha
        mcc::CCC31 => Class::Above as u8, // damma
        mcc::CCC33 => Class::Above as u8, // shadda
        mcc::CCC34 => Class::Above as u8, // sukun
        mcc::CCC35 => Class::Above as u8, // superscript alef
        mcc::CCC36 => Class::Above as u8, // superscript alaph
        mcc::CCC29 => Class::Below as u8, // kasratan
        mcc::CCC32 => Class::Below as u8, // kasra

        // Thai
        mcc::CCC103 => Class::BelowRight as u8, // sara u / sara uu
        mcc::CCC107 => Class::AboveRight as u8, // mai

        // Lao
        mcc::CCC118 => Class::Below as u8, // sign u / sign uu
        mcc::CCC122 => Class::Above as u8, // mai

        // Tibetian
        mcc::CCC129 => Class::Below as u8, // sign aa
        mcc::CCC130 => Class::Above as u8, // sign i
        mcc::CCC132 => Class::Below as u8, // sign u

        _ => class,
    }
}

#[no_mangle]
pub extern "C" fn _rb_ot_shape_fallback_mark_position_recategorize_marks(
    plan: *const ffi::rb_ot_shape_plan_t,
    font: *mut ffi::rb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    recategorize_marks(&plan, font, &mut buffer)
}

fn recategorize_marks(_: &ShapePlan, _: &Font, buffer: &mut Buffer) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        if info.general_category() == GeneralCategory::NonspacingMark {
            let mut class = info.modified_combining_class();
            class = recategorize_combining_class(info.codepoint, class);
            info.set_modified_combining_class(class);
        }
    }
}

fn zero_mark_advances(
    buffer: &mut Buffer,
    start: usize,
    end: usize,
    adjust_offsets_when_zeroing: bool,
) {
    for (info, pos) in buffer.info[start..end].iter().zip(&mut buffer.pos[start..end]) {
        if info.general_category() == GeneralCategory::NonspacingMark {
            if adjust_offsets_when_zeroing {
                pos.x_offset -= pos.x_advance;
                pos.y_offset -= pos.y_advance;
            }
            pos.x_advance = 0;
            pos.y_advance = 0;
        }
    }
}

fn position_mark(
    _: &ShapePlan,
    font: &Font,
    direction: Direction,
    codepoint: u32,
    pos: &mut GlyphPosition,
    base_extents: &mut ffi::rb_glyph_extents_t,
    combining_class: CanonicalCombiningClass,
) {
    use CanonicalCombiningClass as Class;

    let mark_extents = match font.glyph_extents(codepoint) {
        Some(extents) => extents,
        None => return,
    };

    let y_gap = font.units_per_em() / 16;
    pos.x_offset = 0;
    pos.y_offset = 0;

    // We don't position LEFT and RIGHT marks.

    // X positioning
    match combining_class {
        Class::DoubleBelow |
        Class::DoubleAbove if direction.is_horizontal() => {
            pos.x_offset += base_extents.x_bearing
                + if direction.is_forward() { base_extents.width } else { 0 }
                - mark_extents.width / 2 - mark_extents.x_bearing;
        }

        Class::AttachedBelowLeft |
        Class::BelowLeft |
        Class::AboveLeft => {
            // Left align.
            pos.x_offset += base_extents.x_bearing - mark_extents.x_bearing;
        }

        Class::AttachedAboveRight |
        Class::BelowRight |
        Class::AboveRight => {
            // Right align.
            pos.x_offset += base_extents.x_bearing + base_extents.width
                - mark_extents.width - mark_extents.x_bearing;
        }

        Class::AttachedBelow |
        Class::AttachedAbove |
        Class::Below |
        Class::Above |
        _ => {
            // Center align.
            pos.x_offset += base_extents.x_bearing
                + (base_extents.width - mark_extents.width) / 2
                - mark_extents.x_bearing;
        }
    }

    let is_attached = matches!(
        combining_class,
        Class::AttachedBelowLeft |
        Class::AttachedBelow |
        Class::AttachedAbove |
        Class::AttachedAboveRight
    );

    // Y positioning.
    match combining_class {
        Class::DoubleBelow |
        Class::BelowLeft |
        Class::Below |
        Class::BelowRight |
        Class::AttachedBelowLeft |
        Class::AttachedBelow => {
            if !is_attached {
                // Add gap.
                base_extents.height -= y_gap;
            }

            pos.y_offset = base_extents.y_bearing + base_extents.height
                - mark_extents.y_bearing;

            // Never shift up "below" marks.
            if (y_gap > 0) == (pos.y_offset > 0) {
                base_extents.height -= pos.y_offset;
                pos.y_offset = 0;
            }

            base_extents.height += mark_extents.height;
        }

        Class::DoubleAbove |
        Class::AboveLeft |
        Class::Above |
        Class::AboveRight |
        Class::AttachedAbove |
        Class::AttachedAboveRight => {
            if !is_attached {
                // Add gap.
                base_extents.y_bearing += y_gap;
                base_extents.height -= y_gap;
            }

            pos.y_offset = base_extents.y_bearing
                - (mark_extents.y_bearing + mark_extents.height);

            // Don't shift down "above" marks too much.
            if (y_gap > 0) != (pos.y_offset > 0) {
                let correction = -pos.y_offset / 2;
                base_extents.y_bearing += correction;
                base_extents.height -= correction;
                pos.y_offset += correction;
            }

            base_extents.y_bearing -= mark_extents.height;
            base_extents.height += mark_extents.height;
        }

        _ => {}
    }
}

fn position_around_base(
    plan: &ShapePlan,
    font: &Font,
    buffer: &mut Buffer,
    base: usize,
    end: usize,
    adjust_offsets_when_zeroing: bool,
) {
    let mut horizontal_dir = Direction::Invalid;
    buffer.unsafe_to_break(base, end);

    let base_info = &buffer.info[base];
    let base_pos = &buffer.pos[base];
    let mut base_extents = match font.glyph_extents(base_info.codepoint) {
        Some(extents) => extents,
        None => {
            // If extents don't work, zero marks and go home.
            zero_mark_advances(buffer, base + 1, end, adjust_offsets_when_zeroing);
            return;
        }
    };

    base_extents.y_bearing += base_pos.y_offset;
    base_extents.x_bearing = 0;

    // Use horizontal advance for horizontal positioning.
    // Generally a better idea. Also works for zero-ink glyphs. See:
    // https://github.com/harfbuzz/harfbuzz/issues/1532
    base_extents.width = font.glyph_h_advance(base_info.codepoint) as i32;

    let lig_id = base_info.lig_id() as u32;
    let num_lig_components = base_info.lig_num_comps() as i32;

    let mut x_offset = 0;
    let mut y_offset = 0;
    if buffer.direction.is_forward() {
        x_offset -= base_pos.x_advance;
        y_offset -= base_pos.y_advance;
    }

    let mut last_lig_component: i32 = -1;
    let mut last_combining_class: u8 = 255;
    let mut component_extents = base_extents;
    let mut cluster_extents = base_extents;

    for (info, pos) in buffer.info[base+1..end].iter().zip(&mut buffer.pos[base+1..end]) {
        if info.modified_combining_class() != 0 {
            if num_lig_components > 1 {
                let this_lig_id = info.lig_id() as u32;
                let mut this_lig_component = info.lig_comp() as i32 - 1;

                // Conditions for attaching to the last component.
                if lig_id == 0 || lig_id != this_lig_id || this_lig_component >= num_lig_components {
                    this_lig_component = num_lig_components - 1;
                }

                if last_lig_component != this_lig_component {
                    last_lig_component = this_lig_component;
                    last_combining_class = 255;
                    component_extents = base_extents;

                    if horizontal_dir == Direction::Invalid {
                        let plan_dir = plan.direction();
                        horizontal_dir = if plan_dir.is_horizontal() {
                            plan_dir
                        } else {
                            Direction::from_script(plan.script()).unwrap_or_default()
                        };
                    }

                    component_extents.x_bearing +=
                        (if horizontal_dir == Direction::LeftToRight {
                            this_lig_component
                        } else {
                            num_lig_components - 1 - this_lig_component
                        } * component_extents.width) / num_lig_components;

                    component_extents.width /= num_lig_components;
                }
            }

            let this_combining_class = info.modified_combining_class();
            if last_combining_class != this_combining_class {
                last_combining_class = this_combining_class;
                cluster_extents = component_extents;
            }

            position_mark(
                &plan,
                font,
                buffer.direction,
                info.codepoint,
                pos,
                &mut cluster_extents,
                unsafe { std::mem::transmute(this_combining_class) },
            );

            pos.x_advance = 0;
            pos.y_advance = 0;
            pos.x_offset += x_offset;
            pos.y_offset += y_offset;
        } else {
            if buffer.direction.is_forward() {
                x_offset -= pos.x_advance;
                y_offset -= pos.y_advance;
            } else {
                x_offset += pos.x_advance;
                y_offset += pos.y_advance;
            }
        }
    }
}

fn position_cluster(
    plan: &ShapePlan,
    font: &Font,
    buffer: &mut Buffer,
    start: usize,
    end: usize,
    adjust_offsets_when_zeroing: bool,
) {
    if end - start < 2 {
        return;
    }

    // Find the base glyph
    let mut i = start;
    while i < end {
        if !buffer.info[i].is_unicode_mark() {
            // Find mark glyphs
            let mut j = i + 1;
            while j < end && buffer.info[j].is_unicode_mark() {
                j += 1;
            }

            position_around_base(plan, font, buffer, i, j, adjust_offsets_when_zeroing);
            i = j - 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub extern "C" fn _rb_ot_shape_fallback_mark_position(
    plan: *const ffi::rb_ot_shape_plan_t,
    font: *mut ffi::rb_font_t,
    buffer: *mut ffi::rb_buffer_t,
    adjust_offsets_when_zeroing: ffi::rb_bool_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    let adjust_offsets_when_zeroing = adjust_offsets_when_zeroing != 0;
    mark_position(&plan, font, &mut buffer, adjust_offsets_when_zeroing)
}

fn mark_position(
    plan: &ShapePlan,
    font: &Font,
    buffer: &mut Buffer,
    adjust_offsets_when_zeroing: bool,
) {
    let mut start = 0;
    let len = buffer.len;
    for i in 1..len {
        if !buffer.info[i].is_unicode_mark() {
            position_cluster(&plan, font, buffer, start, i, adjust_offsets_when_zeroing);
            start = i;
        }
    }

    position_cluster(&plan, font, buffer, start, len, adjust_offsets_when_zeroing);
}

/// Performs font-assisted kerning.
#[no_mangle]
pub extern "C" fn _rb_ot_shape_fallback_kern(
    _: *const ffi::rb_ot_shape_plan_t,
    _: *mut ffi::rb_font_t,
    _: *mut ffi::rb_buffer_t,
) {}

/// Adjusts width of various spaces.
#[no_mangle]
pub extern "C" fn _rb_ot_shape_fallback_spaces(
    plan: *const ffi::rb_ot_shape_plan_t,
    font: *mut ffi::rb_font_t,
    buffer: *mut ffi::rb_buffer_t,
) {
    let plan = ShapePlan::from_ptr(plan);
    let font = Font::from_ptr(font);
    let mut buffer = Buffer::from_ptr_mut(buffer);
    spaces(&plan, &font, &mut buffer);
}

fn spaces(_: &ShapePlan, font: &Font, buffer: &mut Buffer) {
    let len = buffer.len;
    let horizontal = buffer.direction.is_horizontal();
    for (info, pos) in buffer.info[..len].iter().zip(&mut buffer.pos[..len]) {
        let space_type = match info.space_fallback() {
            Some(fallback) if !info.is_ligated() => fallback,
            _ => continue,
        };

        match space_type {
            Space::Space => {}

            Space::SpaceEm |
            Space::SpaceEm2 |
            Space::SpaceEm3 |
            Space::SpaceEm4 |
            Space::SpaceEm5 |
            Space::SpaceEm6 |
            Space::SpaceEm16 => {
                let length = (font.units_per_em() + (space_type as i32) / 2) / space_type as i32;
                if horizontal {
                    pos.x_advance = length;
                } else {
                    pos.y_advance = -length;
                }
            }

            Space::Space4Em18 => {
                let length = ((font.units_per_em() as i64) * 4 / 18) as i32;
                if horizontal {
                    pos.x_advance = length
                } else {
                    pos.y_advance = -length;
                }
            }

            Space::SpaceFigure => {
                for u in '0'..='9' {
                    if let Some(glyph) = font.glyph_index(u as u32) {
                        if horizontal {
                            pos.x_advance = font.glyph_h_advance(glyph.0 as u32) as i32;
                        } else {
                            pos.y_advance = font.glyph_v_advance(glyph.0 as u32);
                        }
                        break;
                    }
                }
            }

            Space::SpacePunctuation => {
                let punct = font
                    .glyph_index('.' as u32)
                    .or_else(|| font.glyph_index(',' as u32));

                if let Some(glyph) = punct {
                    if horizontal {
                        pos.x_advance = font.glyph_h_advance(glyph.0 as u32) as i32;
                    } else {
                        pos.y_advance = font.glyph_v_advance(glyph.0 as u32);
                    }
                }
            }

            Space::SpaceNarrow => {
                // Half-space?
                // Unicode doc https://unicode.org/charts/PDF/U2000.pdf says ~1/4 or 1/5 of EM.
                // However, in my testing, many fonts have their regular space being about that
                // size. To me, a percentage of the space width makes more sense. Half is as
                // good as any.
                if horizontal {
                    pos.x_advance /= 2;
                } else {
                    pos.y_advance /= 2;
                }
            }
        }
    }
}
