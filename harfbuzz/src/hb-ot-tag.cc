/*
 * Copyright © 2009  Red Hat, Inc.
 * Copyright © 2011  Google, Inc.
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
 * Google Author(s): Behdad Esfahbod, Roozbeh Pournader
 */

#include "hb.hh"

/* rb_script_t */

static rb_tag_t rb_ot_old_tag_from_script(rb_script_t script)
{
    /* This seems to be accurate as of end of 2012. */

    switch ((rb_tag_t)script) {
    case RB_SCRIPT_INVALID:
        return RB_OT_TAG_DEFAULT_SCRIPT;

    /* KATAKANA and HIRAGANA both map to 'kana' */
    case RB_SCRIPT_HIRAGANA:
        return RB_TAG('k', 'a', 'n', 'a');

    /* Spaces at the end are preserved, unlike ISO 15924 */
    case RB_SCRIPT_LAO:
        return RB_TAG('l', 'a', 'o', ' ');
    case RB_SCRIPT_YI:
        return RB_TAG('y', 'i', ' ', ' ');
    /* Unicode-5.0 additions */
    case RB_SCRIPT_NKO:
        return RB_TAG('n', 'k', 'o', ' ');
    /* Unicode-5.1 additions */
    case RB_SCRIPT_VAI:
        return RB_TAG('v', 'a', 'i', ' ');
    }

    /* Else, just change first char to lowercase and return */
    return ((rb_tag_t)script) | 0x20000000u;
}

static rb_script_t rb_ot_old_tag_to_script(rb_tag_t tag)
{
    if (unlikely(tag == RB_OT_TAG_DEFAULT_SCRIPT))
        return RB_SCRIPT_INVALID;

    /* This side of the conversion is fully algorithmic. */

    /* Any spaces at the end of the tag are replaced by repeating the last
     * letter.  Eg 'nko ' -> 'Nkoo' */
    if (unlikely((tag & 0x0000FF00u) == 0x00002000u))
        tag |= (tag >> 8) & 0x0000FF00u; /* Copy second letter to third */
    if (unlikely((tag & 0x000000FFu) == 0x00000020u))
        tag |= (tag >> 8) & 0x000000FFu; /* Copy third letter to fourth */

    /* Change first char to uppercase and return */
    return (rb_script_t)(tag & ~0x20000000u);
}

static rb_tag_t rb_ot_new_tag_from_script(rb_script_t script)
{
    switch ((rb_tag_t)script) {
    case RB_SCRIPT_BENGALI:
        return RB_TAG('b', 'n', 'g', '2');
    case RB_SCRIPT_DEVANAGARI:
        return RB_TAG('d', 'e', 'v', '2');
    case RB_SCRIPT_GUJARATI:
        return RB_TAG('g', 'j', 'r', '2');
    case RB_SCRIPT_GURMUKHI:
        return RB_TAG('g', 'u', 'r', '2');
    case RB_SCRIPT_KANNADA:
        return RB_TAG('k', 'n', 'd', '2');
    case RB_SCRIPT_MALAYALAM:
        return RB_TAG('m', 'l', 'm', '2');
    case RB_SCRIPT_ORIYA:
        return RB_TAG('o', 'r', 'y', '2');
    case RB_SCRIPT_TAMIL:
        return RB_TAG('t', 'm', 'l', '2');
    case RB_SCRIPT_TELUGU:
        return RB_TAG('t', 'e', 'l', '2');
    case RB_SCRIPT_MYANMAR:
        return RB_TAG('m', 'y', 'm', '2');
    }

    return RB_OT_TAG_DEFAULT_SCRIPT;
}

static rb_script_t rb_ot_new_tag_to_script(rb_tag_t tag)
{
    switch (tag) {
    case RB_TAG('b', 'n', 'g', '2'):
        return RB_SCRIPT_BENGALI;
    case RB_TAG('d', 'e', 'v', '2'):
        return RB_SCRIPT_DEVANAGARI;
    case RB_TAG('g', 'j', 'r', '2'):
        return RB_SCRIPT_GUJARATI;
    case RB_TAG('g', 'u', 'r', '2'):
        return RB_SCRIPT_GURMUKHI;
    case RB_TAG('k', 'n', 'd', '2'):
        return RB_SCRIPT_KANNADA;
    case RB_TAG('m', 'l', 'm', '2'):
        return RB_SCRIPT_MALAYALAM;
    case RB_TAG('o', 'r', 'y', '2'):
        return RB_SCRIPT_ORIYA;
    case RB_TAG('t', 'm', 'l', '2'):
        return RB_SCRIPT_TAMIL;
    case RB_TAG('t', 'e', 'l', '2'):
        return RB_SCRIPT_TELUGU;
    case RB_TAG('m', 'y', 'm', '2'):
        return RB_SCRIPT_MYANMAR;
    }

    return RB_SCRIPT_UNKNOWN;
}

/*
 * Complete list at:
 * https://docs.microsoft.com/en-us/typography/opentype/spec/scripttags
 *
 * Most of the script tags are the same as the ISO 15924 tag but lowercased.
 * So we just do that, and handle the exceptional cases in a switch.
 */

static void rb_ot_all_tags_from_script(rb_script_t script, unsigned int *count /* IN/OUT */, rb_tag_t *tags /* OUT */)
{
    unsigned int i = 0;

    rb_tag_t new_tag = rb_ot_new_tag_from_script(script);
    if (unlikely(new_tag != RB_OT_TAG_DEFAULT_SCRIPT)) {
        /* RB_SCRIPT_MYANMAR maps to 'mym2', but there is no 'mym3'. */
        if (new_tag != RB_TAG('m', 'y', 'm', '2'))
            tags[i++] = new_tag | '3';
        if (*count > i)
            tags[i++] = new_tag;
    }

    if (*count > i) {
        rb_tag_t old_tag = rb_ot_old_tag_from_script(script);
        if (old_tag != RB_OT_TAG_DEFAULT_SCRIPT)
            tags[i++] = old_tag;
    }

    *count = i;
}

rb_script_t rb_ot_tag_to_script(rb_tag_t tag)
{
    unsigned char digit = tag & 0x000000FFu;
    if (unlikely(digit == '2' || digit == '3'))
        return rb_ot_new_tag_to_script(tag & 0xFFFFFF32);

    return rb_ot_old_tag_to_script(tag);
}

/* rb_language_t */

static bool subtag_matches(const char *lang_str, const char *limit, const char *subtag)
{
    do {
        const char *s = strstr(lang_str, subtag);
        if (!s || s >= limit)
            return false;
        if (!ISALNUM(s[strlen(subtag)]))
            return true;
        lang_str = s + strlen(subtag);
    } while (true);
}

static rb_bool_t lang_matches(const char *lang_str, const char *spec)
{
    unsigned int len = strlen(spec);

    return strncmp(lang_str, spec, len) == 0 && (lang_str[len] == '\0' || lang_str[len] == '-');
}

static const char *rb_language_from_string(const char *s, size_t len)
{
    return s;
}

struct LangTag
{
    char language[4];
    rb_tag_t tag;

    int cmp(const char *a) const
    {
        const char *b = this->language;
        unsigned int da, db;
        const char *p;

        p = strchr(a, '-');
        da = p ? (unsigned int)(p - a) : strlen(a);

        p = strchr(b, '-');
        db = p ? (unsigned int)(p - b) : strlen(b);

        return strncmp(a, b, rb_max(da, db));
    }
    int cmp(const LangTag *that) const
    {
        return cmp(that->language);
    }
};

#include "hb-ot-tag-table.hh"

/* The corresponding languages IDs for the following IDs are unclear,
 * overlap, or are architecturally weird. Needs more research. */

/*{"??",	{RB_TAG('B','C','R',' ')}},*/ /* Bible Cree */
/*{"zh?",	{RB_TAG('C','H','N',' ')}},*/ /* Chinese (seen in Microsoft fonts) */
/*{"ar-Syrc?",	{RB_TAG('G','A','R',' ')}},*/ /* Garshuni */
/*{"??",	{RB_TAG('N','G','R',' ')}},*/ /* Nagari */
/*{"??",	{RB_TAG('Y','I','C',' ')}},*/ /* Yi Classic */
/*{"zh?",	{RB_TAG('Z','H','P',' ')}},*/ /* Chinese Phonetic */

static void rb_ot_tags_from_language(const char *lang_str, const char *limit, unsigned int *count, rb_tag_t *tags)
{
    const char *s;
    unsigned int tag_idx;

    /* Check for matches of multiple subtags. */
    if (rb_ot_tags_from_complex_language(lang_str, limit, count, tags))
        return;

    /* Find a language matching in the first component. */
    s = strchr(lang_str, '-');
    {
        if (s && limit - lang_str >= 6) {
            const char *extlang_end = strchr(s + 1, '-');
            /* If there is an extended language tag, use it. */
            if (3 == (extlang_end ? extlang_end - s - 1 : strlen(s + 1)) && ISALPHA(s[1]))
                lang_str = s + 1;
        }
        if (rb_sorted_array(ot_languages).bfind(lang_str, &tag_idx)) {
            unsigned int i;
            while (tag_idx != 0 && 0 == strcmp(ot_languages[tag_idx].language, ot_languages[tag_idx - 1].language))
                tag_idx--;
            for (i = 0; i < *count && tag_idx + i < ARRAY_LENGTH(ot_languages) &&
                        0 == strcmp(ot_languages[tag_idx + i].language, ot_languages[tag_idx].language);
                 i++)
                tags[i] = ot_languages[tag_idx + i].tag;
            *count = i;
            return;
        }
    }

    if (!s)
        s = lang_str + strlen(lang_str);
    if (s - lang_str == 3) {
        /* Assume it's ISO-639-3 and upper-case and use it. */
        tags[0] = rb_tag_from_string(lang_str, s - lang_str) & ~0x20202000u;
        *count = 1;
        return;
    }

    *count = 0;
}

static bool parse_private_use_subtag(const char *private_use_subtag,
                                     unsigned int *count,
                                     rb_tag_t *tags,
                                     const char *prefix,
                                     unsigned char (*normalize)(unsigned char))
{
#ifdef RB_NO_LANGUAGE_PRIVATE_SUBTAG
    return false;
#endif

    if (!(private_use_subtag && count && tags && *count))
        return false;

    const char *s = strstr(private_use_subtag, prefix);
    if (!s)
        return false;

    char tag[4];
    int i;
    s += strlen(prefix);
    if (s[0] == '-') {
        s += 1;
        char c;
        for (i = 0; i < 8 && ISHEX(s[i]); i++) {
            c = FROMHEX(s[i]);
            if (i % 2 == 0)
                tag[i / 2] = c << 4;
            else
                tag[i / 2] += c;
        }
        if (i != 8)
            return false;
    } else {
        for (i = 0; i < 4 && ISALNUM(s[i]); i++)
            tag[i] = normalize(s[i]);
        if (!i)
            return false;

        for (; i < 4; i++)
            tag[i] = ' ';
    }
    tags[0] = RB_TAG(tag[0], tag[1], tag[2], tag[3]);
    if ((tags[0] & 0xDFDFDFDF) == RB_OT_TAG_DEFAULT_SCRIPT)
        tags[0] ^= ~0xDFDFDFDF;
    *count = 1;
    return true;
}

/**
 * rb_ot_tags_from_script_and_language:
 * @script: an #rb_script_t to convert.
 * @language: an #rb_language_t to convert.
 * @script_count: (allow-none): maximum number of script tags to retrieve (IN)
 * and actual number of script tags retrieved (OUT)
 * @script_tags: (out) (allow-none): array of size at least @script_count to store the
 * script tag results
 * @language_count: (allow-none): maximum number of language tags to retrieve
 * (IN) and actual number of language tags retrieved (OUT)
 * @language_tags: (out) (allow-none): array of size at least @language_count to store
 * the language tag results
 *
 * Converts an #rb_script_t and an #rb_language_t to script and language tags.
 *
 * Since: 2.0.0
 **/
void rb_ot_tags_from_script_and_language(rb_script_t script,
                                         const char *language,
                                         unsigned int *script_count /* IN/OUT */,
                                         rb_tag_t *script_tags /* OUT */,
                                         unsigned int *language_count /* IN/OUT */,
                                         rb_tag_t *language_tags /* OUT */)
{
    bool needs_script = true;

    if (language == NULL) {
        if (language_count && language_tags && *language_count)
            *language_count = 0;
    } else {
        const char *lang_str, *s, *limit, *private_use_subtag;
        bool needs_language;

        lang_str = language;
        limit = nullptr;
        private_use_subtag = nullptr;
        if (lang_str[0] == 'x' && lang_str[1] == '-') {
            private_use_subtag = lang_str;
        } else {
            for (s = lang_str + 1; *s; s++) {
                if (s[-1] == '-' && s[1] == '-') {
                    if (s[0] == 'x') {
                        private_use_subtag = s;
                        if (!limit)
                            limit = s - 1;
                        break;
                    } else if (!limit) {
                        limit = s - 1;
                    }
                }
            }
            if (!limit)
                limit = s;
        }

        needs_script = !parse_private_use_subtag(private_use_subtag, script_count, script_tags, "-hbsc", TOLOWER);
        needs_language = !parse_private_use_subtag(private_use_subtag, language_count, language_tags, "-hbot", TOUPPER);

        if (needs_language && language_count && language_tags && *language_count)
            rb_ot_tags_from_language(lang_str, limit, language_count, language_tags);
    }

    if (needs_script && script_count && script_tags && *script_count)
        rb_ot_all_tags_from_script(script, script_count, script_tags);
}

/**
 * rb_ot_tag_to_language:
 *
 *
 *
 * Return value: (transfer none):
 *
 * Since: 0.9.2
 **/
const char *rb_ot_tag_to_language(rb_tag_t tag)
{
    unsigned int i;

    if (tag == RB_OT_TAG_DEFAULT_LANGUAGE)
        return nullptr;

    {
        const char *disambiguated_tag = rb_ot_ambiguous_tag_to_language(tag);
        if (disambiguated_tag != NULL)
            return disambiguated_tag;
    }

    for (i = 0; i < ARRAY_LENGTH(ot_languages); i++)
        if (ot_languages[i].tag == tag)
            return rb_language_from_string(ot_languages[i].language, -1);

    /* Return a custom language in the form of "x-hbot-AABBCCDD".
     * If it's three letters long, also guess it's ISO 639-3 and lower-case and
     * prepend it (if it's not a registered tag, the private use subtags will
     * ensure that calling rb_ot_tag_from_language on the result will still return
     * the same tag as the original tag).
     */
    {
        char buf[20];
        char *str = buf;
        if (ISALPHA(tag >> 24) && ISALPHA((tag >> 16) & 0xFF) && ISALPHA((tag >> 8) & 0xFF) && (tag & 0xFF) == ' ') {
            buf[0] = TOLOWER(tag >> 24);
            buf[1] = TOLOWER((tag >> 16) & 0xFF);
            buf[2] = TOLOWER((tag >> 8) & 0xFF);
            buf[3] = '-';
            str += 4;
        }
        snprintf(str, 16, "x-hbot-%08x", tag);
        return rb_language_from_string(&*buf, -1);
    }
}

/**
 * rb_ot_tags_to_script_and_language:
 * @script_tag: a script tag
 * @language_tag: a language tag
 * @script: (allow-none): the #rb_script_t corresponding to @script_tag (OUT).
 * @language: (allow-none): the #rb_language_t corresponding to @script_tag and
 * @language_tag (OUT).
 *
 * Converts a script tag and a language tag to an #rb_script_t and an
 * #rb_language_t.
 *
 * Since: 2.0.0
 **/
void rb_ot_tags_to_script_and_language(rb_tag_t script_tag,
                                       rb_tag_t language_tag,
                                       rb_script_t *script /* OUT */,
                                       const char **language /* OUT */)
{
    rb_script_t script_out = rb_ot_tag_to_script(script_tag);
    if (script)
        *script = script_out;
    if (language) {
        unsigned int script_count = 1;
        rb_tag_t primary_script_tag[1];
        rb_ot_tags_from_script_and_language(script_out, NULL, &script_count, primary_script_tag, nullptr, nullptr);
        *language = rb_ot_tag_to_language(language_tag);
        if (script_count == 0 || primary_script_tag[0] != script_tag) {
            unsigned char *buf;
            const char *lang_str = *language;
            size_t len = strlen(lang_str);
            buf = (unsigned char *)malloc(len + 16);
            if (unlikely(!buf)) {
                *language = nullptr;
            } else {
                int shift;
                memcpy(buf, lang_str, len);
                if (lang_str[0] != 'x' || lang_str[1] != '-') {
                    buf[len++] = '-';
                    buf[len++] = 'x';
                }
                buf[len++] = '-';
                buf[len++] = 'h';
                buf[len++] = 'b';
                buf[len++] = 's';
                buf[len++] = 'c';
                buf[len++] = '-';
                for (shift = 28; shift >= 0; shift -= 4)
                    buf[len++] = TOHEX(script_tag >> shift);
                *language = rb_language_from_string((char *)buf, len);
                free(buf);
            }
        }
    }
}
