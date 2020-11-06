/*
 * Copyright © 2007,2008,2009  Red Hat, Inc.
 * Copyright © 2010,2011,2012  Google, Inc.
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

#ifndef RB_OT_LAYOUT_GDEF_TABLE_HH
#define RB_OT_LAYOUT_GDEF_TABLE_HH

#include "hb-ot-layout-common.hh"

namespace OT {

/*
 * GDEF -- Glyph Definition
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gdef
 */

struct FakeTable {};

struct GDEF
{
    static constexpr rb_tag_t tableTag = RB_OT_TAG_GDEF;

    bool has_glyph_classes() const
    {
        return glyphClassDef != 0;
    }

    RB_INTERNAL bool is_blocklisted(rb_blob_t *blob, rb_face_t *face) const;

    struct accelerator_t
    {
        void init(rb_face_t *face)
        {
            this->table = rb_sanitize_context_t().reference_table<GDEF>(face);
            if (unlikely(this->table->is_blocklisted(this->table.get_blob(), face))) {
                rb_blob_destroy(this->table.get_blob());
                this->table = rb_blob_get_empty();
            }
        }

        void fini()
        {
            this->table.destroy();
        }

        rb_blob_ptr_t<GDEF> table;
    };

    unsigned int get_size() const
    {
        return min_size;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return version.sanitize(c) && likely(version.major == 1);
    }

protected:
    FixedVersion<> version;                   /* Version of the GDEF table--currently
                                               * 0x00010003u */
    OffsetTo<FakeTable> glyphClassDef;         /* Offset to class definition table
                                               * for glyph type--from beginning of
                                               * GDEF header (may be Null) */
public:
    DEFINE_SIZE_MIN(4);
};

struct GDEF_accelerator_t : GDEF::accelerator_t
{
};

} /* namespace OT */

#endif /* RB_OT_LAYOUT_GDEF_TABLE_HH */
