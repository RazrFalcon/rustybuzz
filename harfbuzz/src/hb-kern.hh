/*
 * Copyright © 2017  Google, Inc.
 *
 *  This is part of HarfBuzz, a text shaping library.
 *
 * Permission is hereby granted, without written agreement and without
 * license or royalty fees, to use, copy, modify, and distribute this
 * software and its documentation for any purpose, provided that the
 * above copyright notice and the following two paragraphs appear in
 * all copies of this software.
 *
 * IN NO EVENT SHALL THE COPYRIGHT HOLDER BE LIABLE TO ANY PARTY FOR
 * DIRECT, INDIRECT, SPECIAL, INCIDENTAL, OR CONSEQUENTIAL DAMAGES
 * ARISING OUT OF THE USE OF THIS SOFTWARE AND ITS DOCUMENTATION, EVEN
 * IF THE COPYRIGHT HOLDER HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH
 * DAMAGE.
 *
 * THE COPYRIGHT HOLDER SPECIFICALLY DISCLAIMS ANY WARRANTIES, INCLUDING,
 * BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
 * FITNESS FOR A PARTICULAR PURPOSE.  THE SOFTWARE PROVIDED HEREUNDER IS
 * ON AN "AS IS" BASIS, AND THE COPYRIGHT HOLDER HAS NO OBLIGATION TO
 * PROVIDE MAINTENANCE, SUPPORT, UPDATES, ENHANCEMENTS, OR MODIFICATIONS.
 *
 * Google Author(s): Behdad Esfahbod
 */

#ifndef RB_KERN_HH
#define RB_KERN_HH

#include "hb-open-type.hh"
#include "hb-aat-layout-common.hh"
#include "hb-ot-layout-gpos-table.hh"

namespace OT {

template <typename Driver> struct rb_kern_machine_t
{
    rb_kern_machine_t(const Driver &driver_, bool crossStream_ = false)
        : driver(driver_)
        , crossStream(crossStream_)
    {
    }

    RB_NO_SANITIZE_SIGNED_INTEGER_OVERFLOW
    void kern(rb_font_t *font, rb_buffer_t *buffer, rb_mask_t kern_mask) const
    {
        OT::rb_ot_apply_context_t c(1, font, buffer);
        c.set_lookup_mask(kern_mask);
        c.set_lookup_props(OT::LookupFlag::IgnoreMarks);
        auto &skippy_iter = c.iter_input;

        bool horizontal = RB_DIRECTION_IS_HORIZONTAL(rb_buffer_get_direction(buffer));
        unsigned int count = rb_buffer_get_length(buffer);
        rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
        rb_glyph_position_t *pos = rb_buffer_get_glyph_positions(buffer);
        for (unsigned int idx = 0; idx < count;) {
            if (!(info[idx].mask & kern_mask)) {
                idx++;
                continue;
            }

            skippy_iter.reset(idx, 1);
            if (!skippy_iter.next()) {
                idx++;
                continue;
            }

            unsigned int i = idx;
            unsigned int j = skippy_iter.idx;

            rb_position_t kern = driver.get_kerning(info[i].codepoint, info[j].codepoint);

            if (likely(!kern))
                goto skip;

            if (horizontal) {
                if (crossStream) {
                    pos[j].y_offset = kern;
                    rb_buffer_set_scratch_flags(
                        buffer, rb_buffer_get_scratch_flags(buffer) | RB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT);
                } else {
                    rb_position_t kern1 = kern >> 1;
                    rb_position_t kern2 = kern - kern1;
                    pos[i].x_advance += kern1;
                    pos[j].x_advance += kern2;
                    pos[j].x_offset += kern2;
                }
            } else {
                if (crossStream) {
                    pos[j].x_offset = kern;
                    rb_buffer_set_scratch_flags(
                        buffer, rb_buffer_get_scratch_flags(buffer) | RB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT);
                } else {
                    rb_position_t kern1 = kern >> 1;
                    rb_position_t kern2 = kern - kern1;
                    pos[i].y_advance += kern1;
                    pos[j].y_advance += kern2;
                    pos[j].y_offset += kern2;
                }
            }

            rb_buffer_unsafe_to_break(buffer, i, j + 1);

        skip:
            idx = skippy_iter.idx;
        }
    }

    const Driver &driver;
    bool crossStream;
};

} /* namespace OT */

#endif /* RB_KERN_HH */
