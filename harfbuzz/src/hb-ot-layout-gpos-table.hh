/*
 * Copyright © 2007,2008,2009,2010  Red Hat, Inc.
 * Copyright © 2010,2012,2013  Google, Inc.
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

#ifndef RB_OT_LAYOUT_GPOS_TABLE_HH
#define RB_OT_LAYOUT_GPOS_TABLE_HH

#include "hb-ot-layout-gsubgpos.hh"

extern "C" {
RB_EXTERN rb_bool_t rb_single_pos_apply(const char *data, OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t rb_pair_pos_apply(const char *data, OT::rb_ot_apply_context_t *c);
RB_EXTERN rb_bool_t rb_value_format_apply(unsigned int flags, OT::rb_ot_apply_context_t *c, const char *base, const char *values, unsigned int idx);
RB_EXTERN rb_bool_t rb_anchor_get(const char *data, const OT::rb_ot_apply_context_t *c, float *x, float *y);
}

namespace OT {

struct MarkArray;

/* buffer **position** var allocations */
#define attach_chain()                                                                                                 \
    var.i16[0] /* glyph to which this attaches to, relative to current glyphs; negative for going back, positive for   \
                  forward. */
#define attach_type() var.u8[2] /* attachment type */
/* Note! if attach_chain() is zero, the value of attach_type() is irrelevant. */

enum attach_type_t {
    ATTACH_TYPE_NONE = 0X00,

    /* Each attachment should be either a mark or a cursive; can't be both. */
    ATTACH_TYPE_MARK = 0X01,
    ATTACH_TYPE_CURSIVE = 0X02,
};

/* Shared Tables: ValueRecord, Anchor Table, and MarkArray */

typedef HBUINT16 Value;

typedef UnsizedArrayOf<Value> ValueRecord;

struct ValueFormat : HBUINT16
{
    unsigned int get_len() const
    {
        return rb_popcount((unsigned int)*this);
    }
    unsigned int get_size() const
    {
        return get_len() * Value::static_size;
    }

    bool
    apply_value(rb_ot_apply_context_t *c, const void *base, const Value *values, unsigned int idx) const
    {
        return rb_value_format_apply((unsigned int)*this, c, (const char*)base, (const char*)values, idx);
    }
};

struct Anchor
{
    void get_anchor(rb_ot_apply_context_t *c, rb_codepoint_t glyph_id, float *x, float *y) const
    {
        *x = *y = 0;
        rb_anchor_get((const char*)this, c, x, y);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return true;
    }
};

struct AnchorMatrix
{
    const Anchor &get_anchor(unsigned int row, unsigned int col, unsigned int cols, bool *found) const
    {
        *found = false;
        if (unlikely(row >= rows || col >= cols))
            return Null(Anchor);
        *found = !matrixZ[row * cols + col].is_null();
        return this + matrixZ[row * cols + col];
    }

    bool sanitize(rb_sanitize_context_t *c, unsigned int cols) const
    {
        if (!c->check_struct(this))
            return false;
        if (unlikely(rb_unsigned_mul_overflows(rows, cols)))
            return false;
        unsigned int count = rows * cols;
        if (!c->check_array(matrixZ.arrayZ, count))
            return false;
        for (unsigned int i = 0; i < count; i++)
            if (!matrixZ[i].sanitize(c, this))
                return false;
        return true;
    }

    HBUINT16 rows;                            /* Number of rows */
    UnsizedArrayOf<OffsetTo<Anchor>> matrixZ; /* Matrix of offsets to Anchor tables--
                                               * from beginning of AnchorMatrix table */
public:
    DEFINE_SIZE_ARRAY(2, matrixZ);
};

struct MarkRecord
{
    friend struct MarkArray;

    unsigned get_class() const
    {
        return (unsigned)klass;
    }
    bool sanitize(rb_sanitize_context_t *c, const void *base) const
    {
        return c->check_struct(this) && markAnchor.sanitize(c, base);
    }

protected:
    HBUINT16 klass;              /* Class defined for this mark */
    OffsetTo<Anchor> markAnchor; /* Offset to Anchor table--from
                                  * beginning of MarkArray table */
public:
    DEFINE_SIZE_STATIC(4);
};

struct MarkArray : ArrayOf<MarkRecord> /* Array of MarkRecords--in Coverage order */
{
    bool apply(rb_ot_apply_context_t *c,
               unsigned int mark_index,
               unsigned int glyph_index,
               const AnchorMatrix &anchors,
               unsigned int class_count,
               unsigned int glyph_pos) const
    {
        rb_buffer_t *buffer = c->buffer;
        const MarkRecord &record = ArrayOf<MarkRecord>::operator[](mark_index);
        unsigned int mark_class = record.klass;

        const Anchor &mark_anchor = this + record.markAnchor;
        bool found;
        const Anchor &glyph_anchor = anchors.get_anchor(glyph_index, mark_class, class_count, &found);
        /* If this subtable doesn't have an anchor for this base and this class,
         * return false such that the subsequent subtables have a chance at it. */
        if (unlikely(!found))
            return false;

        float mark_x, mark_y, base_x, base_y;

        rb_buffer_unsafe_to_break(buffer, glyph_pos, rb_buffer_get_index(buffer));
        mark_anchor.get_anchor(c, rb_buffer_get_cur(buffer, 0)->codepoint, &mark_x, &mark_y);
        glyph_anchor.get_anchor(c, rb_buffer_get_glyph_infos(buffer)[glyph_pos].codepoint, &base_x, &base_y);

        rb_glyph_position_t &o = *rb_buffer_get_cur_pos(buffer);
        o.x_offset = roundf(base_x - mark_x);
        o.y_offset = roundf(base_y - mark_y);
        o.attach_type() = ATTACH_TYPE_MARK;
        o.attach_chain() = (int)glyph_pos - (int)rb_buffer_get_index(buffer);
        rb_buffer_set_scratch_flags(buffer,
                                    rb_buffer_get_scratch_flags(buffer) | RB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT);

        rb_buffer_set_index(buffer, rb_buffer_get_index(buffer) + 1);
        return true;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return ArrayOf<MarkRecord>::sanitize(c, this);
    }
};

/* Lookups */

struct SinglePos
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_single_pos_apply((const char*)this, c);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return (format != 1 && format != 2) || coverage.sanitize(c, this);
    }

protected:
    HBUINT16 format;
    OffsetTo<Coverage> coverage;
};

struct PairPos
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return rb_pair_pos_apply((const char*)this, c);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return (format != 1 && format != 2) || coverage.sanitize(c, this);
    }

protected:
    HBUINT16 format;
    OffsetTo<Coverage> coverage;
};

struct EntryExitRecord
{
    friend struct CursivePosFormat1;

    bool sanitize(rb_sanitize_context_t *c, const void *base) const
    {
        return entryAnchor.sanitize(c, base) && exitAnchor.sanitize(c, base);
    }

protected:
    OffsetTo<Anchor> entryAnchor; /* Offset to EntryAnchor table--from
                                   * beginning of CursivePos
                                   * subtable--may be NULL */
    OffsetTo<Anchor> exitAnchor;  /* Offset to ExitAnchor table--from
                                   * beginning of CursivePos
                                   * subtable--may be NULL */
public:
    DEFINE_SIZE_STATIC(4);
};

static void reverse_cursive_minor_offset(rb_glyph_position_t *pos,
                                         unsigned int i,
                                         rb_direction_t direction,
                                         unsigned int new_parent);

struct CursivePosFormat1
{
    const Coverage &get_coverage() const
    {
        return this + coverage;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        rb_buffer_t *buffer = c->buffer;

        const EntryExitRecord &this_record =
            entryExitRecord[(this + coverage).get_coverage(rb_buffer_get_cur(buffer, 0)->codepoint)];
        if (!this_record.entryAnchor)
            return false;

        rb_ot_apply_context_t::skipping_iterator_t &skippy_iter = c->iter_input;
        skippy_iter.reset(rb_buffer_get_index(buffer), 1);
        if (!skippy_iter.prev())
            return false;

        const EntryExitRecord &prev_record =
            entryExitRecord[(this + coverage)
                                .get_coverage(rb_buffer_get_glyph_infos(buffer)[skippy_iter.idx].codepoint)];
        if (!prev_record.exitAnchor)
            return false;

        unsigned int i = skippy_iter.idx;
        unsigned int j = rb_buffer_get_index(buffer);

        rb_buffer_unsafe_to_break(buffer, i, j);
        float entry_x, entry_y, exit_x, exit_y;
        (this + prev_record.exitAnchor).get_anchor(c, rb_buffer_get_glyph_infos(buffer)[i].codepoint, &exit_x, &exit_y);
        (this + this_record.entryAnchor)
            .get_anchor(c, rb_buffer_get_glyph_infos(buffer)[j].codepoint, &entry_x, &entry_y);

        rb_glyph_position_t *pos = rb_buffer_get_glyph_positions(buffer);

        rb_position_t d;
        /* Main-direction adjustment */
        switch (c->direction) {
        case RB_DIRECTION_LTR:
            pos[i].x_advance = roundf(exit_x) + pos[i].x_offset;

            d = roundf(entry_x) + pos[j].x_offset;
            pos[j].x_advance -= d;
            pos[j].x_offset -= d;
            break;
        case RB_DIRECTION_RTL:
            d = roundf(exit_x) + pos[i].x_offset;
            pos[i].x_advance -= d;
            pos[i].x_offset -= d;

            pos[j].x_advance = roundf(entry_x) + pos[j].x_offset;
            break;
        case RB_DIRECTION_TTB:
            pos[i].y_advance = roundf(exit_y) + pos[i].y_offset;

            d = roundf(entry_y) + pos[j].y_offset;
            pos[j].y_advance -= d;
            pos[j].y_offset -= d;
            break;
        case RB_DIRECTION_BTT:
            d = roundf(exit_y) + pos[i].y_offset;
            pos[i].y_advance -= d;
            pos[i].y_offset -= d;

            pos[j].y_advance = roundf(entry_y);
            break;
        case RB_DIRECTION_INVALID:
        default:
            break;
        }

        /* Cross-direction adjustment */

        /* We attach child to parent (think graph theory and rooted trees whereas
         * the root stays on baseline and each node aligns itself against its
         * parent.
         *
         * Optimize things for the case of RightToLeft, as that's most common in
         * Arabic. */
        unsigned int child = i;
        unsigned int parent = j;
        rb_position_t x_offset = entry_x - exit_x;
        rb_position_t y_offset = entry_y - exit_y;
        if (!(c->lookup_props & LookupFlag::RightToLeft)) {
            unsigned int k = child;
            child = parent;
            parent = k;
            x_offset = -x_offset;
            y_offset = -y_offset;
        }

        /* If child was already connected to someone else, walk through its old
         * chain and reverse the link direction, such that the whole tree of its
         * previous connection now attaches to new parent.  Watch out for case
         * where new parent is on the path from old chain...
         */
        reverse_cursive_minor_offset(pos, child, c->direction, parent);

        pos[child].attach_type() = ATTACH_TYPE_CURSIVE;
        pos[child].attach_chain() = (int)parent - (int)child;
        rb_buffer_set_scratch_flags(buffer,
                                    rb_buffer_get_scratch_flags(buffer) | RB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT);
        if (likely(RB_DIRECTION_IS_HORIZONTAL(c->direction)))
            pos[child].y_offset = y_offset;
        else
            pos[child].x_offset = x_offset;

        /* If parent was attached to child, break them free.
         * https://github.com/harfbuzz/harfbuzz/issues/2469
         */
        if (unlikely(pos[parent].attach_chain() == -pos[child].attach_chain()))
            pos[parent].attach_chain() = 0;

        rb_buffer_set_index(buffer, rb_buffer_get_index(buffer) + 1);
        return true;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return coverage.sanitize(c, this) && entryExitRecord.sanitize(c, this);
    }

protected:
    HBUINT16 format;                          /* Format identifier--format = 1 */
    OffsetTo<Coverage> coverage;              /* Offset to Coverage table--from
                                               * beginning of subtable */
    ArrayOf<EntryExitRecord> entryExitRecord; /* Array of EntryExit records--in
                                               * Coverage Index order */
public:
    DEFINE_SIZE_ARRAY(6, entryExitRecord);
};

struct CursivePos
{
    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        if (unlikely(!c->may_dispatch(this, &u.format)))
            return c->no_dispatch_return_value();
        switch (u.format) {
        case 1:
            return c->dispatch(u.format1, rb_forward<Ts>(ds)...);
        default:
            return c->default_return_value();
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        CursivePosFormat1 format1;
    } u;
};

typedef AnchorMatrix BaseArray; /* base-major--
                                 * in order of BaseCoverage Index--,
                                 * mark-minor--
                                 * ordered by class--zero-based. */

struct MarkBasePosFormat1
{
    const Coverage &get_coverage() const
    {
        return this + markCoverage;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        rb_buffer_t *buffer = c->buffer;
        unsigned int mark_index = (this + markCoverage).get_coverage(rb_buffer_get_cur(buffer, 0)->codepoint);
        if (likely(mark_index == NOT_COVERED))
            return false;

        /* Now we search backwards for a non-mark glyph */
        rb_ot_apply_context_t::skipping_iterator_t &skippy_iter = c->iter_input;
        skippy_iter.reset(rb_buffer_get_index(buffer), 1);
        skippy_iter.set_lookup_props(LookupFlag::IgnoreMarks);
        do {
            if (!skippy_iter.prev())
                return false;
            /* We only want to attach to the first of a MultipleSubst sequence.
             * https://github.com/harfbuzz/harfbuzz/issues/740
             * Reject others...
             * ...but stop if we find a mark in the MultipleSubst sequence:
             * https://github.com/harfbuzz/harfbuzz/issues/1020 */
            auto info = rb_buffer_get_glyph_infos(buffer);
            if (!_rb_glyph_info_multiplied(&info[skippy_iter.idx]) ||
                0 == _rb_glyph_info_get_lig_comp(&info[skippy_iter.idx]) ||
                (skippy_iter.idx == 0 || _rb_glyph_info_is_mark(&info[skippy_iter.idx - 1]) ||
                 _rb_glyph_info_get_lig_id(&info[skippy_iter.idx]) !=
                     _rb_glyph_info_get_lig_id(&info[skippy_iter.idx - 1]) ||
                 _rb_glyph_info_get_lig_comp(&info[skippy_iter.idx]) !=
                     _rb_glyph_info_get_lig_comp(&info[skippy_iter.idx - 1]) + 1))
                break;
            skippy_iter.reject();
        } while (true);

        /* Checking that matched glyph is actually a base glyph by GDEF is too strong; disabled */
        // if (!_rb_glyph_info_is_base_glyph (&rb_buffer_get_glyph_infos(buffer)[skippy_iter.idx])) { return_trace
        // (false); }

        unsigned int base_index =
            (this + baseCoverage).get_coverage(rb_buffer_get_glyph_infos(buffer)[skippy_iter.idx].codepoint);
        if (base_index == NOT_COVERED)
            return false;

        return (this + markArray).apply(c, mark_index, base_index, this + baseArray, classCount, skippy_iter.idx);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this) && markCoverage.sanitize(c, this) && baseCoverage.sanitize(c, this) &&
               markArray.sanitize(c, this) && baseArray.sanitize(c, this, (unsigned int)classCount);
    }

protected:
    HBUINT16 format;                 /* Format identifier--format = 1 */
    OffsetTo<Coverage> markCoverage; /* Offset to MarkCoverage table--from
                                      * beginning of MarkBasePos subtable */
    OffsetTo<Coverage> baseCoverage; /* Offset to BaseCoverage table--from
                                      * beginning of MarkBasePos subtable */
    HBUINT16 classCount;             /* Number of classes defined for marks */
    OffsetTo<MarkArray> markArray;   /* Offset to MarkArray table--from
                                      * beginning of MarkBasePos subtable */
    OffsetTo<BaseArray> baseArray;   /* Offset to BaseArray table--from
                                      * beginning of MarkBasePos subtable */
public:
    DEFINE_SIZE_STATIC(12);
};

struct MarkBasePos
{
    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        if (unlikely(!c->may_dispatch(this, &u.format)))
            return c->no_dispatch_return_value();
        switch (u.format) {
        case 1:
            return c->dispatch(u.format1, rb_forward<Ts>(ds)...);
        default:
            return c->default_return_value();
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        MarkBasePosFormat1 format1;
    } u;
};

typedef AnchorMatrix LigatureAttach; /* component-major--
                                      * in order of writing direction--,
                                      * mark-minor--
                                      * ordered by class--zero-based. */

typedef OffsetListOf<LigatureAttach> LigatureArray;
/* Array of LigatureAttach
 * tables ordered by
 * LigatureCoverage Index */

struct MarkLigPosFormat1
{
    const Coverage &get_coverage() const
    {
        return this + markCoverage;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        rb_buffer_t *buffer = c->buffer;
        unsigned int mark_index = (this + markCoverage).get_coverage(rb_buffer_get_cur(buffer, 0)->codepoint);
        if (likely(mark_index == NOT_COVERED))
            return false;

        /* Now we search backwards for a non-mark glyph */
        rb_ot_apply_context_t::skipping_iterator_t &skippy_iter = c->iter_input;
        skippy_iter.reset(rb_buffer_get_index(buffer), 1);
        skippy_iter.set_lookup_props(LookupFlag::IgnoreMarks);
        if (!skippy_iter.prev())
            return false;

        /* Checking that matched glyph is actually a ligature by GDEF is too strong; disabled */
        // if (!_rb_glyph_info_is_ligature (&rb_buffer_get_glyph_infos(buffer)[skippy_iter.idx])) { return_trace
        // (false); }

        unsigned int j = skippy_iter.idx;
        unsigned int lig_index = (this + ligatureCoverage).get_coverage(rb_buffer_get_glyph_infos(buffer)[j].codepoint);
        if (lig_index == NOT_COVERED)
            return false;

        const LigatureArray &lig_array = this + ligatureArray;
        const LigatureAttach &lig_attach = lig_array[lig_index];

        /* Find component to attach to */
        unsigned int comp_count = lig_attach.rows;
        if (unlikely(!comp_count))
            return false;

        /* We must now check whether the ligature ID of the current mark glyph
         * is identical to the ligature ID of the found ligature.  If yes, we
         * can directly use the component index.  If not, we attach the mark
         * glyph to the last component of the ligature. */
        unsigned int comp_index;
        unsigned int lig_id = _rb_glyph_info_get_lig_id(&rb_buffer_get_glyph_infos(buffer)[j]);
        unsigned int mark_id = _rb_glyph_info_get_lig_id(rb_buffer_get_cur(buffer, 0));
        unsigned int mark_comp = _rb_glyph_info_get_lig_comp(rb_buffer_get_cur(buffer, 0));
        if (lig_id && lig_id == mark_id && mark_comp > 0)
            comp_index = rb_min(comp_count, _rb_glyph_info_get_lig_comp(rb_buffer_get_cur(buffer, 0))) - 1;
        else
            comp_index = comp_count - 1;

        return (this + markArray).apply(c, mark_index, comp_index, lig_attach, classCount, j);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this) && markCoverage.sanitize(c, this) && ligatureCoverage.sanitize(c, this) &&
               markArray.sanitize(c, this) && ligatureArray.sanitize(c, this, (unsigned int)classCount);
    }

protected:
    HBUINT16 format;                       /* Format identifier--format = 1 */
    OffsetTo<Coverage> markCoverage;       /* Offset to Mark Coverage table--from
                                            * beginning of MarkLigPos subtable */
    OffsetTo<Coverage> ligatureCoverage;   /* Offset to Ligature Coverage
                                            * table--from beginning of MarkLigPos
                                            * subtable */
    HBUINT16 classCount;                   /* Number of defined mark classes */
    OffsetTo<MarkArray> markArray;         /* Offset to MarkArray table--from
                                            * beginning of MarkLigPos subtable */
    OffsetTo<LigatureArray> ligatureArray; /* Offset to LigatureArray table--from
                                            * beginning of MarkLigPos subtable */
public:
    DEFINE_SIZE_STATIC(12);
};

struct MarkLigPos
{
    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        if (unlikely(!c->may_dispatch(this, &u.format)))
            return c->no_dispatch_return_value();
        switch (u.format) {
        case 1:
            return c->dispatch(u.format1, rb_forward<Ts>(ds)...);
        default:
            return c->default_return_value();
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        MarkLigPosFormat1 format1;
    } u;
};

typedef AnchorMatrix Mark2Array; /* mark2-major--
                                  * in order of Mark2Coverage Index--,
                                  * mark1-minor--
                                  * ordered by class--zero-based. */

struct MarkMarkPosFormat1
{
    const Coverage &get_coverage() const
    {
        return this + mark1Coverage;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        rb_buffer_t *buffer = c->buffer;
        unsigned int mark1_index = (this + mark1Coverage).get_coverage(rb_buffer_get_cur(buffer, 0)->codepoint);
        if (likely(mark1_index == NOT_COVERED))
            return false;

        /* now we search backwards for a suitable mark glyph until a non-mark glyph */
        rb_ot_apply_context_t::skipping_iterator_t &skippy_iter = c->iter_input;
        skippy_iter.reset(rb_buffer_get_index(buffer), 1);
        skippy_iter.set_lookup_props(c->lookup_props & ~LookupFlag::IgnoreFlags);
        if (!skippy_iter.prev())
            return false;

        if (!_rb_glyph_info_is_mark(&rb_buffer_get_glyph_infos(buffer)[skippy_iter.idx])) {
            return false;
        }

        unsigned int j = skippy_iter.idx;

        unsigned int id1 = _rb_glyph_info_get_lig_id(rb_buffer_get_cur(buffer, 0));
        unsigned int id2 = _rb_glyph_info_get_lig_id(&rb_buffer_get_glyph_infos(buffer)[j]);
        unsigned int comp1 = _rb_glyph_info_get_lig_comp(rb_buffer_get_cur(buffer, 0));
        unsigned int comp2 = _rb_glyph_info_get_lig_comp(&rb_buffer_get_glyph_infos(buffer)[j]);

        if (likely(id1 == id2)) {
            if (id1 == 0) /* Marks belonging to the same base. */
                goto good;
            else if (comp1 == comp2) /* Marks belonging to the same ligature component. */
                goto good;
        } else {
            /* If ligature ids don't match, it may be the case that one of the marks
             * itself is a ligature.  In which case match. */
            if ((id1 > 0 && !comp1) || (id2 > 0 && !comp2))
                goto good;
        }

        /* Didn't match. */
        return false;

    good:
        unsigned int mark2_index = (this + mark2Coverage).get_coverage(rb_buffer_get_glyph_infos(buffer)[j].codepoint);
        if (mark2_index == NOT_COVERED)
            return false;

        return (this + mark1Array).apply(c, mark1_index, mark2_index, this + mark2Array, classCount, j);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this) && mark1Coverage.sanitize(c, this) && mark2Coverage.sanitize(c, this) &&
               mark1Array.sanitize(c, this) && mark2Array.sanitize(c, this, (unsigned int)classCount);
    }

protected:
    HBUINT16 format;                  /* Format identifier--format = 1 */
    OffsetTo<Coverage> mark1Coverage; /* Offset to Combining Mark1 Coverage
                                       * table--from beginning of MarkMarkPos
                                       * subtable */
    OffsetTo<Coverage> mark2Coverage; /* Offset to Combining Mark2 Coverage
                                       * table--from beginning of MarkMarkPos
                                       * subtable */
    HBUINT16 classCount;              /* Number of defined mark classes */
    OffsetTo<MarkArray> mark1Array;   /* Offset to Mark1Array table--from
                                       * beginning of MarkMarkPos subtable */
    OffsetTo<Mark2Array> mark2Array;  /* Offset to Mark2Array table--from
                                       * beginning of MarkMarkPos subtable */
public:
    DEFINE_SIZE_STATIC(12);
};

struct MarkMarkPos
{
    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        if (unlikely(!c->may_dispatch(this, &u.format)))
            return c->no_dispatch_return_value();
        switch (u.format) {
        case 1:
            return c->dispatch(u.format1, rb_forward<Ts>(ds)...);
        default:
            return c->default_return_value();
        }
    }

protected:
    union {
        HBUINT16 format; /* Format identifier */
        MarkMarkPosFormat1 format1;
    } u;
};

struct ContextPos : Context
{
};

struct ChainContextPos : ChainContext
{
};

struct ExtensionPos : Extension<ExtensionPos>
{
    typedef struct PosLookupSubTable SubTable;
};

/*
 * PosLookup
 */

struct PosLookupSubTable
{
    friend struct Lookup;
    friend struct PosLookup;

    enum Type {
        Single = 1,
        Pair = 2,
        Cursive = 3,
        MarkBase = 4,
        MarkLig = 5,
        MarkMark = 6,
        Context = 7,
        ChainContext = 8,
        Extension = 9
    };

    template <typename context_t, typename... Ts>
    typename context_t::return_t dispatch(context_t *c, unsigned int lookup_type, Ts &&... ds) const
    {
        switch (lookup_type) {
        case Single:
            return c->dispatch(u.single, rb_forward<Ts>(ds)...);
        case Pair:
            return c->dispatch(u.pair, rb_forward<Ts>(ds)...);
        case Cursive:
            return u.cursive.dispatch(c, rb_forward<Ts>(ds)...);
        case MarkBase:
            return u.markBase.dispatch(c, rb_forward<Ts>(ds)...);
        case MarkLig:
            return u.markLig.dispatch(c, rb_forward<Ts>(ds)...);
        case MarkMark:
            return u.markMark.dispatch(c, rb_forward<Ts>(ds)...);
        case Context:
            return c->dispatch(u.context, rb_forward<Ts>(ds)...);
        case ChainContext:
            return c->dispatch(u.chainContext, rb_forward<Ts>(ds)...);
        case Extension:
            return u.extension.dispatch(c, rb_forward<Ts>(ds)...);
        default:
            return c->default_return_value();
        }
    }

protected:
    union {
        SinglePos single;
        PairPos pair;
        CursivePos cursive;
        MarkBasePos markBase;
        MarkLigPos markLig;
        MarkMarkPos markMark;
        ContextPos context;
        ChainContextPos chainContext;
        ExtensionPos extension;
    } u;

public:
    DEFINE_SIZE_MIN(0);
};

struct PosLookup : Lookup
{
    typedef struct PosLookupSubTable SubTable;

    const SubTable &get_subtable(unsigned int i) const
    {
        return Lookup::get_subtable<SubTable>(i);
    }

    bool is_reverse() const
    {
        return false;
    }

    bool apply(rb_ot_apply_context_t *c) const
    {
        return dispatch(c);
    }

    template <typename set_t> void collect_coverage(set_t *glyphs) const
    {
        rb_collect_coverage_context_t<set_t> c(glyphs);
        dispatch(&c);
    }

    static inline bool apply_recurse_func(rb_ot_apply_context_t *c, unsigned int lookup_index);

    template <typename context_t, typename... Ts> typename context_t::return_t dispatch(context_t *c, Ts &&... ds) const
    {
        return Lookup::dispatch<SubTable>(c, rb_forward<Ts>(ds)...);
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return Lookup::sanitize<SubTable>(c);
    }
};

/*
 * GPOS -- Glyph Positioning
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gpos
 */

struct GPOS : GSUBGPOS
{
    static constexpr rb_tag_t tableTag = RB_OT_TAG_GPOS;

    const PosLookup &get_lookup(unsigned int i) const
    {
        return static_cast<const PosLookup &>(GSUBGPOS::get_lookup(i));
    }

    static inline void position_start(rb_font_t *font, rb_buffer_t *buffer);
    static inline void position_finish_advances(rb_font_t *font, rb_buffer_t *buffer);
    static inline void position_finish_offsets(rb_font_t *font, rb_buffer_t *buffer);

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return GSUBGPOS::sanitize<PosLookup>(c);
    }

    RB_INTERNAL bool is_blocklisted(rb_blob_t *blob, rb_face_t *face) const;

    typedef GSUBGPOS::accelerator_t<GPOS> accelerator_t;
};

static void reverse_cursive_minor_offset(rb_glyph_position_t *pos,
                                         unsigned int i,
                                         rb_direction_t direction,
                                         unsigned int new_parent)
{
    int chain = pos[i].attach_chain(), type = pos[i].attach_type();
    if (likely(!chain || 0 == (type & ATTACH_TYPE_CURSIVE)))
        return;

    pos[i].attach_chain() = 0;

    unsigned int j = (int)i + chain;

    /* Stop if we see new parent in the chain. */
    if (j == new_parent)
        return;

    reverse_cursive_minor_offset(pos, j, direction, new_parent);

    if (RB_DIRECTION_IS_HORIZONTAL(direction))
        pos[j].y_offset = -pos[i].y_offset;
    else
        pos[j].x_offset = -pos[i].x_offset;

    pos[j].attach_chain() = -chain;
    pos[j].attach_type() = type;
}
static void
propagate_attachment_offsets(rb_glyph_position_t *pos, unsigned int len, unsigned int i, rb_direction_t direction)
{
    /* Adjusts offsets of attached glyphs (both cursive and mark) to accumulate
     * offset of glyph they are attached to. */
    int chain = pos[i].attach_chain(), type = pos[i].attach_type();
    if (likely(!chain))
        return;

    pos[i].attach_chain() = 0;

    unsigned int j = (int)i + chain;

    if (unlikely(j >= len))
        return;

    propagate_attachment_offsets(pos, len, j, direction);

    assert(!!(type & ATTACH_TYPE_MARK) ^ !!(type & ATTACH_TYPE_CURSIVE));

    if (type & ATTACH_TYPE_CURSIVE) {
        if (RB_DIRECTION_IS_HORIZONTAL(direction))
            pos[i].y_offset += pos[j].y_offset;
        else
            pos[i].x_offset += pos[j].x_offset;
    } else /*if (type & ATTACH_TYPE_MARK)*/
    {
        pos[i].x_offset += pos[j].x_offset;
        pos[i].y_offset += pos[j].y_offset;

        assert(j < i);
        if (RB_DIRECTION_IS_FORWARD(direction))
            for (unsigned int k = j; k < i; k++) {
                pos[i].x_offset -= pos[k].x_advance;
                pos[i].y_offset -= pos[k].y_advance;
            }
        else
            for (unsigned int k = j + 1; k < i + 1; k++) {
                pos[i].x_offset += pos[k].x_advance;
                pos[i].y_offset += pos[k].y_advance;
            }
    }
}

void GPOS::position_start(rb_font_t *font RB_UNUSED, rb_buffer_t *buffer)
{
    unsigned int count = rb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++)
        rb_buffer_get_glyph_positions(buffer)[i].attach_chain() =
            rb_buffer_get_glyph_positions(buffer)[i].attach_type() = 0;
}

void GPOS::position_finish_advances(rb_font_t *font RB_UNUSED, rb_buffer_t *buffer RB_UNUSED)
{
    //_rb_buffer_assert_gsubgpos_vars (buffer);
}

void GPOS::position_finish_offsets(rb_font_t *font RB_UNUSED, rb_buffer_t *buffer)
{
    unsigned int len = rb_buffer_get_length(buffer);
    rb_glyph_position_t *pos = rb_buffer_get_glyph_positions(buffer);
    rb_direction_t direction = rb_buffer_get_direction(buffer);

    /* Handle attachments */
    if (rb_buffer_get_scratch_flags(buffer) & RB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT)
        for (unsigned int i = 0; i < len; i++)
            propagate_attachment_offsets(pos, len, i, direction);
}

struct GPOS_accelerator_t : GPOS::accelerator_t
{
};

/* Out-of-class implementation for methods recursing */

/*static*/ bool PosLookup::apply_recurse_func(rb_ot_apply_context_t *c, unsigned int lookup_index)
{
    const PosLookup &l = c->face->table.GPOS.get_relaxed()->table->get_lookup(lookup_index);
    unsigned int saved_lookup_props = c->lookup_props;
    unsigned int saved_lookup_index = c->lookup_index;
    c->set_lookup_index(lookup_index);
    c->set_lookup_props(l.get_props());
    bool ret = l.dispatch(c);
    c->set_lookup_index(saved_lookup_index);
    c->set_lookup_props(saved_lookup_props);
    return ret;
}

} /* namespace OT */

#endif /* RB_OT_LAYOUT_GPOS_TABLE_HH */
