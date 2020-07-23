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

#ifndef RB_META_HH
#define RB_META_HH

#include "hb.hh"

/*
 * C++ template meta-programming & fundamentals used with them.
 */

/* Void!  For when we need a expression-type of void. */
struct rb_empty_t
{
};

/* https://en.cppreference.com/w/cpp/types/void_t */
template <typename... Ts> struct _rb_void_t
{
    typedef void type;
};
template <typename... Ts> using rb_void_t = typename _rb_void_t<Ts...>::type;

template <typename Head, typename... Ts> struct _rb_head_t
{
    typedef Head type;
};
template <typename... Ts> using rb_head_t = typename _rb_head_t<Ts...>::type;

template <typename T, T v> struct rb_integral_constant
{
    static constexpr T value = v;
};
template <bool b> using rb_bool_constant = rb_integral_constant<bool, b>;
using rb_true_type = rb_bool_constant<true>;
using rb_false_type = rb_bool_constant<false>;

/* Basic type SFINAE. */

template <bool B, typename T = void> struct rb_enable_if
{
};
template <typename T> struct rb_enable_if<true, T>
{
    typedef T type;
};
#define rb_enable_if(Cond) typename rb_enable_if<(Cond)>::type * = nullptr
/* Concepts/Requires alias: */
#define rb_requires(Cond) rb_enable_if((Cond))

template <typename T, typename T2> struct rb_is_same : rb_false_type
{
};
template <typename T> struct rb_is_same<T, T> : rb_true_type
{
};
#define rb_is_same(T, T2) rb_is_same<T, T2>::value

/* Function overloading SFINAE and priority. */

#define RB_RETURN(Ret, E)                                                                                              \
    ->rb_head_t<Ret, decltype((E))>                                                                                    \
    {                                                                                                                  \
        return (E);                                                                                                    \
    }
#define RB_AUTO_RETURN(E)                                                                                              \
    ->decltype((E))                                                                                                    \
    {                                                                                                                  \
        return (E);                                                                                                    \
    }
#define RB_VOID_RETURN(E)                                                                                              \
    ->rb_void_t<decltype((E))>                                                                                         \
    {                                                                                                                  \
        (E);                                                                                                           \
    }

template <unsigned Pri> struct rb_priority : rb_priority<Pri - 1>
{
};
template <> struct rb_priority<0>
{
};
#define rb_prioritize rb_priority<16>()

#define RB_FUNCOBJ(x) static_const x RB_UNUSED

template <typename T> struct rb_type_identity_t
{
    typedef T type;
};
template <typename T> using rb_type_identity = typename rb_type_identity_t<T>::type;

struct
{
    template <typename T> constexpr T *operator()(T &arg) const
    {
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wcast-align"
        /* https://en.cppreference.com/w/cpp/memory/addressof */
        return reinterpret_cast<T *>(&const_cast<char &>(reinterpret_cast<const volatile char &>(arg)));
#pragma GCC diagnostic pop
    }
} RB_FUNCOBJ(rb_addressof);

template <typename T> static inline T rb_declval();
#define rb_declval(T) (rb_declval<T>())

template <typename T> struct rb_match_const : rb_type_identity_t<T>, rb_bool_constant<false>
{
};
template <typename T> struct rb_match_const<const T> : rb_type_identity_t<T>, rb_bool_constant<true>
{
};
template <typename T> using rb_remove_const = typename rb_match_const<T>::type;
template <typename T> using rb_add_const = const T;
#define rb_is_const(T) rb_match_const<T>::value
template <typename T> struct rb_match_reference : rb_type_identity_t<T>, rb_bool_constant<false>
{
};
template <typename T> struct rb_match_reference<T &> : rb_type_identity_t<T>, rb_bool_constant<true>
{
};
template <typename T> struct rb_match_reference<T &&> : rb_type_identity_t<T>, rb_bool_constant<true>
{
};
template <typename T> using rb_remove_reference = typename rb_match_reference<T>::type;
template <typename T> auto _rb_try_add_lvalue_reference(rb_priority<1>) -> rb_type_identity<T &>;
template <typename T> auto _rb_try_add_lvalue_reference(rb_priority<0>) -> rb_type_identity<T>;
template <typename T> using rb_add_lvalue_reference = decltype(_rb_try_add_lvalue_reference<T>(rb_prioritize));
template <typename T> auto _rb_try_add_rvalue_reference(rb_priority<1>) -> rb_type_identity<T &&>;
template <typename T> auto _rb_try_add_rvalue_reference(rb_priority<0>) -> rb_type_identity<T>;
template <typename T> using rb_add_rvalue_reference = decltype(_rb_try_add_rvalue_reference<T>(rb_prioritize));
#define rb_is_reference(T) rb_match_reference<T>::value
template <typename T> struct rb_match_pointer : rb_type_identity_t<T>, rb_bool_constant<false>
{
};
template <typename T> struct rb_match_pointer<T *> : rb_type_identity_t<T>, rb_bool_constant<true>
{
};
template <typename T> using rb_remove_pointer = typename rb_match_pointer<T>::type;
template <typename T> auto _rb_try_add_pointer(rb_priority<1>) -> rb_type_identity<rb_remove_reference<T> *>;
template <typename T> auto _rb_try_add_pointer(rb_priority<1>) -> rb_type_identity<T>;
template <typename T> using rb_add_pointer = decltype(_rb_try_add_pointer<T>(rb_prioritize));
#define rb_is_pointer(T) rb_match_pointer<T>::value

/* TODO Add feature-parity to std::decay. */
template <typename T> using rb_decay = rb_remove_const<rb_remove_reference<T>>;

template <bool B, class T, class F> struct _rb_conditional
{
    typedef T type;
};
template <class T, class F> struct _rb_conditional<false, T, F>
{
    typedef F type;
};
template <bool B, class T, class F> using rb_conditional = typename _rb_conditional<B, T, F>::type;

template <typename From, typename To> struct rb_is_convertible
{
private:
    static constexpr bool from_void = rb_is_same(void, rb_decay<From>);
    static constexpr bool to_void = rb_is_same(void, rb_decay<To>);
    static constexpr bool either_void = from_void || to_void;
    static constexpr bool both_void = from_void && to_void;

    static rb_true_type impl2(rb_conditional<to_void, int, To>);

    template <typename T> static auto impl(rb_priority<1>) -> decltype(impl2(rb_declval(T)));
    template <typename T> static rb_false_type impl(rb_priority<0>);

public:
    static constexpr bool value =
        both_void || (!either_void && decltype(impl<rb_conditional<from_void, int, From>>(rb_prioritize))::value);
};
#define rb_is_convertible(From, To) rb_is_convertible<From, To>::value

template <typename Base, typename Derived>
using rb_is_base_of = rb_is_convertible<rb_decay<Derived> *, rb_decay<Base> *>;
#define rb_is_base_of(Base, Derived) rb_is_base_of<Base, Derived>::value

template <typename From, typename To>
using rb_is_cr_convertible =
    rb_bool_constant<rb_is_same(rb_decay<From>, rb_decay<To>) && (!rb_is_const(From) || rb_is_const(To)) &&
                     (!rb_is_reference(To) || rb_is_const(To) || rb_is_reference(To))>;
#define rb_is_cr_convertible(From, To) rb_is_cr_convertible<From, To>::value

/* std::move and std::forward */

template <typename T> static constexpr rb_remove_reference<T> &&rb_move(T &&t)
{
    return (rb_remove_reference<T> &&)(t);
}

template <typename T> static constexpr T &&rb_forward(rb_remove_reference<T> &t)
{
    return (T &&) t;
}
template <typename T> static constexpr T &&rb_forward(rb_remove_reference<T> &&t)
{
    return (T &&) t;
}

struct
{
    template <typename T>
    constexpr auto operator()(T &&v) const RB_AUTO_RETURN(rb_forward<T>(v))

        template <typename T>
        constexpr auto operator()(T *v) const RB_AUTO_RETURN(*v)
} RB_FUNCOBJ(rb_deref);

struct
{
    template <typename T>
    constexpr auto operator()(T &&v) const RB_AUTO_RETURN(rb_forward<T>(v))

        template <typename T>
        constexpr auto operator()(T &v) const RB_AUTO_RETURN(rb_addressof(v))
} RB_FUNCOBJ(rb_ref);

template <typename T> struct rb_reference_wrapper
{
    rb_reference_wrapper(T v)
        : v(v)
    {
    }
    bool operator==(const rb_reference_wrapper &o) const
    {
        return v == o.v;
    }
    bool operator!=(const rb_reference_wrapper &o) const
    {
        return v != o.v;
    }
    operator T() const
    {
        return v;
    }
    T get() const
    {
        return v;
    }
    T v;
};
template <typename T> struct rb_reference_wrapper<T &>
{
    rb_reference_wrapper(T &v)
        : v(rb_addressof(v))
    {
    }
    bool operator==(const rb_reference_wrapper &o) const
    {
        return v == o.v;
    }
    bool operator!=(const rb_reference_wrapper &o) const
    {
        return v != o.v;
    }
    operator T &() const
    {
        return *v;
    }
    T &get() const
    {
        return *v;
    }
    T *v;
};

template <typename T>
using rb_is_integral =
    rb_bool_constant<rb_is_same(rb_decay<T>, char) || rb_is_same(rb_decay<T>, signed char) ||
                     rb_is_same(rb_decay<T>, unsigned char) || rb_is_same(rb_decay<T>, signed int) ||
                     rb_is_same(rb_decay<T>, unsigned int) || rb_is_same(rb_decay<T>, signed short) ||
                     rb_is_same(rb_decay<T>, unsigned short) || rb_is_same(rb_decay<T>, signed long) ||
                     rb_is_same(rb_decay<T>, unsigned long) || rb_is_same(rb_decay<T>, signed long long) ||
                     rb_is_same(rb_decay<T>, unsigned long long) || false>;
#define rb_is_integral(T) rb_is_integral<T>::value
template <typename T>
using rb_is_floating_point = rb_bool_constant<rb_is_same(rb_decay<T>, float) || rb_is_same(rb_decay<T>, double) ||
                                              rb_is_same(rb_decay<T>, long double) || false>;
#define rb_is_floating_point(T) rb_is_floating_point<T>::value
template <typename T> using rb_is_arithmetic = rb_bool_constant<rb_is_integral(T) || rb_is_floating_point(T) || false>;
#define rb_is_arithmetic(T) rb_is_arithmetic<T>::value

template <typename T>
using rb_is_signed = rb_conditional<rb_is_arithmetic(T), rb_bool_constant<(T)-1 < (T)0>, rb_false_type>;
#define rb_is_signed(T) rb_is_signed<T>::value
template <typename T>
using rb_is_unsigned = rb_conditional<rb_is_arithmetic(T), rb_bool_constant<(T)0 < (T)-1>, rb_false_type>;
#define rb_is_unsigned(T) rb_is_unsigned<T>::value

template <typename T> struct rb_int_min;
template <> struct rb_int_min<char> : rb_integral_constant<char, CHAR_MIN>
{
};
template <> struct rb_int_min<signed char> : rb_integral_constant<signed char, SCHAR_MIN>
{
};
template <> struct rb_int_min<unsigned char> : rb_integral_constant<unsigned char, 0>
{
};
template <> struct rb_int_min<signed short> : rb_integral_constant<signed short, SHRT_MIN>
{
};
template <> struct rb_int_min<unsigned short> : rb_integral_constant<unsigned short, 0>
{
};
template <> struct rb_int_min<signed int> : rb_integral_constant<signed int, INT_MIN>
{
};
template <> struct rb_int_min<unsigned int> : rb_integral_constant<unsigned int, 0>
{
};
template <> struct rb_int_min<signed long> : rb_integral_constant<signed long, LONG_MIN>
{
};
template <> struct rb_int_min<unsigned long> : rb_integral_constant<unsigned long, 0>
{
};
template <> struct rb_int_min<signed long long> : rb_integral_constant<signed long long, LLONG_MIN>
{
};
template <> struct rb_int_min<unsigned long long> : rb_integral_constant<unsigned long long, 0>
{
};
#define rb_int_min(T) rb_int_min<T>::value
template <typename T> struct rb_int_max;
template <> struct rb_int_max<char> : rb_integral_constant<char, CHAR_MAX>
{
};
template <> struct rb_int_max<signed char> : rb_integral_constant<signed char, SCHAR_MAX>
{
};
template <> struct rb_int_max<unsigned char> : rb_integral_constant<unsigned char, UCHAR_MAX>
{
};
template <> struct rb_int_max<signed short> : rb_integral_constant<signed short, SHRT_MAX>
{
};
template <> struct rb_int_max<unsigned short> : rb_integral_constant<unsigned short, USHRT_MAX>
{
};
template <> struct rb_int_max<signed int> : rb_integral_constant<signed int, INT_MAX>
{
};
template <> struct rb_int_max<unsigned int> : rb_integral_constant<unsigned int, UINT_MAX>
{
};
template <> struct rb_int_max<signed long> : rb_integral_constant<signed long, LONG_MAX>
{
};
template <> struct rb_int_max<unsigned long> : rb_integral_constant<unsigned long, ULONG_MAX>
{
};
template <> struct rb_int_max<signed long long> : rb_integral_constant<signed long long, LLONG_MAX>
{
};
template <> struct rb_int_max<unsigned long long> : rb_integral_constant<unsigned long long, ULLONG_MAX>
{
};
#define rb_int_max(T) rb_int_max<T>::value

template <typename T, typename> struct _rb_is_destructible : rb_false_type
{
};
template <typename T> struct _rb_is_destructible<T, rb_void_t<decltype(rb_declval(T).~T())>> : rb_true_type
{
};
template <typename T> using rb_is_destructible = _rb_is_destructible<T, void>;
#define rb_is_destructible(T) rb_is_destructible<T>::value

template <typename T, typename, typename... Ts> struct _rb_is_constructible : rb_false_type
{
};
template <typename T, typename... Ts>
struct _rb_is_constructible<T, rb_void_t<decltype(T(rb_declval(Ts)...))>, Ts...> : rb_true_type
{
};
template <typename T, typename... Ts> using rb_is_constructible = _rb_is_constructible<T, void, Ts...>;
#define rb_is_constructible(...) rb_is_constructible<__VA_ARGS__>::value

template <typename T> using rb_is_default_constructible = rb_is_constructible<T>;
#define rb_is_default_constructible(T) rb_is_default_constructible<T>::value

template <typename T> using rb_is_copy_constructible = rb_is_constructible<T, rb_add_lvalue_reference<rb_add_const<T>>>;
#define rb_is_copy_constructible(T) rb_is_copy_constructible<T>::value

template <typename T> using rb_is_move_constructible = rb_is_constructible<T, rb_add_rvalue_reference<rb_add_const<T>>>;
#define rb_is_move_constructible(T) rb_is_move_constructible<T>::value

template <typename T, typename U, typename> struct _rb_is_assignable : rb_false_type
{
};
template <typename T, typename U>
struct _rb_is_assignable<T, U, rb_void_t<decltype(rb_declval(T) = rb_declval(U))>> : rb_true_type
{
};
template <typename T, typename U> using rb_is_assignable = _rb_is_assignable<T, U, void>;
#define rb_is_assignable(T, U) rb_is_assignable<T, U>::value

template <typename T>
using rb_is_copy_assignable = rb_is_assignable<rb_add_lvalue_reference<T>, rb_add_lvalue_reference<rb_add_const<T>>>;
#define rb_is_copy_assignable(T) rb_is_copy_assignable<T>::value

template <typename T>
using rb_is_move_assignable = rb_is_assignable<rb_add_lvalue_reference<T>, rb_add_rvalue_reference<T>>;
#define rb_is_move_assignable(T) rb_is_move_assignable<T>::value

/* Trivial versions. */

template <typename T> union rb_trivial {
    T value;
};

/* Don't know how to do the following. */
template <typename T> using rb_is_trivially_destructible = rb_is_destructible<rb_trivial<T>>;
#define rb_is_trivially_destructible(T) rb_is_trivially_destructible<T>::value

/* Don't know how to do the following. */
// template <typename T, typename ...Ts>
// using rb_is_trivially_constructible= rb_is_constructible<rb_trivial<T>, rb_trivial<Ts>...>;
//#define rb_is_trivially_constructible(...) rb_is_trivially_constructible<__VA_ARGS__>::value

template <typename T> using rb_is_trivially_default_constructible = rb_is_default_constructible<rb_trivial<T>>;
#define rb_is_trivially_default_constructible(T) rb_is_trivially_default_constructible<T>::value

template <typename T> using rb_is_trivially_copy_constructible = rb_is_copy_constructible<rb_trivial<T>>;
#define rb_is_trivially_copy_constructible(T) rb_is_trivially_copy_constructible<T>::value

template <typename T> using rb_is_trivially_move_constructible = rb_is_move_constructible<rb_trivial<T>>;
#define rb_is_trivially_move_constructible(T) rb_is_trivially_move_constructible<T>::value

/* Don't know how to do the following. */
// template <typename T, typename U>
// using rb_is_trivially_assignable= rb_is_assignable<rb_trivial<T>, rb_trivial<U>>;
//#define rb_is_trivially_assignable(T,U) rb_is_trivially_assignable<T, U>::value

template <typename T> using rb_is_trivially_copy_assignable = rb_is_copy_assignable<rb_trivial<T>>;
#define rb_is_trivially_copy_assignable(T) rb_is_trivially_copy_assignable<T>::value

template <typename T> using rb_is_trivially_move_assignable = rb_is_move_assignable<rb_trivial<T>>;
#define rb_is_trivially_move_assignable(T) rb_is_trivially_move_assignable<T>::value

template <typename T>
using rb_is_trivially_copyable =
    rb_bool_constant<rb_is_trivially_destructible(T) &&
                     (!rb_is_move_assignable(T) || rb_is_trivially_move_assignable(T)) &&
                     (!rb_is_move_constructible(T) || rb_is_trivially_move_constructible(T)) &&
                     (!rb_is_copy_assignable(T) || rb_is_trivially_copy_assignable(T)) &&
                     (!rb_is_copy_constructible(T) || rb_is_trivially_copy_constructible(T)) && true>;
#define rb_is_trivially_copyable(T) rb_is_trivially_copyable<T>::value

template <typename T>
using rb_is_trivial = rb_bool_constant<rb_is_trivially_copyable(T) && rb_is_trivially_default_constructible(T)>;
#define rb_is_trivial(T) rb_is_trivial<T>::value

/* rb_unwrap_type (T)
 * If T has no T::type, returns T. Otherwise calls itself on T::type recursively.
 */

template <typename T, typename> struct _rb_unwrap_type : rb_type_identity_t<T>
{
};
template <typename T> struct _rb_unwrap_type<T, rb_void_t<typename T::type>> : _rb_unwrap_type<typename T::type, void>
{
};
template <typename T> using rb_unwrap_type = _rb_unwrap_type<T, void>;
#define rb_unwrap_type(T) typename rb_unwrap_type<T>::type

#endif /* RB_META_HH */
