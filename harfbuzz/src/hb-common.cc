/*
 * Copyright © 2009,2010  Red Hat, Inc.
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
 * Red Hat Author(s): Behdad Esfahbod
 * Google Author(s): Behdad Esfahbod
 */

#include "hb.hh"
#include "hb-machinery.hh"

#include <locale.h>

#ifdef HB_NO_SETLOCALE
#define setlocale(Category, Locale) "C"
#endif

/**
 * SECTION:hb-common
 * @title: hb-common
 * @short_description: Common data types
 * @include: hb.h
 *
 * Common data types used across HarfBuzz are defined here.
 **/

/* hb_tag_t */

/**
 * hb_tag_from_string:
 * @str: (array length=len) (element-type uint8_t):
 * @len:
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_tag_t hb_tag_from_string(const char *str, int len)
{
    char tag[4];
    unsigned int i;

    if (!str || !len || !*str)
        return HB_TAG_NONE;

    if (len < 0 || len > 4)
        len = 4;
    for (i = 0; i < (unsigned)len && str[i]; i++)
        tag[i] = str[i];
    for (; i < 4; i++)
        tag[i] = ' ';

    return HB_TAG(tag[0], tag[1], tag[2], tag[3]);
}

/**
 * hb_tag_to_string:
 * @tag:
 * @buf: (out caller-allocates) (array fixed-size=4) (element-type uint8_t):
 *
 *
 *
 * Since: 0.9.5
 **/
void hb_tag_to_string(hb_tag_t tag, char *buf)
{
    buf[0] = (char)(uint8_t)(tag >> 24);
    buf[1] = (char)(uint8_t)(tag >> 16);
    buf[2] = (char)(uint8_t)(tag >> 8);
    buf[3] = (char)(uint8_t)(tag >> 0);
}

const char* hb_language_get_default()
{
    return setlocale(LC_CTYPE, nullptr);
}

/* If there is no visibility control, then hb-static.cc will NOT
 * define anything.  Instead, we get it to define one set in here
 * only, so only libharfbuzz.so defines them, not other libs. */
#ifdef HB_NO_VISIBILITY
#undef HB_NO_VISIBILITY
#include "hb-static.cc"
#define HB_NO_VISIBILITY 1
#endif
