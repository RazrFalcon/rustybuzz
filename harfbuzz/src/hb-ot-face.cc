/*
 * Copyright © 2018  Google, Inc.
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

#include "hb-ot-face.hh"

#include "hb-ot-glyf-table.hh"
#include "hb-ot-kern-table.hh"
#include "hb-ot-color-cbdt-table.hh"
#include "hb-ot-color-sbix-table.hh"
#include "hb-ot-layout-gdef-table.hh"
#include "hb-ot-layout-gsub-table.hh"
#include "hb-ot-layout-gpos-table.hh"


void hb_ot_face_t::init0 (hb_face_t *face)
{
  this->face = face;
  head.init0 ();
  hhea.init0 ();
  OS2.init0 ();
  vhea.init0 ();
  glyf.init0 ();
  gvar.init0 ();
  kern.init0 ();
  GDEF.init0 ();
  GSUB.init0 ();
  GPOS.init0 ();
  morx.init0 ();
  mort.init0 ();
  kerx.init0 ();
  ankr.init0 ();
  trak.init0 ();
  lcar.init0 ();
  ltag.init0 ();
  feat.init0 ();
  CBDT.init0 ();
  sbix.init0 ();
}
void hb_ot_face_t::fini ()
{
  head.fini ();
  hhea.fini ();
  OS2.fini ();
  vhea.fini ();
  glyf.fini ();
  gvar.fini ();
  kern.fini ();
  GDEF.fini ();
  GSUB.fini ();
  GPOS.fini ();
  morx.fini ();
  mort.fini ();
  kerx.fini ();
  ankr.fini ();
  trak.fini ();
  lcar.fini ();
  ltag.fini ();
  feat.fini ();
  CBDT.fini ();
  sbix.fini ();
}
