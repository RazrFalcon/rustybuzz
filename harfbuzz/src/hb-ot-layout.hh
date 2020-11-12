/*
 * Copyright © 2007,2008,2009  Red Hat, Inc.
 * Copyright © 2012,2013  Google, Inc.
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
 * Red Hat Author(s): Behdad Esfahbod
 * Google Author(s): Behdad Esfahbod
 */

#ifndef RB_OT_LAYOUT_HH
#define RB_OT_LAYOUT_HH

#include "hb.hh"

#include "hb-buffer.hh"
#include "hb-open-type.hh"
#include "hb-ot-shape.hh"

#ifndef RB_MAX_CONTEXT_LENGTH
#define RB_MAX_CONTEXT_LENGTH 64
#endif

/*
 * kern
 */

RB_INTERNAL bool rb_ot_layout_has_kerning(rb_face_t *face);

RB_INTERNAL bool rb_ot_layout_has_machine_kerning(rb_face_t *face);

RB_INTERNAL bool rb_ot_layout_has_cross_kerning(rb_face_t *face);

RB_INTERNAL void rb_ot_layout_kern(const rb_ot_shape_plan_t *plan, rb_face_t *face, rb_buffer_t *buffer);

enum attach_type_t {
    ATTACH_TYPE_NONE = 0X00,

    /* Each attachment should be either a mark or a cursive; can't be both. */
    ATTACH_TYPE_MARK = 0X01,
    ATTACH_TYPE_CURSIVE = 0X02,
};

/*
 * Buffer var routines.
 */

/* buffer var allocations, used during the entire shaping process */
#define unicode_props() var2.u16[0]

/* buffer var allocations, used during the GSUB/GPOS processing */
#define attach_chain() var.i16[0] /* Glyph to which this attaches to, relative to current glyphs; \
                                     negative for going back, positive for forward. */
#define attach_type() var.u8[2]   /* Attachment type. Note! if attach_chain() is zero, the \
                                     value of attach_type() is irrelevant. */

/* unicode_props */

/* Design:
 * unicode_props() is a two-byte number.  The low byte includes:
 * - General_Category: 5 bits.
 * - A bit each for:
 *   * Is it Default_Ignorable(); we have a modified Default_Ignorable().
 *   * Whether it's one of the three Mongolian Free Variation Selectors,
 *     CGJ, or other characters that are hidden but should not be ignored
 *     like most other Default_Ignorable()s do during matching.
 *   * Whether it's a grapheme continuation.
 *
 * The high-byte has different meanings, switched by the Gen-Cat:
 * - For Mn,Mc,Me: the modified Combining_Class.
 * - For Cf: whether it's ZWJ, ZWNJ, or something else.
 * - For Ws: index of which space character this is, if space fallback
 *   is needed, ie. we don't set this by default, only if asked to.
 */

enum rb_unicode_props_flags_t {
    UPROPS_MASK_GEN_CAT = 0x001Fu,
    UPROPS_MASK_IGNORABLE = 0x0020u,
    UPROPS_MASK_HIDDEN = 0x0040u, /* MONGOLIAN FREE VARIATION SELECTOR 1..3, or TAG characters */
    UPROPS_MASK_CONTINUATION = 0x0080u,

    /* If GEN_CAT=FORMAT, top byte masks: */
    UPROPS_MASK_Cf_ZWJ = 0x0100u,
    UPROPS_MASK_Cf_ZWNJ = 0x0200u
};
RB_MARK_AS_FLAG_T(rb_unicode_props_flags_t);

static inline rb_unicode_general_category_t rb_glyph_info_get_general_category(const rb_glyph_info_t *info)
{
    return (rb_unicode_general_category_t)(info->unicode_props() & UPROPS_MASK_GEN_CAT);
}

#endif /* RB_OT_LAYOUT_HH */
