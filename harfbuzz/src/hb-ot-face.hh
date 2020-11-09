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

#ifndef RB_OT_FACE_HH
#define RB_OT_FACE_HH

#include "hb.hh"

#include "hb-machinery.hh"

/*
 * rb_ot_face_t
 */

/* Declare tables. */
namespace OT { struct head; }
namespace OT { struct kern; }
namespace AAT { struct morx; }
namespace AAT { struct mort; }
namespace AAT { struct kerx; }
namespace AAT { struct ankr; }
namespace AAT { struct trak; }
namespace AAT { struct feat; }

struct rb_ot_face_t
{
    void init0(rb_face_t *face);
    void fini();

    enum order_t {
        ORDER_ZERO,
        ORDER_OT_head,
        ORDER_OT_kern,
        ORDER_AAT_morx,
        ORDER_AAT_mort,
        ORDER_AAT_kerx,
        ORDER_AAT_ankr,
        ORDER_AAT_trak,
        ORDER_AAT_feat,
    };

    rb_face_t *face;

    rb_table_lazy_loader_t<OT::head, ORDER_OT_head> head;
    rb_table_lazy_loader_t<OT::kern, ORDER_OT_kern> kern;

    rb_table_lazy_loader_t<AAT::morx, ORDER_AAT_morx> morx;
    rb_table_lazy_loader_t<AAT::mort, ORDER_AAT_mort> mort;
    rb_table_lazy_loader_t<AAT::kerx, ORDER_AAT_kerx> kerx;
    rb_table_lazy_loader_t<AAT::ankr, ORDER_AAT_ankr> ankr;
    rb_table_lazy_loader_t<AAT::trak, ORDER_AAT_trak> trak;
    rb_table_lazy_loader_t<AAT::feat, ORDER_AAT_feat> feat;
};

extern "C" {
RB_EXTERN const char *rb_face_get_table_data(const rb_face_t *face, rb_tag_t tag, unsigned int *len);
}

#endif /* RB_OT_FACE_HH */
