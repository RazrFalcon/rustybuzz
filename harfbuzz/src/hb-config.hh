/*
 * Copyright © 2019  Facebook, Inc.
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
 * Facebook Author(s): Behdad Esfahbod
 */

#ifndef HB_CONFIG_HH
#define HB_CONFIG_HH

#if 0 /* Make test happy. */
#include "hb.hh"
#endif

#include "config.h"

/* Closure of options. */

#ifdef HB_NO_AAT
#define HB_NO_OT_NAME_LANGUAGE_AAT
#define HB_NO_AAT_SHAPE
#endif

#ifdef HB_NO_BITMAP
#define HB_NO_OT_FONT_BITMAP
#endif

#ifdef HB_NO_GETENV
#define HB_NO_UNISCRIBE_BUG_COMPATIBLE
#endif

#ifdef HB_NO_LEGACY
#define HB_NO_CMAP_LEGACY_SUBTABLES
#define HB_NO_FALLBACK_SHAPE
#define HB_NO_OT_KERN
#define HB_NO_OT_LAYOUT_BLACKLIST
#define HB_NO_OT_SHAPE_FALLBACK
#endif

#ifdef HB_NO_OT_SHAPE_FALLBACK
#define HB_NO_OT_SHAPE_COMPLEX_ARABIC_FALLBACK
#define HB_NO_OT_SHAPE_COMPLEX_HEBREW_FALLBACK
#define HB_NO_OT_SHAPE_COMPLEX_THAI_FALLBACK
#define HB_NO_OT_SHAPE_COMPLEX_VOWEL_CONSTRAINTS
#endif

#ifdef NDEBUG
#ifndef HB_NDEBUG
#define HB_NDEBUG
#endif
#endif

#endif /* HB_CONFIG_HH */
