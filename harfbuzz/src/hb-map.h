/*
 * Copyright © 2018  Google, Inc.
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

#ifndef RB_MAP_H
#define RB_MAP_H

#include "hb-common.h"

RB_BEGIN_DECLS

/*
 * Since: 1.7.7
 */
#define RB_MAP_VALUE_INVALID ((rb_codepoint_t)-1)

typedef struct rb_map_t rb_map_t;

RB_EXTERN rb_map_t *rb_map_create(void);

RB_EXTERN rb_map_t *rb_map_get_empty(void);

RB_EXTERN rb_map_t *rb_map_reference(rb_map_t *map);

RB_EXTERN void rb_map_destroy(rb_map_t *map);

/* Returns false if allocation has failed before */
RB_EXTERN rb_bool_t rb_map_allocation_successful(const rb_map_t *map);

RB_EXTERN void rb_map_clear(rb_map_t *map);

RB_EXTERN rb_bool_t rb_map_is_empty(const rb_map_t *map);

RB_EXTERN unsigned int rb_map_get_population(const rb_map_t *map);

RB_EXTERN void rb_map_set(rb_map_t *map, rb_codepoint_t key, rb_codepoint_t value);

RB_EXTERN rb_codepoint_t rb_map_get(const rb_map_t *map, rb_codepoint_t key);

RB_EXTERN void rb_map_del(rb_map_t *map, rb_codepoint_t key);

RB_EXTERN rb_bool_t rb_map_has(const rb_map_t *map, rb_codepoint_t key);

RB_END_DECLS

#endif /* RB_MAP_H */
