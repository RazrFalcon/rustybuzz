#pragma once

#include "hb-unicode.h"
#include "hb.hh"

hb_unicode_combining_class_t
hb_ucd_combining_class (hb_codepoint_t unicode);

hb_unicode_general_category_t
hb_ucd_general_category (hb_codepoint_t unicode);

hb_codepoint_t
hb_ucd_mirroring (hb_codepoint_t unicode);

hb_script_t
hb_ucd_script (hb_codepoint_t unicode);

hb_bool_t
hb_ucd_compose (hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab);

hb_bool_t
hb_ucd_decompose (hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b);
