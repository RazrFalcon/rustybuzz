/*
 * Copyright © 2012  Google, Inc.
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

#include "hb-set.hh"

/**
 * SECTION:hb-set
 * @title: hb-set
 * @short_description: Object representing a set of integers
 * @include: hb.h
 *
 * Set objects represent a mathematical set of integer values.  They are
 * used in non-shaping API to query certain set of characters or glyphs,
 * or other integer values.
 **/

/**
 * rb_set_create: (Xconstructor)
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.2
 **/
rb_set_t *rb_set_create()
{
    rb_set_t *set;

    if (!(set = rb_object_create<rb_set_t>()))
        return rb_set_get_empty();

    set->init_shallow();

    return set;
}

/**
 * rb_set_get_empty:
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.2
 **/
rb_set_t *rb_set_get_empty()
{
    return const_cast<rb_set_t *>(&Null(rb_set_t));
}

/**
 * rb_set_reference: (skip)
 * @set: a set.
 *
 * Return value: (transfer full):
 *
 * Since: 0.9.2
 **/
rb_set_t *rb_set_reference(rb_set_t *set)
{
    return rb_object_reference(set);
}

/**
 * rb_set_destroy: (skip)
 * @set: a set.
 *
 * Since: 0.9.2
 **/
void rb_set_destroy(rb_set_t *set)
{
    if (!rb_object_destroy(set))
        return;

    set->fini_shallow();

    free(set);
}

/**
 * rb_set_allocation_successful:
 * @set: a set.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
rb_bool_t rb_set_allocation_successful(const rb_set_t *set)
{
    return set->successful;
}

/**
 * rb_set_clear:
 * @set: a set.
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_set_clear(rb_set_t *set)
{
    set->clear();
}

/**
 * rb_set_is_empty:
 * @set: a set.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.7
 **/
rb_bool_t rb_set_is_empty(const rb_set_t *set)
{
    return set->is_empty();
}

/**
 * rb_set_has:
 * @set: a set.
 * @codepoint:
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.2
 **/
rb_bool_t rb_set_has(const rb_set_t *set, rb_codepoint_t codepoint)
{
    return set->has(codepoint);
}

/**
 * rb_set_add:
 * @set: a set.
 * @codepoint:
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_set_add(rb_set_t *set, rb_codepoint_t codepoint)
{
    set->add(codepoint);
}

/**
 * rb_set_add_range:
 * @set: a set.
 * @first:
 * @last:
 *
 *
 *
 * Since: 0.9.7
 **/
void rb_set_add_range(rb_set_t *set, rb_codepoint_t first, rb_codepoint_t last)
{
    set->add_range(first, last);
}

/**
 * rb_set_del:
 * @set: a set.
 * @codepoint:
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_set_del(rb_set_t *set, rb_codepoint_t codepoint)
{
    set->del(codepoint);
}

/**
 * rb_set_del_range:
 * @set: a set.
 * @first:
 * @last:
 *
 *
 *
 * Since: 0.9.7
 **/
void rb_set_del_range(rb_set_t *set, rb_codepoint_t first, rb_codepoint_t last)
{
    set->del_range(first, last);
}

/**
 * rb_set_is_equal:
 * @set: a set.
 * @other: other set.
 *
 *
 *
 * Return value: %TRUE if the two sets are equal, %FALSE otherwise.
 *
 * Since: 0.9.7
 **/
rb_bool_t rb_set_is_equal(const rb_set_t *set, const rb_set_t *other)
{
    return set->is_equal(other);
}

/**
 * rb_set_is_subset:
 * @set: a set.
 * @larger_set: other set.
 *
 *
 *
 * Return value: %TRUE if the @set is a subset of (or equal to) @larger_set, %FALSE otherwise.
 *
 * Since: 1.8.1
 **/
rb_bool_t rb_set_is_subset(const rb_set_t *set, const rb_set_t *larger_set)
{
    return set->is_subset(larger_set);
}

/**
 * rb_set_set:
 * @set: a set.
 * @other:
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_set_set(rb_set_t *set, const rb_set_t *other)
{
    set->set(other);
}

/**
 * rb_set_union:
 * @set: a set.
 * @other:
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_set_union(rb_set_t *set, const rb_set_t *other)
{
    set->union_(other);
}

/**
 * rb_set_intersect:
 * @set: a set.
 * @other:
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_set_intersect(rb_set_t *set, const rb_set_t *other)
{
    set->intersect(other);
}

/**
 * rb_set_subtract:
 * @set: a set.
 * @other:
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_set_subtract(rb_set_t *set, const rb_set_t *other)
{
    set->subtract(other);
}

/**
 * rb_set_symmetric_difference:
 * @set: a set.
 * @other:
 *
 *
 *
 * Since: 0.9.2
 **/
void rb_set_symmetric_difference(rb_set_t *set, const rb_set_t *other)
{
    set->symmetric_difference(other);
}

/**
 * rb_set_get_population:
 * @set: a set.
 *
 * Returns the number of numbers in the set.
 *
 * Return value: set population.
 *
 * Since: 0.9.7
 **/
unsigned int rb_set_get_population(const rb_set_t *set)
{
    return set->get_population();
}

/**
 * rb_set_get_min:
 * @set: a set.
 *
 * Finds the minimum number in the set.
 *
 * Return value: minimum of the set, or %RB_SET_VALUE_INVALID if set is empty.
 *
 * Since: 0.9.7
 **/
rb_codepoint_t rb_set_get_min(const rb_set_t *set)
{
    return set->get_min();
}

/**
 * rb_set_get_max:
 * @set: a set.
 *
 * Finds the maximum number in the set.
 *
 * Return value: minimum of the set, or %RB_SET_VALUE_INVALID if set is empty.
 *
 * Since: 0.9.7
 **/
rb_codepoint_t rb_set_get_max(const rb_set_t *set)
{
    return set->get_max();
}

/**
 * rb_set_next:
 * @set: a set.
 * @codepoint: (inout):
 *
 * Gets the next number in @set that is greater than current value of @codepoint.
 *
 * Set @codepoint to %RB_SET_VALUE_INVALID to get started.
 *
 * Return value: whether there was a next value.
 *
 * Since: 0.9.2
 **/
rb_bool_t rb_set_next(const rb_set_t *set, rb_codepoint_t *codepoint)
{
    return set->next(codepoint);
}

/**
 * rb_set_previous:
 * @set: a set.
 * @codepoint: (inout):
 *
 * Gets the previous number in @set that is lower than current value of @codepoint.
 *
 * Set @codepoint to %RB_SET_VALUE_INVALID to get started.
 *
 * Return value: whether there was a previous value.
 *
 * Since: 1.8.0
 **/
rb_bool_t rb_set_previous(const rb_set_t *set, rb_codepoint_t *codepoint)
{
    return set->previous(codepoint);
}

/**
 * rb_set_next_range:
 * @set: a set.
 * @first: (out): output first codepoint in the range.
 * @last: (inout): input current last and output last codepoint in the range.
 *
 * Gets the next consecutive range of numbers in @set that
 * are greater than current value of @last.
 *
 * Set @last to %RB_SET_VALUE_INVALID to get started.
 *
 * Return value: whether there was a next range.
 *
 * Since: 0.9.7
 **/
rb_bool_t rb_set_next_range(const rb_set_t *set, rb_codepoint_t *first, rb_codepoint_t *last)
{
    return set->next_range(first, last);
}

/**
 * rb_set_previous_range:
 * @set: a set.
 * @first: (inout): input current first and output first codepoint in the range.
 * @last: (out): output last codepoint in the range.
 *
 * Gets the previous consecutive range of numbers in @set that
 * are less than current value of @first.
 *
 * Set @first to %RB_SET_VALUE_INVALID to get started.
 *
 * Return value: whether there was a previous range.
 *
 * Since: 1.8.0
 **/
rb_bool_t rb_set_previous_range(const rb_set_t *set, rb_codepoint_t *first, rb_codepoint_t *last)
{
    return set->previous_range(first, last);
}
