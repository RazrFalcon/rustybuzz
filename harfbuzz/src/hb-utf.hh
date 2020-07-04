/*
 * Copyright © 2011,2012,2014  Google, Inc.
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

#ifndef HB_UTF_HH
#define HB_UTF_HH

#include "hb.hh"

#include "hb-open-type.hh"

struct hb_utf8_t
{
    typedef uint8_t codepoint_t;

    static const codepoint_t *
    next(const codepoint_t *text, const codepoint_t *end, hb_codepoint_t *unicode, hb_codepoint_t replacement)
    {
        /* Written to only accept well-formed sequences.
         * Based on ideas from ICU's U8_NEXT.
         * Generates one "replacement" for each ill-formed byte. */

        hb_codepoint_t c = *text++;

        if (c > 0x7Fu) {
            if (hb_in_range<hb_codepoint_t>(c, 0xC2u, 0xDFu)) /* Two-byte */
            {
                unsigned int t1;
                if (likely(text < end && (t1 = text[0] - 0x80u) <= 0x3Fu)) {
                    c = ((c & 0x1Fu) << 6) | t1;
                    text++;
                } else
                    goto error;
            } else if (hb_in_range<hb_codepoint_t>(c, 0xE0u, 0xEFu)) /* Three-byte */
            {
                unsigned int t1, t2;
                if (likely(1 < end - text && (t1 = text[0] - 0x80u) <= 0x3Fu && (t2 = text[1] - 0x80u) <= 0x3Fu)) {
                    c = ((c & 0xFu) << 12) | (t1 << 6) | t2;
                    if (unlikely(c < 0x0800u || hb_in_range<hb_codepoint_t>(c, 0xD800u, 0xDFFFu)))
                        goto error;
                    text += 2;
                } else
                    goto error;
            } else if (hb_in_range<hb_codepoint_t>(c, 0xF0u, 0xF4u)) /* Four-byte */
            {
                unsigned int t1, t2, t3;
                if (likely(2 < end - text && (t1 = text[0] - 0x80u) <= 0x3Fu && (t2 = text[1] - 0x80u) <= 0x3Fu &&
                           (t3 = text[2] - 0x80u) <= 0x3Fu)) {
                    c = ((c & 0x7u) << 18) | (t1 << 12) | (t2 << 6) | t3;
                    if (unlikely(!hb_in_range<hb_codepoint_t>(c, 0x10000u, 0x10FFFFu)))
                        goto error;
                    text += 3;
                } else
                    goto error;
            } else
                goto error;
        }

        *unicode = c;
        return text;

    error:
        *unicode = replacement;
        return text;
    }

    static const codepoint_t *
    prev(const codepoint_t *text, const codepoint_t *start, hb_codepoint_t *unicode, hb_codepoint_t replacement)
    {
        const codepoint_t *end = text--;
        while (start < text && (*text & 0xc0) == 0x80 && end - text < 4)
            text--;

        if (likely(next(text, end, unicode, replacement) == end))
            return text;

        *unicode = replacement;
        return end - 1;
    }

    static unsigned int strlen(const codepoint_t *text)
    {
        return ::strlen((const char *)text);
    }

    static unsigned int encode_len(hb_codepoint_t unicode)
    {
        if (unicode < 0x0080u)
            return 1;
        if (unicode < 0x0800u)
            return 2;
        if (unicode < 0x10000u)
            return 3;
        if (unicode < 0x110000u)
            return 4;
        return 3;
    }

    static codepoint_t *encode(codepoint_t *text, const codepoint_t *end, hb_codepoint_t unicode)
    {
        if (unlikely(unicode >= 0xD800u && (unicode <= 0xDFFFu || unicode > 0x10FFFFu)))
            unicode = 0xFFFDu;
        if (unicode < 0x0080u)
            *text++ = unicode;
        else if (unicode < 0x0800u) {
            if (end - text >= 2) {
                *text++ = 0xC0u + (0x1Fu & (unicode >> 6));
                *text++ = 0x80u + (0x3Fu & (unicode));
            }
        } else if (unicode < 0x10000u) {
            if (end - text >= 3) {
                *text++ = 0xE0u + (0x0Fu & (unicode >> 12));
                *text++ = 0x80u + (0x3Fu & (unicode >> 6));
                *text++ = 0x80u + (0x3Fu & (unicode));
            }
        } else {
            if (end - text >= 4) {
                *text++ = 0xF0u + (0x07u & (unicode >> 18));
                *text++ = 0x80u + (0x3Fu & (unicode >> 12));
                *text++ = 0x80u + (0x3Fu & (unicode >> 6));
                *text++ = 0x80u + (0x3Fu & (unicode));
            }
        }
        return text;
    }
};

#endif /* HB_UTF_HH */
