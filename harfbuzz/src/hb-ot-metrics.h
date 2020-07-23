/*
 * Copyright © 2018  Ebrahim Byagowi
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
 */

#ifndef RB_OT_H_IN
#error "Include <hb-ot.h> instead."
#endif

#ifndef RB_OT_METRICS_H
#define RB_OT_METRICS_H

#include "hb.h"
#include "hb-ot-name.h"

RB_BEGIN_DECLS

typedef enum {
    RB_OT_METRICS_TAG_HORIZONTAL_ASCENDER = RB_TAG('h', 'a', 's', 'c'),
    RB_OT_METRICS_TAG_HORIZONTAL_DESCENDER = RB_TAG('h', 'd', 's', 'c'),
    RB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP = RB_TAG('h', 'l', 'g', 'p'),
    RB_OT_METRICS_TAG_VERTICAL_ASCENDER = RB_TAG('v', 'a', 's', 'c'),
    RB_OT_METRICS_TAG_VERTICAL_DESCENDER = RB_TAG('v', 'd', 's', 'c'),
    RB_OT_METRICS_TAG_VERTICAL_LINE_GAP = RB_TAG('v', 'l', 'g', 'p'),
} rb_ot_metrics_tag_t;

RB_END_DECLS

#endif /* RB_OT_METRICS_H */
