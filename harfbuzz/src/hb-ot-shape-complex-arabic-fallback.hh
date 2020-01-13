/*
 * Copyright © 2012  Google, Inc.
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

#ifndef HB_OT_SHAPE_COMPLEX_ARABIC_FALLBACK_HH
#define HB_OT_SHAPE_COMPLEX_ARABIC_FALLBACK_HH

#include "hb.hh"

#include "hb-ot-shape.hh"
#include "hb-ot-layout-gsub-table.hh"


/* Features ordered the same as the entries in shaping_table rows,
 * followed by rlig.  Don't change. */
static const hb_tag_t arabic_fallback_features[] =
{
  HB_TAG('i','n','i','t'),
  HB_TAG('m','e','d','i'),
  HB_TAG('f','i','n','a'),
  HB_TAG('i','s','o','l'),
  HB_TAG('r','l','i','g'),
};

#define ARABIC_FALLBACK_MAX_LOOKUPS 5

struct arabic_fallback_plan_t
{
  unsigned int num_lookups;
  bool free_lookups;

  hb_mask_t mask_array[ARABIC_FALLBACK_MAX_LOOKUPS];
  OT::SubstLookup *lookup_array[ARABIC_FALLBACK_MAX_LOOKUPS];
  OT::hb_ot_layout_lookup_accelerator_t accel_array[ARABIC_FALLBACK_MAX_LOOKUPS];
};

#if defined(_WIN32) && !defined(HB_NO_WIN1256)
#define HB_WITH_WIN1256
#endif

#ifdef HB_WITH_WIN1256
#include "hb-ot-shape-complex-arabic-win1256.hh"
#endif

struct ManifestLookup
{
  public:
  OT::Tag tag;
  OT::OffsetTo<OT::SubstLookup> lookupOffset;
  public:
  DEFINE_SIZE_STATIC (6);
};
typedef OT::ArrayOf<ManifestLookup> Manifest;

static bool
arabic_fallback_plan_init_win1256 (arabic_fallback_plan_t *fallback_plan HB_UNUSED,
				   const hb_ot_shape_plan_t *plan HB_UNUSED,
				   hb_font_t *font HB_UNUSED)
{
#ifdef HB_WITH_WIN1256
  /* Does this font look like it's Windows-1256-encoded? */
  hb_codepoint_t g;
  if (!(hb_font_get_glyph (font, 0x0627u, 0, &g) && g == 199 /* ALEF */ &&
	hb_font_get_glyph (font, 0x0644u, 0, &g) && g == 225 /* LAM */ &&
	hb_font_get_glyph (font, 0x0649u, 0, &g) && g == 236 /* ALEF MAKSURA */ &&
	hb_font_get_glyph (font, 0x064Au, 0, &g) && g == 237 /* YEH */ &&
	hb_font_get_glyph (font, 0x0652u, 0, &g) && g == 250 /* SUKUN */))
    return false;

  const Manifest &manifest = reinterpret_cast<const Manifest&> (arabic_win1256_gsub_lookups.manifest);
  static_assert (sizeof (arabic_win1256_gsub_lookups.manifestData) ==
		 ARABIC_FALLBACK_MAX_LOOKUPS * sizeof (ManifestLookup), "");
  /* TODO sanitize the table? */

  unsigned j = 0;
  unsigned int count = manifest.len;
  for (unsigned int i = 0; i < count; i++)
  {
    fallback_plan->mask_array[j] = plan->map.get_1_mask (manifest[i].tag);
    if (fallback_plan->mask_array[j])
    {
      fallback_plan->lookup_array[j] = const_cast<OT::SubstLookup*> (&(&manifest+manifest[i].lookupOffset));
      if (fallback_plan->lookup_array[j])
      {
	fallback_plan->accel_array[j].init (*fallback_plan->lookup_array[j]);
	j++;
      }
    }
  }

  fallback_plan->num_lookups = j;
  fallback_plan->free_lookups = false;

  return j > 0;
#else
  return false;
#endif
}

static arabic_fallback_plan_t *
arabic_fallback_plan_create (const hb_ot_shape_plan_t *plan,
			     hb_font_t *font)
{
  arabic_fallback_plan_t *fallback_plan = (arabic_fallback_plan_t *) calloc (1, sizeof (arabic_fallback_plan_t));
  if (unlikely (!fallback_plan))
    return const_cast<arabic_fallback_plan_t *> (&Null(arabic_fallback_plan_t));

  fallback_plan->num_lookups = 0;
  fallback_plan->free_lookups = false;

  /* Try synthesizing GSUB table using Unicode Arabic Presentation Forms,
   * in case the font has cmap entries for the presentation-forms characters. */
  // TODO: this
//  if (arabic_fallback_plan_init_unicode (fallback_plan, plan, font))
//    return fallback_plan;

  /* See if this looks like a Windows-1256-encoded font.  If it does, use a
   * hand-coded GSUB table. */
  if (arabic_fallback_plan_init_win1256 (fallback_plan, plan, font))
    return fallback_plan;

  assert (fallback_plan->num_lookups == 0);
  free (fallback_plan);
  return const_cast<arabic_fallback_plan_t *> (&Null(arabic_fallback_plan_t));
}

static void
arabic_fallback_plan_destroy (arabic_fallback_plan_t *fallback_plan)
{
  if (!fallback_plan || fallback_plan->num_lookups == 0)
    return;

  for (unsigned int i = 0; i < fallback_plan->num_lookups; i++)
    if (fallback_plan->lookup_array[i])
    {
      fallback_plan->accel_array[i].fini ();
      if (fallback_plan->free_lookups)
	free (fallback_plan->lookup_array[i]);
    }

  free (fallback_plan);
}

static void
arabic_fallback_plan_shape (arabic_fallback_plan_t *fallback_plan,
			    hb_font_t *font,
			    hb_buffer_t *buffer)
{
  OT::hb_ot_apply_context_t c (0, font, buffer);
  for (unsigned int i = 0; i < fallback_plan->num_lookups; i++)
    if (fallback_plan->lookup_array[i]) {
      c.set_lookup_mask (fallback_plan->mask_array[i]);
      hb_ot_layout_substitute_lookup (&c,
				      *fallback_plan->lookup_array[i],
				      fallback_plan->accel_array[i]);
    }
}


#endif /* HB_OT_SHAPE_COMPLEX_ARABIC_FALLBACK_HH */
