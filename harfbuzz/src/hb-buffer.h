/*
 * Copyright © 1998-2004  David Turner and Werner Lemberg
 * Copyright © 2004,2007,2009  Red Hat, Inc.
 * Copyright © 2011,2012  Google, Inc.
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
 * Red Hat Author(s): Owen Taylor, Behdad Esfahbod
 * Google Author(s): Behdad Esfahbod
 */

#ifndef HB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef HB_BUFFER_H
#define HB_BUFFER_H

#include "hb-common.h"
#include "hb-unicode.h"
#include "hb-font.h"

HB_BEGIN_DECLS

typedef struct hb_glyph_info_t
{
    hb_codepoint_t codepoint;
    /*< private >*/
    hb_mask_t mask;
    /*< public >*/
    uint32_t cluster;

    /*< private >*/
    hb_var_int_t var1;
    hb_var_int_t var2;
} hb_glyph_info_t;

typedef enum {
    HB_GLYPH_FLAG_UNSAFE_TO_BREAK = 0x00000001,

    HB_GLYPH_FLAG_DEFINED = 0x00000001 /* OR of all defined flags */
} hb_glyph_flags_t;

typedef struct hb_glyph_position_t
{
    hb_position_t x_advance;
    hb_position_t y_advance;
    hb_position_t x_offset;
    hb_position_t y_offset;

    /*< private >*/
    hb_var_int_t var;
} hb_glyph_position_t;

typedef struct hb_segment_properties_t
{
    hb_direction_t direction;
    hb_script_t script;
    const char *language;
} hb_segment_properties_t;

typedef struct hb_buffer_t hb_buffer_t;

HB_EXTERN void hb_buffer_set_direction(hb_buffer_t *buffer, hb_direction_t direction);

HB_EXTERN hb_direction_t hb_buffer_get_direction(hb_buffer_t *buffer);

HB_EXTERN hb_script_t hb_buffer_get_script(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_get_segment_properties(hb_buffer_t *buffer, hb_segment_properties_t *props);

typedef enum { /*< flags >*/
               HB_BUFFER_FLAG_DEFAULT = 0x00000000u,
               HB_BUFFER_FLAG_BOT = 0x00000001u, /* Beginning-of-text */
               HB_BUFFER_FLAG_EOT = 0x00000002u, /* End-of-text */
               HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES = 0x00000004u,
               HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES = 0x00000008u,
               HB_BUFFER_FLAG_DO_NOT_INSERT_DOTTED_CIRCLE = 0x00000010u
} hb_buffer_flags_t;

HB_EXTERN hb_buffer_flags_t hb_buffer_get_flags(const hb_buffer_t *buffer);

typedef enum {
    HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES = 0,
    HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS = 1,
    HB_BUFFER_CLUSTER_LEVEL_CHARACTERS = 2,
    HB_BUFFER_CLUSTER_LEVEL_DEFAULT = HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES
} hb_buffer_cluster_level_t;

HB_EXTERN hb_buffer_cluster_level_t hb_buffer_get_cluster_level(hb_buffer_t *buffer);

HB_EXTERN hb_codepoint_t hb_buffer_get_invisible_glyph(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_clear_output(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_reverse(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_reverse_range(hb_buffer_t *buffer, unsigned int start, unsigned int end);

HB_EXTERN void hb_buffer_set_length(hb_buffer_t *buffer, unsigned int length);

HB_EXTERN unsigned int hb_buffer_get_length(const hb_buffer_t *buffer);

HB_EXTERN unsigned int hb_buffer_get_context_len(hb_buffer_t *buffer, unsigned int index);

HB_EXTERN unsigned int hb_buffer_get_index(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_set_index(hb_buffer_t *buffer, unsigned int index);

HB_EXTERN hb_glyph_info_t *hb_buffer_get_glyph_infos(hb_buffer_t *buffer);

HB_EXTERN hb_glyph_info_t *hb_buffer_get_out_infos(hb_buffer_t *buffer);

HB_EXTERN hb_glyph_position_t *hb_buffer_get_glyph_positions(hb_buffer_t *buffer);

HB_EXTERN unsigned int hb_buffer_get_scratch_flags(const hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_set_scratch_flags(hb_buffer_t *buffer, unsigned int flags);

HB_EXTERN void hb_buffer_skip_glyph(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_next_glyph(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_next_glyphs(hb_buffer_t *buffer, unsigned int len);

HB_EXTERN void hb_buffer_copy_glyph(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_delete_glyph(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_replace_glyph(hb_buffer_t *buffer, const hb_codepoint_t glyph_index);

HB_EXTERN void hb_buffer_replace_glyphs(hb_buffer_t *buffer,
                                        unsigned int num_in,
                                        unsigned int num_out,
                                        const hb_codepoint_t *glyph_data);

HB_EXTERN void hb_buffer_output_glyph(hb_buffer_t *buffer, hb_codepoint_t glyph_index);

HB_EXTERN void hb_buffer_output_info(hb_buffer_t *buffer, hb_glyph_info_t ginfo);

HB_EXTERN void hb_buffer_unsafe_to_break(hb_buffer_t *buffer, unsigned int start, unsigned int end);

HB_EXTERN void hb_buffer_unsafe_to_break_from_outbuffer(hb_buffer_t *buffer, unsigned int start, unsigned int end);

HB_EXTERN void hb_buffer_unsafe_to_break_all(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_merge_clusters(hb_buffer_t *buffer, unsigned int start, unsigned int end);

HB_EXTERN void hb_buffer_merge_out_clusters(hb_buffer_t *buffer, unsigned int start, unsigned int end);

HB_EXTERN void hb_buffer_swap_buffers(hb_buffer_t *buffer);

HB_EXTERN unsigned int hb_buffer_next_serial(hb_buffer_t *buffer);

HB_EXTERN unsigned int hb_buffer_get_backtrack_len(hb_buffer_t *buffer);

HB_EXTERN unsigned int hb_buffer_get_lookahead_len(hb_buffer_t *buffer);

HB_EXTERN hb_glyph_info_t *hb_buffer_get_cur(hb_buffer_t *buffer, unsigned int offset);

HB_EXTERN hb_glyph_info_t *hb_buffer_get_prev(hb_buffer_t *buffer);

HB_EXTERN hb_glyph_position_t *hb_buffer_get_cur_pos(hb_buffer_t *buffer);

HB_EXTERN int hb_buffer_decrement_max_ops(hb_buffer_t *buffer);

HB_EXTERN int hb_buffer_get_max_ops(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_set_max_ops(hb_buffer_t *buffer, int len);

HB_EXTERN void hb_buffer_set_max_len(hb_buffer_t *buffer, unsigned int len);

HB_EXTERN unsigned int hb_buffer_get_out_len(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_set_out_len(hb_buffer_t *buffer, unsigned int len);

HB_EXTERN bool hb_buffer_move_to(hb_buffer_t *buffer, unsigned int pos);

HB_EXTERN void hb_buffer_sort(hb_buffer_t *buffer,
                              unsigned int start,
                              unsigned int end,
                              int (*compar)(const hb_glyph_info_t *, const hb_glyph_info_t *));

HB_EXTERN void
hb_buffer_set_masks(hb_buffer_t *buffer, hb_mask_t value, hb_mask_t mask, unsigned int start, unsigned int end);

HB_EXTERN void hb_buffer_reset_masks(hb_buffer_t *buffer, hb_mask_t mask);

HB_EXTERN void hb_buffer_clear_positions(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_set_cluster(hb_buffer_t *buffer, hb_glyph_info_t *info, unsigned int cluster, hb_mask_t mask);

HB_EXTERN bool hb_buffer_has_separate_output(hb_buffer_t *buffer);

HB_EXTERN void hb_buffer_remove_output(hb_buffer_t *buffer);

HB_EXTERN bool hb_buffer_is_allocation_successful(const hb_buffer_t *buffer);

HB_END_DECLS

#endif /* HB_BUFFER_H */
