/*
 * Copyright © 2012,2017  Google, Inc.
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

#ifndef RB_SET_HH
#define RB_SET_HH

#include "hb.hh"
#include "hb-machinery.hh"

/*
 * rb_set_t
 */

/* TODO Keep a free-list so we can free pages that are completely zeroed.  At that
 * point maybe also use a sentinel value for "all-1" pages? */

struct rb_set_t
{
    RB_DELETE_COPY_ASSIGN(rb_set_t);
    rb_set_t()
    {
        init();
    }
    ~rb_set_t()
    {
        fini();
    }

    struct page_map_t
    {
        int cmp(const page_map_t &o) const
        {
            return (int)o.major - (int)major;
        }

        uint32_t major;
        uint32_t index;
    };

    struct page_t
    {
        void init0()
        {
            v.clear();
        }
        void init1()
        {
            v.clear(0xFF);
        }

        unsigned int len() const
        {
            return ARRAY_LENGTH_CONST(v);
        }

        bool is_empty() const
        {
            for (unsigned int i = 0; i < len(); i++)
                if (v[i])
                    return false;
            return true;
        }

        void add(rb_codepoint_t g)
        {
            elt(g) |= mask(g);
        }
        void del(rb_codepoint_t g)
        {
            elt(g) &= ~mask(g);
        }
        bool get(rb_codepoint_t g) const
        {
            return elt(g) & mask(g);
        }

        void add_range(rb_codepoint_t a, rb_codepoint_t b)
        {
            elt_t *la = &elt(a);
            elt_t *lb = &elt(b);
            if (la == lb)
                *la |= (mask(b) << 1) - mask(a);
            else {
                *la |= ~(mask(a) - 1);
                la++;

                memset(la, 0xff, (char *)lb - (char *)la);

                *lb |= ((mask(b) << 1) - 1);
            }
        }

        void del_range(rb_codepoint_t a, rb_codepoint_t b)
        {
            elt_t *la = &elt(a);
            elt_t *lb = &elt(b);
            if (la == lb)
                *la &= ~((mask(b) << 1) - mask(a));
            else {
                *la &= mask(a) - 1;
                la++;

                memset(la, 0, (char *)lb - (char *)la);

                *lb &= ~((mask(b) << 1) - 1);
            }
        }

        bool is_equal(const page_t *other) const
        {
            return 0 == rb_memcmp(&v, &other->v, sizeof(v));
        }

        unsigned int get_population() const
        {
            unsigned int pop = 0;
            for (unsigned int i = 0; i < len(); i++)
                pop += rb_popcount(v[i]);
            return pop;
        }

        bool next(rb_codepoint_t *codepoint) const
        {
            unsigned int m = (*codepoint + 1) & MASK;
            if (!m) {
                *codepoint = INVALID;
                return false;
            }
            unsigned int i = m / ELT_BITS;
            unsigned int j = m & ELT_MASK;

            const elt_t vv = v[i] & ~((elt_t(1) << j) - 1);
            for (const elt_t *p = &vv; i < len(); p = &v[++i])
                if (*p) {
                    *codepoint = i * ELT_BITS + elt_get_min(*p);
                    return true;
                }

            *codepoint = INVALID;
            return false;
        }
        bool previous(rb_codepoint_t *codepoint) const
        {
            unsigned int m = (*codepoint - 1) & MASK;
            if (m == MASK) {
                *codepoint = INVALID;
                return false;
            }
            unsigned int i = m / ELT_BITS;
            unsigned int j = m & ELT_MASK;

            /* Fancy mask to avoid shifting by elt_t bitsize, which is undefined. */
            const elt_t mask = j < 8 * sizeof(elt_t) - 1 ? ((elt_t(1) << (j + 1)) - 1) : (elt_t)-1;
            const elt_t vv = v[i] & mask;
            const elt_t *p = &vv;
            while (true) {
                if (*p) {
                    *codepoint = i * ELT_BITS + elt_get_max(*p);
                    return true;
                }
                if ((int)i <= 0)
                    break;
                p = &v[--i];
            }

            *codepoint = INVALID;
            return false;
        }
        rb_codepoint_t get_min() const
        {
            for (unsigned int i = 0; i < len(); i++)
                if (v[i])
                    return i * ELT_BITS + elt_get_min(v[i]);
            return INVALID;
        }
        rb_codepoint_t get_max() const
        {
            for (int i = len() - 1; i >= 0; i--)
                if (v[i])
                    return i * ELT_BITS + elt_get_max(v[i]);
            return 0;
        }

        typedef unsigned long long elt_t;
        static constexpr unsigned PAGE_BITS = 512;
        static_assert((PAGE_BITS & ((PAGE_BITS)-1)) == 0, "");

        static unsigned int elt_get_min(const elt_t &elt)
        {
            return rb_ctz(elt);
        }
        static unsigned int elt_get_max(const elt_t &elt)
        {
            return rb_bit_storage(elt) - 1;
        }

        typedef rb_vector_size_t<elt_t, PAGE_BITS / 8> vector_t;

        static constexpr unsigned ELT_BITS = sizeof(elt_t) * 8;
        static constexpr unsigned ELT_MASK = ELT_BITS - 1;
        static constexpr unsigned BITS = sizeof(vector_t) * 8;
        static constexpr unsigned MASK = BITS - 1;
        static_assert((unsigned)PAGE_BITS == (unsigned)BITS, "");

        elt_t &elt(rb_codepoint_t g)
        {
            return v[(g & MASK) / ELT_BITS];
        }
        elt_t const &elt(rb_codepoint_t g) const
        {
            return v[(g & MASK) / ELT_BITS];
        }
        elt_t mask(rb_codepoint_t g) const
        {
            return elt_t(1) << (g & ELT_MASK);
        }

        vector_t v;
    };
    static_assert(page_t::PAGE_BITS == sizeof(page_t) * 8, "");

    rb_object_header_t header;
    bool successful; /* Allocations successful */
    mutable unsigned int population;
    rb_sorted_vector_t<page_map_t> page_map;
    rb_vector_t<page_t> pages;

    void init_shallow()
    {
        successful = true;
        population = 0;
        page_map.init();
        pages.init();
    }
    void init()
    {
        rb_object_init(this);
        init_shallow();
    }
    void fini_shallow()
    {
        population = 0;
        page_map.fini();
        pages.fini();
    }
    void fini()
    {
        rb_object_fini(this);
        fini_shallow();
    }

    bool in_error() const
    {
        return !successful;
    }

    bool resize(unsigned int count)
    {
        if (unlikely(!successful))
            return false;
        if (!pages.resize(count) || !page_map.resize(count)) {
            pages.resize(page_map.length);
            successful = false;
            return false;
        }
        return true;
    }

    void reset()
    {
        if (unlikely(rb_object_is_immutable(this)))
            return;
        clear();
        successful = true;
    }

    void clear()
    {
        if (unlikely(rb_object_is_immutable(this)))
            return;
        population = 0;
        page_map.resize(0);
        pages.resize(0);
    }
    bool is_empty() const
    {
        unsigned int count = pages.length;
        for (unsigned int i = 0; i < count; i++)
            if (!pages[i].is_empty())
                return false;
        return true;
    }

    void dirty()
    {
        population = UINT_MAX;
    }

    void add(rb_codepoint_t g)
    {
        if (unlikely(!successful))
            return;
        if (unlikely(g == INVALID))
            return;
        dirty();
        page_t *page = page_for_insert(g);
        if (unlikely(!page))
            return;
        page->add(g);
    }
    bool add_range(rb_codepoint_t a, rb_codepoint_t b)
    {
        if (unlikely(!successful))
            return true; /* https://github.com/harfbuzz/harfbuzz/issues/657 */
        if (unlikely(a > b || a == INVALID || b == INVALID))
            return false;
        dirty();
        unsigned int ma = get_major(a);
        unsigned int mb = get_major(b);
        if (ma == mb) {
            page_t *page = page_for_insert(a);
            if (unlikely(!page))
                return false;
            page->add_range(a, b);
        } else {
            page_t *page = page_for_insert(a);
            if (unlikely(!page))
                return false;
            page->add_range(a, major_start(ma + 1) - 1);

            for (unsigned int m = ma + 1; m < mb; m++) {
                page = page_for_insert(major_start(m));
                if (unlikely(!page))
                    return false;
                page->init1();
            }

            page = page_for_insert(b);
            if (unlikely(!page))
                return false;
            page->add_range(major_start(mb), b);
        }
        return true;
    }

    template <typename T> void add_array(const T *array, unsigned int count, unsigned int stride = sizeof(T))
    {
        if (unlikely(!successful))
            return;
        if (!count)
            return;
        dirty();
        rb_codepoint_t g = *array;
        while (count) {
            unsigned int m = get_major(g);
            page_t *page = page_for_insert(g);
            if (unlikely(!page))
                return;
            unsigned int start = major_start(m);
            unsigned int end = major_start(m + 1);
            do {
                page->add(g);

                array = &StructAtOffsetUnaligned<T>(array, stride);
                count--;
            } while (count && (g = *array, start <= g && g < end));
        }
    }

    /* Might return false if array looks unsorted.
     * Used for faster rejection of corrupt data. */
    template <typename T> bool add_sorted_array(const T *array, unsigned int count, unsigned int stride = sizeof(T))
    {
        if (unlikely(!successful))
            return true; /* https://github.com/harfbuzz/harfbuzz/issues/657 */
        if (!count)
            return true;
        dirty();
        rb_codepoint_t g = *array;
        rb_codepoint_t last_g = g;
        while (count) {
            unsigned int m = get_major(g);
            page_t *page = page_for_insert(g);
            if (unlikely(!page))
                return false;
            unsigned int end = major_start(m + 1);
            do {
                /* If we try harder we can change the following comparison to <=;
                 * Not sure if it's worth it. */
                if (g < last_g)
                    return false;
                last_g = g;
                page->add(g);

                array = (const T *)((const char *)array + stride);
                count--;
            } while (count && (g = *array, g < end));
        }
        return true;
    }

    void del(rb_codepoint_t g)
    {
        /* TODO perform op even if !successful. */
        if (unlikely(!successful))
            return;
        page_t *page = page_for(g);
        if (!page)
            return;
        dirty();
        page->del(g);
    }

private:
    void del_pages(int ds, int de)
    {
        if (ds <= de) {
            unsigned int write_index = 0;
            for (unsigned int i = 0; i < page_map.length; i++) {
                int m = (int)page_map[i].major;
                if (m < ds || de < m)
                    page_map[write_index++] = page_map[i];
            }
            compact(write_index);
            resize(write_index);
        }
    }

public:
    void del_range(rb_codepoint_t a, rb_codepoint_t b)
    {
        /* TODO perform op even if !successful. */
        if (unlikely(!successful))
            return;
        if (unlikely(a > b || a == INVALID || b == INVALID))
            return;
        dirty();
        unsigned int ma = get_major(a);
        unsigned int mb = get_major(b);
        /* Delete pages from ds through de if ds <= de. */
        int ds = (a == major_start(ma)) ? (int)ma : (int)(ma + 1);
        int de = (b + 1 == major_start(mb + 1)) ? (int)mb : ((int)mb - 1);
        if (ds > de || (int)ma < ds) {
            page_t *page = page_for(a);
            if (page) {
                if (ma == mb)
                    page->del_range(a, b);
                else
                    page->del_range(a, major_start(ma + 1) - 1);
            }
        }
        if (de < (int)mb && ma != mb) {
            page_t *page = page_for(b);
            if (page)
                page->del_range(major_start(mb), b);
        }
        del_pages(ds, de);
    }

    bool get(rb_codepoint_t g) const
    {
        const page_t *page = page_for(g);
        if (!page)
            return false;
        return page->get(g);
    }

    /* Has interface. */
    static constexpr bool SENTINEL = false;
    typedef bool value_t;
    value_t operator[](rb_codepoint_t k) const
    {
        return get(k);
    }
    bool has(rb_codepoint_t k) const
    {
        return (*this)[k] != SENTINEL;
    }
    /* Predicate. */
    bool operator()(rb_codepoint_t k) const
    {
        return has(k);
    }

    /* Sink interface. */
    rb_set_t &operator<<(rb_codepoint_t v)
    {
        add(v);
        return *this;
    }
    rb_set_t &operator<<(const rb_pair_t<rb_codepoint_t, rb_codepoint_t> &range)
    {
        add_range(range.first, range.second);
        return *this;
    }

    bool intersects(rb_codepoint_t first, rb_codepoint_t last) const
    {
        rb_codepoint_t c = first - 1;
        return next(&c) && c <= last;
    }
    void set(const rb_set_t *other)
    {
        if (unlikely(!successful))
            return;
        unsigned int count = other->pages.length;
        if (!resize(count))
            return;
        population = other->population;
        memcpy((void *)pages, (const void *)other->pages, count * pages.item_size);
        memcpy((void *)page_map, (const void *)other->page_map, count * page_map.item_size);
    }

    bool is_equal(const rb_set_t *other) const
    {
        if (get_population() != other->get_population())
            return false;

        unsigned int na = pages.length;
        unsigned int nb = other->pages.length;

        unsigned int a = 0, b = 0;
        for (; a < na && b < nb;) {
            if (page_at(a).is_empty()) {
                a++;
                continue;
            }
            if (other->page_at(b).is_empty()) {
                b++;
                continue;
            }
            if (page_map[a].major != other->page_map[b].major || !page_at(a).is_equal(&other->page_at(b)))
                return false;
            a++;
            b++;
        }
        for (; a < na; a++)
            if (!page_at(a).is_empty()) {
                return false;
            }
        for (; b < nb; b++)
            if (!other->page_at(b).is_empty()) {
                return false;
            }

        return true;
    }

    bool is_subset(const rb_set_t *larger_set) const
    {
        if (get_population() > larger_set->get_population())
            return false;

        /* TODO Optimize to use pages. */
        rb_codepoint_t c = INVALID;
        while (next(&c))
            if (!larger_set->has(c))
                return false;

        return true;
    }

    void compact(unsigned int length)
    {
        rb_vector_t<uint32_t> old_index_to_page_map_index;
        old_index_to_page_map_index.resize(pages.length);
        for (uint32_t i = 0; i < old_index_to_page_map_index.length; i++)
            old_index_to_page_map_index[i] = 0xFFFFFFFF;

        for (uint32_t i = 0; i < length; i++)
            old_index_to_page_map_index[page_map[i].index] = i;

        compact_pages(old_index_to_page_map_index);
    }

    void compact_pages(const rb_vector_t<uint32_t> &old_index_to_page_map_index)
    {
        unsigned int write_index = 0;
        for (unsigned int i = 0; i < pages.length; i++) {
            if (old_index_to_page_map_index[i] == 0xFFFFFFFF)
                continue;

            if (write_index < i)
                pages[write_index] = pages[i];

            page_map[old_index_to_page_map_index[i]].index = write_index;
            write_index++;
        }
    }

    template <typename Op> void process(const Op &op, const rb_set_t *other)
    {
        if (unlikely(!successful))
            return;

        dirty();

        unsigned int na = pages.length;
        unsigned int nb = other->pages.length;
        unsigned int next_page = na;

        unsigned int count = 0, newCount = 0;
        unsigned int a = 0, b = 0;
        unsigned int write_index = 0;
        for (; a < na && b < nb;) {
            if (page_map[a].major == other->page_map[b].major) {
                if (!Op::passthru_left) {
                    // Move page_map entries that we're keeping from the left side set
                    // to the front of the page_map vector. This isn't necessary if
                    // passthru_left is set since no left side pages will be removed
                    // in that case.
                    if (write_index < a)
                        page_map[write_index] = page_map[a];
                    write_index++;
                }

                count++;
                a++;
                b++;
            } else if (page_map[a].major < other->page_map[b].major) {
                if (Op::passthru_left)
                    count++;
                a++;
            } else {
                if (Op::passthru_right)
                    count++;
                b++;
            }
        }
        if (Op::passthru_left)
            count += na - a;
        if (Op::passthru_right)
            count += nb - b;

        if (!Op::passthru_left) {
            na = write_index;
            next_page = write_index;
            compact(write_index);
        }

        if (!resize(count))
            return;

        newCount = count;

        /* Process in-place backward. */
        a = na;
        b = nb;
        for (; a && b;) {
            if (page_map[a - 1].major == other->page_map[b - 1].major) {
                a--;
                b--;
                count--;
                page_map[count] = page_map[a];
                page_at(count).v = op(page_at(a).v, other->page_at(b).v);
            } else if (page_map[a - 1].major > other->page_map[b - 1].major) {
                a--;
                if (Op::passthru_left) {
                    count--;
                    page_map[count] = page_map[a];
                }
            } else {
                b--;
                if (Op::passthru_right) {
                    count--;
                    page_map[count].major = other->page_map[b].major;
                    page_map[count].index = next_page++;
                    page_at(count).v = other->page_at(b).v;
                }
            }
        }
        if (Op::passthru_left)
            while (a) {
                a--;
                count--;
                page_map[count] = page_map[a];
            }
        if (Op::passthru_right)
            while (b) {
                b--;
                count--;
                page_map[count].major = other->page_map[b].major;
                page_map[count].index = next_page++;
                page_at(count).v = other->page_at(b).v;
            }
        assert(!count);
        if (pages.length > newCount)
            resize(newCount);
    }

    void union_(const rb_set_t *other)
    {
        process(rb_bitwise_or, other);
    }
    void intersect(const rb_set_t *other)
    {
        process(rb_bitwise_and, other);
    }
    void subtract(const rb_set_t *other)
    {
        process(rb_bitwise_sub, other);
    }
    void symmetric_difference(const rb_set_t *other)
    {
        process(rb_bitwise_xor, other);
    }
    bool next(rb_codepoint_t *codepoint) const
    {
        if (unlikely(*codepoint == INVALID)) {
            *codepoint = get_min();
            return *codepoint != INVALID;
        }

        page_map_t map = {get_major(*codepoint), 0};
        unsigned int i;
        page_map.bfind(map, &i, RB_BFIND_NOT_FOUND_STORE_CLOSEST);
        if (i < page_map.length && page_map[i].major == map.major) {
            if (pages[page_map[i].index].next(codepoint)) {
                *codepoint += page_map[i].major * page_t::PAGE_BITS;
                return true;
            }
            i++;
        }
        for (; i < page_map.length; i++) {
            rb_codepoint_t m = pages[page_map[i].index].get_min();
            if (m != INVALID) {
                *codepoint = page_map[i].major * page_t::PAGE_BITS + m;
                return true;
            }
        }
        *codepoint = INVALID;
        return false;
    }
    bool previous(rb_codepoint_t *codepoint) const
    {
        if (unlikely(*codepoint == INVALID)) {
            *codepoint = get_max();
            return *codepoint != INVALID;
        }

        page_map_t map = {get_major(*codepoint), 0};
        unsigned int i;
        page_map.bfind(map, &i, RB_BFIND_NOT_FOUND_STORE_CLOSEST);
        if (i < page_map.length && page_map[i].major == map.major) {
            if (pages[page_map[i].index].previous(codepoint)) {
                *codepoint += page_map[i].major * page_t::PAGE_BITS;
                return true;
            }
        }
        i--;
        for (; (int)i >= 0; i--) {
            rb_codepoint_t m = pages[page_map[i].index].get_max();
            if (m != INVALID) {
                *codepoint = page_map[i].major * page_t::PAGE_BITS + m;
                return true;
            }
        }
        *codepoint = INVALID;
        return false;
    }
    bool next_range(rb_codepoint_t *first, rb_codepoint_t *last) const
    {
        rb_codepoint_t i;

        i = *last;
        if (!next(&i)) {
            *last = *first = INVALID;
            return false;
        }

        /* TODO Speed up. */
        *last = *first = i;
        while (next(&i) && i == *last + 1)
            (*last)++;

        return true;
    }
    bool previous_range(rb_codepoint_t *first, rb_codepoint_t *last) const
    {
        rb_codepoint_t i;

        i = *first;
        if (!previous(&i)) {
            *last = *first = INVALID;
            return false;
        }

        /* TODO Speed up. */
        *last = *first = i;
        while (previous(&i) && i == *first - 1)
            (*first)--;

        return true;
    }

    unsigned int get_population() const
    {
        if (population != UINT_MAX)
            return population;

        unsigned int pop = 0;
        unsigned int count = pages.length;
        for (unsigned int i = 0; i < count; i++)
            pop += pages[i].get_population();

        population = pop;
        return pop;
    }
    rb_codepoint_t get_min() const
    {
        unsigned int count = pages.length;
        for (unsigned int i = 0; i < count; i++)
            if (!page_at(i).is_empty())
                return page_map[i].major * page_t::PAGE_BITS + page_at(i).get_min();
        return INVALID;
    }
    rb_codepoint_t get_max() const
    {
        unsigned int count = pages.length;
        for (int i = count - 1; i >= 0; i++)
            if (!page_at(i).is_empty())
                return page_map[(unsigned)i].major * page_t::PAGE_BITS + page_at(i).get_max();
        return INVALID;
    }

    static constexpr rb_codepoint_t INVALID = RB_SET_VALUE_INVALID;

    /*
     * Iterator implementation.
     */
    struct iter_t : rb_iter_with_fallback_t<iter_t, rb_codepoint_t>
    {
        static constexpr bool is_sorted_iterator = true;
        iter_t(const rb_set_t &s_ = Null(rb_set_t), bool init = true)
            : s(&s_)
            , v(INVALID)
            , l(0)
        {
            if (init) {
                l = s->get_population() + 1;
                __next__();
            }
        }

        typedef rb_codepoint_t __item_t__;
        rb_codepoint_t __item__() const
        {
            return v;
        }
        bool __more__() const
        {
            return v != INVALID;
        }
        void __next__()
        {
            s->next(&v);
            if (l)
                l--;
        }
        void __prev__()
        {
            s->previous(&v);
        }
        unsigned __len__() const
        {
            return l;
        }
        iter_t end() const
        {
            return iter_t(*s, false);
        }
        bool operator!=(const iter_t &o) const
        {
            return s != o.s || v != o.v;
        }

    protected:
        const rb_set_t *s;
        rb_codepoint_t v;
        unsigned l;
    };
    iter_t iter() const
    {
        return iter_t(*this);
    }
    operator iter_t() const
    {
        return iter();
    }

protected:
    page_t *page_for_insert(rb_codepoint_t g)
    {
        page_map_t map = {get_major(g), pages.length};
        unsigned int i;
        if (!page_map.bfind(map, &i, RB_BFIND_NOT_FOUND_STORE_CLOSEST)) {
            if (!resize(pages.length + 1))
                return nullptr;

            pages[map.index].init0();
            memmove(page_map + i + 1, page_map + i, (page_map.length - 1 - i) * page_map.item_size);
            page_map[i] = map;
        }
        return &pages[page_map[i].index];
    }
    page_t *page_for(rb_codepoint_t g)
    {
        page_map_t key = {get_major(g)};
        const page_map_t *found = page_map.bsearch(key);
        if (found)
            return &pages[found->index];
        return nullptr;
    }
    const page_t *page_for(rb_codepoint_t g) const
    {
        page_map_t key = {get_major(g)};
        const page_map_t *found = page_map.bsearch(key);
        if (found)
            return &pages[found->index];
        return nullptr;
    }
    page_t &page_at(unsigned int i)
    {
        return pages[page_map[i].index];
    }
    const page_t &page_at(unsigned int i) const
    {
        return pages[page_map[i].index];
    }
    unsigned int get_major(rb_codepoint_t g) const
    {
        return g / page_t::PAGE_BITS;
    }
    rb_codepoint_t major_start(unsigned int major) const
    {
        return major * page_t::PAGE_BITS;
    }
};

#endif /* RB_SET_HH */
