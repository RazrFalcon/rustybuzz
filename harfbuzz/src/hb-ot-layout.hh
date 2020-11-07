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
#include "hb-set-digest.hh"

struct rb_ot_shape_plan_t;

/*
 * kern
 */

RB_INTERNAL bool rb_ot_layout_has_kerning(rb_face_t *face);

RB_INTERNAL bool rb_ot_layout_has_machine_kerning(rb_face_t *face);

RB_INTERNAL bool rb_ot_layout_has_cross_kerning(rb_face_t *face);

RB_INTERNAL void rb_ot_layout_kern(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer);

/*
 * GDEF
 */

enum rb_ot_layout_glyph_props_flags_t {
    /* The following three match LookupFlags::Ignore* numbers. */
    RB_OT_LAYOUT_GLYPH_PROPS_BASE_GLYPH = 0x02u,
    RB_OT_LAYOUT_GLYPH_PROPS_LIGATURE = 0x04u,
    RB_OT_LAYOUT_GLYPH_PROPS_MARK = 0x08u,

    RB_OT_LAYOUT_GLYPH_PROPS_CLASS_MASK =
        RB_OT_LAYOUT_GLYPH_PROPS_BASE_GLYPH | RB_OT_LAYOUT_GLYPH_PROPS_LIGATURE | RB_OT_LAYOUT_GLYPH_PROPS_MARK,

    /* The following are used internally; not derived from GDEF. */
    RB_OT_LAYOUT_GLYPH_PROPS_SUBSTITUTED = 0x10u,
    RB_OT_LAYOUT_GLYPH_PROPS_LIGATED = 0x20u,
    RB_OT_LAYOUT_GLYPH_PROPS_MULTIPLIED = 0x40u,
};
RB_MARK_AS_FLAG_T(rb_ot_layout_glyph_props_flags_t);

/*
 * GSUB/GPOS
 */

namespace OT {
struct rb_ot_apply_context_t;
struct SubstLookup;
} // namespace OT

/*
 * Buffer var routines.
 */

/* buffer var allocations, used during the entire shaping process */
#define unicode_props() var2.u16[0]

/* buffer var allocations, used during the GSUB/GPOS processing */
#define glyph_props() var1.u16[0] /* GDEF glyph properties */
#define lig_props() var1.u8[2]    /* GSUB/GPOS ligature tracking */
#define syllable() var1.u8[3]     /* GSUB/GPOS shaping boundaries */

extern "C" {
RB_EXTERN void rb_layout_clear_syllables(const rb_ot_shape_plan_t *plan, rb_font_t *font, rb_buffer_t *buffer);
}

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

extern "C" {
RB_EXTERN void rb_glyph_info_init_unicode_props(rb_glyph_info_t *info, rb_buffer_t *buffer);
}

static inline rb_unicode_general_category_t _rb_glyph_info_get_general_category(const rb_glyph_info_t *info)
{
    return (rb_unicode_general_category_t)(info->unicode_props() & UPROPS_MASK_GEN_CAT);
}

static inline bool _rb_glyph_info_is_unicode_mark(const rb_glyph_info_t *info)
{
    return RB_UNICODE_GENERAL_CATEGORY_IS_MARK(info->unicode_props() & UPROPS_MASK_GEN_CAT);
}

static inline bool _rb_glyph_info_ligated(const rb_glyph_info_t *info)
{
    return !!(info->glyph_props() & RB_OT_LAYOUT_GLYPH_PROPS_LIGATED);
}

static inline rb_bool_t _rb_glyph_info_is_default_ignorable(const rb_glyph_info_t *info)
{
    return (info->unicode_props() & UPROPS_MASK_IGNORABLE) && !_rb_glyph_info_ligated(info);
}

static inline bool _rb_glyph_info_is_default_ignorable_and_not_hidden(const rb_glyph_info_t *info)
{
    return ((info->unicode_props() & (UPROPS_MASK_IGNORABLE | UPROPS_MASK_HIDDEN)) == UPROPS_MASK_IGNORABLE) &&
           !_rb_glyph_info_ligated(info);
}

static inline void _rb_glyph_info_set_continuation(rb_glyph_info_t *info)
{
    info->unicode_props() |= UPROPS_MASK_CONTINUATION;
}

static inline bool _rb_glyph_info_is_continuation(const rb_glyph_info_t *info)
{
    return info->unicode_props() & UPROPS_MASK_CONTINUATION;
}

/* Loop over grapheme. Based on foreach_cluster(). */
#define foreach_grapheme(buffer, start, end)                                                                           \
    for (unsigned int _count = rb_buffer_get_length(buffer),                                                           \
                      start = 0,                                                                                       \
                      end = _count ? rb_buffer_next_grapheme(buffer, 0) : 0;                                           \
         start < _count;                                                                                               \
         start = end, end = rb_buffer_next_grapheme(buffer, start))

static inline unsigned int rb_buffer_next_grapheme(rb_buffer_t *buffer, unsigned int start)
{
    rb_glyph_info_t *info = rb_buffer_get_glyph_infos(buffer);
    unsigned int count = rb_buffer_get_length(buffer);

    while (++start < count && _rb_glyph_info_is_continuation(&info[start]))
        ;

    return start;
}

static inline bool _rb_glyph_info_is_unicode_format(const rb_glyph_info_t *info)
{
    return _rb_glyph_info_get_general_category(info) == RB_UNICODE_GENERAL_CATEGORY_FORMAT;
}

static inline bool _rb_glyph_info_is_zwnj(const rb_glyph_info_t *info)
{
    return _rb_glyph_info_is_unicode_format(info) && (info->unicode_props() & UPROPS_MASK_Cf_ZWNJ);
}

static inline bool _rb_glyph_info_is_zwj(const rb_glyph_info_t *info)
{
    return _rb_glyph_info_is_unicode_format(info) && (info->unicode_props() & UPROPS_MASK_Cf_ZWJ);
}

/* glyph_props: */

static inline void _rb_glyph_info_set_glyph_props(rb_glyph_info_t *info, unsigned int props)
{
    info->glyph_props() = props;
}

static inline bool _rb_glyph_info_is_mark(const rb_glyph_info_t *info)
{
    return !!(info->glyph_props() & RB_OT_LAYOUT_GLYPH_PROPS_MARK);
}

static inline void _rb_glyph_info_clear_substituted(rb_glyph_info_t *info)
{
    info->glyph_props() &= ~(RB_OT_LAYOUT_GLYPH_PROPS_SUBSTITUTED);
}

/* Make sure no one directly touches our props... */
#undef lig_props
#undef glyph_props

#endif /* RB_OT_LAYOUT_HH */
