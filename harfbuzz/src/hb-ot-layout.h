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

#include "hb-ot-name.h"

RB_BEGIN_DECLS

#define RB_OT_TAG_BASE RB_TAG('B', 'A', 'S', 'E')
#define RB_OT_TAG_GDEF RB_TAG('G', 'D', 'E', 'F')
#define RB_OT_TAG_GSUB RB_TAG('G', 'S', 'U', 'B')
#define RB_OT_TAG_GPOS RB_TAG('G', 'P', 'O', 'S')
#define RB_OT_TAG_JSTF RB_TAG('J', 'S', 'T', 'F')

/*
 * Script & Language tags.
 */

#define RB_OT_TAG_DEFAULT_SCRIPT RB_TAG('D', 'F', 'L', 'T')
#define RB_OT_TAG_DEFAULT_LANGUAGE RB_TAG('d', 'f', 'l', 't')

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

RB_EXTERN rb_script_t rb_ot_tag_to_script(rb_tag_t tag);

RB_EXTERN const char *rb_ot_tag_to_language(rb_tag_t tag);

RB_EXTERN void rb_ot_tags_to_script_and_language(rb_tag_t script_tag,
                                                 rb_tag_t language_tag,
                                                 rb_script_t *script /* OUT */,
                                                 const char **language /* OUT */);

/*
 * GDEF
 */

RB_EXTERN rb_bool_t rb_ot_layout_has_glyph_classes(rb_face_t *face);

/**
 * rb_ot_layout_glyph_class_t:
 * @RB_OT_LAYOUT_GLYPH_CLASS_UNCLASSIFIED: Glyphs not matching the other classifications
 * @RB_OT_LAYOUT_GLYPH_CLASS_BASE_GLYPH: Spacing, single characters, capable of accepting marks
 * @RB_OT_LAYOUT_GLYPH_CLASS_LIGATURE: Glyphs that represent ligation of multiple characters
 * @RB_OT_LAYOUT_GLYPH_CLASS_MARK: Non-spacing, combining glyphs that represent marks
 * @RB_OT_LAYOUT_GLYPH_CLASS_COMPONENT: Spacing glyphs that represent part of a single character
 *
 * The GDEF classes defined for glyphs.
 *
 **/
typedef enum {
    RB_OT_LAYOUT_GLYPH_CLASS_UNCLASSIFIED = 0,
    RB_OT_LAYOUT_GLYPH_CLASS_BASE_GLYPH = 1,
    RB_OT_LAYOUT_GLYPH_CLASS_LIGATURE = 2,
    RB_OT_LAYOUT_GLYPH_CLASS_MARK = 3,
    RB_OT_LAYOUT_GLYPH_CLASS_COMPONENT = 4
} rb_ot_layout_glyph_class_t;

RB_EXTERN rb_ot_layout_glyph_class_t rb_ot_layout_get_glyph_class(rb_face_t *face, rb_codepoint_t glyph);

RB_EXTERN void
rb_ot_layout_get_glyphs_in_class(rb_face_t *face, rb_ot_layout_glyph_class_t klass, rb_set_t *glyphs /* OUT */);

/* Not that useful.  Provides list of attach points for a glyph that a
 * client may want to cache */
RB_EXTERN unsigned int rb_ot_layout_get_attach_points(rb_face_t *face,
                                                      rb_codepoint_t glyph,
                                                      unsigned int start_offset,
                                                      unsigned int *point_count /* IN/OUT */,
                                                      unsigned int *point_array /* OUT */);

/*
 * GSUB/GPOS feature query and enumeration interface
 */

#define RB_OT_LAYOUT_NO_SCRIPT_INDEX 0xFFFFu
#define RB_OT_LAYOUT_NO_FEATURE_INDEX 0xFFFFu
#define RB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX 0xFFFFu
#define RB_OT_LAYOUT_NO_VARIATIONS_INDEX 0xFFFFFFFFu

RB_EXTERN unsigned int rb_ot_layout_table_get_script_tags(rb_face_t *face,
                                                          rb_tag_t table_tag,
                                                          unsigned int start_offset,
                                                          unsigned int *script_count /* IN/OUT */,
                                                          rb_tag_t *script_tags /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_table_find_script(rb_face_t *face,
                                                   rb_tag_t table_tag,
                                                   rb_tag_t script_tag,
                                                   unsigned int *script_index /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_table_select_script(rb_face_t *face,
                                                     rb_tag_t table_tag,
                                                     unsigned int script_count,
                                                     const rb_tag_t *script_tags,
                                                     unsigned int *script_index /* OUT */,
                                                     rb_tag_t *chosen_script /* OUT */);

RB_EXTERN unsigned int rb_ot_layout_table_get_feature_tags(rb_face_t *face,
                                                           rb_tag_t table_tag,
                                                           unsigned int start_offset,
                                                           unsigned int *feature_count /* IN/OUT */,
                                                           rb_tag_t *feature_tags /* OUT */);

RB_EXTERN unsigned int rb_ot_layout_script_get_language_tags(rb_face_t *face,
                                                             rb_tag_t table_tag,
                                                             unsigned int script_index,
                                                             unsigned int start_offset,
                                                             unsigned int *language_count /* IN/OUT */,
                                                             rb_tag_t *language_tags /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_script_select_language(rb_face_t *face,
                                                        rb_tag_t table_tag,
                                                        unsigned int script_index,
                                                        unsigned int language_count,
                                                        const rb_tag_t *language_tags,
                                                        unsigned int *language_index /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_language_get_required_feature_index(rb_face_t *face,
                                                                     rb_tag_t table_tag,
                                                                     unsigned int script_index,
                                                                     unsigned int language_index,
                                                                     unsigned int *feature_index /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_language_get_required_feature(rb_face_t *face,
                                                               rb_tag_t table_tag,
                                                               unsigned int script_index,
                                                               unsigned int language_index,
                                                               unsigned int *feature_index /* OUT */,
                                                               rb_tag_t *feature_tag /* OUT */);

RB_EXTERN unsigned int rb_ot_layout_language_get_feature_indexes(rb_face_t *face,
                                                                 rb_tag_t table_tag,
                                                                 unsigned int script_index,
                                                                 unsigned int language_index,
                                                                 unsigned int start_offset,
                                                                 unsigned int *feature_count /* IN/OUT */,
                                                                 unsigned int *feature_indexes /* OUT */);

RB_EXTERN unsigned int rb_ot_layout_language_get_feature_tags(rb_face_t *face,
                                                              rb_tag_t table_tag,
                                                              unsigned int script_index,
                                                              unsigned int language_index,
                                                              unsigned int start_offset,
                                                              unsigned int *feature_count /* IN/OUT */,
                                                              rb_tag_t *feature_tags /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_language_find_feature(rb_face_t *face,
                                                       rb_tag_t table_tag,
                                                       unsigned int script_index,
                                                       unsigned int language_index,
                                                       rb_tag_t feature_tag,
                                                       unsigned int *feature_index /* OUT */);

RB_EXTERN unsigned int rb_ot_layout_feature_get_lookups(rb_face_t *face,
                                                        rb_tag_t table_tag,
                                                        unsigned int feature_index,
                                                        unsigned int start_offset,
                                                        unsigned int *lookup_count /* IN/OUT */,
                                                        unsigned int *lookup_indexes /* OUT */);

RB_EXTERN unsigned int rb_ot_layout_table_get_lookup_count(rb_face_t *face, rb_tag_t table_tag);

RB_EXTERN void rb_ot_layout_collect_features(rb_face_t *face,
                                             rb_tag_t table_tag,
                                             const rb_tag_t *scripts,
                                             const rb_tag_t *languages,
                                             const rb_tag_t *features,
                                             rb_set_t *feature_indexes /* OUT */);

RB_EXTERN void rb_ot_layout_collect_lookups(rb_face_t *face,
                                            rb_tag_t table_tag,
                                            const rb_tag_t *scripts,
                                            const rb_tag_t *languages,
                                            const rb_tag_t *features,
                                            rb_set_t *lookup_indexes /* OUT */);

RB_EXTERN void rb_ot_layout_lookup_collect_glyphs(rb_face_t *face,
                                                  rb_tag_t table_tag,
                                                  unsigned int lookup_index,
                                                  rb_set_t *glyphs_before, /* OUT.  May be NULL */
                                                  rb_set_t *glyphs_input,  /* OUT.  May be NULL */
                                                  rb_set_t *glyphs_after,  /* OUT.  May be NULL */
                                                  rb_set_t *glyphs_output /* OUT.  May be NULL */);

#ifdef RB_NOT_IMPLEMENTED
typedef struct
{
    const rb_codepoint_t *before, unsigned int before_length, const rb_codepoint_t *input, unsigned int input_length,
        const rb_codepoint_t *after, unsigned int after_length,
} rb_ot_layout_glyph_sequence_t;

typedef rb_bool_t (*rb_ot_layout_glyph_sequence_func_t)(rb_font_t *font,
                                                        rb_tag_t table_tag,
                                                        unsigned int lookup_index,
                                                        const rb_ot_layout_glyph_sequence_t *sequence,
                                                        void *user_data);

RB_EXTERN void Xrb_ot_layout_lookup_enumerate_sequences(rb_face_t *face,
                                                        rb_tag_t table_tag,
                                                        unsigned int lookup_index,
                                                        rb_ot_layout_glyph_sequence_func_t callback,
                                                        void *user_data);
#endif

/* Variations support */

RB_EXTERN rb_bool_t rb_ot_layout_table_find_feature_variations(rb_face_t *face,
                                                               rb_tag_t table_tag,
                                                               const int *coords,
                                                               unsigned int num_coords,
                                                               unsigned int *variations_index /* out */);

RB_EXTERN unsigned int rb_ot_layout_feature_with_variations_get_lookups(rb_face_t *face,
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

RB_EXTERN unsigned rb_ot_layout_lookup_get_glyph_alternates(rb_face_t *face,
                                                            unsigned lookup_index,
                                                            rb_codepoint_t glyph,
                                                            unsigned start_offset,
                                                            unsigned *alternate_count /* IN/OUT */,
                                                            rb_codepoint_t *alternate_glyphs /* OUT */);

RB_EXTERN rb_bool_t rb_ot_layout_lookup_would_substitute(rb_face_t *face,
                                                         unsigned int lookup_index,
                                                         const rb_codepoint_t *glyphs,
                                                         unsigned int glyphs_length,
                                                         rb_bool_t zero_context);

RB_EXTERN void rb_ot_layout_lookup_substitute_closure(rb_face_t *face, unsigned int lookup_index, rb_set_t *glyphs
                                                      /*TODO , rb_bool_t  inclusive */);

RB_EXTERN void rb_ot_layout_lookups_substitute_closure(rb_face_t *face, const rb_set_t *lookups, rb_set_t *glyphs);

#ifdef RB_NOT_IMPLEMENTED
/* Note: You better have GDEF when using this API, or marks won't do much. */
RB_EXTERN rb_bool_t Xrb_ot_layout_lookup_substitute(rb_font_t *font,
                                                    unsigned int lookup_index,
                                                    const rb_ot_layout_glyph_sequence_t *sequence,
                                                    unsigned int out_size,
                                                    rb_codepoint_t *glyphs_out, /* OUT */
                                                    unsigned int *clusters_out, /* OUT */
                                                    unsigned int *out_length /* OUT */);
#endif

/*
 * GPOS
 */

RB_EXTERN rb_bool_t rb_ot_layout_has_positioning(rb_face_t *face);

#ifdef RB_NOT_IMPLEMENTED
/* Note: You better have GDEF when using this API, or marks won't do much. */
RB_EXTERN rb_bool_t Xrb_ot_layout_lookup_position(rb_font_t *font,
                                                  unsigned int lookup_index,
                                                  const rb_ot_layout_glyph_sequence_t *sequence,
                                                  rb_glyph_position_t *positions /* IN / OUT */);
#endif

/* Optical 'size' feature info.  Returns true if found.
 * https://docs.microsoft.com/en-us/typography/opentype/spec/features_pt#size */
RB_EXTERN rb_bool_t rb_ot_layout_get_size_params(rb_face_t *face,
                                                 unsigned int *design_size,          /* OUT.  May be NULL */
                                                 unsigned int *subfamily_id,         /* OUT.  May be NULL */
                                                 rb_ot_name_id_t *subfamily_name_id, /* OUT.  May be NULL */
                                                 unsigned int *range_start,          /* OUT.  May be NULL */
                                                 unsigned int *range_end /* OUT.  May be NULL */);

RB_EXTERN rb_bool_t rb_ot_layout_feature_get_name_ids(rb_face_t *face,
                                                      rb_tag_t table_tag,
                                                      unsigned int feature_index,
                                                      rb_ot_name_id_t *label_id /* OUT.  May be NULL */,
                                                      rb_ot_name_id_t *tooltip_id /* OUT.  May be NULL */,
                                                      rb_ot_name_id_t *sample_id /* OUT.  May be NULL */,
                                                      unsigned int *num_named_parameters /* OUT.  May be NULL */,
                                                      rb_ot_name_id_t *first_param_id /* OUT.  May be NULL */);

RB_EXTERN unsigned int rb_ot_layout_feature_get_characters(rb_face_t *face,
                                                           rb_tag_t table_tag,
                                                           unsigned int feature_index,
                                                           unsigned int start_offset,
                                                           unsigned int *char_count /* IN/OUT.  May be NULL */,
                                                           rb_codepoint_t *characters /* OUT.     May be NULL */);

RB_END_DECLS

#endif /* RB_OT_LAYOUT_H */
