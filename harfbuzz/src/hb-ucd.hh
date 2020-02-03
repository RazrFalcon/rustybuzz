#pragma once

#include "hb-unicode.h"
#include "hb.hh"

extern "C" {
hb_script_t rb_ucd_script(hb_codepoint_t unicode);
hb_bool_t rb_ucd_decompose(hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b);
hb_bool_t rb_ucd_compose(hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab);
hb_unicode_general_category_t rb_ucd_general_category(hb_codepoint_t unicode);
hb_unicode_combining_class_t rb_ucd_combining_class(hb_codepoint_t unicode);
hb_codepoint_t rb_ucd_mirroring(hb_codepoint_t unicode);
}

inline hb_bool_t hb_ucd_compose(hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
    *ab = 0;
    if (unlikely(!a || !b))
        return false;
    return rb_ucd_compose(a, b, ab);
}

inline hb_bool_t hb_ucd_decompose(hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b)
{
    *a = ab;
    *b = 0;
    return rb_ucd_decompose(ab, a, b);
}
