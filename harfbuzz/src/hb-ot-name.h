/*
 * Copyright © 2018  Ebrahim Byagowi.
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

#ifndef RB_OT_NAME_H
#define RB_OT_NAME_H

#include "hb.h"

RB_BEGIN_DECLS

/**
 * rb_ot_name_id_t:
 * @RB_OT_NAME_ID_INVALID: Value to represent a nonexistent name ID.
 *
 * An integral type representing an OpenType 'name' table name identifier.
 * There are predefined name IDs, as well as name IDs return from other
 * API.  These can be used to fetch name strings from a font face.
 *
 * Since: 2.0.0
 **/
enum {
    RB_OT_NAME_ID_COPYRIGHT = 0,
    RB_OT_NAME_ID_FONT_FAMILY = 1,
    RB_OT_NAME_ID_FONT_SUBFAMILY = 2,
    RB_OT_NAME_ID_UNIQUE_ID = 3,
    RB_OT_NAME_ID_FULL_NAME = 4,
    RB_OT_NAME_ID_VERSION_STRING = 5,
    RB_OT_NAME_ID_POSTSCRIPT_NAME = 6,
    RB_OT_NAME_ID_TRADEMARK = 7,
    RB_OT_NAME_ID_MANUFACTURER = 8,
    RB_OT_NAME_ID_DESIGNER = 9,
    RB_OT_NAME_ID_DESCRIPTION = 10,
    RB_OT_NAME_ID_VENDOR_URL = 11,
    RB_OT_NAME_ID_DESIGNER_URL = 12,
    RB_OT_NAME_ID_LICENSE = 13,
    RB_OT_NAME_ID_LICENSE_URL = 14,
    /*RB_OT_NAME_ID_RESERVED		= 15,*/
    RB_OT_NAME_ID_TYPOGRAPHIC_FAMILY = 16,
    RB_OT_NAME_ID_TYPOGRAPHIC_SUBFAMILY = 17,
    RB_OT_NAME_ID_MAC_FULL_NAME = 18,
    RB_OT_NAME_ID_SAMPLE_TEXT = 19,
    RB_OT_NAME_ID_CID_FINDFONT_NAME = 20,
    RB_OT_NAME_ID_WWS_FAMILY = 21,
    RB_OT_NAME_ID_WWS_SUBFAMILY = 22,
    RB_OT_NAME_ID_LIGHT_BACKGROUND = 23,
    RB_OT_NAME_ID_DARK_BACKGROUND = 24,
    RB_OT_NAME_ID_VARIATIONS_PS_PREFIX = 25,

    RB_OT_NAME_ID_INVALID = 0xFFFF
};

typedef unsigned int rb_ot_name_id_t;

RB_END_DECLS

#endif /* RB_OT_NAME_H */
