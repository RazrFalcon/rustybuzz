/*
 * Copyright © 2007,2008,2009,2010  Red Hat, Inc.
 * Copyright © 2012,2018  Google, Inc.
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

#ifndef RB_MACHINERY_HH
#define RB_MACHINERY_HH

#include "hb.hh"
#include "hb-blob.hh"

#include "hb-dispatch.hh"
#include "hb-sanitize.hh"

/*
 * Casts
 */

/* StructAtOffset<T>(P,Ofs) returns the struct T& that is placed at memory
 * location pointed to by P plus Ofs bytes. */
template <typename Type> static inline const Type &StructAtOffset(const void *P, unsigned int offset)
{
    return *reinterpret_cast<const Type *>((const char *)P + offset);
}
template <typename Type> static inline Type &StructAtOffset(void *P, unsigned int offset)
{
    return *reinterpret_cast<Type *>((char *)P + offset);
}
template <typename Type> static inline const Type &StructAtOffsetUnaligned(const void *P, unsigned int offset)
{
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wcast-align"
    return *reinterpret_cast<const Type *>((const char *)P + offset);
#pragma GCC diagnostic pop
}
template <typename Type> static inline Type &StructAtOffsetUnaligned(void *P, unsigned int offset)
{
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wcast-align"
    return *reinterpret_cast<Type *>((char *)P + offset);
#pragma GCC diagnostic pop
}

/* StructAfter<T>(X) returns the struct T& that is placed after X.
 * Works with X of variable size also.  X must implement get_size() */
template <typename Type, typename TObject> static inline const Type &StructAfter(const TObject &X)
{
    return StructAtOffset<Type>(&X, X.get_size());
}
template <typename Type, typename TObject> static inline Type &StructAfter(TObject &X)
{
    return StructAtOffset<Type>(&X, X.get_size());
}

/*
 * Size checking
 */

/* Check _assertion in a method environment */
#define _DEFINE_INSTANCE_ASSERTION1(_line, _assertion)                                                                 \
    void _instance_assertion_on_line_##_line() const                                                                   \
    {                                                                                                                  \
        static_assert((_assertion), "");                                                                               \
    }
#define _DEFINE_INSTANCE_ASSERTION0(_line, _assertion) _DEFINE_INSTANCE_ASSERTION1(_line, _assertion)
#define DEFINE_INSTANCE_ASSERTION(_assertion) _DEFINE_INSTANCE_ASSERTION0(__LINE__, _assertion)

/* Check that _code compiles in a method environment */
#define _DEFINE_COMPILES_ASSERTION1(_line, _code)                                                                      \
    void _compiles_assertion_on_line_##_line() const                                                                   \
    {                                                                                                                  \
        _code;                                                                                                         \
    }
#define _DEFINE_COMPILES_ASSERTION0(_line, _code) _DEFINE_COMPILES_ASSERTION1(_line, _code)
#define DEFINE_COMPILES_ASSERTION(_code) _DEFINE_COMPILES_ASSERTION0(__LINE__, _code)

#define DEFINE_SIZE_STATIC(size)                                                                                       \
    DEFINE_INSTANCE_ASSERTION(sizeof(*this) == (size))                                                                 \
    unsigned int get_size() const                                                                                      \
    {                                                                                                                  \
        return (size);                                                                                                 \
    }                                                                                                                  \
    static constexpr unsigned null_size = (size);                                                                      \
    static constexpr unsigned min_size = (size);                                                                       \
    static constexpr unsigned static_size = (size)

#define DEFINE_SIZE_UNION(size, _member)                                                                               \
    DEFINE_COMPILES_ASSERTION((void)this->u._member.static_size)                                                       \
    DEFINE_INSTANCE_ASSERTION(sizeof(this->u._member) == (size))                                                       \
    static constexpr unsigned null_size = (size);                                                                      \
    static constexpr unsigned min_size = (size)

#define DEFINE_SIZE_MIN(size)                                                                                          \
    DEFINE_INSTANCE_ASSERTION(sizeof(*this) >= (size))                                                                 \
    static constexpr unsigned null_size = (size);                                                                      \
    static constexpr unsigned min_size = (size)

#define DEFINE_SIZE_UNBOUNDED(size)                                                                                    \
    DEFINE_INSTANCE_ASSERTION(sizeof(*this) >= (size))                                                                 \
    static constexpr unsigned min_size = (size)

#define DEFINE_SIZE_ARRAY(size, array)                                                                                 \
    DEFINE_COMPILES_ASSERTION((void)(array)[0].static_size)                                                            \
    DEFINE_INSTANCE_ASSERTION(sizeof(*this) == (size) + (RB_VAR_ARRAY + 0) * sizeof((array)[0]))                       \
    static constexpr unsigned null_size = (size);                                                                      \
    static constexpr unsigned min_size = (size)

#endif /* RB_MACHINERY_HH */
