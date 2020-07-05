/*
 * Copyright © 2011,2012  Google, Inc.
 * Copyright © 2018  Ebrahim Byagowi
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

#ifndef HB_OT_OS2_TABLE_HH
#define HB_OT_OS2_TABLE_HH

#include "hb-open-type.hh"
#include "hb-ot-cmap-table.hh"

#include "hb-set.hh"

/*
 * OS/2 and Windows Metrics
 * https://docs.microsoft.com/en-us/typography/opentype/spec/os2
 */
#define HB_OT_TAG_OS2 HB_TAG('O', 'S', '/', '2')

namespace OT {

struct OS2V1Tail
{
    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this));
    }

public:
    HBUINT32 ulCodePageRange1;
    HBUINT32 ulCodePageRange2;

public:
    DEFINE_SIZE_STATIC(8);
};

struct OS2V2Tail
{
    bool has_data() const
    {
        return sxHeight || sCapHeight;
    }

    const OS2V2Tail *operator->() const
    {
        return this;
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this));
    }

public:
    HBINT16 sxHeight;
    HBINT16 sCapHeight;
    HBUINT16 usDefaultChar;
    HBUINT16 usBreakChar;
    HBUINT16 usMaxContext;

public:
    DEFINE_SIZE_STATIC(10);
};

struct OS2V5Tail
{
    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        return_trace(c->check_struct(this));
    }

public:
    HBUINT16 usLowerOpticalPointSize;
    HBUINT16 usUpperOpticalPointSize;

public:
    DEFINE_SIZE_STATIC(4);
};

struct OS2
{
    static constexpr hb_tag_t tableTag = HB_OT_TAG_OS2;

    bool has_data() const
    {
        return usWeightClass || usWidthClass || usFirstCharIndex || usLastCharIndex;
    }

    const OS2V1Tail &v1() const
    {
        return version >= 1 ? v1X : Null(OS2V1Tail);
    }
    const OS2V2Tail &v2() const
    {
        return version >= 2 ? v2X : Null(OS2V2Tail);
    }
    const OS2V5Tail &v5() const
    {
        return version >= 5 ? v5X : Null(OS2V5Tail);
    }

    enum selection_flag_t {
        ITALIC = 1u << 0,
        UNDERSCORE = 1u << 1,
        NEGATIVE = 1u << 2,
        OUTLINED = 1u << 3,
        STRIKEOUT = 1u << 4,
        BOLD = 1u << 5,
        REGULAR = 1u << 6,
        USE_TYPO_METRICS = 1u << 7,
        WWS = 1u << 8,
        OBLIQUE = 1u << 9
    };

    bool use_typo_metrics() const
    {
        return fsSelection & USE_TYPO_METRICS;
    }

    unsigned get_size() const
    {
        unsigned result = min_size;
        if (version >= 1)
            result += v1X.get_size();
        if (version >= 2)
            result += v2X.get_size();
        if (version >= 5)
            result += v5X.get_size();
        return result;
    }

    bool sanitize(hb_sanitize_context_t *c) const
    {
        TRACE_SANITIZE(this);
        if (unlikely(!c->check_struct(this)))
            return_trace(false);
        if (unlikely(version >= 1 && !v1X.sanitize(c)))
            return_trace(false);
        if (unlikely(version >= 2 && !v2X.sanitize(c)))
            return_trace(false);
        if (unlikely(version >= 5 && !v5X.sanitize(c)))
            return_trace(false);
        return_trace(true);
    }

public:
    HBUINT16 version;
    HBINT16 xAvgCharWidth;
    HBUINT16 usWeightClass;
    HBUINT16 usWidthClass;
    HBUINT16 fsType;
    HBINT16 ySubscriptXSize;
    HBINT16 ySubscriptYSize;
    HBINT16 ySubscriptXOffset;
    HBINT16 ySubscriptYOffset;
    HBINT16 ySuperscriptXSize;
    HBINT16 ySuperscriptYSize;
    HBINT16 ySuperscriptXOffset;
    HBINT16 ySuperscriptYOffset;
    HBINT16 yStrikeoutSize;
    HBINT16 yStrikeoutPosition;
    HBINT16 sFamilyClass;
    HBUINT8 panose[10];
    HBUINT32 ulUnicodeRange[4];
    Tag achVendID;
    HBUINT16 fsSelection;
    HBUINT16 usFirstCharIndex;
    HBUINT16 usLastCharIndex;
    HBINT16 sTypoAscender;
    HBINT16 sTypoDescender;
    HBINT16 sTypoLineGap;
    HBUINT16 usWinAscent;
    HBUINT16 usWinDescent;
    OS2V1Tail v1X;
    OS2V2Tail v2X;
    OS2V5Tail v5X;

public:
    DEFINE_SIZE_MIN(78);
};

} /* namespace OT */

#endif /* HB_OT_OS2_TABLE_HH */
