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

/* hb_options_t */

hb_atomic_int_t _hb_options;

void _hb_options_init()
{
    hb_options_union_t u;
    u.i = 0;
    u.opts.initialized = true;

    const char *c = getenv("HB_OPTIONS");
    if (c) {
        while (*c) {
            const char *p = strchr(c, ':');
            if (!p)
                p = c + strlen(c);

#define OPTION(name, symbol)                                                                                           \
    if (0 == strncmp(c, name, p - c) && strlen(name) == static_cast<size_t>(p - c))                                    \
        do {                                                                                                           \
            u.opts.symbol = true;                                                                                      \
    } while (0)

            OPTION("uniscribe-bug-compatible", uniscribe_bug_compatible);

#undef OPTION

            c = *p ? p + 1 : p;
        }
    }

    /* This is idempotent and threadsafe. */
    _hb_options.set_relaxed(u.i);
}

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

/* hb_direction_t */

const char direction_strings[][4] = {"ltr", "rtl", "ttb", "btt"};

/**
 * hb_direction_from_string:
 * @str: (array length=len) (element-type uint8_t):
 * @len:
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_direction_t hb_direction_from_string(const char *str, int len)
{
    if (unlikely(!str || !len || !*str))
        return HB_DIRECTION_INVALID;

    /* Lets match loosely: just match the first letter, such that
     * all of "ltr", "left-to-right", etc work!
     */
    char c = TOLOWER(str[0]);
    for (unsigned int i = 0; i < ARRAY_LENGTH(direction_strings); i++)
        if (c == direction_strings[i][0])
            return (hb_direction_t)(HB_DIRECTION_LTR + i);

    return HB_DIRECTION_INVALID;
}

/**
 * hb_direction_to_string:
 * @direction:
 *
 *
 *
 * Return value: (transfer none):
 *
 * Since: 0.9.2
 **/
const char *hb_direction_to_string(hb_direction_t direction)
{
    if (likely((unsigned int)(direction - HB_DIRECTION_LTR) < ARRAY_LENGTH(direction_strings)))
        return direction_strings[direction - HB_DIRECTION_LTR];

    return "invalid";
}

/* hb_language_t */

struct hb_language_impl_t
{
    const char s[1];
};

static const char canon_map[256] = {
    0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
    0,   '-', 0,   0,   '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 0,   0,   0,   0,   0,   0,   0,   'a',
    'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
    'x', 'y', 'z', 0,   0,   0,   0,   '-', 0,   'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 0,   0,   0,   0,   0};

static bool lang_equal(hb_language_t v1, const void *v2)
{
    const unsigned char *p1 = (const unsigned char *)v1;
    const unsigned char *p2 = (const unsigned char *)v2;

    while (*p1 && *p1 == canon_map[*p2]) {
        p1++;
        p2++;
    }

    return *p1 == canon_map[*p2];
}

struct hb_language_item_t
{

    struct hb_language_item_t *next;
    hb_language_t lang;

    bool operator==(const char *s) const
    {
        return lang_equal(lang, s);
    }

    hb_language_item_t &operator=(const char *s)
    {
        /* If a custom allocated is used calling strdup() pairs
        badly with a call to the custom free() in fini() below.
        Therefore don't call strdup(), implement its behavior.
        */
        size_t len = strlen(s) + 1;
        lang = (hb_language_t)malloc(len);
        if (likely(lang)) {
            memcpy((unsigned char *)lang, s, len);
            for (unsigned char *p = (unsigned char *)lang; *p; p++)
                *p = canon_map[*p];
        }

        return *this;
    }

    void fini()
    {
        free((void *)lang);
    }
};

/* Thread-safe lock-free language list */

static hb_atomic_ptr_t<hb_language_item_t> langs;

#if HB_USE_ATEXIT
static void free_langs()
{
retry:
    hb_language_item_t *first_lang = langs;
    if (unlikely(!langs.cmpexch(first_lang, nullptr)))
        goto retry;

    while (first_lang) {
        hb_language_item_t *next = first_lang->next;
        first_lang->fini();
        free(first_lang);
        first_lang = next;
    }
}
#endif

static hb_language_item_t *lang_find_or_insert(const char *key)
{
retry:
    hb_language_item_t *first_lang = langs;

    for (hb_language_item_t *lang = first_lang; lang; lang = lang->next)
        if (*lang == key)
            return lang;

    /* Not found; allocate one. */
    hb_language_item_t *lang = (hb_language_item_t *)calloc(1, sizeof(hb_language_item_t));
    if (unlikely(!lang))
        return nullptr;
    lang->next = first_lang;
    *lang = key;
    if (unlikely(!lang->lang)) {
        free(lang);
        return nullptr;
    }

    if (unlikely(!langs.cmpexch(first_lang, lang))) {
        lang->fini();
        free(lang);
        goto retry;
    }

#if HB_USE_ATEXIT
    if (!first_lang)
        atexit(free_langs); /* First person registers atexit() callback. */
#endif

    return lang;
}

/**
 * hb_language_from_string:
 * @str: (array length=len) (element-type uint8_t): a string representing
 *       a BCP 47 language tag
 * @len: length of the @str, or -1 if it is %NULL-terminated.
 *
 * Converts @str representing a BCP 47 language tag to the corresponding
 * #hb_language_t.
 *
 * Return value: (transfer none):
 * The #hb_language_t corresponding to the BCP 47 language tag.
 *
 * Since: 0.9.2
 **/
hb_language_t hb_language_from_string(const char *str, int len)
{
    if (!str || !len || !*str)
        return HB_LANGUAGE_INVALID;

    hb_language_item_t *item = nullptr;
    if (len >= 0) {
        /* NUL-terminate it. */
        char strbuf[64];
        len = hb_min(len, (int)sizeof(strbuf) - 1);
        memcpy(strbuf, str, len);
        strbuf[len] = '\0';
        item = lang_find_or_insert(strbuf);
    } else
        item = lang_find_or_insert(str);

    return likely(item) ? item->lang : HB_LANGUAGE_INVALID;
}

/**
 * hb_language_to_string:
 * @language: an #hb_language_t to convert.
 *
 * See hb_language_from_string().
 *
 * Return value: (transfer none):
 * A %NULL-terminated string representing the @language. Must not be freed by
 * the caller.
 *
 * Since: 0.9.2
 **/
const char *hb_language_to_string(hb_language_t language)
{
    if (unlikely(!language))
        return nullptr;

    return language->s;
}

/**
 * hb_language_get_default:
 *
 * Get default language from current locale.
 *
 * Note that the first time this function is called, it calls
 * "setlocale (LC_CTYPE, nullptr)" to fetch current locale.  The underlying
 * setlocale function is, in many implementations, NOT threadsafe.  To avoid
 * problems, call this function once before multiple threads can call it.
 * This function is only used from hb_buffer_guess_segment_properties() by
 * HarfBuzz itself.
 *
 * Return value: (transfer none):
 *
 * Since: 0.9.2
 **/
hb_language_t hb_language_get_default()
{
    static hb_atomic_ptr_t<hb_language_t> default_language;

    hb_language_t language = default_language;
    if (unlikely(language == HB_LANGUAGE_INVALID)) {
        language = hb_language_from_string(setlocale(LC_CTYPE, nullptr), -1);
        (void)default_language.cmpexch(HB_LANGUAGE_INVALID, language);
    }

    return language;
}

/* If there is no visibility control, then hb-static.cc will NOT
 * define anything.  Instead, we get it to define one set in here
 * only, so only libharfbuzz.so defines them, not other libs. */
#ifdef HB_NO_VISIBILITY
#undef HB_NO_VISIBILITY
#include "hb-static.cc"
#define HB_NO_VISIBILITY 1
#endif
