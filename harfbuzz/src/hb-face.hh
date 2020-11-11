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
 * Google Author(s): Behdad Esfahbod
 */

#ifndef RB_FACE_HH
#define RB_FACE_HH

#include "hb.hh"

namespace OT { struct kern; }
namespace AAT { struct morx; }
namespace AAT { struct mort; }
namespace AAT { struct kerx; }
namespace AAT { struct ankr; }
namespace AAT { struct trak; }
namespace AAT { struct feat; }

const OT::kern  *rb_face_get_kern_table(rb_face_t *face);
const AAT::morx *rb_face_get_morx_table(rb_face_t *face);
const AAT::mort *rb_face_get_mort_table(rb_face_t *face);
const AAT::kerx *rb_face_get_kerx_table(rb_face_t *face);
const AAT::ankr *rb_face_get_ankr_table(rb_face_t *face);
const AAT::trak *rb_face_get_trak_table(rb_face_t *face);
const AAT::feat *rb_face_get_feat_table(rb_face_t *face);

#endif /* RB_FACE_HH */
