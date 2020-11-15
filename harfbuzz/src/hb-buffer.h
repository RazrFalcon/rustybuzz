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

#ifndef RB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef RB_BUFFER_H
#define RB_BUFFER_H

#include "hb-common.h"

RB_BEGIN_DECLS

typedef struct rb_glyph_info_t
{
    rb_codepoint_t codepoint;
    /*< private >*/
    rb_mask_t mask;
    /*< public >*/
    uint32_t cluster;

    /*< private >*/
    rb_var_int_t var1;
    rb_var_int_t var2;
} rb_glyph_info_t;

typedef struct rb_glyph_position_t
{
    rb_position_t x_advance;
    rb_position_t y_advance;
    rb_position_t x_offset;
    rb_position_t y_offset;

    /*< private >*/
    rb_var_int_t var;
} rb_glyph_position_t;

typedef struct rb_buffer_t rb_buffer_t;

RB_EXTERN rb_direction_t rb_buffer_get_direction(rb_buffer_t *buffer);

RB_EXTERN void rb_buffer_reverse(rb_buffer_t *buffer);

RB_EXTERN unsigned int rb_buffer_get_length(const rb_buffer_t *buffer);

RB_EXTERN rb_glyph_info_t *rb_buffer_get_cur(rb_buffer_t *buffer, unsigned int offset);

RB_EXTERN rb_glyph_position_t *rb_buffer_get_cur_pos(rb_buffer_t *buffer);

RB_EXTERN unsigned int rb_buffer_get_backtrack_len(rb_buffer_t *buffer);

RB_EXTERN bool rb_buffer_move_to(rb_buffer_t *buffer, unsigned int pos);

RB_EXTERN void rb_buffer_swap_buffers(rb_buffer_t *buffer);

RB_EXTERN void rb_buffer_clear_output(rb_buffer_t *buffer);

RB_EXTERN void rb_buffer_merge_clusters(rb_buffer_t *buffer, unsigned int start, unsigned int end);

RB_EXTERN void rb_buffer_merge_out_clusters(rb_buffer_t *buffer, unsigned int start, unsigned int end);

RB_EXTERN void rb_buffer_unsafe_to_break(rb_buffer_t *buffer, unsigned int start, unsigned int end);

RB_EXTERN void rb_buffer_unsafe_to_break_from_outbuffer(rb_buffer_t *buffer, unsigned int start, unsigned int end);

RB_EXTERN void rb_buffer_replace_glyph(rb_buffer_t *buffer, const rb_codepoint_t glyph_index);

RB_EXTERN void rb_buffer_output_glyph(rb_buffer_t *buffer, rb_codepoint_t glyph_index);

RB_EXTERN void rb_buffer_copy_glyph(rb_buffer_t *buffer);

RB_EXTERN void rb_buffer_next_glyph(rb_buffer_t *buffer);

RB_EXTERN void rb_buffer_skip_glyph(rb_buffer_t *buffer);

RB_EXTERN void rb_buffer_delete_glyph(rb_buffer_t *buffer);

RB_EXTERN void rb_buffer_delete_glyphs_inplace(rb_buffer_t *buffer, rb_bool_t(*filter)(const rb_glyph_info_t *info));

RB_EXTERN rb_glyph_info_t *rb_buffer_get_glyph_infos(rb_buffer_t *buffer);

RB_EXTERN int rb_buffer_decrement_max_ops(rb_buffer_t *buffer, int count);

RB_EXTERN unsigned int rb_buffer_get_index(rb_buffer_t *buffer);

RB_EXTERN void rb_buffer_set_index(rb_buffer_t *buffer, unsigned int index);

RB_EXTERN unsigned int rb_buffer_get_out_len(rb_buffer_t *buffer);

RB_EXTERN rb_glyph_position_t *rb_buffer_get_glyph_positions(rb_buffer_t *buffer);

RB_EXTERN unsigned int rb_buffer_get_scratch_flags(const rb_buffer_t *buffer);

RB_EXTERN void rb_buffer_set_scratch_flags(rb_buffer_t *buffer, unsigned int flags);

RB_EXTERN bool rb_buffer_is_allocation_successful(const rb_buffer_t *buffer);

RB_EXTERN unsigned int rb_buffer_next_grapheme(const rb_buffer_t *buffer, unsigned int start);

RB_END_DECLS

#endif /* RB_BUFFER_H */
