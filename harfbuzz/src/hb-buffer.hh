/*
 * Copyright © 1998-2004  David Turner and Werner Lemberg
 * Copyright © 2004,2007,2009,2010  Red Hat, Inc.
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

#ifndef RB_BUFFER_HH
#define RB_BUFFER_HH

#include "hb.hh"

#ifndef RB_BUFFER_MAX_LEN_FACTOR
#define RB_BUFFER_MAX_LEN_FACTOR 32
#endif
#ifndef RB_BUFFER_MAX_LEN_MIN
#define RB_BUFFER_MAX_LEN_MIN 8192
#endif
#ifndef RB_BUFFER_MAX_LEN_DEFAULT
#define RB_BUFFER_MAX_LEN_DEFAULT 0x3FFFFFFF /* Shaping more than a billion chars? Let us know! */
#endif

#ifndef RB_BUFFER_MAX_OPS_FACTOR
#define RB_BUFFER_MAX_OPS_FACTOR 64
#endif
#ifndef RB_BUFFER_MAX_OPS_MIN
#define RB_BUFFER_MAX_OPS_MIN 1024
#endif
#ifndef RB_BUFFER_MAX_OPS_DEFAULT
#define RB_BUFFER_MAX_OPS_DEFAULT 0x1FFFFFFF /* Shaping more than a billion operations? Let us know! */
#endif

static_assert((sizeof(rb_glyph_info_t) == 20), "");
static_assert((sizeof(rb_glyph_info_t) == sizeof(rb_glyph_position_t)), "");

RB_MARK_AS_FLAG_T(rb_buffer_flags_t);

enum rb_buffer_scratch_flags_t {
    RB_BUFFER_SCRATCH_FLAG_DEFAULT = 0x00000000u,
    RB_BUFFER_SCRATCH_FLAG_HAS_NON_ASCII = 0x00000001u,
    RB_BUFFER_SCRATCH_FLAG_HAS_DEFAULT_IGNORABLES = 0x00000002u,
    RB_BUFFER_SCRATCH_FLAG_HAS_SPACE_FALLBACK = 0x00000004u,
    RB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT = 0x00000008u,
    RB_BUFFER_SCRATCH_FLAG_HAS_UNSAFE_TO_BREAK = 0x00000010u,
    RB_BUFFER_SCRATCH_FLAG_HAS_CGJ = 0x00000020u,

    /* Reserved for complex shapers' internal use. */
    RB_BUFFER_SCRATCH_FLAG_COMPLEX0 = 0x01000000u,
    RB_BUFFER_SCRATCH_FLAG_COMPLEX1 = 0x02000000u,
    RB_BUFFER_SCRATCH_FLAG_COMPLEX2 = 0x04000000u,
    RB_BUFFER_SCRATCH_FLAG_COMPLEX3 = 0x08000000u,
};
RB_MARK_AS_FLAG_T(rb_buffer_scratch_flags_t);

/* Loop over grapheme. */
#define foreach_grapheme(buffer, start, end)                                                                           \
    for (unsigned int _count = rb_buffer_get_length(buffer),                                                           \
                      start = 0,                                                                                       \
                      end = _count ? rb_buffer_next_grapheme(buffer, 0) : 0;                                           \
         start < _count;                                                                                               \
         start = end, end = rb_buffer_next_grapheme(buffer, start))

#endif /* RB_BUFFER_HH */
