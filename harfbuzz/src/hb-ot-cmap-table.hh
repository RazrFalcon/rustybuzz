/*
 * Copyright © 2014  Google, Inc.
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

#ifndef HB_OT_CMAP_TABLE_HH
#define HB_OT_CMAP_TABLE_HH

#include "hb-open-type.hh"
#include "hb-set.hh"
#include "hb-map.hh"

/*
 * cmap -- Character to Glyph Index Mapping
 * https://docs.microsoft.com/en-us/typography/opentype/spec/cmap
 */
#define HB_OT_TAG_cmap HB_TAG('c', 'm', 'a', 'p')

namespace OT {

struct CmapSubtableFormat0
{
    bool get_glyph(hb_codepoint_t codepoint, hb_codepoint_t *glyph) const
    {
        hb_codepoint_t gid = codepoint < 256 ? glyphIdArray[codepoint] : 0;
        if (!gid)
            return false;
        *glyph = gid;
        return true;
    }
    void collect_unicodes(hb_set_t *out) const
    {
        for (unsigned int i = 0; i < 256; i++)
            if (glyphIdArray[i])
                out->add(i);
    }

    void collect_mapping(hb_set_t *unicodes, /* OUT */
                         hb_map_t *mapping /* OUT */) const
    {
        for (unsigned i = 0; i < 256; i++)
            if (glyphIdArray[i]) {
                hb_codepoint_t glyph = glyphIdArray[i];
                unicodes->add(i);
                mapping->set(i, glyph);
            }
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this));
    }

protected:
    HBUINT16 format;           /* Format number is set to 0. */
    HBUINT16 length;           /* Byte length of this subtable. */
    HBUINT16 language;         /* Ignore. */
    HBUINT8 glyphIdArray[256]; /* An array that maps character
                                * code to glyph index values. */
public:
    DEFINE_SIZE_STATIC(6 + 256);
};

struct CmapSubtableFormat4
{
    struct accelerator_t
    {
        accelerator_t() {}
        accelerator_t(const CmapSubtableFormat4 *subtable)
        {
            init(subtable);
        }
        ~accelerator_t()
        {
            fini();
        }

        void init(const CmapSubtableFormat4 *subtable)
        {
            segCount = subtable->segCountX2 / 2;
            endCount = subtable->values.arrayZ;
            startCount = endCount + segCount + 1;
            idDelta = startCount + segCount;
            idRangeOffset = idDelta + segCount;
            glyphIdArray = idRangeOffset + segCount;
            glyphIdArrayLength = (subtable->length - 16 - 8 * segCount) / 2;
        }
        void fini() {}

        bool get_glyph(hb_codepoint_t codepoint, hb_codepoint_t *glyph) const
        {
            struct CustomRange
            {
                int cmp(hb_codepoint_t k, unsigned distance) const
                {
                    if (k > last)
                        return +1;
                    if (k < (&last)[distance])
                        return -1;
                    return 0;
                }
                HBUINT16 last;
            };

            const HBUINT16 *found = hb_bsearch(codepoint,
                                               this->endCount,
                                               this->segCount,
                                               2,
                                               _hb_cmp_method<hb_codepoint_t, CustomRange, unsigned>,
                                               this->segCount + 1);
            if (!found)
                return false;
            unsigned int i = found - endCount;

            hb_codepoint_t gid;
            unsigned int rangeOffset = this->idRangeOffset[i];
            if (rangeOffset == 0)
                gid = codepoint + this->idDelta[i];
            else {
                /* Somebody has been smoking... */
                unsigned int index = rangeOffset / 2 + (codepoint - this->startCount[i]) + i - this->segCount;
                if (unlikely(index >= this->glyphIdArrayLength))
                    return false;
                gid = this->glyphIdArray[index];
                if (unlikely(!gid))
                    return false;
                gid += this->idDelta[i];
            }
            gid &= 0xFFFFu;
            if (!gid)
                return false;
            *glyph = gid;
            return true;
        }

        HB_INTERNAL static bool get_glyph_func(const void *obj, hb_codepoint_t codepoint, hb_codepoint_t *glyph)
        {
            return ((const accelerator_t *)obj)->get_glyph(codepoint, glyph);
        }

        void collect_unicodes(hb_set_t *out) const
        {
            unsigned int count = this->segCount;
            if (count && this->startCount[count - 1] == 0xFFFFu)
                count--; /* Skip sentinel segment. */
            for (unsigned int i = 0; i < count; i++) {
                hb_codepoint_t start = this->startCount[i];
                hb_codepoint_t end = this->endCount[i];
                unsigned int rangeOffset = this->idRangeOffset[i];
                if (rangeOffset == 0) {
                    for (hb_codepoint_t codepoint = start; codepoint <= end; codepoint++) {
                        hb_codepoint_t gid = (codepoint + this->idDelta[i]) & 0xFFFFu;
                        if (unlikely(!gid))
                            continue;
                        out->add(codepoint);
                    }
                } else {
                    for (hb_codepoint_t codepoint = start; codepoint <= end; codepoint++) {
                        unsigned int index = rangeOffset / 2 + (codepoint - this->startCount[i]) + i - this->segCount;
                        if (unlikely(index >= this->glyphIdArrayLength))
                            break;
                        hb_codepoint_t gid = this->glyphIdArray[index];
                        if (unlikely(!gid))
                            continue;
                        out->add(codepoint);
                    }
                }
            }
        }

        void collect_mapping(hb_set_t *unicodes, /* OUT */
                             hb_map_t *mapping /* OUT */) const
        {
            unsigned count = this->segCount;
            if (count && this->startCount[count - 1] == 0xFFFFu)
                count--; /* Skip sentinel segment. */
            for (unsigned i = 0; i < count; i++) {
                hb_codepoint_t start = this->startCount[i];
                hb_codepoint_t end = this->endCount[i];
                unsigned rangeOffset = this->idRangeOffset[i];
                if (rangeOffset == 0) {
                    for (hb_codepoint_t codepoint = start; codepoint <= end; codepoint++) {
                        hb_codepoint_t gid = (codepoint + this->idDelta[i]) & 0xFFFFu;
                        if (unlikely(!gid))
                            continue;
                        unicodes->add(codepoint);
                        mapping->set(codepoint, gid);
                    }
                } else {
                    for (hb_codepoint_t codepoint = start; codepoint <= end; codepoint++) {
                        unsigned index = rangeOffset / 2 + (codepoint - this->startCount[i]) + i - this->segCount;
                        if (unlikely(index >= this->glyphIdArrayLength))
                            break;
                        hb_codepoint_t gid = this->glyphIdArray[index];
                        if (unlikely(!gid))
                            continue;
                        unicodes->add(codepoint);
                        mapping->set(codepoint, gid);
                    }
                }
            }
        }

        const HBUINT16 *endCount;
        const HBUINT16 *startCount;
        const HBUINT16 *idDelta;
        const HBUINT16 *idRangeOffset;
        const HBUINT16 *glyphIdArray;
        unsigned int segCount;
        unsigned int glyphIdArrayLength;
    };

    bool get_glyph(hb_codepoint_t codepoint, hb_codepoint_t *glyph) const
    {
        accelerator_t accel(this);
        return accel.get_glyph_func(&accel, codepoint, glyph);
    }
    void collect_unicodes(hb_set_t *out) const
    {
        accelerator_t accel(this);
        accel.collect_unicodes(out);
    }

    void collect_mapping(hb_set_t *unicodes, /* OUT */
                         hb_map_t *mapping /* OUT */) const
    {
        accelerator_t accel(this);
        accel.collect_mapping(unicodes, mapping);
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        if (unlikely(!c->check_struct(this)))
            return_trace(false);

        if (unlikely(!c->check_range(this, length))) {
            /* Some broken fonts have too long of a "length" value.
             * If that is the case, just change the value to truncate
             * the subtable at the end of the blob. */
            uint16_t new_length = (uint16_t)hb_min((uintptr_t)65535, (uintptr_t)(c->end - (char *)this));
            if (!c->try_set(&length, new_length))
                return_trace(false);
        }

        return_trace(16 + 4 * (unsigned int)segCountX2 <= length);
    }

protected:
    HBUINT16 format;        /* Format number is set to 4. */
    HBUINT16 length;        /* This is the length in bytes of the
                             * subtable. */
    HBUINT16 language;      /* Ignore. */
    HBUINT16 segCountX2;    /* 2 x segCount. */
    HBUINT16 searchRange;   /* 2 * (2**floor(log2(segCount))) */
    HBUINT16 entrySelector; /* log2(searchRange/2) */
    HBUINT16 rangeShift;    /* 2 x segCount - searchRange */

    UnsizedArrayOf<HBUINT16> values;
#if 0
  HBUINT16	endCount[segCount];	/* End characterCode for each segment,
                     * last=0xFFFFu. */
  HBUINT16	reservedPad;		/* Set to 0. */
  HBUINT16	startCount[segCount];	/* Start character code for each segment. */
  HBINT16		idDelta[segCount];	/* Delta for all character codes in segment. */
  HBUINT16	idRangeOffset[segCount];/* Offsets into glyphIdArray or 0 */
  UnsizedArrayOf<HBUINT16>
        glyphIdArray;	/* Glyph index array (arbitrary length) */
#endif

public:
    DEFINE_SIZE_ARRAY(14, values);
};

struct CmapSubtableLongGroup
{
    friend struct CmapSubtableFormat12;
    friend struct CmapSubtableFormat13;
    template <typename U> friend struct CmapSubtableLongSegmented;
    friend struct cmap;

    int cmp(hb_codepoint_t codepoint) const
    {
        if (codepoint < startCharCode)
            return -1;
        if (codepoint > endCharCode)
            return +1;
        return 0;
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this));
    }

private:
    HBUINT32 startCharCode; /* First character code in this group. */
    HBUINT32 endCharCode;   /* Last character code in this group. */
    HBUINT32 glyphID;       /* Glyph index; interpretation depends on
                             * subtable format. */
public:
    DEFINE_SIZE_STATIC(12);
};
DECLARE_NULL_NAMESPACE_BYTES(OT, CmapSubtableLongGroup);

template <typename UINT> struct CmapSubtableTrimmed
{
    bool get_glyph(hb_codepoint_t codepoint, hb_codepoint_t *glyph) const
    {
        /* Rely on our implicit array bound-checking. */
        hb_codepoint_t gid = glyphIdArray[codepoint - startCharCode];
        if (!gid)
            return false;
        *glyph = gid;
        return true;
    }
    void collect_unicodes(hb_set_t *out) const
    {
        hb_codepoint_t start = startCharCode;
        unsigned int count = glyphIdArray.len;
        for (unsigned int i = 0; i < count; i++)
            if (glyphIdArray[i])
                out->add(start + i);
    }

    void collect_mapping(hb_set_t *unicodes, /* OUT */
                         hb_map_t *mapping /* OUT */) const
    {
        hb_codepoint_t start_cp = startCharCode;
        unsigned count = glyphIdArray.len;
        for (unsigned i = 0; i < count; i++)
            if (glyphIdArray[i]) {
                hb_codepoint_t unicode = start_cp + i;
                hb_codepoint_t glyphid = glyphIdArray[i];
                unicodes->add(unicode);
                mapping->set(unicode, glyphid);
            }
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this) && glyphIdArray.sanitize(c));
    }

protected:
    UINT formatReserved;                   /* Subtable format and (maybe) padding. */
    UINT length;                           /* Byte length of this subtable. */
    UINT language;                         /* Ignore. */
    UINT startCharCode;                    /* First character code covered. */
    ArrayOf<HBGlyphID, UINT> glyphIdArray; /* Array of glyph index values for character
                                            * codes in the range. */
public:
    DEFINE_SIZE_ARRAY(5 * sizeof(UINT), glyphIdArray);
};

struct CmapSubtableFormat6 : CmapSubtableTrimmed<HBUINT16>
{
};
struct CmapSubtableFormat10 : CmapSubtableTrimmed<HBUINT32>
{
};

template <typename T> struct CmapSubtableLongSegmented
{
    friend struct cmap;

    bool get_glyph(hb_codepoint_t codepoint, hb_codepoint_t *glyph) const
    {
        hb_codepoint_t gid = T::group_get_glyph(groups.bsearch(codepoint), codepoint);
        if (!gid)
            return false;
        *glyph = gid;
        return true;
    }

    void collect_unicodes(hb_set_t *out, unsigned int num_glyphs) const
    {
        for (unsigned int i = 0; i < this->groups.len; i++) {
            hb_codepoint_t start = this->groups[i].startCharCode;
            hb_codepoint_t end = hb_min((hb_codepoint_t)this->groups[i].endCharCode, (hb_codepoint_t)HB_UNICODE_MAX);
            hb_codepoint_t gid = this->groups[i].glyphID;
            if (!gid) {
                /* Intention is: if (hb_is_same (T, CmapSubtableFormat13)) continue; */
                if (!T::group_get_glyph(this->groups[i], end))
                    continue;
                start++;
                gid++;
            }
            if (unlikely((unsigned int)gid >= num_glyphs))
                continue;
            if (unlikely((unsigned int)(gid + end - start) >= num_glyphs))
                end = start + (hb_codepoint_t)num_glyphs - gid;

            out->add_range(start, end);
        }
    }

    void collect_mapping(hb_set_t *unicodes, /* OUT */
                         hb_map_t *mapping,  /* OUT */
                         unsigned num_glyphs) const
    {
        for (unsigned i = 0; i < this->groups.len; i++) {
            hb_codepoint_t start = this->groups[i].startCharCode;
            hb_codepoint_t end = hb_min((hb_codepoint_t)this->groups[i].endCharCode, (hb_codepoint_t)HB_UNICODE_MAX);
            hb_codepoint_t gid = this->groups[i].glyphID;
            if (!gid) {
                /* Intention is: if (hb_is_same (T, CmapSubtableFormat13)) continue; */
                if (!T::group_get_glyph(this->groups[i], end))
                    continue;
                start++;
                gid++;
            }
            if (unlikely((unsigned int)gid >= num_glyphs))
                continue;
            if (unlikely((unsigned int)(gid + end - start) >= num_glyphs))
                end = start + (hb_codepoint_t)num_glyphs - gid;

            for (unsigned cp = start; cp <= end; cp++) {
                unicodes->add(cp);
                mapping->set(cp, gid);
                gid++;
            }
        }
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this) && groups.sanitize(c));
    }

protected:
    HBUINT16 format;                                       /* Subtable format; set to 12. */
    HBUINT16 reserved;                                     /* Reserved; set to 0. */
    HBUINT32 length;                                       /* Byte length of this subtable. */
    HBUINT32 language;                                     /* Ignore. */
    SortedArrayOf<CmapSubtableLongGroup, HBUINT32> groups; /* Groupings. */
public:
    DEFINE_SIZE_ARRAY(16, groups);
};

struct CmapSubtableFormat12 : CmapSubtableLongSegmented<CmapSubtableFormat12>
{
    static hb_codepoint_t group_get_glyph(const CmapSubtableLongGroup &group, hb_codepoint_t u)
    {
        return likely(group.startCharCode <= group.endCharCode) ? group.glyphID + (u - group.startCharCode) : 0;
    }

    static size_t get_sub_table_size(const hb_sorted_vector_t<CmapSubtableLongGroup> &groups_data)
    {
        return 16 + 12 * groups_data.length;
    }

private:
    static bool _is_gid_consecutive(hb_codepoint_t endCharCode,
                                    hb_codepoint_t startCharCode,
                                    hb_codepoint_t glyphID,
                                    hb_codepoint_t cp,
                                    hb_codepoint_t new_gid)
    {
        return (cp - 1 == endCharCode) && new_gid == glyphID + (cp - startCharCode);
    }
};

struct CmapSubtableFormat13 : CmapSubtableLongSegmented<CmapSubtableFormat13>
{
    static hb_codepoint_t group_get_glyph(const CmapSubtableLongGroup &group, hb_codepoint_t u HB_UNUSED)
    {
        return group.glyphID;
    }
};

typedef enum { GLYPH_VARIANT_NOT_FOUND = 0, GLYPH_VARIANT_FOUND = 1, GLYPH_VARIANT_USE_DEFAULT = 2 } glyph_variant_t;

struct UnicodeValueRange
{
    int cmp(const hb_codepoint_t &codepoint) const
    {
        if (codepoint < startUnicodeValue)
            return -1;
        if (codepoint > startUnicodeValue + additionalCount)
            return +1;
        return 0;
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this));
    }

    HBUINT24 startUnicodeValue; /* First value in this range. */
    HBUINT8 additionalCount;    /* Number of additional values in this
                                 * range. */
public:
    DEFINE_SIZE_STATIC(4);
};

struct DefaultUVS : SortedArrayOf<UnicodeValueRange, HBUINT32>
{
    void collect_unicodes(hb_set_t *out) const
    {
        unsigned int count = len;
        for (unsigned int i = 0; i < count; i++) {
            hb_codepoint_t first = arrayZ[i].startUnicodeValue;
            hb_codepoint_t last =
                hb_min((hb_codepoint_t)(first + arrayZ[i].additionalCount), (hb_codepoint_t)HB_UNICODE_MAX);
            out->add_range(first, last);
        }
    }

public:
    DEFINE_SIZE_ARRAY(4, *this);
};

struct UVSMapping
{
    int cmp(const hb_codepoint_t &codepoint) const
    {
        return unicodeValue.cmp(codepoint);
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this));
    }

    HBUINT24 unicodeValue; /* Base Unicode value of the UVS */
    HBGlyphID glyphID;     /* Glyph ID of the UVS */
public:
    DEFINE_SIZE_STATIC(5);
};

struct NonDefaultUVS : SortedArrayOf<UVSMapping, HBUINT32>
{
    void collect_unicodes(hb_set_t *out) const
    {
        unsigned int count = len;
        for (unsigned int i = 0; i < count; i++)
            out->add(arrayZ[i].unicodeValue);
    }

    void collect_mapping(hb_set_t *unicodes, /* OUT */
                         hb_map_t *mapping /* OUT */) const
    {
        unsigned count = len;
        for (unsigned i = 0; i < count; i++) {
            hb_codepoint_t unicode = arrayZ[i].unicodeValue;
            hb_codepoint_t glyphid = arrayZ[i].glyphID;
            unicodes->add(unicode);
            mapping->set(unicode, glyphid);
        }
    }

    void closure_glyphs(const hb_set_t *unicodes, hb_set_t *glyphset) const
    {
        +as_array() | hb_filter(unicodes, &UVSMapping::unicodeValue) | hb_map(&UVSMapping::glyphID) | hb_sink(glyphset);
    }

public:
    DEFINE_SIZE_ARRAY(4, *this);
};

struct VariationSelectorRecord
{
    glyph_variant_t get_glyph(hb_codepoint_t codepoint, hb_codepoint_t *glyph, const void *base) const
    {
        if ((base + defaultUVS).bfind(codepoint))
            return GLYPH_VARIANT_USE_DEFAULT;
        const UVSMapping &nonDefault = (base + nonDefaultUVS).bsearch(codepoint);
        if (nonDefault.glyphID) {
            *glyph = nonDefault.glyphID;
            return GLYPH_VARIANT_FOUND;
        }
        return GLYPH_VARIANT_NOT_FOUND;
    }

    VariationSelectorRecord(const VariationSelectorRecord &other)
    {
        *this = other;
    }

    void operator=(const VariationSelectorRecord &other)
    {
        varSelector = other.varSelector;
        HBUINT32 offset = other.defaultUVS;
        defaultUVS = offset;
        offset = other.nonDefaultUVS;
        nonDefaultUVS = offset;
    }

    void collect_unicodes(hb_set_t *out, const void *base) const
    {
        (base + defaultUVS).collect_unicodes(out);
        (base + nonDefaultUVS).collect_unicodes(out);
    }

    void collect_mapping(const void *base,
                         hb_set_t *unicodes, /* OUT */
                         hb_map_t *mapping /* OUT */) const
    {
        (base + defaultUVS).collect_unicodes(unicodes);
        (base + nonDefaultUVS).collect_mapping(unicodes, mapping);
    }

    int cmp(const hb_codepoint_t &variation_selector) const
    {
        return varSelector.cmp(variation_selector);
    }

    bool sanitize(hb_sanitize_context_t *c, const void *base) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this) && defaultUVS.sanitize(c, base) && nonDefaultUVS.sanitize(c, base));
    }

    HBUINT24 varSelector;                   /* Variation selector. */
    LOffsetTo<DefaultUVS> defaultUVS;       /* Offset to Default UVS Table.  May be 0. */
    LOffsetTo<NonDefaultUVS> nonDefaultUVS; /* Offset to Non-Default UVS Table.  May be 0. */
public:
    DEFINE_SIZE_STATIC(11);
};

struct CmapSubtableFormat14
{
    glyph_variant_t
    get_glyph_variant(hb_codepoint_t codepoint, hb_codepoint_t variation_selector, hb_codepoint_t *glyph) const
    {
        return record.bsearch(variation_selector).get_glyph(codepoint, glyph, this);
    }

    void collect_variation_selectors(hb_set_t *out) const
    {
        unsigned int count = record.len;
        for (unsigned int i = 0; i < count; i++)
            out->add(record.arrayZ[i].varSelector);
    }
    void collect_variation_unicodes(hb_codepoint_t variation_selector, hb_set_t *out) const
    {
        record.bsearch(variation_selector).collect_unicodes(out, this);
    }

    void _reverse_variation_records()
    {
        record.as_array().reverse();
    }

    void closure_glyphs(const hb_set_t *unicodes, hb_set_t *glyphset) const
    {
        +hb_iter(record) | hb_filter(hb_bool, &VariationSelectorRecord::nonDefaultUVS) |
            hb_map(&VariationSelectorRecord::nonDefaultUVS) | hb_map(hb_add(this)) |
            hb_apply([=](const NonDefaultUVS &_) { _.closure_glyphs(unicodes, glyphset); });
    }

    void collect_unicodes(hb_set_t *out) const
    {
        for (const VariationSelectorRecord &_ : record)
            _.collect_unicodes(out, this);
    }

    void collect_mapping(hb_set_t *unicodes, /* OUT */
                         hb_map_t *mapping /* OUT */) const
    {
        for (const VariationSelectorRecord &_ : record)
            _.collect_mapping(this, unicodes, mapping);
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this) && record.sanitize(c, this));
    }

protected:
    HBUINT16 format;                                         /* Format number is set to 14. */
    HBUINT32 length;                                         /* Byte length of this subtable. */
    SortedArrayOf<VariationSelectorRecord, HBUINT32> record; /* Variation selector records; sorted
                                                              * in increasing order of `varSelector'. */
public:
    DEFINE_SIZE_ARRAY(10, record);
};

struct CmapSubtable
{
    /* Note: We intentionally do NOT implement subtable formats 2 and 8. */

    bool get_glyph(hb_codepoint_t codepoint, hb_codepoint_t *glyph) const
    {
        switch (u.format) {
        case 0:
            return u.format0.get_glyph(codepoint, glyph);
        case 4:
            return u.format4.get_glyph(codepoint, glyph);
        case 6:
            return u.format6.get_glyph(codepoint, glyph);
        case 10:
            return u.format10.get_glyph(codepoint, glyph);
        case 12:
            return u.format12.get_glyph(codepoint, glyph);
        case 13:
            return u.format13.get_glyph(codepoint, glyph);
        case 14:
        default:
            return false;
        }
    }
    void collect_unicodes(hb_set_t *out, unsigned int num_glyphs = UINT_MAX) const
    {
        switch (u.format) {
        case 0:
            u.format0.collect_unicodes(out);
            return;
        case 4:
            u.format4.collect_unicodes(out);
            return;
        case 6:
            u.format6.collect_unicodes(out);
            return;
        case 10:
            u.format10.collect_unicodes(out);
            return;
        case 12:
            u.format12.collect_unicodes(out, num_glyphs);
            return;
        case 13:
            u.format13.collect_unicodes(out, num_glyphs);
            return;
        case 14:
        default:
            return;
        }
    }

    void collect_mapping(hb_set_t *unicodes, /* OUT */
                         hb_map_t *mapping,  /* OUT */
                         unsigned num_glyphs = UINT_MAX) const
    {
        switch (u.format) {
        case 0:
            u.format0.collect_mapping(unicodes, mapping);
            return;
        case 4:
            u.format4.collect_mapping(unicodes, mapping);
            return;
        case 6:
            u.format6.collect_mapping(unicodes, mapping);
            return;
        case 10:
            u.format10.collect_mapping(unicodes, mapping);
            return;
        case 12:
            u.format12.collect_mapping(unicodes, mapping, num_glyphs);
            return;
        case 13:
            u.format13.collect_mapping(unicodes, mapping, num_glyphs);
            return;
        case 14:
        default:
            return;
        }
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        if (!u.format.sanitize(c))
            return_trace(false);
        switch (u.format) {
        case 0:
            return_trace(u.format0.sanitize(c));
        case 4:
            return_trace(u.format4.sanitize(c));
        case 6:
            return_trace(u.format6.sanitize(c));
        case 10:
            return_trace(u.format10.sanitize(c));
        case 12:
            return_trace(u.format12.sanitize(c));
        case 13:
            return_trace(u.format13.sanitize(c));
        case 14:
            return_trace(u.format14.sanitize(c));
        default:
            return_trace(true);
        }
    }

public:
    union {
        HBUINT16 format; /* Format identifier */
        CmapSubtableFormat0 format0;
        CmapSubtableFormat4 format4;
        CmapSubtableFormat6 format6;
        CmapSubtableFormat10 format10;
        CmapSubtableFormat12 format12;
        CmapSubtableFormat13 format13;
        CmapSubtableFormat14 format14;
    } u;

public:
    DEFINE_SIZE_UNION(2, format);
};

struct EncodingRecord
{
    int cmp(const EncodingRecord &other) const
    {
        int ret;
        ret = platformID.cmp(other.platformID);
        if (ret)
            return ret;
        ret = encodingID.cmp(other.encodingID);
        if (ret)
            return ret;
        return 0;
    }

    bool sanitize(hb_sanitize_context_t *c, const void *base) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this) && subtable.sanitize(c, base));
    }

    HBUINT16 platformID;              /* Platform ID. */
    HBUINT16 encodingID;              /* Platform-specific encoding ID. */
    LOffsetTo<CmapSubtable> subtable; /* Byte offset from beginning of table to the subtable for this encoding. */
public:
    DEFINE_SIZE_STATIC(8);
};

struct cmap
{
    static constexpr hb_tag_t tableTag = HB_OT_TAG_cmap;

    void closure_glyphs(const hb_set_t *unicodes, hb_set_t *glyphset) const
    {
        +hb_iter(encodingRecord) | hb_map(&EncodingRecord::subtable) | hb_map(hb_add(this)) |
            hb_filter([&](const CmapSubtable &_) { return _.u.format == 14; }) |
            hb_apply([=](const CmapSubtable &_) { _.u.format14.closure_glyphs(unicodes, glyphset); });
    }

    const CmapSubtable *find_best_subtable(bool *symbol = nullptr) const
    {
        if (symbol)
            *symbol = false;

        const CmapSubtable *subtable;

        /* Symbol subtable.
         * Prefer symbol if available.
         * https://github.com/harfbuzz/harfbuzz/issues/1918 */
        if ((subtable = this->find_subtable(3, 0))) {
            if (symbol)
                *symbol = true;
            return subtable;
        }

        /* 32-bit subtables. */
        if ((subtable = this->find_subtable(3, 10)))
            return subtable;
        if ((subtable = this->find_subtable(0, 6)))
            return subtable;
        if ((subtable = this->find_subtable(0, 4)))
            return subtable;

        /* 16-bit subtables. */
        if ((subtable = this->find_subtable(3, 1)))
            return subtable;
        if ((subtable = this->find_subtable(0, 3)))
            return subtable;
        if ((subtable = this->find_subtable(0, 2)))
            return subtable;
        if ((subtable = this->find_subtable(0, 1)))
            return subtable;
        if ((subtable = this->find_subtable(0, 0)))
            return subtable;

        /* Meh. */
        return &Null(CmapSubtable);
    }

    struct accelerator_t
    {
        void init(hb_face_t *face)
        {
            this->table = hb_sanitize_context_t().reference_table<cmap>(face);
            bool symbol;
            this->subtable = table->find_best_subtable(&symbol);
            this->subtable_uvs = &Null(CmapSubtableFormat14);
            {
                const CmapSubtable *st = table->find_subtable(0, 5);
                if (st && st->u.format == 14)
                    subtable_uvs = &st->u.format14;
            }

            this->get_glyph_data = subtable;
            if (unlikely(symbol))
                this->get_glyph_funcZ = get_glyph_from_symbol<CmapSubtable>;
            else {
                switch (subtable->u.format) {
                /* Accelerate format 4 and format 12. */
                default:
                    this->get_glyph_funcZ = get_glyph_from<CmapSubtable>;
                    break;
                case 12:
                    this->get_glyph_funcZ = get_glyph_from<CmapSubtableFormat12>;
                    break;
                case 4: {
                    this->format4_accel.init(&subtable->u.format4);
                    this->get_glyph_data = &this->format4_accel;
                    this->get_glyph_funcZ = this->format4_accel.get_glyph_func;
                    break;
                }
                }
            }
        }

        void fini()
        {
            this->table.destroy();
        }

        bool get_nominal_glyph(hb_codepoint_t unicode, hb_codepoint_t *glyph) const
        {
            if (unlikely(!this->get_glyph_funcZ))
                return false;
            return this->get_glyph_funcZ(this->get_glyph_data, unicode, glyph);
        }
        unsigned int get_nominal_glyphs(unsigned int count,
                                        const hb_codepoint_t *first_unicode,
                                        unsigned int unicode_stride,
                                        hb_codepoint_t *first_glyph,
                                        unsigned int glyph_stride) const
        {
            if (unlikely(!this->get_glyph_funcZ))
                return 0;

            hb_cmap_get_glyph_func_t get_glyph_funcZ = this->get_glyph_funcZ;
            const void *get_glyph_data = this->get_glyph_data;

            unsigned int done;
            for (done = 0; done < count && get_glyph_funcZ(get_glyph_data, *first_unicode, first_glyph); done++) {
                first_unicode = &StructAtOffsetUnaligned<hb_codepoint_t>(first_unicode, unicode_stride);
                first_glyph = &StructAtOffsetUnaligned<hb_codepoint_t>(first_glyph, glyph_stride);
            }
            return done;
        }

        bool get_variation_glyph(hb_codepoint_t unicode, hb_codepoint_t variation_selector, hb_codepoint_t *glyph) const
        {
            switch (this->subtable_uvs->get_glyph_variant(unicode, variation_selector, glyph)) {
            case GLYPH_VARIANT_NOT_FOUND:
                return false;
            case GLYPH_VARIANT_FOUND:
                return true;
            case GLYPH_VARIANT_USE_DEFAULT:
                break;
            }

            return get_nominal_glyph(unicode, glyph);
        }

        void collect_unicodes(hb_set_t *out, unsigned int num_glyphs) const
        {
            subtable->collect_unicodes(out, num_glyphs);
        }
        void collect_mapping(hb_set_t *unicodes, hb_map_t *mapping, unsigned num_glyphs = UINT_MAX) const
        {
            subtable->collect_mapping(unicodes, mapping, num_glyphs);
        }
        void collect_variation_selectors(hb_set_t *out) const
        {
            subtable_uvs->collect_variation_selectors(out);
        }
        void collect_variation_unicodes(hb_codepoint_t variation_selector, hb_set_t *out) const
        {
            subtable_uvs->collect_variation_unicodes(variation_selector, out);
        }

    protected:
        typedef bool (*hb_cmap_get_glyph_func_t)(const void *obj, hb_codepoint_t codepoint, hb_codepoint_t *glyph);

        template <typename Type>
        HB_INTERNAL static bool get_glyph_from(const void *obj, hb_codepoint_t codepoint, hb_codepoint_t *glyph)
        {
            const Type *typed_obj = (const Type *)obj;
            return typed_obj->get_glyph(codepoint, glyph);
        }

        template <typename Type>
        HB_INTERNAL static bool get_glyph_from_symbol(const void *obj, hb_codepoint_t codepoint, hb_codepoint_t *glyph)
        {
            const Type *typed_obj = (const Type *)obj;
            if (likely(typed_obj->get_glyph(codepoint, glyph)))
                return true;

            if (codepoint <= 0x00FFu) {
                /* For symbol-encoded OpenType fonts, we duplicate the
                 * U+F000..F0FF range at U+0000..U+00FF.  That's what
                 * Windows seems to do, and that's hinted about at:
                 * https://docs.microsoft.com/en-us/typography/opentype/spec/recom
                 * under "Non-Standard (Symbol) Fonts". */
                return typed_obj->get_glyph(0xF000u + codepoint, glyph);
            }

            return false;
        }

    private:
        hb_nonnull_ptr_t<const CmapSubtable> subtable;
        hb_nonnull_ptr_t<const CmapSubtableFormat14> subtable_uvs;

        hb_cmap_get_glyph_func_t get_glyph_funcZ;
        const void *get_glyph_data;

        CmapSubtableFormat4::accelerator_t format4_accel;

    public:
        hb_blob_ptr_t<cmap> table;
    };

protected:
    const CmapSubtable *find_subtable(unsigned int platform_id, unsigned int encoding_id) const
    {
        EncodingRecord key;
        key.platformID = platform_id;
        key.encodingID = encoding_id;

        const EncodingRecord &result = encodingRecord.bsearch(key);
        if (!result.subtable)
            return nullptr;

        return &(this + result.subtable);
    }

    const EncodingRecord *find_encodingrec(unsigned int platform_id, unsigned int encoding_id) const
    {
        EncodingRecord key;
        key.platformID = platform_id;
        key.encodingID = encoding_id;

        return encodingRecord.as_array().bsearch(key);
    }

    bool find_subtable(unsigned format) const
    {
        auto it = +hb_iter(encodingRecord) | hb_map(&EncodingRecord::subtable) | hb_map(hb_add(this)) |
                  hb_filter([&](const CmapSubtable &_) { return _.u.format == format; });

        return it.len();
    }

public:
    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this) && likely(version == 0) && encodingRecord.sanitize(c, this));
    }

protected:
    HBUINT16 version;                             /* Table version number (0). */
    SortedArrayOf<EncodingRecord> encodingRecord; /* Encoding tables. */
public:
    DEFINE_SIZE_ARRAY(4, encodingRecord);
};

struct cmap_accelerator_t : cmap::accelerator_t
{
};

} /* namespace OT */

#endif /* HB_OT_CMAP_TABLE_HH */
