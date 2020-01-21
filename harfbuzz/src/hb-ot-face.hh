/*
 * Copyright © 2007,2008,2009  Red Hat, Inc.
 * Copyright © 2012,2013  Google, Inc.
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

#pragma once

#include "hb.hh"

#include "hb-machinery.hh"


/*
 * hb_ot_face_t
 */

/* Declare tables. */
namespace OT { struct glyf_accelerator_t; }
namespace OT { struct gvar_accelerator_t; }
namespace OT { struct kern; }
namespace OT { struct GDEF_accelerator_t; }
namespace OT { struct GSUB_accelerator_t; }
namespace OT { struct GPOS_accelerator_t; }
namespace AAT { struct morx; }
namespace AAT { struct mort; }
namespace AAT { struct kerx; }
namespace AAT { struct ankr; }
namespace AAT { struct trak; }
namespace AAT { struct lcar; }
namespace AAT { struct ltag; }
namespace AAT { struct feat; }
namespace OT { struct CBDT_accelerator_t; }
namespace OT { struct sbix_accelerator_t; }

struct hb_ot_face_t
{
  void init0 (hb_face_t *face);
  void fini ();

  enum order_t
  {
    ORDER_ZERO,

    ORDER_OT_glyf,
    ORDER_OT_gvar,
    ORDER_OT_kern,
    ORDER_OT_GDEF,
    ORDER_OT_GSUB,
    ORDER_OT_GPOS,
    ORDER_AAT_morx,
    ORDER_AAT_mort,
    ORDER_AAT_kerx,
    ORDER_AAT_ankr,
    ORDER_AAT_trak,
    ORDER_AAT_lcar,
    ORDER_AAT_ltag,
    ORDER_AAT_feat,
    ORDER_OT_CBDT,
    ORDER_OT_sbix,
  };

  hb_face_t *face;

  hb_face_lazy_loader_t<OT::glyf_accelerator_t, ORDER_OT_glyf> glyf;
  hb_face_lazy_loader_t<OT::gvar_accelerator_t, ORDER_OT_gvar> gvar;
  hb_table_lazy_loader_t<OT::kern, ORDER_OT_kern> kern;
  hb_face_lazy_loader_t<OT::GDEF_accelerator_t, ORDER_OT_GDEF> GDEF;
  hb_face_lazy_loader_t<OT::GSUB_accelerator_t, ORDER_OT_GSUB> GSUB;
  hb_face_lazy_loader_t<OT::GPOS_accelerator_t, ORDER_OT_GPOS> GPOS;
  hb_table_lazy_loader_t<AAT::morx, ORDER_AAT_morx> morx;
  hb_table_lazy_loader_t<AAT::mort, ORDER_AAT_mort> mort;
  hb_table_lazy_loader_t<AAT::kerx, ORDER_AAT_kerx> kerx;
  hb_table_lazy_loader_t<AAT::ankr, ORDER_AAT_ankr> ankr;
  hb_table_lazy_loader_t<AAT::trak, ORDER_AAT_trak> trak;
  hb_table_lazy_loader_t<AAT::lcar, ORDER_AAT_lcar> lcar;
  hb_table_lazy_loader_t<AAT::ltag, ORDER_AAT_ltag> ltag;
  hb_table_lazy_loader_t<AAT::feat, ORDER_AAT_feat> feat;
  hb_face_lazy_loader_t<OT::CBDT_accelerator_t, ORDER_OT_CBDT> CBDT;
  hb_face_lazy_loader_t<OT::sbix_accelerator_t, ORDER_OT_sbix> sbix;
};

