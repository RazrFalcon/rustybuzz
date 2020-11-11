/*
 * Copyright © 2007,2008,2009,2010  Red Hat, Inc.
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
 * Red Hat Author(s): Behdad Esfahbod
 * Google Author(s): Behdad Esfahbod
 */

#ifndef RB_OPEN_TYPE_HH
#define RB_OPEN_TYPE_HH

#include "hb.hh"
#include "hb-blob.hh"
#include "hb-face.hh"
#include "hb-machinery.hh"

namespace OT {

/*
 *
 * The OpenType Font File: Data Types
 */

/* "The following data types are used in the OpenType font file.
 *  All OpenType fonts use Motorola-style byte ordering (Big Endian):" */

/*
 * Int types
 */

/* Integer types in big-endian order and no alignment requirement */
template <typename Type, unsigned int Size> struct IntType
{
    typedef Type type;
    typedef rb_conditional<rb_is_signed(Type), signed, unsigned> wide_type;

    IntType &operator=(wide_type i)
    {
        v = i;
        return *this;
    }
    operator wide_type() const
    {
        return v;
    }
    bool operator==(const IntType &o) const
    {
        return (Type)v == (Type)o.v;
    }
    bool operator!=(const IntType &o) const
    {
        return !(*this == o);
    }

    IntType &operator+=(unsigned count)
    {
        *this = *this + count;
        return *this;
    }
    IntType &operator-=(unsigned count)
    {
        *this = *this - count;
        return *this;
    }
    IntType &operator++()
    {
        *this += 1;
        return *this;
    }
    IntType &operator--()
    {
        *this -= 1;
        return *this;
    }
    IntType operator++(int)
    {
        IntType c(*this);
        ++*this;
        return c;
    }
    IntType operator--(int)
    {
        IntType c(*this);
        --*this;
        return c;
    }

    RB_INTERNAL static int cmp(const IntType *a, const IntType *b)
    {
        return b->cmp(*a);
    }
    RB_INTERNAL static int cmp(const void *a, const void *b)
    {
        IntType *pa = (IntType *)a;
        IntType *pb = (IntType *)b;

        return pb->cmp(*pa);
    }
    template <typename Type2> int cmp(Type2 a) const
    {
        Type b = v;
        if (sizeof(Type) < sizeof(int) && sizeof(Type2) < sizeof(int))
            return (int)a - (int)b;
        else
            return a < b ? -1 : a == b ? 0 : +1;
    }
    bool sanitize(rb_sanitize_context_t *c) const
    {
        return likely(c->check_struct(this));
    }

protected:
    BEInt<Type, Size> v;

public:
    DEFINE_SIZE_STATIC(Size);
};

typedef IntType<uint8_t, 1> HBUINT8;   /* 8-bit unsigned integer. */
typedef IntType<int8_t, 1> HBINT8;     /* 8-bit signed integer. */
typedef IntType<uint16_t, 2> HBUINT16; /* 16-bit unsigned integer. */
typedef IntType<int16_t, 2> HBINT16;   /* 16-bit signed integer. */
typedef IntType<uint32_t, 4> HBUINT32; /* 32-bit unsigned integer. */
typedef IntType<int32_t, 4> HBINT32;   /* 32-bit signed integer. */
/* Note: we cannot defined a signed HBINT24 because there's no corresponding C type.
 * Works for unsigned, but not signed, since we rely on compiler for sign-extension. */
typedef IntType<uint32_t, 3> HBUINT24; /* 24-bit unsigned integer. */

/* 16-bit signed integer (HBINT16) that describes a quantity in FUnits. */
typedef HBINT16 FWORD;

/* 32-bit signed integer (HBINT32) that describes a quantity in FUnits. */
typedef HBINT32 FWORD32;

/* 16-bit unsigned integer (HBUINT16) that describes a quantity in FUnits. */
typedef HBUINT16 UFWORD;

/* 16-bit signed fixed number with the low 14 bits of fraction (2.14). */
struct F2DOT14 : HBINT16
{
    F2DOT14 &operator=(uint16_t i)
    {
        HBINT16::operator=(i);
        return *this;
    }
    // 16384 means 1<<14
    float to_float() const
    {
        return ((int32_t)v) / 16384.f;
    }
    void set_float(float f)
    {
        v = roundf(f * 16384.f);
    }

public:
    DEFINE_SIZE_STATIC(2);
};

/* 32-bit signed fixed-point number (16.16). */
struct HBFixed : HBINT32
{
    HBFixed &operator=(uint32_t i)
    {
        HBINT32::operator=(i);
        return *this;
    }
    // 65536 means 1<<16
    float to_float() const
    {
        return ((int32_t)v) / 65536.f;
    }
    void set_float(float f)
    {
        v = roundf(f * 65536.f);
    }

public:
    DEFINE_SIZE_STATIC(4);
};

/* Date represented in number of seconds since 12:00 midnight, January 1,
 * 1904. The value is represented as a signed 64-bit integer. */
struct LONGDATETIME
{
    bool sanitize(rb_sanitize_context_t *c) const
    {
        return likely(c->check_struct(this));
    }

protected:
    HBINT32 major;
    HBUINT32 minor;

public:
    DEFINE_SIZE_STATIC(8);
};

/* Array of four uint8s (length = 32 bits) used to identify a script, language
 * system, feature, or baseline */
struct Tag : HBUINT32
{
    Tag &operator=(rb_tag_t i)
    {
        HBUINT32::operator=(i);
        return *this;
    }
    /* What the char* converters return is NOT nul-terminated.  Print using "%.4s" */
    operator const char *() const
    {
        return reinterpret_cast<const char *>(&this->v);
    }
    operator char *()
    {
        return reinterpret_cast<char *>(&this->v);
    }

public:
    DEFINE_SIZE_STATIC(4);
};

/* Glyph index number, same as uint16 (length = 16 bits) */
struct HBGlyphID : HBUINT16
{
    HBGlyphID &operator=(uint16_t i)
    {
        HBUINT16::operator=(i);
        return *this;
    }
};

/* Script/language-system/feature index */
struct Index : HBUINT16
{
    static constexpr unsigned NOT_FOUND_INDEX = 0xFFFFu;
    Index &operator=(uint16_t i)
    {
        HBUINT16::operator=(i);
        return *this;
    }
};
DECLARE_NULL_NAMESPACE_BYTES(OT, Index);

typedef Index NameID;

/* Offset, Null offset = 0 */
template <typename Type, bool has_null = true> struct Offset : Type
{
    Offset &operator=(typename Type::type i)
    {
        Type::operator=(i);
        return *this;
    }

    typedef Type type;

    bool is_null() const
    {
        return has_null && 0 == *this;
    }

public:
    DEFINE_SIZE_STATIC(sizeof(Type));
};

typedef Offset<HBUINT16> Offset16;
typedef Offset<HBUINT32> Offset32;

/* CheckSum */
struct CheckSum : HBUINT32
{
    CheckSum &operator=(uint32_t i)
    {
        HBUINT32::operator=(i);
        return *this;
    }

    /* This is reference implementation from the spec. */
    static uint32_t CalcTableChecksum(const HBUINT32 *Table, uint32_t Length)
    {
        uint32_t Sum = 0L;
        assert(0 == (Length & 3));
        const HBUINT32 *EndPtr = Table + Length / HBUINT32::static_size;

        while (Table < EndPtr)
            Sum += *Table++;
        return Sum;
    }

    /* Note: data should be 4byte aligned and have 4byte padding at the end. */
    void set_for_data(const void *data, unsigned int length)
    {
        *this = CalcTableChecksum((const HBUINT32 *)data, length);
    }

public:
    DEFINE_SIZE_STATIC(4);
};

/*
 * Version Numbers
 */

template <typename FixedType = HBUINT16> struct FixedVersion
{
    uint32_t to_int() const
    {
        return (major << (sizeof(FixedType) * 8)) + minor;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this);
    }

    FixedType major;
    FixedType minor;

public:
    DEFINE_SIZE_STATIC(2 * sizeof(FixedType));
};

/*
 * Template subclasses of Offset that do the dereferencing.
 * Use: (base+offset)
 */

template <typename Type, bool has_null> struct _rb_has_null
{
    static const Type *get_null()
    {
        return nullptr;
    }
    static Type *get_crap()
    {
        return nullptr;
    }
};
template <typename Type> struct _rb_has_null<Type, true>
{
    static const Type *get_null()
    {
        return &Null(Type);
    }
    static Type *get_crap()
    {
        return &Crap(Type);
    }
};

template <typename Type, typename OffsetType = HBUINT16, bool has_null = true>
struct OffsetTo : Offset<OffsetType, has_null>
{
    RB_DELETE_COPY_ASSIGN(OffsetTo);
    OffsetTo() = default;

    OffsetTo &operator=(typename OffsetType::type i)
    {
        OffsetType::operator=(i);
        return *this;
    }

    const Type &operator()(const void *base) const
    {
        if (unlikely(this->is_null()))
            return *_rb_has_null<Type, has_null>::get_null();
        return StructAtOffset<const Type>(base, *this);
    }
    Type &operator()(void *base) const
    {
        if (unlikely(this->is_null()))
            return *_rb_has_null<Type, has_null>::get_crap();
        return StructAtOffset<Type>(base, *this);
    }

    template <typename Base, rb_enable_if(rb_is_convertible(const Base, const void *))>
    friend const Type &operator+(const Base &base, const OffsetTo &offset)
    {
        return offset((const void *)base);
    }
    template <typename Base, rb_enable_if(rb_is_convertible(const Base, const void *))>
    friend const Type &operator+(const OffsetTo &offset, const Base &base)
    {
        return offset((const void *)base);
    }
    template <typename Base, rb_enable_if(rb_is_convertible(Base, void *))>
    friend Type &operator+(Base &&base, OffsetTo &offset)
    {
        return offset((void *)base);
    }
    template <typename Base, rb_enable_if(rb_is_convertible(Base, void *))>
    friend Type &operator+(OffsetTo &offset, Base &&base)
    {
        return offset((void *)base);
    }

    bool sanitize_shallow(rb_sanitize_context_t *c, const void *base) const
    {
        if (unlikely(!c->check_struct(this)))
            return false;
        if (unlikely(this->is_null()))
            return true;
        if (unlikely(!c->check_range(base, *this)))
            return false;
        return true;
    }

    template <typename... Ts> bool sanitize(rb_sanitize_context_t *c, const void *base, Ts &&... ds) const
    {
        return sanitize_shallow(c, base) &&
               (this->is_null() || c->dispatch(StructAtOffset<Type>(base, *this), rb_forward<Ts>(ds)...));
    }
    DEFINE_SIZE_STATIC(sizeof(OffsetType));
};
/* Partial specializations. */
template <typename Type, bool has_null = true> using LOffsetTo = OffsetTo<Type, HBUINT32, has_null>;
template <typename Type, typename OffsetType = HBUINT16> using NNOffsetTo = OffsetTo<Type, OffsetType, false>;
template <typename Type> using LNNOffsetTo = LOffsetTo<Type, false>;

/*
 * Array Types
 */

template <typename Type> struct UnsizedArrayOf
{
    typedef Type item_t;
    static constexpr unsigned item_size = rb_static_size(Type);

    RB_DELETE_CREATE_COPY_ASSIGN(UnsizedArrayOf);

    const Type &operator[](int i_) const
    {
        unsigned int i = (unsigned int)i_;
        const Type *p = &arrayZ[i];
        if (unlikely(p < arrayZ))
            return Null(Type); /* Overflowed. */
        return *p;
    }
    Type &operator[](int i_)
    {
        unsigned int i = (unsigned int)i_;
        Type *p = &arrayZ[i];
        if (unlikely(p < arrayZ))
            return Crap(Type); /* Overflowed. */
        return *p;
    }

    unsigned int get_size(unsigned int len) const
    {
        return len * Type::static_size;
    }

    template <typename T> operator T *()
    {
        return arrayZ;
    }
    template <typename T> operator const T *() const
    {
        return arrayZ;
    }
    rb_array_t<Type> as_array(unsigned int len)
    {
        return rb_array(arrayZ, len);
    }
    rb_array_t<const Type> as_array(unsigned int len) const
    {
        return rb_array(arrayZ, len);
    }
    operator rb_array_t<Type>()
    {
        return as_array();
    }
    operator rb_array_t<const Type>() const
    {
        return as_array();
    }

    template <typename... Ts> bool sanitize(rb_sanitize_context_t *c, unsigned int count, Ts &&... ds) const
    {
        if (unlikely(!sanitize_shallow(c, count)))
            return false;
        if (!sizeof...(Ts) && rb_is_trivially_copyable(Type))
            return true;
        for (unsigned int i = 0; i < count; i++)
            if (unlikely(!c->dispatch(arrayZ[i], rb_forward<Ts>(ds)...)))
                return false;
        return true;
    }

    bool sanitize_shallow(rb_sanitize_context_t *c, unsigned int count) const
    {
        return c->check_array(arrayZ, count);
    }

public:
    Type arrayZ[RB_VAR_ARRAY];

public:
    DEFINE_SIZE_UNBOUNDED(0);
};

/* Unsized array of offset's */
template <typename Type, typename OffsetType, bool has_null = true>
using UnsizedOffsetArrayOf = UnsizedArrayOf<OffsetTo<Type, OffsetType, has_null>>;

/* Unsized array of offsets relative to the beginning of the array itself. */
template <typename Type, typename OffsetType, bool has_null = true>
struct UnsizedOffsetListOf : UnsizedOffsetArrayOf<Type, OffsetType, has_null>
{
    const Type &operator[](int i_) const
    {
        unsigned int i = (unsigned int)i_;
        const OffsetTo<Type, OffsetType, has_null> *p = &this->arrayZ[i];
        if (unlikely(p < this->arrayZ))
            return Null(Type); /* Overflowed. */
        return this + *p;
    }
    Type &operator[](int i_)
    {
        unsigned int i = (unsigned int)i_;
        const OffsetTo<Type, OffsetType, has_null> *p = &this->arrayZ[i];
        if (unlikely(p < this->arrayZ))
            return Crap(Type); /* Overflowed. */
        return this + *p;
    }

    template <typename... Ts> bool sanitize(rb_sanitize_context_t *c, unsigned int count, Ts &&... ds) const
    {
        return (UnsizedOffsetArrayOf<Type, OffsetType, has_null>::sanitize(c, count, this, rb_forward<Ts>(ds)...));
    }
};

/* An array with sorted elements.  Supports binary searching. */
template <typename Type> struct SortedUnsizedArrayOf : UnsizedArrayOf<Type>
{
    rb_sorted_array_t<Type> as_array(unsigned int len)
    {
        return rb_sorted_array(this->arrayZ, len);
    }
    rb_sorted_array_t<const Type> as_array(unsigned int len) const
    {
        return rb_sorted_array(this->arrayZ, len);
    }
    operator rb_sorted_array_t<Type>()
    {
        return as_array();
    }
    operator rb_sorted_array_t<const Type>() const
    {
        return as_array();
    }

    template <typename T> Type &bsearch(unsigned int len, const T &x, Type &not_found = Crap(Type))
    {
        return *as_array(len).bsearch(x, &not_found);
    }
    template <typename T> const Type &bsearch(unsigned int len, const T &x, const Type &not_found = Null(Type)) const
    {
        return *as_array(len).bsearch(x, &not_found);
    }
    template <typename T>
    bool bfind(unsigned int len,
               const T &x,
               unsigned int *i = nullptr,
               rb_bfind_not_found_t not_found = RB_BFIND_NOT_FOUND_DONT_STORE,
               unsigned int to_store = (unsigned int)-1) const
    {
        return as_array(len).bfind(x, i, not_found, to_store);
    }
};

/* An array with a number of elements. */
template <typename Type, typename LenType = HBUINT16> struct ArrayOf
{
    RB_DELETE_CREATE_COPY_ASSIGN(ArrayOf);

    const Type &operator[](int i_) const
    {
        unsigned int i = (unsigned int)i_;
        if (unlikely(i >= len))
            return Null(Type);
        return arrayZ[i];
    }
    Type &operator[](int i_)
    {
        unsigned int i = (unsigned int)i_;
        if (unlikely(i >= len))
            return Crap(Type);
        return arrayZ[i];
    }

    unsigned int get_size() const
    {
        return len.static_size + len * Type::static_size;
    }

    template <typename... Ts> bool sanitize(rb_sanitize_context_t *c, Ts &&... ds) const
    {
        if (unlikely(!sanitize_shallow(c)))
            return false;
        if (!sizeof...(Ts) && rb_is_trivially_copyable(Type))
            return true;
        unsigned int count = len;
        for (unsigned int i = 0; i < count; i++)
            if (unlikely(!c->dispatch(arrayZ[i], rb_forward<Ts>(ds)...)))
                return false;
        return true;
    }

    bool sanitize_shallow(rb_sanitize_context_t *c) const
    {
        return len.sanitize(c) && c->check_array(arrayZ, len);
    }

public:
    LenType len;
    Type arrayZ[RB_VAR_ARRAY];

public:
    DEFINE_SIZE_ARRAY(sizeof(LenType), arrayZ);
};
template <typename Type> using LArrayOf = ArrayOf<Type, HBUINT32>;

/* An array storing length-1. */
template <typename Type, typename LenType = HBUINT16> struct ArrayOfM1
{
    RB_DELETE_CREATE_COPY_ASSIGN(ArrayOfM1);

    const Type &operator[](int i_) const
    {
        unsigned int i = (unsigned int)i_;
        if (unlikely(i > lenM1))
            return Null(Type);
        return arrayZ[i];
    }
    Type &operator[](int i_)
    {
        unsigned int i = (unsigned int)i_;
        if (unlikely(i > lenM1))
            return Crap(Type);
        return arrayZ[i];
    }
    unsigned int get_size() const
    {
        return lenM1.static_size + (lenM1 + 1) * Type::static_size;
    }

    template <typename... Ts> bool sanitize(rb_sanitize_context_t *c, Ts &&... ds) const
    {
        if (unlikely(!sanitize_shallow(c)))
            return false;
        unsigned int count = lenM1 + 1;
        for (unsigned int i = 0; i < count; i++)
            if (unlikely(!c->dispatch(arrayZ[i], rb_forward<Ts>(ds)...)))
                return false;
        return true;
    }

private:
    bool sanitize_shallow(rb_sanitize_context_t *c) const
    {
        return lenM1.sanitize(c) && (c->check_array(arrayZ, lenM1 + 1));
    }

public:
    LenType lenM1;
    Type arrayZ[RB_VAR_ARRAY];

public:
    DEFINE_SIZE_ARRAY(sizeof(LenType), arrayZ);
};

/* An array with sorted elements.  Supports binary searching. */
template <typename Type, typename LenType = HBUINT16> struct SortedArrayOf : ArrayOf<Type, LenType>
{
    rb_sorted_array_t<Type> as_array()
    {
        return rb_sorted_array(this->arrayZ, this->len);
    }
    rb_sorted_array_t<const Type> as_array() const
    {
        return rb_sorted_array(this->arrayZ, this->len);
    }

    rb_sorted_array_t<const Type> sub_array(unsigned int start_offset, unsigned int count) const
    {
        return as_array().sub_array(start_offset, count);
    }
    rb_sorted_array_t<const Type> sub_array(unsigned int start_offset, unsigned int *count = nullptr /* IN/OUT */) const
    {
        return as_array().sub_array(start_offset, count);
    }
    rb_sorted_array_t<Type> sub_array(unsigned int start_offset, unsigned int count)
    {
        return as_array().sub_array(start_offset, count);
    }
    rb_sorted_array_t<Type> sub_array(unsigned int start_offset, unsigned int *count = nullptr /* IN/OUT */)
    {
        return as_array().sub_array(start_offset, count);
    }

    template <typename T> Type &bsearch(const T &x, Type &not_found = Crap(Type))
    {
        return *as_array().bsearch(x, &not_found);
    }
    template <typename T> const Type &bsearch(const T &x, const Type &not_found = Null(Type)) const
    {
        return *as_array().bsearch(x, &not_found);
    }
    template <typename T>
    bool bfind(const T &x,
               unsigned int *i = nullptr,
               rb_bfind_not_found_t not_found = RB_BFIND_NOT_FOUND_DONT_STORE,
               unsigned int to_store = (unsigned int)-1) const
    {
        return as_array().bfind(x, i, not_found, to_store);
    }
};

/*
 * Binary-search arrays
 */

template <typename LenType = HBUINT16> struct BinSearchHeader
{
    operator uint32_t() const
    {
        return len;
    }

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this);
    }

    BinSearchHeader &operator=(unsigned int v)
    {
        len = v;
        assert(len == v);
        entrySelector = rb_max(1u, rb_bit_storage(v)) - 1;
        searchRange = 16 * (1u << entrySelector);
        rangeShift = v * 16 > searchRange ? 16 * v - searchRange : 0;
        return *this;
    }

protected:
    LenType len;
    LenType searchRange;
    LenType entrySelector;
    LenType rangeShift;

public:
    DEFINE_SIZE_STATIC(8);
};

template <typename Type, typename LenType = HBUINT16>
using BinSearchArrayOf = SortedArrayOf<Type, BinSearchHeader<LenType>>;

struct VarSizedBinSearchHeader
{

    bool sanitize(rb_sanitize_context_t *c) const
    {
        return c->check_struct(this);
    }

    HBUINT16 unitSize;      /* Size of a lookup unit for this search in bytes. */
    HBUINT16 nUnits;        /* Number of units of the preceding size to be searched. */
    HBUINT16 searchRange;   /* The value of unitSize times the largest power of 2
                             * that is less than or equal to the value of nUnits. */
    HBUINT16 entrySelector; /* The log base 2 of the largest power of 2 less than
                             * or equal to the value of nUnits. */
    HBUINT16 rangeShift;    /* The value of unitSize times the difference of the
                             * value of nUnits minus the largest power of 2 less
                             * than or equal to the value of nUnits. */
public:
    DEFINE_SIZE_STATIC(10);
};

template <typename Type> struct VarSizedBinSearchArrayOf
{
    static constexpr unsigned item_size = Type::static_size;

    RB_DELETE_CREATE_COPY_ASSIGN(VarSizedBinSearchArrayOf);

    bool last_is_terminator() const
    {
        if (unlikely(!header.nUnits))
            return false;

        /* Gah.
         *
         * "The number of termination values that need to be included is table-specific.
         * The value that indicates binary search termination is 0xFFFF." */
        const HBUINT16 *words = &StructAtOffset<HBUINT16>(&bytesZ, (header.nUnits - 1) * header.unitSize);
        unsigned int count = Type::TerminationWordCount;
        for (unsigned int i = 0; i < count; i++)
            if (words[i] != 0xFFFFu)
                return false;
        return true;
    }

    const Type &operator[](int i_) const
    {
        unsigned int i = (unsigned int)i_;
        if (unlikely(i >= get_length()))
            return Null(Type);
        return StructAtOffset<Type>(&bytesZ, i * header.unitSize);
    }
    Type &operator[](int i_)
    {
        unsigned int i = (unsigned int)i_;
        if (unlikely(i >= get_length()))
            return Crap(Type);
        return StructAtOffset<Type>(&bytesZ, i * header.unitSize);
    }
    unsigned int get_length() const
    {
        return header.nUnits - last_is_terminator();
    }
    unsigned int get_size() const
    {
        return header.static_size + header.nUnits * header.unitSize;
    }

    template <typename... Ts> bool sanitize(rb_sanitize_context_t *c, Ts &&... ds) const
    {
        if (unlikely(!sanitize_shallow(c)))
            return false;
        if (!sizeof...(Ts) && rb_is_trivially_copyable(Type))
            return true;
        unsigned int count = get_length();
        for (unsigned int i = 0; i < count; i++)
            if (unlikely(!(*this)[i].sanitize(c, rb_forward<Ts>(ds)...)))
                return false;
        return true;
    }

    template <typename T> const Type *bsearch(const T &key) const
    {
        unsigned pos;
        return rb_bsearch_impl(&pos, key, (const void *)bytesZ, get_length(), header.unitSize, _rb_cmp_method<T, Type>)
                   ? (const Type *)(((const char *)&bytesZ) + (pos * header.unitSize))
                   : nullptr;
    }

private:
    bool sanitize_shallow(rb_sanitize_context_t *c) const
    {
        return header.sanitize(c) && Type::static_size <= header.unitSize &&
               c->check_range(bytesZ.arrayZ, header.nUnits, header.unitSize);
    }

protected:
    VarSizedBinSearchHeader header;
    UnsizedArrayOf<HBUINT8> bytesZ;

public:
    DEFINE_SIZE_ARRAY(10, bytesZ);
};

} /* namespace OT */

#endif /* RB_OPEN_TYPE_HH */
