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

struct FakeTable {};

struct MarkGlyphSetsFormat1
{
    bool covers(unsigned int set_index, rb_codepoint_t glyph_id) const
    {
        return (this + coverage[set_index]).get_coverage(glyph_id) != NOT_COVERED;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return coverage.sanitize(c, this);
    }

protected:
    HBUINT16 format;                       /* Format identifier--format = 1 */
    ArrayOf<LOffsetTo<Coverage>> coverage; /* Array of long offsets to mark set
                                            * coverage tables */
public:
    DEFINE_SIZE_ARRAY(4, coverage);
};

struct MarkGlyphSets
{
    bool covers(unsigned int set_index, rb_codepoint_t glyph_id) const
    {
        switch (u.format) {
        case 1:
            return u.format1.covers(set_index, glyph_id);
        default:
            return false;
        }
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        if (!u.format.sanitize(c))
            return false;
        switch (u.format) {
        case 1:
            return u.format1.sanitize(c);
        default:
            return true;
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        MarkGlyphSetsFormat1 format1;
    } u;

public:
    DEFINE_SIZE_UNION(2, format);
};

/*
 * GDEF -- Glyph Definition
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gdef
 */

struct GDEF
{
    static constexpr rb_tag_t tableTag = RB_OT_TAG_GDEF;

    enum GlyphClasses { UnclassifiedGlyph = 0, BaseGlyph = 1, LigatureGlyph = 2, MarkGlyph = 3, ComponentGlyph = 4 };

    bool has_glyph_classes() const
    {
        return glyphClassDef != 0;
    }
    unsigned int get_glyph_class(rb_codepoint_t glyph) const
    {
        return (this + glyphClassDef).get_class(glyph);
    }

    unsigned int get_mark_attachment_type(rb_codepoint_t glyph) const
    {
        return (this + markAttachClassDef).get_class(glyph);
    }

    bool mark_set_covers(unsigned int set_index, rb_codepoint_t glyph_id) const
    {
        return version.to_int() >= 0x00010002u && (this + markGlyphSetsDef).covers(set_index, glyph_id);
    }

    const VariationStore &get_var_store() const
    {
        return version.to_int() >= 0x00010003u ? this + varStore : Null(VariationStore);
    }

    /* glyph_props is a 16-bit integer where the lower 8-bit have bits representing
     * glyph class and other bits, and high 8-bit the mark attachment type (if any).
     * Not to be confused with lookup_props which is very similar. */
    unsigned int get_glyph_props(rb_codepoint_t glyph) const
    {
        unsigned int klass = get_glyph_class(glyph);

        static_assert(((unsigned int)RB_OT_LAYOUT_GLYPH_PROPS_BASE_GLYPH == (unsigned int)LookupFlag::IgnoreBaseGlyphs),
                      "");
        static_assert(((unsigned int)RB_OT_LAYOUT_GLYPH_PROPS_LIGATURE == (unsigned int)LookupFlag::IgnoreLigatures),
                      "");
        static_assert(((unsigned int)RB_OT_LAYOUT_GLYPH_PROPS_MARK == (unsigned int)LookupFlag::IgnoreMarks), "");

        switch (klass) {
        default:
            return 0;
        case BaseGlyph:
            return RB_OT_LAYOUT_GLYPH_PROPS_BASE_GLYPH;
        case LigatureGlyph:
            return RB_OT_LAYOUT_GLYPH_PROPS_LIGATURE;
        case MarkGlyph:
            klass = get_mark_attachment_type(glyph);
            return RB_OT_LAYOUT_GLYPH_PROPS_MARK | (klass << 8);
        }
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
        return min_size + (version.to_int() >= 0x00010002u ? markGlyphSetsDef.static_size : 0) +
               (version.to_int() >= 0x00010003u ? varStore.static_size : 0);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return version.sanitize(c) && likely(version.major == 1) && glyphClassDef.sanitize(c, this) &&
               markAttachClassDef.sanitize(c, this) &&
               (version.to_int() < 0x00010002u || markGlyphSetsDef.sanitize(c, this)) &&
               (version.to_int() < 0x00010003u || varStore.sanitize(c, this));
    }

protected:
    FixedVersion<> version;                   /* Version of the GDEF table--currently
                                               * 0x00010003u */
    OffsetTo<ClassDef> glyphClassDef;         /* Offset to class definition table
                                               * for glyph type--from beginning of
                                               * GDEF header (may be Null) */
    OffsetTo<FakeTable> attachList;           /* Not used */
    OffsetTo<FakeTable> ligCaretList;         /* Not used */
    OffsetTo<ClassDef> markAttachClassDef;    /* Offset to class definition table for
                                               * mark attachment type--from beginning
                                               * of GDEF header (may be Null) */
    OffsetTo<MarkGlyphSets> markGlyphSetsDef; /* Offset to the table of mark set
                                               * definitions--from beginning of GDEF
                                               * header (may be NULL).  Introduced
                                               * in version 0x00010002. */
    LOffsetTo<VariationStore> varStore;       /* Offset to the table of Item Variation
                                               * Store--from beginning of GDEF
                                               * header (may be NULL).  Introduced
                                               * in version 0x00010003. */
public:
    DEFINE_SIZE_MIN(12);
};

struct GDEF_accelerator_t : GDEF::accelerator_t
{
};

} /* namespace OT */

#endif /* RB_OT_LAYOUT_GDEF_TABLE_HH */
