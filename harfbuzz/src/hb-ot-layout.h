/*
 * Copyright © 2007,2008,2009  Red Hat, Inc.
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
 */

#ifndef RB_OT_H_IN
#error "Include <hb-ot.h> instead."
#endif

#ifndef RB_OT_LAYOUT_H
#define RB_OT_LAYOUT_H

#include "hb.h"

RB_BEGIN_DECLS

#define RB_OT_TAG_GSUB RB_TAG('G', 'S', 'U', 'B')
#define RB_OT_TAG_GPOS RB_TAG('G', 'P', 'O', 'S')

/*
 * Script & Language tags.
 */

#define RB_OT_TAG_DEFAULT_SCRIPT RB_TAG('D', 'F', 'L', 'T')

/**
 * RB_OT_MAX_TAGS_PER_SCRIPT:
 *
 * Since: 2.0.0
 **/
#define RB_OT_MAX_TAGS_PER_SCRIPT 3u
/**
 * RB_OT_MAX_TAGS_PER_LANGUAGE:
 *
 * Since: 2.0.0
 **/
#define RB_OT_MAX_TAGS_PER_LANGUAGE 3u

RB_EXTERN void rb_ot_tags_from_script_and_language(rb_script_t script,
                                                   const char *language,
                                                   unsigned int *script_count /* IN/OUT */,
                                                   rb_tag_t *script_tags /* OUT */,
                                                   unsigned int *language_count /* IN/OUT */,
                                                   rb_tag_t *language_tags /* OUT */);

/*
 * GDEF
 */

RB_EXTERN rb_bool_t rb_ot_layout_has_glyph_classes(rb_face_t *face);

/*
 * GSUB/GPOS feature query and enumeration interface
 */

#define RB_OT_LAYOUT_NO_FEATURE_INDEX 0xFFFFu

RB_EXTERN rb_bool_t rb_ot_layout_table_select_script(rb_face_t *face,
                                                     rb_tag_t table_tag,
                                                     unsigned int script_count,
                                                     const rb_tag_t *script_tags,
                                                     unsigned int *script_index /* OUT */,
                                                     rb_tag_t *chosen_script /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_table_find_feature(rb_face_t *face,
                                                    rb_tag_t table_tag,
                                                    rb_tag_t feature_tag,
                                                    unsigned int *feature_index /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_script_select_language(rb_face_t *face,
                                                        rb_tag_t table_tag,
                                                        unsigned int script_index,
                                                        unsigned int language_count,
                                                        const rb_tag_t *language_tags,
                                                        unsigned int *language_index /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_language_get_required_feature(rb_face_t *face,
                                                               rb_tag_t table_tag,
                                                               unsigned int script_index,
                                                               unsigned int language_index,
                                                               unsigned int *feature_index /* OUT */,
                                                               rb_tag_t *feature_tag /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_language_find_feature(rb_face_t *face,
                                                       rb_tag_t table_tag,
                                                       unsigned int script_index,
                                                       unsigned int language_index,
                                                       rb_tag_t feature_tag,
                                                       unsigned int *feature_index /* OUT */);

RB_EXTERN unsigned int rb_ot_layout_table_get_lookup_count(rb_face_t *face, rb_tag_t table_tag);

/* Variations support */

RB_EXTERN rb_bool_t rb_ot_layout_table_find_feature_variations(rb_face_t *face,
                                                               rb_tag_t table_tag,
                                                               const int *coords,
                                                               unsigned int num_coords,
                                                               unsigned int *variations_index /* out */);

RB_EXTERN void rb_ot_layout_feature_with_variations_get_lookups(rb_face_t *face,
                                                                rb_tag_t table_tag,
                                                                unsigned int feature_index,
                                                                unsigned int variations_index,
                                                                unsigned int start_offset,
                                                                unsigned int *lookup_count /* IN/OUT */,
                                                                unsigned int *lookup_indexes /* OUT */);

/*
 * GSUB
 */

RB_EXTERN rb_bool_t rb_ot_layout_has_substitution(rb_face_t *face);

RB_EXTERN rb_bool_t rb_ot_layout_lookup_would_substitute(rb_face_t *face,
                                                         unsigned int lookup_index,
                                                         const rb_codepoint_t *glyphs,
                                                         unsigned int glyphs_length,
                                                         rb_bool_t zero_context);

RB_EXTERN void rb_ot_layout_substitute_start(rb_font_t *font, rb_buffer_t *buffer);

/*
 * GPOS
 */

RB_EXTERN rb_bool_t rb_ot_layout_has_positioning(rb_face_t *face);

RB_EXTERN void rb_ot_layout_position_start(rb_font_t *font, rb_buffer_t *buffer);

RB_EXTERN void rb_ot_layout_position_finish_advances(rb_font_t *font, rb_buffer_t *buffer);

RB_EXTERN void rb_ot_layout_position_finish_offsets(rb_font_t *font, rb_buffer_t *buffer);

RB_END_DECLS

#endif /* RB_OT_LAYOUT_H */
