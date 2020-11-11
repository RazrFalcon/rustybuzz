/*
 * Copyright © 2018  Google, Inc.
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

#ifndef RB_ARRAY_HH
#define RB_ARRAY_HH

#include "hb.hh"
#include "hb-algs.hh"
#include "hb-iter.hh"
#include "hb-null.hh"

template <typename Type> struct rb_sorted_array_t;

template <typename Type> struct rb_array_t : rb_iter_with_fallback_t<rb_array_t<Type>, Type &>
{
    /*
     * Constructors.
     */
    rb_array_t()
        : arrayZ(nullptr)
        , length(0)
        , backwards_length(0)
    {
    }
    rb_array_t(Type *array_, unsigned int length_)
        : arrayZ(array_)
        , length(length_)
        , backwards_length(0)
    {
    }
    template <unsigned int length_>
    rb_array_t(Type (&array_)[length_])
        : arrayZ(array_)
        , length(length_)
        , backwards_length(0)
    {
    }

    template <typename U, rb_enable_if(rb_is_cr_convertible(U, Type))>
    rb_array_t(const rb_array_t<U> &o)
        : rb_iter_with_fallback_t<rb_array_t, Type &>()
        , arrayZ(o.arrayZ)
        , length(o.length)
        , backwards_length(o.backwards_length)
    {
    }

    /*
     * Iterator implementation.
     */
    typedef Type &__item_t__;
    static constexpr bool is_random_access_iterator = true;
    Type &__item_at__(unsigned i) const
    {
        if (unlikely(i >= length))
            return CrapOrNull(Type);
        return arrayZ[i];
    }
    void __forward__(unsigned n)
    {
        if (unlikely(n > length))
            n = length;
        length -= n;
        backwards_length += n;
        arrayZ += n;
    }
    void __rewind__(unsigned n)
    {
        if (unlikely(n > backwards_length))
            n = backwards_length;
        length += n;
        backwards_length -= n;
        arrayZ -= n;
    }
    unsigned __len__() const
    {
        return length;
    }
    /* Ouch. The operator== compares the contents of the array.  For range-based for loops,
     * it's best if we can just compare arrayZ, though comparing contents is still fast,
     * but also would require that Type has operator==.  As such, we optimize this operator
     * for range-based for loop and just compare arrayZ.  No need to compare length, as we
     * assume we're only compared to .end(). */
    bool operator!=(const rb_array_t &o) const
    {
        return arrayZ != o.arrayZ;
    }

    /* Extra operators.
     */
    Type *operator&() const
    {
        return arrayZ;
    }
    operator rb_array_t<const Type>()
    {
        return rb_array_t<const Type>(arrayZ, length);
    }
    template <typename T> operator T *() const
    {
        return arrayZ;
    }

    RB_INTERNAL bool operator==(const rb_array_t &o) const;

    uint32_t hash() const
    {
        uint32_t current = 0;
        for (unsigned int i = 0; i < this->length; i++) {
            current = current * 31 + rb_hash(this->arrayZ[i]);
        }
        return current;
    }

    /*
     * Compare, Sort, and Search.
     */

    template <typename T> Type *lsearch(const T &x, Type *not_found = nullptr)
    {
        unsigned i;
        return lfind(x, &i) ? &this->arrayZ[i] : not_found;
    }
    template <typename T> const Type *lsearch(const T &x, const Type *not_found = nullptr) const
    {
        unsigned i;
        return lfind(x, &i) ? &this->arrayZ[i] : not_found;
    }
    template <typename T> bool lfind(const T &x, unsigned *pos = nullptr) const
    {
        for (unsigned i = 0; i < length; ++i)
            if (!this->arrayZ[i].cmp(x)) {
                if (pos)
                    *pos = i;
                return true;
            }

        return false;
    }

    rb_sorted_array_t<Type> qsort(int (*cmp_)(const void *, const void *))
    {
        if (likely(length))
            rb_qsort(arrayZ, length, this->get_item_size(), cmp_);
        return rb_sorted_array_t<Type>(*this);
    }
    rb_sorted_array_t<Type> qsort()
    {
        if (likely(length))
            rb_qsort(arrayZ, length, this->get_item_size(), Type::cmp);
        return rb_sorted_array_t<Type>(*this);
    }
    void qsort(unsigned int start, unsigned int end)
    {
        end = rb_min(end, length);
        assert(start <= end);
        if (likely(start < end))
            rb_qsort(arrayZ + start, end - start, this->get_item_size(), Type::cmp);
    }

    /*
     * Other methods.
     */

    unsigned int get_size() const
    {
        return length * this->get_item_size();
    }

    rb_array_t sub_array(unsigned int start_offset = 0, unsigned int *seg_count = nullptr /* IN/OUT */) const
    {
        if (!start_offset && !seg_count)
            return *this;

        unsigned int count = length;
        if (unlikely(start_offset > count))
            count = 0;
        else
            count -= start_offset;
        if (seg_count)
            count = *seg_count = rb_min(count, *seg_count);
        return rb_array_t(arrayZ + start_offset, count);
    }

    template <typename T, unsigned P = sizeof(Type), rb_enable_if(P == 1)> const T *as() const
    {
        return length < rb_null_size(T) ? &Null(T) : reinterpret_cast<const T *>(arrayZ);
    }

    template <typename rb_sanitize_context_t> bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_array(arrayZ, length);
    }

    /*
     * Members
     */

public:
    Type *arrayZ;
    unsigned int length;
    unsigned int backwards_length;
};
template <typename T> inline rb_array_t<T> rb_array(T *array, unsigned int length)
{
    return rb_array_t<T>(array, length);
}
template <typename T, unsigned int length_> inline rb_array_t<T> rb_array(T (&array_)[length_])
{
    return rb_array_t<T>(array_);
}

enum rb_bfind_not_found_t {
    RB_BFIND_NOT_FOUND_DONT_STORE,
    RB_BFIND_NOT_FOUND_STORE,
    RB_BFIND_NOT_FOUND_STORE_CLOSEST,
};

template <typename Type> struct rb_sorted_array_t : rb_iter_t<rb_sorted_array_t<Type>, Type &>, rb_array_t<Type>
{
    typedef rb_iter_t<rb_sorted_array_t, Type &> iter_base_t;
    RB_ITER_USING(iter_base_t);
    static constexpr bool is_random_access_iterator = true;
    static constexpr bool is_sorted_iterator = true;

    rb_sorted_array_t()
        : rb_array_t<Type>()
    {
    }
    rb_sorted_array_t(Type *array_, unsigned int length_)
        : rb_array_t<Type>(array_, length_)
    {
    }
    template <unsigned int length_>
    rb_sorted_array_t(Type (&array_)[length_])
        : rb_array_t<Type>(array_)
    {
    }

    template <typename U, rb_enable_if(rb_is_cr_convertible(U, Type))>
    rb_sorted_array_t(const rb_array_t<U> &o)
        : rb_iter_t<rb_sorted_array_t, Type &>()
        , rb_array_t<Type>(o)
    {
    }

    template <typename T> Type *bsearch(const T &x, Type *not_found = nullptr)
    {
        unsigned int i;
        return bfind(x, &i) ? &this->arrayZ[i] : not_found;
    }
    template <typename T> const Type *bsearch(const T &x, const Type *not_found = nullptr) const
    {
        unsigned int i;
        return bfind(x, &i) ? &this->arrayZ[i] : not_found;
    }
    template <typename T>
    bool bfind(const T &x,
               unsigned int *i = nullptr,
               rb_bfind_not_found_t not_found = RB_BFIND_NOT_FOUND_DONT_STORE,
               unsigned int to_store = (unsigned int)-1) const
    {
        unsigned pos;

        if (bsearch_impl(x, &pos)) {
            if (i)
                *i = pos;
            return true;
        }

        if (i) {
            switch (not_found) {
            case RB_BFIND_NOT_FOUND_DONT_STORE:
                break;

            case RB_BFIND_NOT_FOUND_STORE:
                *i = to_store;
                break;

            case RB_BFIND_NOT_FOUND_STORE_CLOSEST:
                *i = pos;
                break;
            }
        }
        return false;
    }
    template <typename T> bool bsearch_impl(const T &x, unsigned *pos) const
    {
        return rb_bsearch_impl(pos, x, this->arrayZ, this->length, sizeof(Type), _rb_cmp_method<T, Type>);
    }
};
template <typename T> inline rb_sorted_array_t<T> rb_sorted_array(T *array, unsigned int length)
{
    return rb_sorted_array_t<T>(array, length);
}
template <typename T, unsigned int length_> inline rb_sorted_array_t<T> rb_sorted_array(T (&array_)[length_])
{
    return rb_sorted_array_t<T>(array_);
}

typedef rb_array_t<const char> rb_bytes_t;

#endif /* RB_ARRAY_HH */
