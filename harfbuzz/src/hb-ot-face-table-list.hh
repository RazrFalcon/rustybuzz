/*
 * Copyright © 2007,2008,2009  Red Hat, Inc.
 * Copyright © 2012,2013  Google, Inc.
 * Copyright © 2019, Facebook Inc.
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
 * Facebook Author(s): Behdad Esfahbod
 */

#ifndef HB_OT_FACE_TABLE_LIST_HH
#define HB_OT_FACE_TABLE_LIST_HH
#endif /* HB_OT_FACE_TABLE_LIST_HH */ /* Dummy header guards */

#ifndef HB_OT_ACCELERATOR
#define HB_OT_ACCELERATOR(Namespace, Type) HB_OT_TABLE (Namespace, Type)
#define _HB_OT_ACCELERATOR_UNDEF
#endif


/* This lists font tables that the hb_face_t will contain and lazily
 * load.  Don't add a table unless it's used though.  This is not
 * exactly free. */

/* v--- Add new tables in the right place here. */


/* OpenType fundamentals. */
HB_OT_TABLE (OT, head)
HB_OT_TABLE (OT, hhea)
HB_OT_ACCELERATOR (OT, hmtx)
HB_OT_TABLE (OT, OS2)

/* Vertical layout. */
HB_OT_TABLE (OT, vhea)
HB_OT_ACCELERATOR (OT, vmtx)

/* TrueType outlines. */
HB_OT_ACCELERATOR (OT, glyf)

/* CFF outlines. */
HB_OT_TABLE (OT, VORG)

/* OpenType variations. */
HB_OT_TABLE (OT, fvar)
HB_OT_TABLE (OT, avar)
HB_OT_ACCELERATOR (OT, gvar)
HB_OT_TABLE (OT, MVAR)

/* Legacy kern. */
#ifndef HB_NO_OT_KERN
HB_OT_TABLE (OT, kern)
#endif

/* OpenType shaping. */
#ifndef HB_NO_OT_LAYOUT
HB_OT_ACCELERATOR (OT, GDEF)
HB_OT_ACCELERATOR (OT, GSUB)
HB_OT_ACCELERATOR (OT, GPOS)
#endif

/* AAT shaping. */
#ifndef HB_NO_AAT
HB_OT_TABLE (AAT, morx)
HB_OT_TABLE (AAT, mort)
HB_OT_TABLE (AAT, kerx)
HB_OT_TABLE (AAT, ankr)
HB_OT_TABLE (AAT, trak)
HB_OT_TABLE (AAT, lcar)
HB_OT_TABLE (AAT, ltag)
HB_OT_TABLE (AAT, feat)
#endif

/* OpenType color fonts. */
HB_OT_ACCELERATOR (OT, CBDT)
HB_OT_ACCELERATOR (OT, sbix)

#ifdef _HB_OT_ACCELERATOR_UNDEF
#undef HB_OT_ACCELERATOR
#endif
