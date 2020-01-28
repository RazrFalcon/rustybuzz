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

void
_hb_options_init ()
{
  hb_options_union_t u;
  u.i = 0;
  u.opts.initialized = true;

  const char *c = getenv ("HB_OPTIONS");
  if (c)
  {
    while (*c)
    {
      const char *p = strchr (c, ':');
      if (!p)
        p = c + strlen (c);

#define OPTION(name, symbol) \
        if (0 == strncmp (c, name, p - c) && strlen (name) == static_cast<size_t>(p - c)) do { u.opts.symbol = true; } while (0)

      OPTION ("uniscribe-bug-compatible", uniscribe_bug_compatible);
      OPTION ("aat", aat);

#undef OPTION

      c = *p ? p + 1 : p;
    }

  }

  /* This is idempotent and threadsafe. */
  _hb_options.set_relaxed (u.i);
}

/* hb_script_t */

/**
 * hb_script_get_horizontal_direction:
 * @script:
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
hb_direction_t
hb_script_get_horizontal_direction (hb_script_t script)
{
  /* https://docs.google.com/spreadsheets/d/1Y90M0Ie3MUJ6UVCRDOypOtijlMDLNNyyLk36T6iMu0o */
  switch ((hb_tag_t) script)
  {
    /* Unicode-1.1 additions */
    case HB_SCRIPT_ARABIC:
    case HB_SCRIPT_HEBREW:

    /* Unicode-3.0 additions */
    case HB_SCRIPT_SYRIAC:
    case HB_SCRIPT_THAANA:

    /* Unicode-4.0 additions */
    case HB_SCRIPT_CYPRIOT:

    /* Unicode-4.1 additions */
    case HB_SCRIPT_KHAROSHTHI:

    /* Unicode-5.0 additions */
    case HB_SCRIPT_PHOENICIAN:
    case HB_SCRIPT_NKO:

    /* Unicode-5.1 additions */
    case HB_SCRIPT_LYDIAN:

    /* Unicode-5.2 additions */
    case HB_SCRIPT_AVESTAN:
    case HB_SCRIPT_IMPERIAL_ARAMAIC:
    case HB_SCRIPT_INSCRIPTIONAL_PAHLAVI:
    case HB_SCRIPT_INSCRIPTIONAL_PARTHIAN:
    case HB_SCRIPT_OLD_SOUTH_ARABIAN:
    case HB_SCRIPT_OLD_TURKIC:
    case HB_SCRIPT_SAMARITAN:

    /* Unicode-6.0 additions */
    case HB_SCRIPT_MANDAIC:

    /* Unicode-6.1 additions */
    case HB_SCRIPT_MEROITIC_CURSIVE:
    case HB_SCRIPT_MEROITIC_HIEROGLYPHS:

    /* Unicode-7.0 additions */
    case HB_SCRIPT_MANICHAEAN:
    case HB_SCRIPT_MENDE_KIKAKUI:
    case HB_SCRIPT_NABATAEAN:
    case HB_SCRIPT_OLD_NORTH_ARABIAN:
    case HB_SCRIPT_PALMYRENE:
    case HB_SCRIPT_PSALTER_PAHLAVI:

    /* Unicode-8.0 additions */
    case HB_SCRIPT_HATRAN:

    /* Unicode-9.0 additions */
    case HB_SCRIPT_ADLAM:

    /* Unicode-11.0 additions */
    case HB_SCRIPT_HANIFI_ROHINGYA:
    case HB_SCRIPT_OLD_SOGDIAN:
    case HB_SCRIPT_SOGDIAN:

      return HB_DIRECTION_RTL;


    /* https://github.com/harfbuzz/harfbuzz/issues/1000 */
    case HB_SCRIPT_OLD_HUNGARIAN:
    case HB_SCRIPT_OLD_ITALIC:
    case HB_SCRIPT_RUNIC:

      return HB_DIRECTION_INVALID;
  }

  return HB_DIRECTION_LTR;
}
