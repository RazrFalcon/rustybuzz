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

#include "hb-map.hh"

/**
 * SECTION:hb-map
 * @title: hb-map
 * @short_description: Object representing integer to integer mapping
 * @include: hb.h
 *
 * Map objects are integer-to-integer hash-maps.  Currently they are
 * not used in the HarfBuzz public API, but are provided for client's
 * use if desired.
 **/

/**
 * rb_map_create: (Xconstructor)
 *
 * Return value: (transfer full):
 *
 * Since: 1.7.7
 **/
rb_map_t *rb_map_create()
{
    rb_map_t *map;

    if (!(map = rb_object_create<rb_map_t>()))
        return rb_map_get_empty();

    map->init_shallow();

    return map;
}

/**
 * rb_map_get_empty:
 *
 * Return value: (transfer full):
 *
 * Since: 1.7.7
 **/
rb_map_t *rb_map_get_empty()
{
    return const_cast<rb_map_t *>(&Null(rb_map_t));
}

/**
 * rb_map_reference: (skip)
 * @map: a map.
 *
 * Return value: (transfer full):
 *
 * Since: 1.7.7
 **/
rb_map_t *rb_map_reference(rb_map_t *map)
{
    return rb_object_reference(map);
}

/**
 * rb_map_destroy: (skip)
 * @map: a map.
 *
 * Since: 1.7.7
 **/
void rb_map_destroy(rb_map_t *map)
{
    if (!rb_object_destroy(map))
        return;

    map->fini_shallow();

    free(map);
}

/**
 * rb_map_allocation_successful:
 * @map: a map.
 *
 *
 *
 * Return value:
 *
 * Since: 1.7.7
 **/
rb_bool_t rb_map_allocation_successful(const rb_map_t *map)
{
    return map->successful;
}

/**
 * rb_map_set:
 * @map: a map.
 * @key:
 * @value:
 *
 *
 *
 * Since: 1.7.7
 **/
void rb_map_set(rb_map_t *map, rb_codepoint_t key, rb_codepoint_t value)
{
    map->set(key, value);
}

/**
 * rb_map_get:
 * @map: a map.
 * @key:
 *
 *
 *
 * Since: 1.7.7
 **/
rb_codepoint_t rb_map_get(const rb_map_t *map, rb_codepoint_t key)
{
    return map->get(key);
}

/**
 * rb_map_del:
 * @map: a map.
 * @key:
 *
 *
 *
 * Since: 1.7.7
 **/
void rb_map_del(rb_map_t *map, rb_codepoint_t key)
{
    map->del(key);
}

/**
 * rb_map_has:
 * @map: a map.
 * @key:
 *
 *
 *
 * Since: 1.7.7
 **/
rb_bool_t rb_map_has(const rb_map_t *map, rb_codepoint_t key)
{
    return map->has(key);
}

/**
 * rb_map_clear:
 * @map: a map.
 *
 *
 *
 * Since: 1.7.7
 **/
void rb_map_clear(rb_map_t *map)
{
    return map->clear();
}

/**
 * rb_map_is_empty:
 * @map: a map.
 *
 *
 *
 * Since: 1.7.7
 **/
rb_bool_t rb_map_is_empty(const rb_map_t *map)
{
    return map->is_empty();
}

/**
 * rb_map_get_population:
 * @map: a map.
 *
 *
 *
 * Since: 1.7.7
 **/
unsigned int rb_map_get_population(const rb_map_t *map)
{
    return map->get_population();
}
