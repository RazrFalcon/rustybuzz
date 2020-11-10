/*
 * Copyright © 2007  Chris Wilson
 * Copyright © 2009,2010  Red Hat, Inc.
 * Copyright © 2011,2012  Google, Inc.
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
 * Contributor(s):
 *	Chris Wilson <chris@chris-wilson.co.uk>
 * Red Hat Author(s): Behdad Esfahbod
 * Google Author(s): Behdad Esfahbod
 */

#ifndef RB_OBJECT_HH
#define RB_OBJECT_HH

#include "hb.hh"
#include "hb-atomic.hh"
#include "hb-vector.hh"

/*
 * Reference-count.
 */

#define RB_REFERENCE_COUNT_INERT_VALUE 0
#define RB_REFERENCE_COUNT_POISON_VALUE -0x0000DEAD
#define RB_REFERENCE_COUNT_INIT                                                                                        \
    {                                                                                                                  \
        RB_ATOMIC_INT_INIT(RB_REFERENCE_COUNT_INERT_VALUE)                                                             \
    }

struct rb_reference_count_t
{
    mutable rb_atomic_int_t ref_count;

    void init(int v = 1)
    {
        ref_count.set_relaxed(v);
    }
    int get_relaxed() const
    {
        return ref_count.get_relaxed();
    }
    int inc() const
    {
        return ref_count.inc();
    }
    int dec() const
    {
        return ref_count.dec();
    }
    void fini()
    {
        ref_count.set_relaxed(RB_REFERENCE_COUNT_POISON_VALUE);
    }

    bool is_inert() const
    {
        return ref_count.get_relaxed() == RB_REFERENCE_COUNT_INERT_VALUE;
    }
    bool is_valid() const
    {
        return ref_count.get_relaxed() > 0;
    }
};

/*
 * Object header
 */

struct rb_object_header_t
{
    rb_reference_count_t ref_count;
    mutable rb_atomic_int_t writable;
};
#define RB_OBJECT_HEADER_STATIC                                                                                        \
    {                                                                                                                  \
        RB_REFERENCE_COUNT_INIT, RB_ATOMIC_INT_INIT(false)                                                             \
    }

/*
 * Object
 */

template <typename Type> static inline void rb_object_trace(const Type *obj, const char *function)
{
    //    DEBUG_MSG(OBJECT, (void *)obj, "%s refcount=%d", function, obj ? obj->header.ref_count.get_relaxed() : 0);
}

template <typename Type> static inline Type *rb_object_create()
{
    Type *obj = (Type *)calloc(1, sizeof(Type));

    if (unlikely(!obj))
        return obj;

    rb_object_init(obj);
    rb_object_trace(obj, RB_FUNC);
    return obj;
}
template <typename Type> static inline void rb_object_init(Type *obj)
{
    obj->header.ref_count.init();
    obj->header.writable.set_relaxed(true);
}
template <typename Type> static inline bool rb_object_is_inert(const Type *obj)
{
    return unlikely(obj->header.ref_count.is_inert());
}
template <typename Type> static inline bool rb_object_is_valid(const Type *obj)
{
    return likely(obj->header.ref_count.is_valid());
}
template <typename Type> static inline Type *rb_object_reference(Type *obj)
{
    rb_object_trace(obj, RB_FUNC);
    if (unlikely(!obj || rb_object_is_inert(obj)))
        return obj;
    assert(rb_object_is_valid(obj));
    obj->header.ref_count.inc();
    return obj;
}
template <typename Type> static inline bool rb_object_destroy(Type *obj)
{
    rb_object_trace(obj, RB_FUNC);
    if (unlikely(!obj || rb_object_is_inert(obj)))
        return false;
    assert(rb_object_is_valid(obj));
    if (obj->header.ref_count.dec() != 1)
        return false;

    rb_object_fini(obj);
    return true;
}
template <typename Type> static inline void rb_object_fini(Type *obj)
{
    obj->header.ref_count.fini(); /* Do this before user_data */
}
#endif /* RB_OBJECT_HH */
