/*
 * Copyright © 2018 Adobe Inc.
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
 * Adobe Author(s): Michiharu Ariza
 */
#ifndef HB_OT_CFF_COMMON_HH
#define HB_OT_CFF_COMMON_HH

#include "hb-open-type.hh"
#include "hb-ot-layout-common.hh"
#include "hb-cff-interp-dict-common.hh"

namespace CFF {

using namespace OT;

#define CFF_UNDEF_CODE 0xFFFFFFFF

/* utility macro */
template <typename Type> static inline const Type &StructAtOffsetOrNull(const void *P, unsigned int offset)
{
    return offset ? StructAtOffset<Type>(P, offset) : Null(Type);
}

inline unsigned int calcOffSize(unsigned int dataSize)
{
    unsigned int size = 1;
    unsigned int offset = dataSize + 1;
    while (offset & ~0xFF) {
        size++;
        offset >>= 8;
    }
    /* format does not support size > 4; caller should handle it as an error */
    return size;
}

struct code_pair_t
{
    hb_codepoint_t code;
    hb_codepoint_t glyph;
};

typedef hb_vector_t<unsigned char> str_buff_t;
struct str_buff_vec_t : hb_vector_t<str_buff_t>
{
    void fini()
    {
        SUPER::fini_deep();
    }

    unsigned int total_size() const
    {
        unsigned int size = 0;
        for (unsigned int i = 0; i < length; i++)
            size += (*this)[i].length;
        return size;
    }

private:
    typedef hb_vector_t<str_buff_t> SUPER;
};

/* CFF INDEX */
template <typename COUNT> struct CFFIndex
{
    static unsigned int calculate_offset_array_size(unsigned int offSize, unsigned int count)
    {
        return offSize * (count + 1);
    }

    unsigned int offset_array_size() const
    {
        return calculate_offset_array_size(offSize, count);
    }

    void set_offset_at(unsigned int index, unsigned int offset)
    {
        HBUINT8 *p = offsets + offSize * index + offSize;
        unsigned int size = offSize;
        for (; size; size--) {
            --p;
            *p = offset & 0xFF;
            offset >>= 8;
        }
    }

    unsigned int offset_at(unsigned int index) const
    {
        assert(index <= count);
        const HBUINT8 *p = offsets + offSize * index;
        unsigned int size = offSize;
        unsigned int offset = 0;
        for (; size; size--)
            offset = (offset << 8) + *p++;
        return offset;
    }

    unsigned int length_at(unsigned int index) const
    {
        if (unlikely((offset_at(index + 1) < offset_at(index)) || (offset_at(index + 1) > offset_at(count))))
            return 0;
        return offset_at(index + 1) - offset_at(index);
    }

    const unsigned char *data_base() const
    {
        return (const unsigned char *)this + min_size + offset_array_size();
    }

    unsigned int data_size() const
    {
        return HBINT8::static_size;
    }

    byte_str_t operator[](unsigned int index) const
    {
        if (unlikely(index >= count))
            return Null(byte_str_t);
        return byte_str_t(data_base() + offset_at(index) - 1, length_at(index));
    }

    unsigned int get_size() const
    {
        if (this == &Null(CFFIndex))
            return 0;
        if (count > 0)
            return min_size + offset_array_size() + (offset_at(count) - 1);
        return count.static_size; /* empty CFFIndex contains count only */
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(likely((c->check_struct(this) && count == 0) || /* empty INDEX */
                            (c->check_struct(this) && offSize >= 1 && offSize <= 4 &&
                             c->check_array(offsets, offSize, count + 1) &&
                             c->check_array((const HBUINT8 *)data_base(), 1, max_offset() - 1))));
    }

protected:
    unsigned int max_offset() const
    {
        unsigned int max = 0;
        for (unsigned int i = 0; i < count + 1u; i++) {
            unsigned int off = offset_at(i);
            if (off > max)
                max = off;
        }
        return max;
    }

public:
    COUNT count;     /* Number of object data. Note there are (count+1) offsets */
    HBUINT8 offSize; /* The byte size of each offset in the offsets array. */
    HBUINT8 offsets[HB_VAR_ARRAY];
    /* The array of (count + 1) offsets into objects array (1-base). */
    /* HBUINT8 data[HB_VAR_ARRAY];	Object data */
public:
    DEFINE_SIZE_ARRAY(COUNT::static_size + HBUINT8::static_size, offsets);
};

template <typename COUNT, typename TYPE> struct CFFIndexOf : CFFIndex<COUNT>
{
    const byte_str_t operator[](unsigned int index) const
    {
        if (likely(index < CFFIndex<COUNT>::count))
            return byte_str_t(CFFIndex<COUNT>::data_base() + CFFIndex<COUNT>::offset_at(index) - 1,
                              CFFIndex<COUNT>::length_at(index));
        return Null(byte_str_t);
    }
};

/* Top Dict, Font Dict, Private Dict */
struct Dict : UnsizedByteStr
{
};

struct TopDict : Dict
{
};
struct FontDict : Dict
{
};
struct PrivateDict : Dict
{
};

struct table_info_t
{
    void init()
    {
        offset = size = 0;
//        link = 0;
    }

    unsigned int offset;
    unsigned int size;
//    objidx_t link;
};

template <typename COUNT> struct FDArray : CFFIndexOf<COUNT, FontDict>
{
};

/* FDSelect */
struct FDSelect0
{
    bool sanitize(hb_sanitize_context_t *c, unsigned int fdcount) const
    {
        TRACE_SANITIZE(this);
        if (unlikely(!(c->check_struct(this))))
            return_trace(false);
        for (unsigned int i = 0; i < c->get_num_glyphs(); i++)
            if (unlikely(!fds[i].sanitize(c)))
                return_trace(false);

        return_trace(true);
    }

    hb_codepoint_t get_fd(hb_codepoint_t glyph) const
    {
        return (hb_codepoint_t)fds[glyph];
    }

    unsigned int get_size(unsigned int num_glyphs) const
    {
        return HBUINT8::static_size * num_glyphs;
    }

    HBUINT8 fds[HB_VAR_ARRAY];

    DEFINE_SIZE_MIN(0);
};

template <typename GID_TYPE, typename FD_TYPE> struct FDSelect3_4_Range
{
    bool sanitize(hb_sanitize_context_t *c, const void * /*nullptr*/, unsigned int fdcount) const
    {
        TRACE_SANITIZE(this);
        return_trace(first < c->get_num_glyphs() && (fd < fdcount));
    }

    GID_TYPE first;
    FD_TYPE fd;

public:
    DEFINE_SIZE_STATIC(GID_TYPE::static_size + FD_TYPE::static_size);
};

template <typename GID_TYPE, typename FD_TYPE> struct FDSelect3_4
{
    unsigned int get_size() const
    {
        return GID_TYPE::static_size * 2 + ranges.get_size();
    }

    bool sanitize(hb_sanitize_context_t *c, unsigned int fdcount) const
    {
        TRACE_SANITIZE(this);
        if (unlikely(!c->check_struct(this) || !ranges.sanitize(c, nullptr, fdcount) || (nRanges() == 0) ||
                     ranges[0].first != 0))
            return_trace(false);

        for (unsigned int i = 1; i < nRanges(); i++)
            if (unlikely(ranges[i - 1].first >= ranges[i].first))
                return_trace(false);

        if (unlikely(!sentinel().sanitize(c) || (sentinel() != c->get_num_glyphs())))
            return_trace(false);

        return_trace(true);
    }

    hb_codepoint_t get_fd(hb_codepoint_t glyph) const
    {
        unsigned int i;
        for (i = 1; i < nRanges(); i++)
            if (glyph < ranges[i].first)
                break;

        return (hb_codepoint_t)ranges[i - 1].fd;
    }

    GID_TYPE &nRanges()
    {
        return ranges.len;
    }
    GID_TYPE nRanges() const
    {
        return ranges.len;
    }
    GID_TYPE &sentinel()
    {
        return StructAfter<GID_TYPE>(ranges[nRanges() - 1]);
    }
    const GID_TYPE &sentinel() const
    {
        return StructAfter<GID_TYPE>(ranges[nRanges() - 1]);
    }

    ArrayOf<FDSelect3_4_Range<GID_TYPE, FD_TYPE>, GID_TYPE> ranges;
    /* GID_TYPE sentinel */

    DEFINE_SIZE_ARRAY(GID_TYPE::static_size, ranges);
};

typedef FDSelect3_4<HBUINT16, HBUINT8> FDSelect3;
typedef FDSelect3_4_Range<HBUINT16, HBUINT8> FDSelect3_Range;

struct FDSelect
{
    unsigned int get_size(unsigned int num_glyphs) const
    {
        switch (format) {
        case 0:
            return format.static_size + u.format0.get_size(num_glyphs);
        case 3:
            return format.static_size + u.format3.get_size();
        default:
            return 0;
        }
    }

    hb_codepoint_t get_fd(hb_codepoint_t glyph) const
    {
        if (this == &Null(FDSelect))
            return 0;

        switch (format) {
        case 0:
            return u.format0.get_fd(glyph);
        case 3:
            return u.format3.get_fd(glyph);
        default:
            return 0;
        }
    }

    bool sanitize(hb_sanitize_context_t *c, unsigned int fdcount) const
    {
        TRACE_SANITIZE(this);
        if (unlikely(!c->check_struct(this)))
            return_trace(false);

        switch (format) {
        case 0:
            return_trace(u.format0.sanitize(c, fdcount));
        case 3:
            return_trace(u.format3.sanitize(c, fdcount));
        default:
            return_trace(false);
        }
    }

    HBUINT8 format;
    union {
        FDSelect0 format0;
        FDSelect3 format3;
    } u;

public:
    DEFINE_SIZE_MIN(1);
};

template <typename COUNT> struct Subrs : CFFIndex<COUNT>
{
    typedef COUNT count_type;
    typedef CFFIndex<COUNT> SUPER;
};

} /* namespace CFF */

#endif /* HB_OT_CFF_COMMON_HH */
