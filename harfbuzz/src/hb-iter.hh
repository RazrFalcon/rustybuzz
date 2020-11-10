/*
 * Copyright © 2018  Google, Inc.
 * Copyright © 2019  Facebook, Inc.
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
 * Facebook Author(s): Behdad Esfahbod
 */

#ifndef RB_ITER_HH
#define RB_ITER_HH

#include "hb.hh"
#include "hb-algs.hh"
#include "hb-meta.hh"

/* Unified iterator object.
 *
 * The goal of this template is to make the same iterator interface
 * available to all types, and make it very easy and compact to use.
 * rb_iter_tator objects are small, light-weight, objects that can be
 * copied by value.  If the collection / object being iterated on
 * is writable, then the iterator returns lvalues, otherwise it
 * returns rvalues.
 *
 * TODO Document more.
 *
 * If iterator implementation implements operator!=, then can be
 * used in range-based for loop.  That comes free if the iterator
 * is random-access.  Otherwise, the range-based for loop incurs
 * one traversal to find end(), which can be avoided if written
 * as a while-style for loop, or if iterator implements a faster
 * __end__() method.
 * TODO When opting in for C++17, address this by changing return
 * type of .end()?
 */

/*
 * Base classes for iterators.
 */

/* Base class for all iterators. */
template <typename iter_t, typename Item = typename iter_t::__item_t__> struct rb_iter_t
{
    typedef Item item_t;
    constexpr unsigned get_item_size() const
    {
        return rb_static_size(Item);
    }
    static constexpr bool is_iterator = true;
    static constexpr bool is_random_access_iterator = false;
    static constexpr bool is_sorted_iterator = false;

private:
    /* https://en.wikipedia.org/wiki/Curiously_recurring_template_pattern */
    const iter_t *thiz() const
    {
        return static_cast<const iter_t *>(this);
    }
    iter_t *thiz()
    {
        return static_cast<iter_t *>(this);
    }

public:
    /* TODO:
     * Port operators below to use rb_enable_if to sniff which method implements
     * an operator and use it, and remove rb_iter_fallback_mixin_t completely. */

    /* Operators. */
    iter_t iter() const
    {
        return *thiz();
    }
    iter_t operator+() const
    {
        return *thiz();
    }
    iter_t begin() const
    {
        return *thiz();
    }
    iter_t end() const
    {
        return thiz()->__end__();
    }
    explicit operator bool() const
    {
        return thiz()->__more__();
    }
    unsigned len() const
    {
        return thiz()->__len__();
    }
    /* The following can only be enabled if item_t is reference type.  Otherwise
     * it will be returning pointer to temporary rvalue.
     * TODO Use a wrapper return type to fix for non-reference type. */
    template <typename T = item_t, rb_enable_if(rb_is_reference(T))> rb_remove_reference<item_t> *operator->() const
    {
        return rb_addressof(**thiz());
    }
    item_t operator*() const
    {
        return thiz()->__item__();
    }
    item_t operator*()
    {
        return thiz()->__item__();
    }
    item_t operator[](unsigned i) const
    {
        return thiz()->__item_at__(i);
    }
    item_t operator[](unsigned i)
    {
        return thiz()->__item_at__(i);
    }
    iter_t &operator+=(unsigned count) &
    {
        thiz()->__forward__(count);
        return *thiz();
    }
    iter_t operator+=(unsigned count) &&
    {
        thiz()->__forward__(count);
        return *thiz();
    }
    iter_t &operator++() &
    {
        thiz()->__next__();
        return *thiz();
    }
    iter_t operator++() &&
    {
        thiz()->__next__();
        return *thiz();
    }
    iter_t &operator-=(unsigned count) &
    {
        thiz()->__rewind__(count);
        return *thiz();
    }
    iter_t operator-=(unsigned count) &&
    {
        thiz()->__rewind__(count);
        return *thiz();
    }
    iter_t &operator--() &
    {
        thiz()->__prev__();
        return *thiz();
    }
    iter_t operator--() &&
    {
        thiz()->__prev__();
        return *thiz();
    }
    iter_t operator+(unsigned count) const
    {
        auto c = thiz()->iter();
        c += count;
        return c;
    }
    friend iter_t operator+(unsigned count, const iter_t &it)
    {
        return it + count;
    }
    iter_t operator++(int)
    {
        iter_t c(*thiz());
        ++*thiz();
        return c;
    }
    iter_t operator-(unsigned count) const
    {
        auto c = thiz()->iter();
        c -= count;
        return c;
    }
    iter_t operator--(int)
    {
        iter_t c(*thiz());
        --*thiz();
        return c;
    }
    template <typename T> iter_t &operator>>(T &v) &
    {
        v = **thiz();
        ++*thiz();
        return *thiz();
    }
    template <typename T> iter_t operator>>(T &v) &&
    {
        v = **thiz();
        ++*thiz();
        return *thiz();
    }
    template <typename T> iter_t &operator<<(const T v) &
    {
        **thiz() = v;
        ++*thiz();
        return *thiz();
    }
    template <typename T> iter_t operator<<(const T v) &&
    {
        **thiz() = v;
        ++*thiz();
        return *thiz();
    }

protected:
    rb_iter_t() = default;
    rb_iter_t(const rb_iter_t &o RB_UNUSED) = default;
    rb_iter_t(rb_iter_t &&o RB_UNUSED) = default;
    rb_iter_t &operator=(const rb_iter_t &o RB_UNUSED) = default;
    rb_iter_t &operator=(rb_iter_t &&o RB_UNUSED) = default;
};

#define RB_ITER_USING(Name)                                                                                            \
    using item_t = typename Name::item_t;                                                                              \
    using Name::begin;                                                                                                 \
    using Name::end;                                                                                                   \
    using Name::get_item_size;                                                                                         \
    using Name::is_iterator;                                                                                           \
    using Name::iter;                                                                                                  \
    using Name::operator bool;                                                                                         \
    using Name::len;                                                                                                   \
    using Name::operator->;                                                                                            \
    using Name::operator*;                                                                                             \
    using Name::operator[];                                                                                            \
    using Name::operator+=;                                                                                            \
    using Name::operator++;                                                                                            \
    using Name::operator-=;                                                                                            \
    using Name::operator--;                                                                                            \
    using Name::operator+;                                                                                             \
    using Name::operator-;                                                                                             \
    using Name::operator>>;                                                                                            \
    using Name::operator<<;                                                                                            \
    static_assert(true, "")

/* Returns iterator / item type of a type. */
template <typename Iterable> using rb_iter_type = decltype(rb_deref(rb_declval(Iterable)).iter());

template <typename> struct rb_array_t;
template <typename> struct rb_sorted_array_t;

struct
{
    template <typename T> rb_iter_type<T> operator()(T &&c) const
    {
        return rb_deref(rb_forward<T>(c)).iter();
    }

    /* Specialization for C arrays. */

    template <typename Type> inline rb_array_t<Type> operator()(Type *array, unsigned int length) const
    {
        return rb_array_t<Type>(array, length);
    }

    template <typename Type, unsigned int length> rb_array_t<Type> operator()(Type (&array)[length]) const
    {
        return rb_array_t<Type>(array, length);
    }

} RB_FUNCOBJ(rb_iter);

/* Mixin to fill in what the subclass doesn't provide. */
template <typename iter_t, typename item_t = typename iter_t::__item_t__> struct rb_iter_fallback_mixin_t
{
private:
    /* https://en.wikipedia.org/wiki/Curiously_recurring_template_pattern */
    const iter_t *thiz() const
    {
        return static_cast<const iter_t *>(this);
    }
    iter_t *thiz()
    {
        return static_cast<iter_t *>(this);
    }

public:
    /* Access: Implement __item__(), or __item_at__() if random-access. */
    item_t __item__() const
    {
        return (*thiz())[0];
    }
    item_t __item_at__(unsigned i) const
    {
        return *(*thiz() + i);
    }

    /* Termination: Implement __more__(), or __len__() if random-access. */
    bool __more__() const
    {
        return bool(thiz()->len());
    }
    unsigned __len__() const
    {
        iter_t c(*thiz());
        unsigned l = 0;
        while (c) {
            c++;
            l++;
        }
        return l;
    }

    /* Advancing: Implement __next__(), or __forward__() if random-access. */
    void __next__()
    {
        *thiz() += 1;
    }
    void __forward__(unsigned n)
    {
        while (*thiz() && n--)
            ++*thiz();
    }

    /* Rewinding: Implement __prev__() or __rewind__() if bidirectional. */
    void __prev__()
    {
        *thiz() -= 1;
    }
    void __rewind__(unsigned n)
    {
        while (*thiz() && n--)
            --*thiz();
    }

    /* Range-based for: Implement __end__() if can be done faster,
     * and operator!=. */
    iter_t __end__() const
    {
        if (thiz()->is_random_access_iterator)
            return *thiz() + thiz()->len();
        /* Above expression loops twice. Following loops once. */
        auto it = *thiz();
        while (it)
            ++it;
        return it;
    }

protected:
    rb_iter_fallback_mixin_t() = default;
    rb_iter_fallback_mixin_t(const rb_iter_fallback_mixin_t &o RB_UNUSED) = default;
    rb_iter_fallback_mixin_t(rb_iter_fallback_mixin_t &&o RB_UNUSED) = default;
    rb_iter_fallback_mixin_t &operator=(const rb_iter_fallback_mixin_t &o RB_UNUSED) = default;
    rb_iter_fallback_mixin_t &operator=(rb_iter_fallback_mixin_t &&o RB_UNUSED) = default;
};

template <typename iter_t, typename item_t = typename iter_t::__item_t__>
struct rb_iter_with_fallback_t : rb_iter_t<iter_t, item_t>, rb_iter_fallback_mixin_t<iter_t, item_t>
{
protected:
    rb_iter_with_fallback_t() = default;
    rb_iter_with_fallback_t(const rb_iter_with_fallback_t &o RB_UNUSED) = default;
    rb_iter_with_fallback_t(rb_iter_with_fallback_t &&o RB_UNUSED) = default;
    rb_iter_with_fallback_t &operator=(const rb_iter_with_fallback_t &o RB_UNUSED) = default;
    rb_iter_with_fallback_t &operator=(rb_iter_with_fallback_t &&o RB_UNUSED) = default;
};

/*
 * Meta-programming predicates.
 */

/* rb_is_iterator() / rb_is_iterator_of() */

template <typename Iter, typename Item> struct rb_is_iterator_of
{
    template <typename Item2 = Item>
    static rb_true_type impl(rb_priority<2>, rb_iter_t<Iter, rb_type_identity<Item2>> *);
    static rb_false_type impl(rb_priority<0>, const void *);

public:
    static constexpr bool value = decltype(impl(rb_prioritize, rb_declval(Iter *)))::value;
};
#define rb_is_iterator_of(Iter, Item) rb_is_iterator_of<Iter, Item>::value
#define rb_is_iterator(Iter) rb_is_iterator_of(Iter, typename Iter::item_t)

/*
 * Adaptors, combiners, etc.
 */

template <typename Lhs, typename Rhs, rb_requires(rb_is_iterator(Lhs))>
static inline auto operator|(Lhs &&lhs, Rhs &&rhs) RB_AUTO_RETURN(rb_forward<Rhs>(rhs)(rb_forward<Lhs>(lhs)))

/* rb_map() */

enum class rb_function_sortedness_t {
    NOT_SORTED,
    RETAINS_SORTING,
    SORTED,
};

template <typename Iter, typename Proj, rb_function_sortedness_t Sorted, rb_requires(rb_is_iterator(Iter))>
struct rb_map_iter_t
    : rb_iter_t<rb_map_iter_t<Iter, Proj, Sorted>, decltype(rb_get(rb_declval(Proj), *rb_declval(Iter)))>
{
    rb_map_iter_t(const Iter &it, Proj f_)
        : it(it)
        , f(f_)
    {
    }

    typedef decltype(rb_get(rb_declval(Proj), *rb_declval(Iter))) __item_t__;
    static constexpr bool is_random_access_iterator = Iter::is_random_access_iterator;
    static constexpr bool is_sorted_iterator =
        Sorted == rb_function_sortedness_t::SORTED
            ? true
            : Sorted == rb_function_sortedness_t::RETAINS_SORTING ? Iter::is_sorted_iterator : false;
    __item_t__ __item__() const
    {
        return rb_get(f.get(), *it);
    }
    __item_t__ __item_at__(unsigned i) const
    {
        return rb_get(f.get(), it[i]);
    }
    bool __more__() const
    {
        return bool(it);
    }
    unsigned __len__() const
    {
        return it.len();
    }
    void __next__()
    {
        ++it;
    }
    void __forward__(unsigned n)
    {
        it += n;
    }
    void __prev__()
    {
        --it;
    }
    void __rewind__(unsigned n)
    {
        it -= n;
    }
    rb_map_iter_t __end__() const
    {
        return rb_map_iter_t(it.end(), f);
    }
    bool operator!=(const rb_map_iter_t &o) const
    {
        return it != o.it;
    }

private:
    Iter it;
    rb_reference_wrapper<Proj> f;
};

template <typename Proj, rb_function_sortedness_t Sorted> struct rb_map_iter_factory_t
{
    rb_map_iter_factory_t(Proj f)
        : f(f)
    {
    }

    template <typename Iter, rb_requires(rb_is_iterator(Iter))> rb_map_iter_t<Iter, Proj, Sorted> operator()(Iter it)
    {
        return rb_map_iter_t<Iter, Proj, Sorted>(it, f);
    }

private:
    Proj f;
};
struct
{
    template <typename Proj>
    rb_map_iter_factory_t<Proj, rb_function_sortedness_t::NOT_SORTED> operator()(Proj &&f) const
    {
        return rb_map_iter_factory_t<Proj, rb_function_sortedness_t::NOT_SORTED>(f);
    }
} RB_FUNCOBJ(rb_map);

/* rb_sink() */

template <typename Sink> struct rb_sink_t
{
    rb_sink_t(Sink s)
        : s(s)
    {
    }

    template <typename Iter, rb_requires(rb_is_iterator(Iter))> void operator()(Iter it)
    {
        for (; it; ++it)
            s << *it;
    }

private:
    Sink s;
};
struct
{
    template <typename Sink> rb_sink_t<Sink> operator()(Sink &&s) const
    {
        return rb_sink_t<Sink>(s);
    }

    template <typename Sink> rb_sink_t<Sink &> operator()(Sink *s) const
    {
        return rb_sink_t<Sink &>(*s);
    }
} RB_FUNCOBJ(rb_sink);

/*
 * Algorithms operating on iterators.
 */

template <typename S, typename D> inline void rb_copy(S &&is, D &&id)
{
    rb_iter(is) | rb_sink(id);
}

#endif /* RB_ITER_HH */
