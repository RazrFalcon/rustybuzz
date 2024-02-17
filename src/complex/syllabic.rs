use crate::buffer::{hb_buffer_t, BufferFlags};
use crate::{hb_font_t, hb_glyph_info_t};

pub fn insert_dotted_circles(
    face: &hb_font_t,
    buffer: &mut hb_buffer_t,
    broken_syllable_type: u8,
    dottedcircle_category: u8,
    repha_category: Option<u8>,
    dottedcircle_position: Option<u8>,
) {
    if buffer
        .flags
        .contains(BufferFlags::DO_NOT_INSERT_DOTTED_CIRCLE)
    {
        return;
    }

    // Note: This loop is extra overhead, but should not be measurable.
    // TODO Use a buffer scratch flag to remove the loop.
    let has_broken_syllables = buffer
        .info_slice()
        .iter()
        .any(|info| info.syllable() & 0x0F == broken_syllable_type);

    if !has_broken_syllables {
        return;
    }

    let dottedcircle_glyph = match face.glyph_index(0x25CC) {
        Some(g) => g.0 as u32,
        None => return,
    };

    let mut dottedcircle = hb_glyph_info_t {
        glyph_id: 0x25CC,
        ..hb_glyph_info_t::default()
    };
    dottedcircle.set_complex_var_u8_category(dottedcircle_category);
    if let Some(dottedcircle_position) = dottedcircle_position {
        dottedcircle.set_complex_var_u8_auxiliary(dottedcircle_position);
    }
    dottedcircle.glyph_id = dottedcircle_glyph;

    buffer.clear_output();

    buffer.idx = 0;
    let mut last_syllable = 0;
    while buffer.idx < buffer.len {
        let syllable = buffer.cur(0).syllable();
        if last_syllable != syllable && (syllable & 0x0F) == broken_syllable_type {
            last_syllable = syllable;

            let mut ginfo = dottedcircle;
            ginfo.cluster = buffer.cur(0).cluster;
            ginfo.mask = buffer.cur(0).mask;
            ginfo.set_syllable(buffer.cur(0).syllable());

            // Insert dottedcircle after possible Repha.
            if let Some(repha_category) = repha_category {
                while buffer.idx < buffer.len
                    && last_syllable == buffer.cur(0).syllable()
                    && buffer.cur(0).complex_var_u8_category() == repha_category
                {
                    buffer.next_glyph();
                }
            }

            buffer.output_info(ginfo);
        } else {
            buffer.next_glyph();
        }
    }

    buffer.sync();
}
