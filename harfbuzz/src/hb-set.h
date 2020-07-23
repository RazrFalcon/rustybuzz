/*
 * Copyright © 2012  Google, Inc.
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

#ifndef RB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef RB_SET_H
#define RB_SET_H

#include "hb-common.h"

RB_BEGIN_DECLS

/*
 * Since: 0.9.21
 */
#define RB_SET_VALUE_INVALID ((rb_codepoint_t)-1)

typedef struct rb_set_t rb_set_t;

RB_EXTERN rb_set_t *rb_set_create(void);

RB_EXTERN rb_set_t *rb_set_get_empty(void);

RB_EXTERN rb_set_t *rb_set_reference(rb_set_t *set);

RB_EXTERN void rb_set_destroy(rb_set_t *set);

/* Returns false if allocation has failed before */
RB_EXTERN rb_bool_t rb_set_allocation_successful(const rb_set_t *set);

RB_EXTERN void rb_set_clear(rb_set_t *set);

RB_EXTERN rb_bool_t rb_set_is_empty(const rb_set_t *set);

RB_EXTERN rb_bool_t rb_set_has(const rb_set_t *set, rb_codepoint_t codepoint);

RB_EXTERN void rb_set_add(rb_set_t *set, rb_codepoint_t codepoint);

RB_EXTERN void rb_set_add_range(rb_set_t *set, rb_codepoint_t first, rb_codepoint_t last);

RB_EXTERN void rb_set_del(rb_set_t *set, rb_codepoint_t codepoint);

RB_EXTERN void rb_set_del_range(rb_set_t *set, rb_codepoint_t first, rb_codepoint_t last);

RB_EXTERN rb_bool_t rb_set_is_equal(const rb_set_t *set, const rb_set_t *other);

RB_EXTERN rb_bool_t rb_set_is_subset(const rb_set_t *set, const rb_set_t *larger_set);

RB_EXTERN void rb_set_set(rb_set_t *set, const rb_set_t *other);

RB_EXTERN void rb_set_union(rb_set_t *set, const rb_set_t *other);

RB_EXTERN void rb_set_intersect(rb_set_t *set, const rb_set_t *other);

RB_EXTERN void rb_set_subtract(rb_set_t *set, const rb_set_t *other);

RB_EXTERN void rb_set_symmetric_difference(rb_set_t *set, const rb_set_t *other);

RB_EXTERN unsigned int rb_set_get_population(const rb_set_t *set);

/* Returns RB_SET_VALUE_INVALID if set empty. */
RB_EXTERN rb_codepoint_t rb_set_get_min(const rb_set_t *set);

/* Returns RB_SET_VALUE_INVALID if set empty. */
RB_EXTERN rb_codepoint_t rb_set_get_max(const rb_set_t *set);

/* Pass RB_SET_VALUE_INVALID in to get started. */
RB_EXTERN rb_bool_t rb_set_next(const rb_set_t *set, rb_codepoint_t *codepoint);

/* Pass RB_SET_VALUE_INVALID in to get started. */
RB_EXTERN rb_bool_t rb_set_previous(const rb_set_t *set, rb_codepoint_t *codepoint);

/* Pass RB_SET_VALUE_INVALID for first and last to get started. */
RB_EXTERN rb_bool_t rb_set_next_range(const rb_set_t *set, rb_codepoint_t *first, rb_codepoint_t *last);

/* Pass RB_SET_VALUE_INVALID for first and last to get started. */
RB_EXTERN rb_bool_t rb_set_previous_range(const rb_set_t *set, rb_codepoint_t *first, rb_codepoint_t *last);

RB_END_DECLS

#endif /* RB_SET_H */
