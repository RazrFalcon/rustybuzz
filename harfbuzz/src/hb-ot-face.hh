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
#define RB_OT_TABLE(Namespace, Type)                                                                                   \
    namespace Namespace {                                                                                              \
    struct Type;                                                                                                       \
    }
#define RB_OT_ACCELERATOR(Namespace, Type) RB_OT_TABLE(Namespace, Type##_accelerator_t)
#include "hb-ot-face-table-list.hh"
#undef RB_OT_ACCELERATOR
#undef RB_OT_TABLE

struct rb_ot_face_t
{
    RB_INTERNAL void init0(rb_face_t *face);
    RB_INTERNAL void fini();

#define RB_OT_TABLE_ORDER(Namespace, Type) RB_PASTE(ORDER_, RB_PASTE(Namespace, RB_PASTE(_, Type)))
    enum order_t {
        ORDER_ZERO,
#define RB_OT_TABLE(Namespace, Type) RB_OT_TABLE_ORDER(Namespace, Type),
#include "hb-ot-face-table-list.hh"
#undef RB_OT_TABLE
    };

    rb_face_t *face; /* MUST be JUST before the lazy loaders. */
#define RB_OT_TABLE(Namespace, Type) rb_table_lazy_loader_t<Namespace::Type, RB_OT_TABLE_ORDER(Namespace, Type)> Type;
#define RB_OT_ACCELERATOR(Namespace, Type)                                                                             \
    rb_face_lazy_loader_t<Namespace::Type##_accelerator_t, RB_OT_TABLE_ORDER(Namespace, Type)> Type;
#include "hb-ot-face-table-list.hh"
#undef RB_OT_ACCELERATOR
#undef RB_OT_TABLE
};

#endif /* RB_OT_FACE_HH */
