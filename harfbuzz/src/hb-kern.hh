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

extern "C" {
RB_EXTERN void rb_kern_machine_kern(
    rb_font_t *font,
    rb_buffer_t *buffer,
    rb_mask_t kern_mask,
    rb_bool_t cross_stream,
    const void *machine,
    rb_position_t (*machine_get_kerning)(const void *machine, rb_codepoint_t left, rb_codepoint_t right)
);
}

namespace OT {

template <typename Driver> struct rb_kern_machine_t
{
    rb_kern_machine_t(const Driver &driver_, bool crossStream_ = false)
        : driver(driver_)
        , crossStream(crossStream_)
    {}

    static rb_position_t machine_get_kerning(const void *machine, rb_codepoint_t left, rb_codepoint_t right)
    {
        return ((const rb_kern_machine_t*)machine)->driver.get_kerning(left, right);
    }

    RB_NO_SANITIZE_SIGNED_INTEGER_OVERFLOW
    void kern(rb_font_t *font, rb_buffer_t *buffer, rb_mask_t kern_mask) const
    {
        rb_kern_machine_kern(
            font,
            buffer,
            kern_mask,
            crossStream,
            (const void*)this,
            machine_get_kerning
        );
    }

    const Driver &driver;
    bool crossStream;
};

} /* namespace OT */

#endif /* RB_KERN_HH */
