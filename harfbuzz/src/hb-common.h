/*
 * Copyright © 2007,2008,2009  Red Hat, Inc.
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
 * Red Hat Author(s): Behdad Esfahbod
 * Google Author(s): Behdad Esfahbod
 */

#ifndef RB_H_IN
#error "Include <hb.h> instead."
#endif

#ifndef RB_COMMON_H
#define RB_COMMON_H

#ifndef RB_EXTERN
#define RB_EXTERN extern
#endif

#ifndef RB_BEGIN_DECLS
#ifdef __cplusplus
#define RB_BEGIN_DECLS extern "C" {
#define RB_END_DECLS }
#else /* !__cplusplus */
#define RB_BEGIN_DECLS
#define RB_END_DECLS
#endif /* !__cplusplus */
#endif

#if defined(_SVR4) || defined(SVR4) || defined(__OpenBSD__) || defined(_sgi) || defined(__sun) || defined(sun) ||      \
    defined(__digital__) || defined(__HP_cc)
#include <inttypes.h>
#elif defined(_AIX)
#include <sys/inttypes.h>
#elif defined(_MSC_VER) && _MSC_VER < 1600
/* VS 2010 (_MSC_VER 1600) has stdint.h */
typedef __int8 int8_t;
typedef unsigned __int8 uint8_t;
typedef __int16 int16_t;
typedef unsigned __int16 uint16_t;
typedef __int32 int32_t;
typedef unsigned __int32 uint32_t;
typedef __int64 int64_t;
typedef unsigned __int64 uint64_t;
#elif defined(__KERNEL__)
#include <linux/types.h>
#else
#include <stdint.h>
#endif

#if defined(__GNUC__) && ((__GNUC__ > 3) || (__GNUC__ == 3 && __GNUC_MINOR__ >= 1))
#define RB_DEPRECATED __attribute__((__deprecated__))
#elif defined(_MSC_VER) && (_MSC_VER >= 1300)
#define RB_DEPRECATED __declspec(deprecated)
#else
#define RB_DEPRECATED
#endif

#if defined(__GNUC__) && ((__GNUC__ > 4) || (__GNUC__ == 4 && __GNUC_MINOR__ >= 5))
#define RB_DEPRECATED_FOR(f) __attribute__((__deprecated__("Use '" #f "' instead")))
#elif defined(_MSC_FULL_VER) && (_MSC_FULL_VER > 140050320)
#define RB_DEPRECATED_FOR(f) __declspec(deprecated("is deprecated. Use '" #f "' instead"))
#else
#define RB_DEPRECATED_FOR(f) RB_DEPRECATED
#endif

RB_BEGIN_DECLS

typedef int rb_bool_t;

typedef uint32_t rb_codepoint_t;
typedef int32_t rb_position_t;
typedef uint32_t rb_mask_t;

typedef union _rb_var_int_t {
    uint32_t u32;
    int32_t i32;
    uint16_t u16[2];
    int16_t i16[2];
    uint8_t u8[4];
    int8_t i8[4];
} rb_var_int_t;

/* rb_tag_t */

typedef uint32_t rb_tag_t;

#define RB_TAG(c1, c2, c3, c4)                                                                                         \
    ((rb_tag_t)((((uint32_t)(c1)&0xFF) << 24) | (((uint32_t)(c2)&0xFF) << 16) | (((uint32_t)(c3)&0xFF) << 8) |         \
                ((uint32_t)(c4)&0xFF)))
#define RB_UNTAG(tag)                                                                                                  \
    (uint8_t)(((tag) >> 24) & 0xFF), (uint8_t)(((tag) >> 16) & 0xFF), (uint8_t)(((tag) >> 8) & 0xFF),                  \
        (uint8_t)((tag)&0xFF)

#define RB_TAG_NONE RB_TAG(0, 0, 0, 0)
#define RB_TAG_MAX RB_TAG(0xff, 0xff, 0xff, 0xff)
#define RB_TAG_MAX_SIGNED RB_TAG(0x7f, 0xff, 0xff, 0xff)

/**
 * rb_direction_t:
 * @RB_DIRECTION_INVALID: Initial, unset direction.
 * @RB_DIRECTION_LTR: Text is set horizontally from left to right.
 * @RB_DIRECTION_RTL: Text is set horizontally from right to left.
 * @RB_DIRECTION_TTB: Text is set vertically from top to bottom.
 * @RB_DIRECTION_BTT: Text is set vertically from bottom to top.
 */
typedef enum {
    RB_DIRECTION_INVALID = 0,
    RB_DIRECTION_LTR = 4,
    RB_DIRECTION_RTL,
    RB_DIRECTION_TTB,
    RB_DIRECTION_BTT
} rb_direction_t;

#define RB_DIRECTION_IS_VALID(dir) ((((unsigned int)(dir)) & ~3U) == 4)
/* Direction must be valid for the following */
#define RB_DIRECTION_IS_HORIZONTAL(dir) ((((unsigned int)(dir)) & ~1U) == 4)
#define RB_DIRECTION_IS_VERTICAL(dir) ((((unsigned int)(dir)) & ~1U) == 6)
#define RB_DIRECTION_IS_FORWARD(dir) ((((unsigned int)(dir)) & ~2U) == 4)
#define RB_DIRECTION_IS_BACKWARD(dir) ((((unsigned int)(dir)) & ~2U) == 5)
#define RB_DIRECTION_REVERSE(dir) ((rb_direction_t)(((unsigned int)(dir)) ^ 1))

/* Script functions */

typedef void (*rb_destroy_func_t)(void *user_data);

/* Font features and variations. */

RB_END_DECLS

#endif /* RB_COMMON_H */
