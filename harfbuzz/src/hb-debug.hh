/*
 * Copyright © 2017  Google, Inc.
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

#ifndef RB_DEBUG_HH
#define RB_DEBUG_HH

#include "hb.hh"
#include "hb-atomic.hh"
#include "hb-algs.hh"

#ifndef RB_DEBUG
#define RB_DEBUG 0
#endif

/*
 * Debug output (needs enabling at compile time.)
 */

static inline bool _rb_debug(unsigned int level, unsigned int max_level)
{
    return level < max_level;
}

#define DEBUG_LEVEL_ENABLED(WHAT, LEVEL) (_rb_debug((LEVEL), RB_DEBUG_##WHAT))
#define DEBUG_ENABLED(WHAT) (DEBUG_LEVEL_ENABLED(WHAT, 0))

static inline void _rb_print_func(const char *func)
{
    if (func) {
        unsigned int func_len = strlen(func);
        /* Skip "static" */
        if (0 == strncmp(func, "static ", 7))
            func += 7;
        /* Skip "typename" */
        if (0 == strncmp(func, "typename ", 9))
            func += 9;
        /* Skip return type */
        const char *space = strchr(func, ' ');
        if (space)
            func = space + 1;
        /* Skip parameter list */
        const char *paren = strchr(func, '(');
        if (paren)
            func_len = paren - func;
        fprintf(stderr, "%.*s", func_len, func);
    }
}

template <int max_level>
static inline void _rb_debug_msg_va(const char *what,
                                    const void *obj,
                                    const char *func,
                                    bool indented,
                                    unsigned int level,
                                    int level_dir,
                                    const char *message,
                                    va_list ap) RB_PRINTF_FUNC(7, 0);
template <int max_level>
static inline void _rb_debug_msg_va(const char *what,
                                    const void *obj,
                                    const char *func,
                                    bool indented,
                                    unsigned int level,
                                    int level_dir,
                                    const char *message,
                                    va_list ap)
{
    if (!_rb_debug(level, max_level))
        return;

    fprintf(stderr, "%-10s", what ? what : "");

    if (obj)
        fprintf(stderr, "(%*p) ", (unsigned int)(2 * sizeof(void *)), obj);
    else
        fprintf(stderr, " %*s  ", (unsigned int)(2 * sizeof(void *)), "");

    if (indented) {
#define VBAR "\342\224\202"  /* U+2502 BOX DRAWINGS LIGHT VERTICAL */
#define VRBAR "\342\224\234" /* U+251C BOX DRAWINGS LIGHT VERTICAL AND RIGHT */
#define DLBAR "\342\225\256" /* U+256E BOX DRAWINGS LIGHT ARC DOWN AND LEFT */
#define ULBAR "\342\225\257" /* U+256F BOX DRAWINGS LIGHT ARC UP AND LEFT */
#define LBAR "\342\225\264"  /* U+2574 BOX DRAWINGS LIGHT LEFT */
        static const char bars[] = VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR
            VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR
                VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR VBAR;
        fprintf(stderr,
                "%2u %s" VRBAR "%s",
                level,
                bars + sizeof(bars) - 1 -
                    rb_min((unsigned int)sizeof(bars) - 1, (unsigned int)(sizeof(VBAR) - 1) * level),
                level_dir ? (level_dir > 0 ? DLBAR : ULBAR) : LBAR);
    } else
        fprintf(stderr, "   " VRBAR LBAR);

    _rb_print_func(func);

    if (message) {
        fprintf(stderr, ": ");
        vfprintf(stderr, message, ap);
    }

    fprintf(stderr, "\n");
}
template <>
inline void RB_PRINTF_FUNC(7, 0) _rb_debug_msg_va<0>(const char *what RB_UNUSED,
                                                     const void *obj RB_UNUSED,
                                                     const char *func RB_UNUSED,
                                                     bool indented RB_UNUSED,
                                                     unsigned int level RB_UNUSED,
                                                     int level_dir RB_UNUSED,
                                                     const char *message RB_UNUSED,
                                                     va_list ap RB_UNUSED)
{
}

template <int max_level>
static inline void _rb_debug_msg(const char *what,
                                 const void *obj,
                                 const char *func,
                                 bool indented,
                                 unsigned int level,
                                 int level_dir,
                                 const char *message,
                                 ...) RB_PRINTF_FUNC(7, 8);
template <int max_level>
static inline void RB_PRINTF_FUNC(7, 8) _rb_debug_msg(const char *what,
                                                      const void *obj,
                                                      const char *func,
                                                      bool indented,
                                                      unsigned int level,
                                                      int level_dir,
                                                      const char *message,
                                                      ...)
{
    va_list ap;
    va_start(ap, message);
    _rb_debug_msg_va<max_level>(what, obj, func, indented, level, level_dir, message, ap);
    va_end(ap);
}
template <>
inline void _rb_debug_msg<0>(const char *what RB_UNUSED,
                             const void *obj RB_UNUSED,
                             const char *func RB_UNUSED,
                             bool indented RB_UNUSED,
                             unsigned int level RB_UNUSED,
                             int level_dir RB_UNUSED,
                             const char *message RB_UNUSED,
                             ...) RB_PRINTF_FUNC(7, 8);
template <>
inline void RB_PRINTF_FUNC(7, 8) _rb_debug_msg<0>(const char *what RB_UNUSED,
                                                  const void *obj RB_UNUSED,
                                                  const char *func RB_UNUSED,
                                                  bool indented RB_UNUSED,
                                                  unsigned int level RB_UNUSED,
                                                  int level_dir RB_UNUSED,
                                                  const char *message RB_UNUSED,
                                                  ...)
{
}

#define DEBUG_MSG_LEVEL(WHAT, OBJ, LEVEL, LEVEL_DIR, ...)                                                              \
    _rb_debug_msg<RB_DEBUG_##WHAT>(#WHAT, (OBJ), nullptr, true, (LEVEL), (LEVEL_DIR), __VA_ARGS__)
#define DEBUG_MSG(WHAT, OBJ, ...) _rb_debug_msg<RB_DEBUG_##WHAT>(#WHAT, (OBJ), nullptr, false, 0, 0, __VA_ARGS__)
#define DEBUG_MSG_FUNC(WHAT, OBJ, ...) _rb_debug_msg<RB_DEBUG_##WHAT>(#WHAT, (OBJ), RB_FUNC, false, 0, 0, __VA_ARGS__)

/*
 * Printer
 */

template <typename T> struct rb_printer_t
{
    const char *print(const T &)
    {
        return "something";
    }
};

template <> struct rb_printer_t<bool>
{
    const char *print(bool v)
    {
        return v ? "true" : "false";
    }
};

template <> struct rb_printer_t<rb_empty_t>
{
    const char *print(rb_empty_t)
    {
        return "";
    }
};

/*
 * Trace
 */

template <typename T> static inline void _rb_warn_no_return(bool returned)
{
    if (unlikely(!returned)) {
        fprintf(stderr, "OUCH, returned with no call to return_trace().  This is a bug, please report.\n");
    }
}
template <>
/*static*/ inline void _rb_warn_no_return<rb_empty_t>(bool returned RB_UNUSED)
{
}

template <int max_level, typename ret_t> struct rb_auto_trace_t
{
    explicit inline rb_auto_trace_t(
        unsigned int *plevel_, const char *what_, const void *obj_, const char *func, const char *message, ...)
        RB_PRINTF_FUNC(6, 7)
        : plevel(plevel_)
        , what(what_)
        , obj(obj_)
        , returned(false)
    {
        if (plevel)
            ++*plevel;

        va_list ap;
        va_start(ap, message);
        _rb_debug_msg_va<max_level>(what, obj, func, true, plevel ? *plevel : 0, +1, message, ap);
        va_end(ap);
    }
    ~rb_auto_trace_t()
    {
        _rb_warn_no_return<ret_t>(returned);
        if (!returned) {
            _rb_debug_msg<max_level>(what, obj, nullptr, true, plevel ? *plevel : 1, -1, " ");
        }
        if (plevel)
            --*plevel;
    }

    template <typename T> T ret(T &&v, const char *func = "", unsigned int line = 0)
    {
        if (unlikely(returned)) {
            fprintf(stderr, "OUCH, double calls to return_trace().  This is a bug, please report.\n");
            return rb_forward<T>(v);
        }

        _rb_debug_msg<max_level>(what,
                                 obj,
                                 func,
                                 true,
                                 plevel ? *plevel : 1,
                                 -1,
                                 "return %s (line %d)",
                                 rb_printer_t<decltype(v)>().print(v),
                                 line);
        if (plevel)
            --*plevel;
        plevel = nullptr;
        returned = true;
        return rb_forward<T>(v);
    }

private:
    unsigned int *plevel;
    const char *what;
    const void *obj;
    bool returned;
};
template <typename ret_t> /* Make sure we don't use rb_auto_trace_t when not tracing. */
struct rb_auto_trace_t<0, ret_t>
{
    explicit inline rb_auto_trace_t(
        unsigned int *plevel_, const char *what_, const void *obj_, const char *func, const char *message, ...)
        RB_PRINTF_FUNC(6, 7)
    {
    }

    template <typename T> T ret(T &&v, const char *func RB_UNUSED = nullptr, unsigned int line RB_UNUSED = 0)
    {
        return rb_forward<T>(v);
    }
};

/* For disabled tracing; optimize out everything.
 * https://github.com/harfbuzz/harfbuzz/pull/605 */
template <typename ret_t> struct rb_no_trace_t
{
    template <typename T> T ret(T &&v, const char *func RB_UNUSED = nullptr, unsigned int line RB_UNUSED = 0)
    {
        return rb_forward<T>(v);
    }
};

#define return_trace(RET) return trace.ret(RET, RB_FUNC, __LINE__)

/*
 * Instances.
 */

#ifndef RB_DEBUG_ARABIC
#define RB_DEBUG_ARABIC (RB_DEBUG + 0)
#endif

#ifndef RB_DEBUG_BLOB
#define RB_DEBUG_BLOB (RB_DEBUG + 0)
#endif

#ifndef RB_DEBUG_CORETEXT
#define RB_DEBUG_CORETEXT (RB_DEBUG + 0)
#endif

#ifndef RB_DEBUG_DIRECTWRITE
#define RB_DEBUG_DIRECTWRITE (RB_DEBUG + 0)
#endif

#ifndef RB_DEBUG_FT
#define RB_DEBUG_FT (RB_DEBUG + 0)
#endif

#ifndef RB_DEBUG_OBJECT
#define RB_DEBUG_OBJECT (RB_DEBUG + 0)
#endif

#ifndef RB_DEBUG_SHAPE_PLAN
#define RB_DEBUG_SHAPE_PLAN (RB_DEBUG + 0)
#endif

#ifndef RB_DEBUG_UNISCRIBE
#define RB_DEBUG_UNISCRIBE (RB_DEBUG + 0)
#endif

/*
 * With tracing.
 */

#ifndef RB_DEBUG_APPLY
#define RB_DEBUG_APPLY (RB_DEBUG + 0)
#endif
#if RB_DEBUG_APPLY
#define TRACE_APPLY(this)                                                                                              \
    rb_auto_trace_t<RB_DEBUG_APPLY, bool> trace(&c->debug_depth,                                                       \
                                                c->get_name(),                                                         \
                                                this,                                                                  \
                                                RB_FUNC,                                                               \
                                                "idx %d gid %u lookup %d",                                             \
                                                c->buffer->idx,                                                        \
                                                c->buffer->cur().codepoint,                                            \
                                                (int)c->lookup_index)
#else
#define TRACE_APPLY(this) rb_no_trace_t<bool> trace
#endif

#ifndef RB_DEBUG_SANITIZE
#define RB_DEBUG_SANITIZE (RB_DEBUG + 0)
#endif
#if RB_DEBUG_SANITIZE
#define TRACE_SANITIZE(this)                                                                                           \
    rb_auto_trace_t<RB_DEBUG_SANITIZE, bool> trace(&c->debug_depth, c->get_name(), this, RB_FUNC, " ")
#else
#define TRACE_SANITIZE(this) rb_no_trace_t<bool> trace
#endif

#ifndef RB_DEBUG_SERIALIZE
#define RB_DEBUG_SERIALIZE (RB_DEBUG + 0)
#endif
#if RB_DEBUG_SERIALIZE
#define TRACE_SERIALIZE(this)                                                                                          \
    rb_auto_trace_t<RB_DEBUG_SERIALIZE, bool> trace(&c->debug_depth, "SERIALIZE", c, RB_FUNC, " ")
#else
#define TRACE_SERIALIZE(this) rb_no_trace_t<bool> trace
#endif

#ifndef RB_DEBUG_SUBSET
#define RB_DEBUG_SUBSET (RB_DEBUG + 0)
#endif
#if RB_DEBUG_SUBSET
#define TRACE_SUBSET(this)                                                                                             \
    rb_auto_trace_t<RB_DEBUG_SUBSET, bool> trace(&c->debug_depth, c->get_name(), this, RB_FUNC, " ")
#else
#define TRACE_SUBSET(this) rb_no_trace_t<bool> trace
#endif

#ifndef RB_DEBUG_DISPATCH
#define RB_DEBUG_DISPATCH (RB_DEBUG_APPLY + RB_DEBUG_SANITIZE + RB_DEBUG_SERIALIZE + RB_DEBUG_SUBSET + 0)
#endif
#if RB_DEBUG_DISPATCH
#define TRACE_DISPATCH(this, format)                                                                                   \
    rb_auto_trace_t<context_t::max_debug_depth, typename context_t::return_t> trace(                                   \
        &c->debug_depth, c->get_name(), this, RB_FUNC, "format %d", (int)format)
#else
#define TRACE_DISPATCH(this, format) rb_no_trace_t<typename context_t::return_t> trace
#endif

#endif /* RB_DEBUG_HH */
