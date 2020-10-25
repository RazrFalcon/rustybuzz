/*
 * Copyright © 2007,2008,2009  Red Hat, Inc.
 * Copyright © 2010,2012  Google, Inc.
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

#ifndef RB_OT_LAYOUT_COMMON_HH
#define RB_OT_LAYOUT_COMMON_HH

#include "hb.hh"
#include "hb-ot-layout.hh"
#include "hb-open-type.hh"
#include "hb-set.hh"
#include "hb-map.hh"

#ifndef RB_MAX_NESTING_LEVEL
#define RB_MAX_NESTING_LEVEL 6
#endif
#ifndef RB_MAX_CONTEXT_LENGTH
#define RB_MAX_CONTEXT_LENGTH 64
#endif
#ifndef RB_CLOSURE_MAX_STAGES
/*
 * The maximum number of times a lookup can be applied during shaping.
 * Used to limit the number of iterations of the closure algorithm.
 * This must be larger than the number of times add_pause() is
 * called in a collect_features call of any shaper.
 */
#define RB_CLOSURE_MAX_STAGES 32
#endif

#ifndef RB_MAX_SCRIPTS
#define RB_MAX_SCRIPTS 500
#endif

#ifndef RB_MAX_LANGSYS
#define RB_MAX_LANGSYS 2000
#endif

#ifndef RB_MAX_FEATURES
#define RB_MAX_FEATURES 750
#endif

#ifndef RB_MAX_FEATURE_INDICES
#define RB_MAX_FEATURE_INDICES 1500
#endif

#ifndef RB_MAX_LOOKUP_INDICES
#define RB_MAX_LOOKUP_INDICES 20000
#endif

namespace OT {

#define NOT_COVERED ((unsigned int)-1)

/*
 *
 * OpenType Layout Common Table Formats
 *
 */

/*
 * Script, ScriptList, LangSys, Feature, FeatureList, Lookup, LookupList
 */

struct Record_sanitize_closure_t
{
    rb_tag_t tag;
    const void *list_base;
};

template <typename Type> struct Record
{
    int cmp(rb_tag_t a) const
    {
        return tag.cmp(a);
    }

    bool sanitize(rb_sanitize_context_t *c, const void *base) const
    {
        const Record_sanitize_closure_t closure = {tag, base};
        return c->check_struct(this) && offset.sanitize(c, base, &closure);
    }

    Tag tag;               /* 4-byte Tag identifier */
    OffsetTo<Type> offset; /* Offset from beginning of object holding
                            * the Record */
public:
    DEFINE_SIZE_STATIC(6);
};

template <typename Type> struct RecordArrayOf : SortedArrayOf<Record<Type>>
{
    const OffsetTo<Type> &get_offset(unsigned int i) const
    {
        return (*this)[i].offset;
    }
    OffsetTo<Type> &get_offset(unsigned int i)
    {
        return (*this)[i].offset;
    }
    const Tag &get_tag(unsigned int i) const
    {
        return (*this)[i].tag;
    }
    bool find_index(rb_tag_t tag, unsigned int *index) const
    {
        return this->bfind(tag, index, RB_BFIND_NOT_FOUND_STORE, Index::NOT_FOUND_INDEX);
    }
};

template <typename Type> struct RecordListOf : RecordArrayOf<Type>
{
    const Type &operator[](unsigned int i) const
    {
        return this + this->get_offset(i);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return RecordArrayOf<Type>::sanitize(c, this);
    }
};

struct Feature;

struct RecordListOfFeature : RecordListOf<Feature>
{
};

struct RangeRecord
{
    int cmp(rb_codepoint_t g) const
    {
        return g < first ? -1 : g <= last ? 0 : +1;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this);
    }

    template <typename set_t> bool collect_coverage(set_t *glyphs) const
    {
        return glyphs->add_range(first, last);
    }

    HBGlyphID first; /* First GlyphID in the range */
    HBGlyphID last;  /* Last GlyphID in the range */
    HBUINT16 value;  /* Value */
public:
    DEFINE_SIZE_STATIC(6);
};
DECLARE_NULL_NAMESPACE_BYTES(OT, RangeRecord);

struct IndexArray : ArrayOf<Index>
{
    unsigned int
    get_indexes(unsigned int start_offset, unsigned int *_count /* IN/OUT */, unsigned int *_indexes /* OUT */) const
    {
        if (_count) {
            +this->sub_array(start_offset, _count) | rb_sink(rb_array(_indexes, *_count));
        }
        return this->len;
    }
};

struct LangSys
{
    unsigned int get_feature_count() const
    {
        return featureIndex.len;
    }
    rb_tag_t get_feature_index(unsigned int i) const
    {
        return featureIndex[i];
    }

    bool has_required_feature() const
    {
        return reqFeatureIndex != 0xFFFFu;
    }
    unsigned int get_required_feature_index() const
    {
        if (reqFeatureIndex == 0xFFFFu)
            return Index::NOT_FOUND_INDEX;
        return reqFeatureIndex;
    }

    bool operator==(const LangSys &o) const
    {
        if (featureIndex.len != o.featureIndex.len || reqFeatureIndex != o.reqFeatureIndex)
            return false;

        for (const auto _ : +rb_zip(featureIndex, o.featureIndex))
            if (_.first != _.second)
                return false;

        return true;
    }

    bool sanitize(rb_sanitize_context_t *c, const Record_sanitize_closure_t * = nullptr) const
    {
        return c->check_struct(this) && featureIndex.sanitize(c);
    }

    Offset16 lookupOrderZ;    /* = Null (reserved for an offset to a
                               * reordering table) */
    HBUINT16 reqFeatureIndex; /* Index of a feature required for this
                               * language system--if no required features
                               * = 0xFFFFu */
    IndexArray featureIndex;  /* Array of indices into the FeatureList */
public:
    DEFINE_SIZE_ARRAY_SIZED(6, featureIndex);
};
DECLARE_NULL_NAMESPACE_BYTES(OT, LangSys);

struct Script
{
    unsigned int get_lang_sys_count() const
    {
        return langSys.len;
    }
    const Tag &get_lang_sys_tag(unsigned int i) const
    {
        return langSys.get_tag(i);
    }
    const LangSys &get_lang_sys(unsigned int i) const
    {
        if (i == Index::NOT_FOUND_INDEX)
            return get_default_lang_sys();
        return this + langSys[i].offset;
    }
    bool find_lang_sys_index(rb_tag_t tag, unsigned int *index) const
    {
        return langSys.find_index(tag, index);
    }

    bool has_default_lang_sys() const
    {
        return defaultLangSys != 0;
    }
    const LangSys &get_default_lang_sys() const
    {
        return this + defaultLangSys;
    }

    bool sanitize(rb_sanitize_context_t *c, const Record_sanitize_closure_t * = nullptr) const
    {
        return defaultLangSys.sanitize(c, this) && langSys.sanitize(c, this);
    }

protected:
    OffsetTo<LangSys> defaultLangSys; /* Offset to DefaultLangSys table--from
                                       * beginning of Script table--may be Null */
    RecordArrayOf<LangSys> langSys;   /* Array of LangSysRecords--listed
                                       * alphabetically by LangSysTag */
public:
    DEFINE_SIZE_ARRAY_SIZED(4, langSys);
};

typedef RecordListOf<Script> ScriptList;

struct FakeFeatureParams {};

struct Feature
{
    unsigned int get_lookup_count() const
    {
        return lookupIndex.len;
    }
    rb_tag_t get_lookup_index(unsigned int i) const
    {
        return lookupIndex[i];
    }
    unsigned int get_lookup_indexes(unsigned int start_index,
                                    unsigned int *lookup_count /* IN/OUT */,
                                    unsigned int *lookup_tags /* OUT */) const
    {
        return lookupIndex.get_indexes(start_index, lookup_count, lookup_tags);
    }

    bool sanitize(rb_sanitize_context_t *c, const Record_sanitize_closure_t *closure = nullptr) const
    {
        return c->check_struct(this) && lookupIndex.sanitize(c);
    }

    OffsetTo<FakeFeatureParams> featureParams; /* Offset to Feature Parameters table (if one
                                                * has been defined for the feature), relative
                                                * to the beginning of the Feature Table; = Null
                                                * if not required */
    IndexArray lookupIndex;                    /* Array of LookupList indices */
public:
    DEFINE_SIZE_ARRAY_SIZED(4, lookupIndex);
};

typedef RecordListOf<Feature> FeatureList;

struct LookupFlag : HBUINT16
{
    enum Flags {
        RightToLeft = 0x0001u,
        IgnoreBaseGlyphs = 0x0002u,
        IgnoreLigatures = 0x0004u,
        IgnoreMarks = 0x0008u,
        IgnoreFlags = 0x000Eu,
        UseMarkFilteringSet = 0x0010u,
        Reserved = 0x00E0u,
        MarkAttachmentType = 0xFF00u
    };

public:
    DEFINE_SIZE_STATIC(2);
};

} /* namespace OT */
/* This has to be outside the namespace. */
RB_MARK_AS_FLAG_T(OT::LookupFlag::Flags);
namespace OT {

struct Lookup
{
    unsigned int get_subtable_count() const
    {
        return subTable.len;
    }

    template <typename TSubTable> const OffsetArrayOf<TSubTable> &get_subtables() const
    {
        return reinterpret_cast<const OffsetArrayOf<TSubTable> &>(subTable);
    }
    template <typename TSubTable> OffsetArrayOf<TSubTable> &get_subtables()
    {
        return reinterpret_cast<OffsetArrayOf<TSubTable> &>(subTable);
    }

    template <typename TSubTable> const TSubTable &get_subtable(unsigned int i) const
    {
        return this + get_subtables<TSubTable>()[i];
    }
    template <typename TSubTable> TSubTable &get_subtable(unsigned int i)
    {
        return this + get_subtables<TSubTable>()[i];
    }

    unsigned int get_size() const
    {
        const HBUINT16 &markFilteringSet = StructAfter<const HBUINT16>(subTable);
        if (lookupFlag & LookupFlag::UseMarkFilteringSet)
            return (const char *)&StructAfter<const char>(markFilteringSet) - (const char *)this;
        return (const char *)&markFilteringSet - (const char *)this;
    }

    unsigned int get_type() const
    {
        return lookupType;
    }

    /* lookup_props is a 32-bit integer where the lower 16-bit is LookupFlag and
     * higher 16-bit is mark-filtering-set if the lookup uses one.
     * Not to be confused with glyph_props which is very similar. */
    uint32_t get_props() const
    {
        unsigned int flag = lookupFlag;
        if (unlikely(flag & LookupFlag::UseMarkFilteringSet)) {
            const HBUINT16 &markFilteringSet = StructAfter<HBUINT16>(subTable);
            flag += (markFilteringSet << 16);
        }
        return flag;
    }

    template <typename TSubTable, typename context_t, typename... Ts>
    typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        unsigned int lookup_type = get_type();
        unsigned int count = get_subtable_count();
        for (unsigned int i = 0; i < count; i++) {
            typename context_t::return_t r = get_subtable<TSubTable>(i).dispatch(c, lookup_type, rb_forward<Ts>(ds)...);
            if (c->stop_sublookup_iteration(r))
                return r;
        }
        return c->default_return_value();
    }

    template <typename TSubTable> bool sanitize(rb_sanitize_context_t *c) const
    {
        if (!(c->check_struct(this) && subTable.sanitize(c)))
            return false;

        unsigned subtables = get_subtable_count();
        if (unlikely(!c->visit_subtables(subtables)))
            return false;

        if (lookupFlag & LookupFlag::UseMarkFilteringSet) {
            const HBUINT16 &markFilteringSet = StructAfter<HBUINT16>(subTable);
            if (!markFilteringSet.sanitize(c))
                return false;
        }

        if (unlikely(!get_subtables<TSubTable>().sanitize(c, this, get_type())))
            return false;

        if (unlikely(get_type() == TSubTable::Extension && !c->get_edit_count())) {
            /* The spec says all subtables of an Extension lookup should
             * have the same type, which shall not be the Extension type
             * itself (but we already checked for that).
             * This is specially important if one has a reverse type!
             *
             * We only do this if sanitizer edit_count is zero.  Otherwise,
             * some of the subtables might have become insane after they
             * were sanity-checked by the edits of subsequent subtables.
             * https://bugs.chromium.org/p/chromium/issues/detail?id=960331
             */
            unsigned int type = get_subtable<TSubTable>(0).u.extension.get_type();
            for (unsigned int i = 1; i < subtables; i++)
                if (get_subtable<TSubTable>(i).u.extension.get_type() != type)
                    return false;
        }
        return true;
    }

private:
    HBUINT16 lookupType;                           /* Different enumerations for GSUB and GPOS */
    HBUINT16 lookupFlag;                           /* Lookup qualifiers */
    ArrayOf<Offset16> subTable;                    /* Array of SubTables */
    /*HBUINT16	markFilteringSetX[RB_VAR_ARRAY];*/ /* Index (base 0) into GDEF mark glyph sets
                                                    * structure. This field is only present if bit
                                                    * UseMarkFilteringSet of lookup flags is set. */
public:
    DEFINE_SIZE_ARRAY(6, subTable);
};

typedef OffsetListOf<Lookup> LookupList;

template <typename TLookup> struct LookupOffsetList : OffsetListOf<TLookup>
{
    bool sanitize(rb_sanitize_context_t *c) const
    {
        return OffsetListOf<TLookup>::sanitize(c, this);
    }
};

/*
 * Coverage Table
 */

struct CoverageFormat1
{
    friend struct Coverage;

private:
    unsigned int get_coverage(rb_codepoint_t glyph_id) const
    {
        unsigned int i;
        glyphArray.bfind(glyph_id, &i, RB_BFIND_NOT_FOUND_STORE, NOT_COVERED);
        return i;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return glyphArray.sanitize(c);
    }

    template <typename set_t> bool collect_coverage(set_t *glyphs) const
    {
        return glyphs->add_sorted_array(glyphArray.arrayZ, glyphArray.len);
    }

protected:
    HBUINT16 coverageFormat;             /* Format identifier--format = 1 */
    SortedArrayOf<HBGlyphID> glyphArray; /* Array of GlyphIDs--in numerical order */
public:
    DEFINE_SIZE_ARRAY(4, glyphArray);
};

struct CoverageFormat2
{
    friend struct Coverage;

private:
    unsigned int get_coverage(rb_codepoint_t glyph_id) const
    {
        const RangeRecord &range = rangeRecord.bsearch(glyph_id);
        return likely(range.first <= range.last) ? (unsigned int)range.value + (glyph_id - range.first) : NOT_COVERED;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return rangeRecord.sanitize(c);
    }

    template <typename set_t> bool collect_coverage(set_t *glyphs) const
    {
        unsigned int count = rangeRecord.len;
        for (unsigned int i = 0; i < count; i++)
            if (unlikely(!rangeRecord[i].collect_coverage(glyphs)))
                return false;
        return true;
    }

protected:
    HBUINT16 coverageFormat;                /* Format identifier--format = 2 */
    SortedArrayOf<RangeRecord> rangeRecord; /* Array of glyph ranges--ordered by
                                             * Start GlyphID. rangeCount entries
                                             * long */
public:
    DEFINE_SIZE_ARRAY(4, rangeRecord);
};

struct Coverage
{
    /* Has interface. */
    static constexpr unsigned SENTINEL = NOT_COVERED;
    typedef unsigned int value_t;
    value_t operator[](rb_codepoint_t k) const
    {
        return get(k);
    }
    bool has(rb_codepoint_t k) const
    {
        return (*this)[k] != SENTINEL;
    }
    /* Predicate. */
    bool operator()(rb_codepoint_t k) const
    {
        return has(k);
    }

    unsigned int get(rb_codepoint_t k) const
    {
        return get_coverage(k);
    }
    unsigned int get_coverage(rb_codepoint_t glyph_id) const
    {
        switch (u.format) {
        case 1:
            return u.format1.get_coverage(glyph_id);
        case 2:
            return u.format2.get_coverage(glyph_id);
        default:
            return NOT_COVERED;
        }
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        if (!u.format.sanitize(c))
            return false;
        switch (u.format) {
        case 1:
            return u.format1.sanitize(c);
        case 2:
            return u.format2.sanitize(c);
        default:
            return true;
        }
    }

    /* Might return false if array looks unsorted.
     * Used for faster rejection of corrupt data. */
    template <typename set_t> bool collect_coverage(set_t *glyphs) const
    {
        switch (u.format) {
        case 1:
            return u.format1.collect_coverage(glyphs);
        case 2:
            return u.format2.collect_coverage(glyphs);
        default:
            return false;
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        CoverageFormat1 format1;
        CoverageFormat2 format2;
    } u;

public:
    DEFINE_SIZE_UNION(2, format);
};

/*
 * Class Definition Table
 */

struct ClassDefFormat1
{
    friend struct ClassDef;

private:
    unsigned int get_class(rb_codepoint_t glyph_id) const
    {
        return classValue[(unsigned int)(glyph_id - startGlyph)];
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this) && classValue.sanitize(c);
    }

protected:
    HBUINT16 classFormat;         /* Format identifier--format = 1 */
    HBGlyphID startGlyph;         /* First GlyphID of the classValueArray */
    ArrayOf<HBUINT16> classValue; /* Array of Class Values--one per GlyphID */
public:
    DEFINE_SIZE_ARRAY(6, classValue);
};

struct ClassDefFormat2
{
    friend struct ClassDef;

private:
    unsigned int get_class(rb_codepoint_t glyph_id) const
    {
        return rangeRecord.bsearch(glyph_id).value;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return rangeRecord.sanitize(c);
    }

protected:
    HBUINT16 classFormat;                   /* Format identifier--format = 2 */
    SortedArrayOf<RangeRecord> rangeRecord; /* Array of glyph ranges--ordered by
                                             * Start GlyphID */
public:
    DEFINE_SIZE_ARRAY(4, rangeRecord);
};

struct ClassDef
{
    /* Has interface. */
    static constexpr unsigned SENTINEL = 0;
    typedef unsigned int value_t;
    value_t operator[](rb_codepoint_t k) const
    {
        return get(k);
    }
    bool has(rb_codepoint_t k) const
    {
        return (*this)[k] != SENTINEL;
    }
    /* Projection. */
    rb_codepoint_t operator()(rb_codepoint_t k) const
    {
        return get(k);
    }

    unsigned int get(rb_codepoint_t k) const
    {
        return get_class(k);
    }
    unsigned int get_class(rb_codepoint_t glyph_id) const
    {
        switch (u.format) {
        case 1:
            return u.format1.get_class(glyph_id);
        case 2:
            return u.format2.get_class(glyph_id);
        default:
            return 0;
        }
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        if (!u.format.sanitize(c))
            return false;
        switch (u.format) {
        case 1:
            return u.format1.sanitize(c);
        case 2:
            return u.format2.sanitize(c);
        default:
            return true;
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        ClassDefFormat1 format1;
        ClassDefFormat2 format2;
    } u;

public:
    DEFINE_SIZE_UNION(2, format);
};

/*
 * Item Variation Store
 */

struct VarRegionAxis
{
    float evaluate(int coord) const
    {
        int start = startCoord, peak = peakCoord, end = endCoord;

        /* TODO Move these to sanitize(). */
        if (unlikely(start > peak || peak > end))
            return 1.;
        if (unlikely(start < 0 && end > 0 && peak != 0))
            return 1.;

        if (peak == 0 || coord == peak)
            return 1.;

        if (coord <= start || end <= coord)
            return 0.;

        /* Interpolate */
        if (coord < peak)
            return float(coord - start) / (peak - start);
        else
            return float(end - coord) / (end - peak);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this);
        /* TODO Handle invalid start/peak/end configs, so we don't
         * have to do that at runtime. */
    }

public:
    F2DOT14 startCoord;
    F2DOT14 peakCoord;
    F2DOT14 endCoord;

public:
    DEFINE_SIZE_STATIC(6);
};

struct VarRegionList
{
    float evaluate(unsigned int region_index, const int *coords, unsigned int coord_len) const
    {
        if (unlikely(region_index >= regionCount))
            return 0.;

        const VarRegionAxis *axes = axesZ.arrayZ + (region_index * axisCount);

        float v = 1.;
        unsigned int count = axisCount;
        for (unsigned int i = 0; i < count; i++) {
            int coord = i < coord_len ? coords[i] : 0;
            float factor = axes[i].evaluate(coord);
            if (factor == 0.f)
                return 0.;
            v *= factor;
        }
        return v;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this) && axesZ.sanitize(c, (unsigned int)axisCount * (unsigned int)regionCount);
    }

protected:
    HBUINT16 axisCount;
    HBUINT16 regionCount;
    UnsizedArrayOf<VarRegionAxis> axesZ;

public:
    DEFINE_SIZE_ARRAY(4, axesZ);
};

struct VarData
{
    unsigned int get_region_index_count() const
    {
        return regionIndices.len;
    }

    unsigned int get_row_size() const
    {
        return shortCount + regionIndices.len;
    }

    unsigned int get_size() const
    {
        return itemCount * get_row_size();
    }

    float get_delta(unsigned int inner, const int *coords, unsigned int coord_count, const VarRegionList &regions) const
    {
        if (unlikely(inner >= itemCount))
            return 0.;

        unsigned int count = regionIndices.len;
        unsigned int scount = shortCount;

        const HBUINT8 *bytes = get_delta_bytes();
        const HBUINT8 *row = bytes + inner * (scount + count);

        float delta = 0.;
        unsigned int i = 0;

        const HBINT16 *scursor = reinterpret_cast<const HBINT16 *>(row);
        for (; i < scount; i++) {
            float scalar = regions.evaluate(regionIndices.arrayZ[i], coords, coord_count);
            delta += scalar * *scursor++;
        }
        const HBINT8 *bcursor = reinterpret_cast<const HBINT8 *>(scursor);
        for (; i < count; i++) {
            float scalar = regions.evaluate(regionIndices.arrayZ[i], coords, coord_count);
            delta += scalar * *bcursor++;
        }

        return delta;
    }

    void get_scalars(const int *coords,
                     unsigned int coord_count,
                     const VarRegionList &regions,
                     float *scalars /*OUT */,
                     unsigned int num_scalars) const
    {
        unsigned count = rb_min(num_scalars, regionIndices.len);
        for (unsigned int i = 0; i < count; i++)
            scalars[i] = regions.evaluate(regionIndices.arrayZ[i], coords, coord_count);
        for (unsigned int i = count; i < num_scalars; i++)
            scalars[i] = 0.f;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this) && regionIndices.sanitize(c) && shortCount <= regionIndices.len &&
               c->check_range(get_delta_bytes(), itemCount, get_row_size());
    }

protected:
    const HBUINT8 *get_delta_bytes() const
    {
        return &StructAfter<HBUINT8>(regionIndices);
    }

    HBUINT8 *get_delta_bytes()
    {
        return &StructAfter<HBUINT8>(regionIndices);
    }

protected:
    HBUINT16 itemCount;
    HBUINT16 shortCount;
    ArrayOf<HBUINT16> regionIndices;
    /*UnsizedArrayOf<HBUINT8>bytesX;*/
public:
    DEFINE_SIZE_ARRAY(6, regionIndices);
};

struct VariationStore
{
    float get_delta(unsigned int outer, unsigned int inner, const int *coords, unsigned int coord_count) const
    {
        if (unlikely(outer >= dataSets.len))
            return 0.f;

        return (this + dataSets[outer]).get_delta(inner, coords, coord_count, this + regions);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this) && format == 1 && regions.sanitize(c, this) && dataSets.sanitize(c, this);
    }

protected:
    HBUINT16 format;
    LOffsetTo<VarRegionList> regions;
    LOffsetArrayOf<VarData> dataSets;

public:
    DEFINE_SIZE_ARRAY(8, dataSets);
};

/*
 * Feature Variations
 */

struct ConditionFormat1
{
    friend struct Condition;

private:
    bool evaluate(const int *coords, unsigned int coord_len) const
    {
        int coord = axisIndex < coord_len ? coords[axisIndex] : 0;
        return filterRangeMinValue <= coord && coord <= filterRangeMaxValue;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this);
    }

protected:
    HBUINT16 format; /* Format identifier--format = 1 */
    HBUINT16 axisIndex;
    F2DOT14 filterRangeMinValue;
    F2DOT14 filterRangeMaxValue;

public:
    DEFINE_SIZE_STATIC(8);
};

struct Condition
{
    bool evaluate(const int *coords, unsigned int coord_len) const
    {
        switch (u.format) {
        case 1:
            return u.format1.evaluate(coords, coord_len);
        default:
            return false;
        }
    }

    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        if (unlikely(!c->may_dispatch(this, &u.format)))
            return_trace(c->no_dispatch_return_value());
        switch (u.format) {
        case 1:
            return_trace(c->dispatch(u.format1, rb_forward<Ts>(ds)...));
        default:
            return_trace(c->default_return_value());
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
        ConditionFormat1 format1;
    } u;

public:
    DEFINE_SIZE_UNION(2, format);
};

struct ConditionSet
{
    bool evaluate(const int *coords, unsigned int coord_len) const
    {
        unsigned int count = conditions.len;
        for (unsigned int i = 0; i < count; i++)
            if (!(this + conditions.arrayZ[i]).evaluate(coords, coord_len))
                return false;
        return true;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return conditions.sanitize(c, this);
    }

protected:
    LOffsetArrayOf<Condition> conditions;

public:
    DEFINE_SIZE_ARRAY(2, conditions);
};

struct FeatureTableSubstitutionRecord
{
    friend struct FeatureTableSubstitution;

    bool sanitize(rb_sanitize_context_t *c, const void *base) const
    {
        return c->check_struct(this) && feature.sanitize(c, base);
    }

protected:
    HBUINT16 featureIndex;
    LOffsetTo<Feature> feature;

public:
    DEFINE_SIZE_STATIC(6);
};

struct FeatureTableSubstitution
{
    const Feature *find_substitute(unsigned int feature_index) const
    {
        unsigned int count = substitutions.len;
        for (unsigned int i = 0; i < count; i++) {
            const FeatureTableSubstitutionRecord &record = substitutions.arrayZ[i];
            if (record.featureIndex == feature_index)
                return &(this + record.feature);
        }
        return nullptr;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return version.sanitize(c) && likely(version.major == 1) && substitutions.sanitize(c, this);
    }

protected:
    FixedVersion<> version; /* Version--0x00010000u */
    ArrayOf<FeatureTableSubstitutionRecord> substitutions;

public:
    DEFINE_SIZE_ARRAY(6, substitutions);
};

struct FeatureVariationRecord
{
    friend struct FeatureVariations;

    bool sanitize(rb_sanitize_context_t *c, const void *base) const
    {
        return conditions.sanitize(c, base) && substitutions.sanitize(c, base);
    }

protected:
    LOffsetTo<ConditionSet> conditions;
    LOffsetTo<FeatureTableSubstitution> substitutions;

public:
    DEFINE_SIZE_STATIC(8);
};

struct FeatureVariations
{
    static constexpr unsigned NOT_FOUND_INDEX = 0xFFFFFFFFu;

    bool find_index(const int *coords, unsigned int coord_len, unsigned int *index) const
    {
        unsigned int count = varRecords.len;
        for (unsigned int i = 0; i < count; i++) {
            const FeatureVariationRecord &record = varRecords.arrayZ[i];
            if ((this + record.conditions).evaluate(coords, coord_len)) {
                *index = i;
                return true;
            }
        }
        *index = NOT_FOUND_INDEX;
        return false;
    }

    const Feature *find_substitute(unsigned int variations_index, unsigned int feature_index) const
    {
        const FeatureVariationRecord &record = varRecords[variations_index];
        return (this + record.substitutions).find_substitute(feature_index);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return version.sanitize(c) && likely(version.major == 1) && varRecords.sanitize(c, this);
    }

protected:
    FixedVersion<> version; /* Version--0x00010000u */
    LArrayOf<FeatureVariationRecord> varRecords;

public:
    DEFINE_SIZE_ARRAY_SIZED(8, varRecords);
};

/*
 * Device Tables
 */

struct HintingDevice
{
    friend struct Device;

private:
    rb_position_t get_x_delta(rb_font_t *font) const
    {
        return get_delta(rb_font_get_ppem_x(font), rb_font_get_upem(font));
    }

    rb_position_t get_y_delta(rb_font_t *font) const
    {
        return get_delta(rb_font_get_ppem_y(font), rb_font_get_upem(font));
    }

public:
    unsigned int get_size() const
    {
        unsigned int f = deltaFormat;
        if (unlikely(f < 1 || f > 3 || startSize > endSize))
            return 3 * HBUINT16::static_size;
        return HBUINT16::static_size * (4 + ((endSize - startSize) >> (4 - f)));
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this) && c->check_range(this, this->get_size());
    }

private:
    int get_delta(unsigned int ppem, int scale) const
    {
        if (!ppem)
            return 0;

        int pixels = get_delta_pixels(ppem);

        if (!pixels)
            return 0;

        return (int)(pixels * (int64_t)scale / ppem);
    }
    int get_delta_pixels(unsigned int ppem_size) const
    {
        unsigned int f = deltaFormat;
        if (unlikely(f < 1 || f > 3))
            return 0;

        if (ppem_size < startSize || ppem_size > endSize)
            return 0;

        unsigned int s = ppem_size - startSize;

        unsigned int byte = deltaValueZ[s >> (4 - f)];
        unsigned int bits = (byte >> (16 - (((s & ((1 << (4 - f)) - 1)) + 1) << f)));
        unsigned int mask = (0xFFFFu >> (16 - (1 << f)));

        int delta = bits & mask;

        if ((unsigned int)delta >= ((mask + 1) >> 1))
            delta -= mask + 1;

        return delta;
    }

protected:
    HBUINT16 startSize;                   /* Smallest size to correct--in ppem */
    HBUINT16 endSize;                     /* Largest size to correct--in ppem */
    HBUINT16 deltaFormat;                 /* Format of DeltaValue array data: 1, 2, or 3
                                           * 1	Signed 2-bit value, 8 values per uint16
                                           * 2	Signed 4-bit value, 4 values per uint16
                                           * 3	Signed 8-bit value, 2 values per uint16
                                           */
    UnsizedArrayOf<HBUINT16> deltaValueZ; /* Array of compressed data */
public:
    DEFINE_SIZE_ARRAY(6, deltaValueZ);
};

struct VariationDevice
{
    friend struct Device;

private:
    rb_position_t get_x_delta(rb_font_t *font, const VariationStore &store) const
    {
        return (rb_position_t)roundf(get_delta(font, store));
    }

    rb_position_t get_y_delta(rb_font_t *font, const VariationStore &store) const
    {
        return (rb_position_t)roundf(get_delta(font, store));
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this);
    }

private:
    float get_delta(rb_font_t *font, const VariationStore &store) const
    {
        return store.get_delta(outerIndex, innerIndex, rb_font_get_coords(font), rb_font_get_num_coords(font));
    }

protected:
    HBUINT16 outerIndex;
    HBUINT16 innerIndex;
    HBUINT16 deltaFormat; /* Format identifier for this table: 0x0x8000 */
public:
    DEFINE_SIZE_STATIC(6);
};

struct DeviceHeader
{
protected:
    HBUINT16 reserved1;
    HBUINT16 reserved2;

public:
    HBUINT16 format; /* Format identifier */
public:
    DEFINE_SIZE_STATIC(6);
};

struct Device
{
    rb_position_t get_x_delta(rb_font_t *font, const VariationStore &store = Null(VariationStore)) const
    {
        switch (u.b.format) {
#ifndef RB_NO_HINTING
        case 1:
        case 2:
        case 3:
            return u.hinting.get_x_delta(font);
#endif
        case 0x8000:
            return u.variation.get_x_delta(font, store);
        default:
            return 0;
        }
    }
    rb_position_t get_y_delta(rb_font_t *font, const VariationStore &store = Null(VariationStore)) const
    {
        switch (u.b.format) {
        case 1:
        case 2:
        case 3:
#ifndef RB_NO_HINTING
            return u.hinting.get_y_delta(font);
#endif
        case 0x8000:
            return u.variation.get_y_delta(font, store);
        default:
            return 0;
        }
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        if (!u.b.format.sanitize(c))
            return false;
        switch (u.b.format) {
#ifndef RB_NO_HINTING
        case 1:
        case 2:
        case 3:
            return u.hinting.sanitize(c);
#endif
        case 0x8000:
            return u.variation.sanitize(c);
        default:
            return true;
        }
    }

protected:
    union {
        DeviceHeader b;
        HintingDevice hinting;
        VariationDevice variation;
    } u;

public:
    DEFINE_SIZE_UNION(6, b);
};

} /* namespace OT */

#endif /* RB_OT_LAYOUT_COMMON_HH */
