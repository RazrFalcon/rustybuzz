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

} /* namespace OT */

#endif /* RB_OT_LAYOUT_COMMON_HH */
