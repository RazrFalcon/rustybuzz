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
#include "hb-unicode.hh"
#include "hb-machinery.hh"
#include "hb-ucd.hh"

#include "hb-ucd-table.hh"

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
  return _hb_ucd_sc_map[_hb_ucd_sc (unicode)];
}


#define SBASE 0xAC00u
#define LBASE 0x1100u
#define VBASE 0x1161u
#define TBASE 0x11A7u
#define SCOUNT 11172u
#define LCOUNT 19u
#define VCOUNT 21u
#define TCOUNT 28u
#define NCOUNT (VCOUNT * TCOUNT)

static inline bool
_hb_ucd_decompose_hangul (hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b)
{
  unsigned si = ab - SBASE;

  if (si >= SCOUNT)
    return false;

  if (si % TCOUNT)
  {
    /* LV,T */
    *a = SBASE + (si / TCOUNT) * TCOUNT;
    *b = TBASE + (si % TCOUNT);
    return true;
  } else {
    /* L,V */
    *a = LBASE + (si / NCOUNT);
    *b = VBASE + (si % NCOUNT) / TCOUNT;
    return true;
  }
}

static inline bool
_hb_ucd_compose_hangul (hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
  if (a >= SBASE && a < (SBASE + SCOUNT) && b > TBASE && b < (TBASE + TCOUNT) &&
    !((a - SBASE) % TCOUNT))
  {
    /* LV,T */
    *ab = a + (b - TBASE);
    return true;
  }
  else if (a >= LBASE && a < (LBASE + LCOUNT) && b >= VBASE && b < (VBASE + VCOUNT))
  {
    /* L,V */
    int li = a - LBASE;
    int vi = b - VBASE;
    *ab = SBASE + li * NCOUNT + vi * TCOUNT;
    return true;
  }
  else
    return false;
}

static int
_cmp_pair (const void *_key, const void *_item)
{
  uint64_t& a = * (uint64_t*) _key;
  uint64_t b = (* (uint64_t*) _item) & HB_CODEPOINT_ENCODE3(0x1FFFFFu, 0x1FFFFFu, 0);

  return a < b ? -1 : a > b ? +1 : 0;
}
static int
_cmp_pair_11_7_14 (const void *_key, const void *_item)
{
  uint32_t& a = * (uint32_t*) _key;
  uint32_t b = (* (uint32_t*) _item) & HB_CODEPOINT_ENCODE3_11_7_14(0x1FFFFFu, 0x1FFFFFu, 0);

  return a < b ? -1 : a > b ? +1 : 0;
}

hb_bool_t
hb_ucd_compose (hb_codepoint_t a, hb_codepoint_t b, hb_codepoint_t *ab)
{
  if (_hb_ucd_compose_hangul (a, b, ab)) return true;

  hb_codepoint_t u = 0;

  if ((a & 0xFFFFF800u) == 0x0000u && (b & 0xFFFFFF80) == 0x0300u)
  {
    uint32_t k = HB_CODEPOINT_ENCODE3_11_7_14 (a, b, 0);
    uint32_t *v = (uint32_t*) hb_bsearch (&k, _hb_ucd_dm2_u32_map,
					  ARRAY_LENGTH (_hb_ucd_dm2_u32_map),
					  sizeof (*_hb_ucd_dm2_u32_map),
					  _cmp_pair_11_7_14);
    if (likely (!v)) return false;
    u = HB_CODEPOINT_DECODE3_11_7_14_3 (*v);
  }
  else
  {
    uint64_t k = HB_CODEPOINT_ENCODE3 (a, b, 0);
    uint64_t *v = (uint64_t*) hb_bsearch (&k, _hb_ucd_dm2_u64_map,
					  ARRAY_LENGTH (_hb_ucd_dm2_u64_map),
					  sizeof (*_hb_ucd_dm2_u64_map),
					  _cmp_pair);
    if (likely (!v)) return false;
    u = HB_CODEPOINT_DECODE3_3 (*v);
  }

  if (unlikely (!u)) return false;
  *ab = u;
  return true;
}

hb_bool_t
hb_ucd_decompose (hb_codepoint_t ab, hb_codepoint_t *a, hb_codepoint_t *b)
{
  if (_hb_ucd_decompose_hangul (ab, a, b)) return true;

  unsigned i = _hb_ucd_dm (ab);

  if (likely (!i)) return false;
  i--;

  if (i < ARRAY_LENGTH (_hb_ucd_dm1_p0_map) + ARRAY_LENGTH (_hb_ucd_dm1_p2_map))
  {
    if (i < ARRAY_LENGTH (_hb_ucd_dm1_p0_map))
      *a = _hb_ucd_dm1_p0_map[i];
    else
    {
      i -= ARRAY_LENGTH (_hb_ucd_dm1_p0_map);
      *a = 0x20000 | _hb_ucd_dm1_p2_map[i];
    }
    *b = 0;
    return true;
  }
  i -= ARRAY_LENGTH (_hb_ucd_dm1_p0_map) + ARRAY_LENGTH (_hb_ucd_dm1_p2_map);

  if (i < ARRAY_LENGTH (_hb_ucd_dm2_u32_map))
  {
    uint32_t v = _hb_ucd_dm2_u32_map[i];
    *a = HB_CODEPOINT_DECODE3_11_7_14_1 (v);
    *b = HB_CODEPOINT_DECODE3_11_7_14_2 (v);
    return true;
  }
  i -= ARRAY_LENGTH (_hb_ucd_dm2_u32_map);

  uint64_t v = _hb_ucd_dm2_u64_map[i];
  *a = HB_CODEPOINT_DECODE3_1 (v);
  *b = HB_CODEPOINT_DECODE3_2 (v);
  return true;
}
