/*
 * Copyright (C) 2012 Grigori Goronzy <greg@kinoho.net>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

#include "hb.hh"
#include "hb-ucd.hh"

#include "hb-ucd-table.hh"

extern "C" {
  hb_script_t rb_ucd_script (hb_codepoint_t unicode);
  hb_bool_t rb_ucd_decompose (hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b);
  hb_bool_t rb_ucd_compose (hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab);
}

hb_unicode_combining_class_t
hb_ucd_combining_class (hb_codepoint_t unicode)
{
  return (hb_unicode_combining_class_t) _hb_ucd_ccc (unicode);
}

hb_unicode_general_category_t
hb_ucd_general_category (hb_codepoint_t unicode)
{
  return (hb_unicode_general_category_t) _hb_ucd_gc (unicode);
}

hb_codepoint_t
hb_ucd_mirroring (hb_codepoint_t unicode)
{
  return unicode + _hb_ucd_bmg (unicode);
}

hb_script_t
hb_ucd_script (hb_codepoint_t unicode)
{
  return rb_ucd_script (unicode);
}

hb_bool_t
hb_ucd_compose (hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
  *ab = 0;
  if (unlikely (!a || !b)) return false;
  return rb_ucd_compose(a, b, ab);
}

hb_bool_t
hb_ucd_decompose (hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b)
{
  *a = ab; *b = 0;
  return rb_ucd_decompose(ab, a, b);
}
