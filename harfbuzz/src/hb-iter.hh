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
template <typename Iterable> using rb_item_type = decltype(*rb_deref(rb_declval(Iterable)).iter());

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
struct
{
    template <typename T> unsigned operator()(T &&c) const
    {
        return c.len();
    }

} RB_FUNCOBJ(rb_len);

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

/* rb_is_iterable() */

template <typename T> struct rb_is_iterable
{
private:
    template <typename U> static auto impl(rb_priority<1>) -> decltype(rb_declval(U).iter(), rb_true_type());

    template <typename> static rb_false_type impl(rb_priority<0>);

public:
    static constexpr bool value = decltype(impl<T>(rb_prioritize))::value;
};
#define rb_is_iterable(Iterable) rb_is_iterable<Iterable>::value

/* rb_is_source_of() / rb_is_sink_of() */

template <typename Iter, typename Item> struct rb_is_source_of
{
private:
    template <typename Iter2 = Iter,
              rb_enable_if(rb_is_convertible(typename Iter2::item_t, rb_add_lvalue_reference<rb_add_const<Item>>))>
    static rb_true_type impl(rb_priority<2>);
    template <typename Iter2 = Iter>
    static auto impl(rb_priority<1>) -> decltype(rb_declval(Iter2) >> rb_declval(Item &), rb_true_type());
    static rb_false_type impl(rb_priority<0>);

public:
    static constexpr bool value = decltype(impl(rb_prioritize))::value;
};
#define rb_is_source_of(Iter, Item) rb_is_source_of<Iter, Item>::value

template <typename Iter, typename Item> struct rb_is_sink_of
{
private:
    template <typename Iter2 = Iter,
              rb_enable_if(rb_is_convertible(typename Iter2::item_t, rb_add_lvalue_reference<Item>))>
    static rb_true_type impl(rb_priority<2>);
    template <typename Iter2 = Iter>
    static auto impl(rb_priority<1>) -> decltype(rb_declval(Iter2) << rb_declval(Item), rb_true_type());
    static rb_false_type impl(rb_priority<0>);

public:
    static constexpr bool value = decltype(impl(rb_prioritize))::value;
};
#define rb_is_sink_of(Iter, Item) rb_is_sink_of<Iter, Item>::value

/* This is commonly used, so define: */
#define rb_is_sorted_source_of(Iter, Item) (rb_is_source_of(Iter, Item) && Iter::is_sorted_iterator)

/* Range-based 'for' for iterables. */

template <typename Iterable, rb_requires(rb_is_iterable(Iterable))>
static inline auto begin(Iterable &&iterable) RB_AUTO_RETURN(rb_iter(iterable).begin())

    template <typename Iterable, rb_requires(rb_is_iterable(Iterable))>
    static inline auto end(Iterable &&iterable) RB_AUTO_RETURN(rb_iter(iterable).end())

    /* begin()/end() are NOT looked up non-ADL.  So each namespace must declare them.
     * Do it for namespace OT. */
    namespace OT
{

    template <typename Iterable, rb_requires(rb_is_iterable(Iterable))>
    static inline auto begin(Iterable && iterable) RB_AUTO_RETURN(rb_iter(iterable).begin())

        template <typename Iterable, rb_requires(rb_is_iterable(Iterable))>
        static inline auto end(Iterable && iterable) RB_AUTO_RETURN(rb_iter(iterable).end())
}

/*
 * Adaptors, combiners, etc.
 */

template <typename Lhs, typename Rhs, rb_requires(rb_is_iterator(Lhs))>
static inline auto operator|(Lhs &&lhs, Rhs &&rhs) RB_AUTO_RETURN(rb_forward<Rhs>(rhs)(rb_forward<Lhs>(lhs)))

    /* rb_map(), rb_filter(), rb_reduce() */

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
struct
{
    template <typename Proj>
    rb_map_iter_factory_t<Proj, rb_function_sortedness_t::RETAINS_SORTING> operator()(Proj &&f) const
    {
        return rb_map_iter_factory_t<Proj, rb_function_sortedness_t::RETAINS_SORTING>(f);
    }
} RB_FUNCOBJ(rb_map_retains_sorting);
struct
{
    template <typename Proj> rb_map_iter_factory_t<Proj, rb_function_sortedness_t::SORTED> operator()(Proj &&f) const
    {
        return rb_map_iter_factory_t<Proj, rb_function_sortedness_t::SORTED>(f);
    }
} RB_FUNCOBJ(rb_map_sorted);

template <typename Iter, typename Pred, typename Proj, rb_requires(rb_is_iterator(Iter))>
struct rb_filter_iter_t : rb_iter_with_fallback_t<rb_filter_iter_t<Iter, Pred, Proj>, typename Iter::item_t>
{
    rb_filter_iter_t(const Iter &it_, Pred p_, Proj f_)
        : it(it_)
        , p(p_)
        , f(f_)
    {
        while (it && !rb_has(p.get(), rb_get(f.get(), *it)))
            ++it;
    }

    typedef typename Iter::item_t __item_t__;
    static constexpr bool is_sorted_iterator = Iter::is_sorted_iterator;
    __item_t__ __item__() const
    {
        return *it;
    }
    bool __more__() const
    {
        return bool(it);
    }
    void __next__()
    {
        do
            ++it;
        while (it && !rb_has(p.get(), rb_get(f.get(), *it)));
    }
    void __prev__()
    {
        do
            --it;
        while (it && !rb_has(p.get(), rb_get(f.get(), *it)));
    }
    rb_filter_iter_t __end__() const
    {
        return rb_filter_iter_t(it.end(), p, f);
    }
    bool operator!=(const rb_filter_iter_t &o) const
    {
        return it != o.it;
    }

private:
    Iter it;
    rb_reference_wrapper<Pred> p;
    rb_reference_wrapper<Proj> f;
};
template <typename Pred, typename Proj> struct rb_filter_iter_factory_t
{
    rb_filter_iter_factory_t(Pred p, Proj f)
        : p(p)
        , f(f)
    {
    }

    template <typename Iter, rb_requires(rb_is_iterator(Iter))> rb_filter_iter_t<Iter, Pred, Proj> operator()(Iter it)
    {
        return rb_filter_iter_t<Iter, Pred, Proj>(it, p, f);
    }

private:
    Pred p;
    Proj f;
};
struct
{
    template <typename Pred = decltype((rb_identity)), typename Proj = decltype((rb_identity))>
    rb_filter_iter_factory_t<Pred, Proj> operator()(Pred &&p = rb_identity, Proj &&f = rb_identity) const
    {
        return rb_filter_iter_factory_t<Pred, Proj>(p, f);
    }
} RB_FUNCOBJ(rb_filter);

template <typename Redu, typename InitT> struct rb_reduce_t
{
    rb_reduce_t(Redu r, InitT init_value)
        : r(r)
        , init_value(init_value)
    {
    }

    template <
        typename Iter,
        rb_requires(rb_is_iterator(Iter)),
        typename AccuT = rb_decay<decltype(rb_declval(Redu)(rb_declval(InitT), rb_declval(typename Iter::item_t)))>>
    AccuT operator()(Iter it)
    {
        AccuT value = init_value;
        for (; it; ++it)
            value = r(value, *it);
        return value;
    }

private:
    Redu r;
    InitT init_value;
};
struct
{
    template <typename Redu, typename InitT> rb_reduce_t<Redu, InitT> operator()(Redu &&r, InitT init_value) const
    {
        return rb_reduce_t<Redu, InitT>(r, init_value);
    }
} RB_FUNCOBJ(rb_reduce);

/* rb_zip() */

template <typename A, typename B>
struct rb_zip_iter_t : rb_iter_t<rb_zip_iter_t<A, B>, rb_pair_t<typename A::item_t, typename B::item_t>>
{
    rb_zip_iter_t() {}
    rb_zip_iter_t(const A &a, const B &b)
        : a(a)
        , b(b)
    {
    }

    typedef rb_pair_t<typename A::item_t, typename B::item_t> __item_t__;
    static constexpr bool is_random_access_iterator = A::is_random_access_iterator && B::is_random_access_iterator;
    /* Note.  The following categorization is only valid if A is strictly sorted,
     * ie. does NOT have duplicates.  Previously I tried to categorize sortedness
     * more granularly, see commits:
     *
     *   513762849a683914fc266a17ddf38f133cccf072
     *   4d3cf2adb669c345cc43832d11689271995e160a
     *
     * However, that was not enough, since rb_sorted_array_t, rb_sorted_vector_t,
     * SortedArrayOf, etc all needed to be updated to add more variants.  At that
     * point I saw it not worth the effort, and instead we now deem all sorted
     * collections as essentially strictly-sorted for the purposes of zip.
     *
     * The above assumption is not as bad as it sounds.  Our "sorted" comes with
     * no guarantees.  It's just a contract, put in place to help you remember,
     * and think about, whether an iterator you receive is expected to be
     * sorted or not.  As such, it's not perfect by definition, and should not
     * be treated so.  The inaccuracy here just errs in the direction of being
     * more permissive, so your code compiles instead of erring on the side of
     * marking your zipped iterator unsorted in which case your code won't
     * compile.
     *
     * This semantical limitation does NOT affect logic in any other place I
     * know of as of this writing.
     */
    static constexpr bool is_sorted_iterator = A::is_sorted_iterator;

    __item_t__ __item__() const
    {
        return __item_t__(*a, *b);
    }
    __item_t__ __item_at__(unsigned i) const
    {
        return __item_t__(a[i], b[i]);
    }
    bool __more__() const
    {
        return bool(a) && bool(b);
    }
    unsigned __len__() const
    {
        return rb_min(a.len(), b.len());
    }
    void __next__()
    {
        ++a;
        ++b;
    }
    void __forward__(unsigned n)
    {
        a += n;
        b += n;
    }
    void __prev__()
    {
        --a;
        --b;
    }
    void __rewind__(unsigned n)
    {
        a -= n;
        b -= n;
    }
    rb_zip_iter_t __end__() const
    {
        return rb_zip_iter_t(a.end(), b.end());
    }
    /* Note, we should stop if ANY of the iters reaches end.  As such two compare
     * unequal if both items are unequal, NOT if either is unequal. */
    bool operator!=(const rb_zip_iter_t &o) const
    {
        return a != o.a && b != o.b;
    }

private:
    A a;
    B b;
};
struct
{
    RB_PARTIALIZE(2);
    template <typename A, typename B, rb_requires(rb_is_iterable(A) && rb_is_iterable(B))>
    rb_zip_iter_t<rb_iter_type<A>, rb_iter_type<B>> operator()(A &&a, B &&b) const
    {
        return rb_zip_iter_t<rb_iter_type<A>, rb_iter_type<B>>(rb_iter(a), rb_iter(b));
    }
} RB_FUNCOBJ(rb_zip);

/* rb_apply() */

template <typename Appl> struct rb_apply_t
{
    rb_apply_t(Appl a)
        : a(a)
    {
    }

    template <typename Iter, rb_requires(rb_is_iterator(Iter))> void operator()(Iter it)
    {
        for (; it; ++it)
            (void)rb_invoke(a, *it);
    }

private:
    Appl a;
};
struct
{
    template <typename Appl> rb_apply_t<Appl> operator()(Appl &&a) const
    {
        return rb_apply_t<Appl>(a);
    }

    template <typename Appl> rb_apply_t<Appl &> operator()(Appl *a) const
    {
        return rb_apply_t<Appl &>(*a);
    }
} RB_FUNCOBJ(rb_apply);

/* rb_range()/rb_iota()/rb_repeat() */

template <typename T, typename S> struct rb_range_iter_t : rb_iter_t<rb_range_iter_t<T, S>, T>
{
    rb_range_iter_t(T start, T end_, S step)
        : v(start)
        , end_(end_for(start, end_, step))
        , step(step)
    {
    }

    typedef T __item_t__;
    static constexpr bool is_random_access_iterator = true;
    static constexpr bool is_sorted_iterator = true;
    __item_t__ __item__() const
    {
        return rb_ridentity(v);
    }
    __item_t__ __item_at__(unsigned j) const
    {
        return v + j * step;
    }
    bool __more__() const
    {
        return v != end_;
    }
    unsigned __len__() const
    {
        return !step ? UINT_MAX : (end_ - v) / step;
    }
    void __next__()
    {
        v += step;
    }
    void __forward__(unsigned n)
    {
        v += n * step;
    }
    void __prev__()
    {
        v -= step;
    }
    void __rewind__(unsigned n)
    {
        v -= n * step;
    }
    rb_range_iter_t __end__() const
    {
        return rb_range_iter_t(end_, end_, step);
    }
    bool operator!=(const rb_range_iter_t &o) const
    {
        return v != o.v;
    }

private:
    static inline T end_for(T start, T end_, S step)
    {
        if (!step)
            return end_;
        auto res = (end_ - start) % step;
        if (!res)
            return end_;
        end_ += step - res;
        return end_;
    }

private:
    T v;
    T end_;
    S step;
};
struct
{
    template <typename T = unsigned> rb_range_iter_t<T, unsigned> operator()(T end = (unsigned)-1) const
    {
        return rb_range_iter_t<T, unsigned>(0, end, 1u);
    }

    template <typename T, typename S = unsigned> rb_range_iter_t<T, S> operator()(T start, T end, S step = 1u) const
    {
        return rb_range_iter_t<T, S>(start, end, step);
    }
} RB_FUNCOBJ(rb_range);

template <typename T, typename S> struct rb_iota_iter_t : rb_iter_with_fallback_t<rb_iota_iter_t<T, S>, T>
{
    rb_iota_iter_t(T start, S step)
        : v(start)
        , step(step)
    {
    }

private:
    template <typename S2 = S>
    auto inc(rb_type_identity<S2> s, rb_priority<1>)
        -> rb_void_t<decltype(rb_invoke(rb_forward<S2>(s), rb_declval<T &>()))>
    {
        v = rb_invoke(rb_forward<S2>(s), v);
    }

    void inc(S s, rb_priority<0>)
    {
        v += s;
    }

public:
    typedef T __item_t__;
    static constexpr bool is_random_access_iterator = true;
    static constexpr bool is_sorted_iterator = true;
    __item_t__ __item__() const
    {
        return rb_ridentity(v);
    }
    bool __more__() const
    {
        return true;
    }
    unsigned __len__() const
    {
        return UINT_MAX;
    }
    void __next__()
    {
        inc(step, rb_prioritize);
    }
    void __prev__()
    {
        v -= step;
    }
    rb_iota_iter_t __end__() const
    {
        return *this;
    }
    bool operator!=(const rb_iota_iter_t &o) const
    {
        return true;
    }

private:
    T v;
    S step;
};
struct
{
    template <typename T = unsigned, typename S = unsigned>
    rb_iota_iter_t<T, S> operator()(T start = 0u, S step = 1u) const
    {
        return rb_iota_iter_t<T, S>(start, step);
    }
} RB_FUNCOBJ(rb_iota);

template <typename T> struct rb_repeat_iter_t : rb_iter_t<rb_repeat_iter_t<T>, T>
{
    rb_repeat_iter_t(T value)
        : v(value)
    {
    }

    typedef T __item_t__;
    static constexpr bool is_random_access_iterator = true;
    static constexpr bool is_sorted_iterator = true;
    __item_t__ __item__() const
    {
        return v;
    }
    __item_t__ __item_at__(unsigned j) const
    {
        return v;
    }
    bool __more__() const
    {
        return true;
    }
    unsigned __len__() const
    {
        return UINT_MAX;
    }
    void __next__() {}
    void __forward__(unsigned) {}
    void __prev__() {}
    void __rewind__(unsigned) {}
    rb_repeat_iter_t __end__() const
    {
        return *this;
    }
    bool operator!=(const rb_repeat_iter_t &o) const
    {
        return true;
    }

private:
    T v;
};
struct
{
    template <typename T> rb_repeat_iter_t<T> operator()(T value) const
    {
        return rb_repeat_iter_t<T>(value);
    }
} RB_FUNCOBJ(rb_repeat);

/* rb_enumerate()/rb_take() */

struct
{
    template <typename Iterable, typename Index = unsigned, rb_requires(rb_is_iterable(Iterable))>
    auto operator()(Iterable &&it, Index start = 0u) const RB_AUTO_RETURN(rb_zip(rb_iota(start), it))
} RB_FUNCOBJ(rb_enumerate);

struct
{
    RB_PARTIALIZE(2);
    template <typename Iterable, rb_requires(rb_is_iterable(Iterable))>
    auto operator()(Iterable &&it, unsigned count) const RB_AUTO_RETURN(rb_zip(rb_range(count), it) | rb_map(rb_second))

        /* Specialization arrays. */

        template <typename Type>
        inline rb_array_t<Type> operator()(rb_array_t<Type> array, unsigned count) const
    {
        return array.sub_array(0, count);
    }

    template <typename Type>
    inline rb_sorted_array_t<Type> operator()(rb_sorted_array_t<Type> array, unsigned count) const
    {
        return array.sub_array(0, count);
    }
} RB_FUNCOBJ(rb_take);

struct
{
    RB_PARTIALIZE(2);
    template <typename Iter, rb_requires(rb_is_iterator(Iter))>
    auto operator()(Iter it, unsigned count) const
        RB_AUTO_RETURN(+rb_iota(it, rb_add(count)) | rb_map(rb_take(count)) | rb_take((rb_len(it) + count - 1) / count))
} RB_FUNCOBJ(rb_chop);

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

/* hb-drain: rb_sink to void / blackhole / /dev/null. */

struct
{
    template <typename Iter, rb_requires(rb_is_iterator(Iter))> void operator()(Iter it) const
    {
        for (; it; ++it)
            (void)*it;
    }
} RB_FUNCOBJ(rb_drain);

/* rb_unzip(): unzip and sink to two sinks. */

template <typename Sink1, typename Sink2> struct rb_unzip_t
{
    rb_unzip_t(Sink1 s1, Sink2 s2)
        : s1(s1)
        , s2(s2)
    {
    }

    template <typename Iter, rb_requires(rb_is_iterator(Iter))> void operator()(Iter it)
    {
        for (; it; ++it) {
            const auto &v = *it;
            s1 << v.first;
            s2 << v.second;
        }
    }

private:
    Sink1 s1;
    Sink2 s2;
};
struct
{
    template <typename Sink1, typename Sink2> rb_unzip_t<Sink1, Sink2> operator()(Sink1 &&s1, Sink2 &&s2) const
    {
        return rb_unzip_t<Sink1, Sink2>(s1, s2);
    }

    template <typename Sink1, typename Sink2> rb_unzip_t<Sink1 &, Sink2 &> operator()(Sink1 *s1, Sink2 *s2) const
    {
        return rb_unzip_t<Sink1 &, Sink2 &>(*s1, *s2);
    }
} RB_FUNCOBJ(rb_unzip);

/* hb-all, hb-any, hb-none. */

struct
{
    template <typename Iterable,
              typename Pred = decltype((rb_identity)),
              typename Proj = decltype((rb_identity)),
              rb_requires(rb_is_iterable(Iterable))>
    bool operator()(Iterable &&c, Pred &&p = rb_identity, Proj &&f = rb_identity) const
    {
        for (auto it = rb_iter(c); it; ++it)
            if (!rb_match(rb_forward<Pred>(p), rb_get(rb_forward<Proj>(f), *it)))
                return false;
        return true;
    }
} RB_FUNCOBJ(rb_all);
struct
{
    template <typename Iterable,
              typename Pred = decltype((rb_identity)),
              typename Proj = decltype((rb_identity)),
              rb_requires(rb_is_iterable(Iterable))>
    bool operator()(Iterable &&c, Pred &&p = rb_identity, Proj &&f = rb_identity) const
    {
        for (auto it = rb_iter(c); it; ++it)
            if (rb_match(rb_forward<Pred>(p), rb_get(rb_forward<Proj>(f), *it)))
                return true;
        return false;
    }
} RB_FUNCOBJ(rb_any);
struct
{
    template <typename Iterable,
              typename Pred = decltype((rb_identity)),
              typename Proj = decltype((rb_identity)),
              rb_requires(rb_is_iterable(Iterable))>
    bool operator()(Iterable &&c, Pred &&p = rb_identity, Proj &&f = rb_identity) const
    {
        for (auto it = rb_iter(c); it; ++it)
            if (rb_match(rb_forward<Pred>(p), rb_get(rb_forward<Proj>(f), *it)))
                return false;
        return true;
    }
} RB_FUNCOBJ(rb_none);

/*
 * Algorithms operating on iterators.
 */

template <typename C, typename V, rb_requires(rb_is_iterable(C))> inline void rb_fill(C &c, const V &v)
{
    for (auto i = rb_iter(c); i; i++)
        *i = v;
}

template <typename S, typename D> inline void rb_copy(S &&is, D &&id)
{
    rb_iter(is) | rb_sink(id);
}

#endif /* RB_ITER_HH */
