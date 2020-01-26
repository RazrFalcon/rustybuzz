/*
 * Copyright © 1998-2004  David Turner and Werner Lemberg
 * Copyright © 2004,2007,2009,2010  Red Hat, Inc.
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
 * Red Hat Author(s): Owen Taylor, Behdad Esfahbod
 * Google Author(s): Behdad Esfahbod
 */

#include "hb-buffer.hh"
#include "hb-utf.hh"


/**
 * SECTION: hb-buffer
 * @title: hb-buffer
 * @short_description: Input and output buffers
 * @include: hb.h
 *
 * Buffers serve dual role in HarfBuzz; they hold the input characters that are
 * passed to hb_shape(), and after shaping they hold the output glyphs.
 **/

/**
 * hb_segment_properties_equal:
 * @a: first #hb_segment_properties_t to compare.
 * @b: second #hb_segment_properties_t to compare.
 *
 * Checks the equality of two #hb_segment_properties_t's.
 *
 * Return value:
 * %true if all properties of @a equal those of @b, false otherwise.
 *
 * Since: 0.9.7
 **/
hb_bool_t
hb_segment_properties_equal (const hb_segment_properties_t *a,
                             const hb_segment_properties_t *b)
{
  return a->direction == b->direction &&
         a->script    == b->script    &&
         a->language  == b->language;

}

/**
 * hb_segment_properties_hash:
 * @p: #hb_segment_properties_t to hash.
 *
 * Creates a hash representing @p.
 *
 * Return value:
 * A hash of @p.
 *
 * Since: 0.9.7
 **/
unsigned int
hb_segment_properties_hash (const hb_segment_properties_t *p)
{
  return (unsigned int) p->direction ^
         (unsigned int) p->script ^
         (intptr_t) (p->language);
}



/* Here is how the buffer works internally:
 *
 * There are two info pointers: info and out_info.  They always have
 * the same allocated size, but different lengths.
 *
 * As an optimization, both info and out_info may point to the
 * same piece of memory, which is owned by info.  This remains the
 * case as long as out_len doesn't exceed i at any time.
 * In that case, swap_buffers() is no-op and the glyph operations operate
 * mostly in-place.
 *
 * As soon as out_info gets longer than info, out_info is moved over
 * to an alternate buffer (which we reuse the pos buffer for!), and its
 * current contents (out_len entries) are copied to the new place.
 * This should all remain transparent to the user.  swap_buffers() then
 * switches info and out_info.
 */



/* Internal API */

static void
_unsafe_to_break_set_mask (hb_buffer_t *buffer,
                           hb_glyph_info_t *infos,
                           unsigned int start, unsigned int end,
                           unsigned int cluster)
{
  for (unsigned int i = start; i < end; i++)
    if (cluster != infos[i].cluster)
    {
      buffer->scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_UNSAFE_TO_BREAK;
      infos[i].mask |= HB_GLYPH_FLAG_UNSAFE_TO_BREAK;
    }
}

static int
_unsafe_to_break_find_min_cluster (const hb_glyph_info_t *infos,
                                   unsigned int start, unsigned int end,
                                   unsigned int cluster)
{
  for (unsigned int i = start; i < end; i++)
    cluster = hb_min (cluster, infos[i].cluster);
  return cluster;
}

static bool make_room_for (hb_buffer_t *buffer,
                           unsigned int num_in,
                           unsigned int num_out)
{
  hb_buffer_pre_allocate (buffer, hb_buffer_get_out_len(buffer) + num_out);

  if (!buffer->have_separate_output &&
      hb_buffer_get_out_len(buffer) + num_out > hb_buffer_get_idx(buffer) + num_in)
  {
    assert (buffer->have_output);

    buffer->have_separate_output = true;
    for (unsigned i = 0; i < hb_buffer_get_out_len(buffer); ++i) {
      hb_buffer_get_out_info(buffer)[i] = hb_buffer_get_info(buffer)[i];
    }
  }

  return true;
}

static bool shift_forward (hb_buffer_t *buffer, unsigned int count)
{
  assert (buffer->have_output);
  hb_buffer_pre_allocate (buffer, hb_buffer_get_length(buffer) + count);

  for (unsigned i = 0; i < (hb_buffer_get_length(buffer) - hb_buffer_get_idx(buffer)); ++i) {
    hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer) + count + i] = hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer) + i];
  }

  if (hb_buffer_get_idx(buffer) + count > hb_buffer_get_length(buffer))
  {
    /* Under memory failure we might expose this area.  At least
     * clean it up.  Oh well...
     *
     * Ideally, we should at least set Default_Ignorable bits on
     * these, as well as consistent cluster values.  But the former
     * is layering violation... */
    memset (hb_buffer_get_info(buffer) + hb_buffer_get_length(buffer), 0, (hb_buffer_get_idx(buffer) + count - hb_buffer_get_length(buffer)) * sizeof (hb_buffer_get_info(buffer)[0]));
  }
  buffer->info_vec.length += count;
  hb_buffer_set_idx(buffer, hb_buffer_get_idx(buffer) + count);

  return true;
}

/* HarfBuzz-Internal API */

static void add (hb_buffer_t *buffer, hb_codepoint_t codepoint, unsigned int cluster)
{
  hb_glyph_info_t *glyph;

  hb_buffer_pre_allocate (buffer, hb_buffer_get_length(buffer) + 1);

  glyph = &hb_buffer_get_info(buffer)[hb_buffer_get_length(buffer)];

  memset (glyph, 0, sizeof (*glyph));
  glyph->codepoint = codepoint;
  glyph->mask = 0;
  glyph->cluster = cluster;

  buffer->info_vec.length++;
}

/* Public API */

DEFINE_NULL_INSTANCE (hb_buffer_t) =
{
  HB_OBJECT_HEADER_STATIC,

  HB_BUFFER_FLAG_DEFAULT,
  HB_BUFFER_CLUSTER_LEVEL_DEFAULT,
  HB_BUFFER_REPLACEMENT_CODEPOINT_DEFAULT,
  0, /* invisible */
  HB_BUFFER_SCRATCH_FLAG_DEFAULT,
  HB_BUFFER_MAX_OPS_DEFAULT,

  HB_BUFFER_CONTENT_TYPE_INVALID,
  HB_SEGMENT_PROPERTIES_DEFAULT,
  false, /* successful */
  true, /* have_output */
  true  /* have_positions */

  /* Zero is good enough for everything else. */
};


/**
 * hb_buffer_create: (Xconstructor)
 *
 * Creates a new #hb_buffer_t with all properties to defaults.
 *
 * Return value: (transfer full):
 * A newly allocated #hb_buffer_t with a reference count of 1. The initial
 * reference count should be released with hb_buffer_destroy() when you are done
 * using the #hb_buffer_t. This function never returns %NULL. If memory cannot
 * be allocated, a special #hb_buffer_t object will be returned on which
 * hb_buffer_allocation_successful() returns %false.
 *
 * Since: 0.9.2
 **/
hb_buffer_t *
hb_buffer_create ()
{
  hb_buffer_t *buffer;

  if (!(buffer = hb_object_create<hb_buffer_t> ()))
    return const_cast<hb_buffer_t *> (&Null(hb_buffer_t));

  buffer->max_ops = HB_BUFFER_MAX_OPS_DEFAULT;

  hb_buffer_reset (buffer);

  return buffer;
}

/**
 * hb_buffer_destroy: (skip)
 * @buffer: an #hb_buffer_t.
 *
 * Deallocate the @buffer.
 * Decreases the reference count on @buffer by one. If the result is zero, then
 * @buffer and all associated resources are freed. See hb_buffer_reference().
 *
 * Since: 0.9.2
 **/
void
hb_buffer_destroy (hb_buffer_t *buffer)
{
  if (!hb_object_destroy (buffer)) return;

  free (hb_buffer_get_info(buffer));
  free (hb_buffer_get_pos(buffer));
  free (buffer);
}

/**
 * hb_buffer_set_content_type:
 * @buffer: an #hb_buffer_t.
 * @content_type: the type of buffer contents to set
 *
 * Sets the type of @buffer contents, buffers are either empty, contain
 * characters (before shaping) or glyphs (the result of shaping).
 *
 * Since: 0.9.5
 **/
void
hb_buffer_set_content_type (hb_buffer_t              *buffer,
                            hb_buffer_content_type_t  content_type)
{
  buffer->content_type = content_type;
}

/**
 * hb_buffer_get_content_type:
 * @buffer: an #hb_buffer_t.
 *
 * see hb_buffer_set_content_type().
 *
 * Return value:
 * The type of @buffer contents.
 *
 * Since: 0.9.5
 **/
hb_buffer_content_type_t
hb_buffer_get_content_type (hb_buffer_t *buffer)
{
  return buffer->content_type;
}

/**
 * hb_buffer_set_direction:
 * @buffer: an #hb_buffer_t.
 * @direction: the #hb_direction_t of the @buffer
 *
 * Set the text flow direction of the buffer. No shaping can happen without
 * setting @buffer direction, and it controls the visual direction for the
 * output glyphs; for RTL direction the glyphs will be reversed. Many layout
 * features depend on the proper setting of the direction, for example,
 * reversing RTL text before shaping, then shaping with LTR direction is not
 * the same as keeping the text in logical order and shaping with RTL
 * direction.
 *
 * Since: 0.9.2
 **/
void
hb_buffer_set_direction (hb_buffer_t    *buffer,
                         hb_direction_t  direction)

{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->props.direction = direction;
}

/**
 * hb_buffer_get_direction:
 * @buffer: an #hb_buffer_t.
 *
 * See hb_buffer_set_direction()
 *
 * Return value:
 * The direction of the @buffer.
 *
 * Since: 0.9.2
 **/
hb_direction_t
hb_buffer_get_direction (hb_buffer_t    *buffer)
{
  return buffer->props.direction;
}

/**
 * hb_buffer_set_script:
 * @buffer: an #hb_buffer_t.
 * @script: an #hb_script_t to set.
 *
 * Sets the script of @buffer to @script.
 *
 * Script is crucial for choosing the proper shaping behaviour for scripts that
 * require it (e.g. Arabic) and the which OpenType features defined in the font
 * to be applied.
 *
 * You can pass one of the predefined #hb_script_t values, or use
 * hb_script_from_string() or hb_script_from_iso15924_tag() to get the
 * corresponding script from an ISO 15924 script tag.
 *
 * Since: 0.9.2
 **/
void
hb_buffer_set_script (hb_buffer_t *buffer,
                      hb_script_t  script)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->props.script = script;
}

/**
 * hb_buffer_get_script:
 * @buffer: an #hb_buffer_t.
 *
 * See hb_buffer_set_script().
 *
 * Return value:
 * The #hb_script_t of the @buffer.
 *
 * Since: 0.9.2
 **/
hb_script_t
hb_buffer_get_script (hb_buffer_t *buffer)
{
  return buffer->props.script;
}

/**
 * hb_buffer_set_language:
 * @buffer: an #hb_buffer_t.
 * @language: an hb_language_t to set.
 *
 * Sets the language of @buffer to @language.
 *
 * Languages are crucial for selecting which OpenType feature to apply to the
 * buffer which can result in applying language-specific behaviour. Languages
 * are orthogonal to the scripts, and though they are related, they are
 * different concepts and should not be confused with each other.
 *
 * Use hb_language_from_string() to convert from BCP 47 language tags to
 * #hb_language_t.
 *
 * Since: 0.9.2
 **/
void
hb_buffer_set_language (hb_buffer_t   *buffer,
                        hb_language_t  language)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->props.language = language;
}

/**
 * hb_buffer_get_language:
 * @buffer: an #hb_buffer_t.
 *
 * See hb_buffer_set_language().
 *
 * Return value: (transfer none):
 * The #hb_language_t of the buffer. Must not be freed by the caller.
 *
 * Since: 0.9.2
 **/
hb_language_t
hb_buffer_get_language (hb_buffer_t *buffer)
{
  return buffer->props.language;
}

/**
 * hb_buffer_set_segment_properties:
 * @buffer: an #hb_buffer_t.
 * @props: an #hb_segment_properties_t to use.
 *
 * Sets the segment properties of the buffer, a shortcut for calling
 * hb_buffer_set_direction(), hb_buffer_set_script() and
 * hb_buffer_set_language() individually.
 *
 * Since: 0.9.7
 **/
void
hb_buffer_set_segment_properties (hb_buffer_t *buffer,
                                  const hb_segment_properties_t *props)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->props = *props;
}

/**
 * hb_buffer_set_flags:
 * @buffer: an #hb_buffer_t.
 * @flags: the buffer flags to set.
 *
 * Sets @buffer flags to @flags. See #hb_buffer_flags_t.
 *
 * Since: 0.9.7
 **/
void
hb_buffer_set_flags (hb_buffer_t       *buffer,
                     hb_buffer_flags_t  flags)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->flags = flags;
}

/**
 * hb_buffer_get_flags:
 * @buffer: an #hb_buffer_t.
 *
 * See hb_buffer_set_flags().
 *
 * Return value:
 * The @buffer flags.
 *
 * Since: 0.9.7
 **/
hb_buffer_flags_t
hb_buffer_get_flags (hb_buffer_t *buffer)
{
  return buffer->flags;
}

/**
 * hb_buffer_set_cluster_level:
 * @buffer: an #hb_buffer_t.
 * @cluster_level:
 *
 *
 *
 * Since: 0.9.42
 **/
void
hb_buffer_set_cluster_level (hb_buffer_t       *buffer,
                     hb_buffer_cluster_level_t  cluster_level)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->cluster_level = cluster_level;
}

/**
 * hb_buffer_get_cluster_level:
 * @buffer: an #hb_buffer_t.
 *
 *
 *
 * Return value:
 *
 * Since: 0.9.42
 **/
hb_buffer_cluster_level_t
hb_buffer_get_cluster_level (hb_buffer_t *buffer)
{
  return buffer->cluster_level;
}


/**
 * hb_buffer_set_replacement_codepoint:
 * @buffer: an #hb_buffer_t.
 * @replacement: the replacement #hb_codepoint_t
 *
 * Sets the #hb_codepoint_t that replaces invalid entries for a given encoding
 * when adding text to @buffer.
 *
 * Default is %HB_BUFFER_REPLACEMENT_CODEPOINT_DEFAULT.
 *
 * Since: 0.9.31
 **/
void
hb_buffer_set_replacement_codepoint (hb_buffer_t    *buffer,
                                     hb_codepoint_t  replacement)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->replacement = replacement;
}

/**
 * hb_buffer_get_replacement_codepoint:
 * @buffer: an #hb_buffer_t.
 *
 * See hb_buffer_set_replacement_codepoint().
 *
 * Return value:
 * The @buffer replacement #hb_codepoint_t.
 *
 * Since: 0.9.31
 **/
hb_codepoint_t
hb_buffer_get_replacement_codepoint (hb_buffer_t    *buffer)
{
  return buffer->replacement;
}


/**
 * hb_buffer_set_invisible_glyph:
 * @buffer: an #hb_buffer_t.
 * @invisible: the invisible #hb_codepoint_t
 *
 * Sets the #hb_codepoint_t that replaces invisible characters in
 * the shaping result.  If set to zero (default), the glyph for the
 * U+0020 SPACE character is used.  Otherwise, this value is used
 * verbatim.
 *
 * Since: 2.0.0
 **/
void
hb_buffer_set_invisible_glyph (hb_buffer_t    *buffer,
                               hb_codepoint_t  invisible)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->invisible = invisible;
}

/**
 * hb_buffer_get_invisible_glyph:
 * @buffer: an #hb_buffer_t.
 *
 * See hb_buffer_set_invisible_glyph().
 *
 * Return value:
 * The @buffer invisible #hb_codepoint_t.
 *
 * Since: 2.0.0
 **/
hb_codepoint_t
hb_buffer_get_invisible_glyph (hb_buffer_t    *buffer)
{
  return buffer->invisible;
}


/**
 * hb_buffer_reset:
 * @buffer: an #hb_buffer_t.
 *
 * Resets the buffer to its initial status, as if it was just newly created
 * with hb_buffer_create().
 *
 * Since: 0.9.2
 **/
void
hb_buffer_reset (hb_buffer_t *buffer)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->flags = HB_BUFFER_FLAG_DEFAULT;
  buffer->replacement = HB_BUFFER_REPLACEMENT_CODEPOINT_DEFAULT;
  buffer->invisible = 0;

  hb_buffer_clear_contents (buffer);
}

/**
 * hb_buffer_clear_contents:
 * @buffer: an #hb_buffer_t.
 *
 * Similar to hb_buffer_reset(), but does not clear the Unicode functions and
 * the replacement code point.
 *
 * Since: 0.9.11
 **/
void
hb_buffer_clear_contents (hb_buffer_t *buffer)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  hb_segment_properties_t default_props = HB_SEGMENT_PROPERTIES_DEFAULT;
  buffer->props = default_props;
  buffer->scratch_flags = HB_BUFFER_SCRATCH_FLAG_DEFAULT;

  buffer->content_type = HB_BUFFER_CONTENT_TYPE_INVALID;
  buffer->have_output = false;
  buffer->have_positions = false;

  hb_buffer_set_idx(buffer, 0);
  buffer->info_vec.length = 0;
  hb_buffer_set_out_len(buffer, 0);
  buffer->have_separate_output = false;

  buffer->serial = 0;

  memset (buffer->context, 0, sizeof buffer->context);
  memset (buffer->context_len, 0, sizeof buffer->context_len);
}

/**
 * hb_buffer_pre_allocate:
 * @buffer: an #hb_buffer_t.
 * @size: number of items to pre allocate.
 *
 * Pre allocates memory for @buffer to fit at least @size number of items.
 *
 * Return value:
 * %true if @buffer memory allocation succeeded, %false otherwise.
 *
 * Since: 0.9.2
 **/
void
hb_buffer_pre_allocate (hb_buffer_t *buffer, unsigned int size)
{
  static_assert ((sizeof (hb_buffer_get_info(buffer)[0]) == sizeof (hb_buffer_get_pos(buffer)[0])), "");

  buffer->info_vec.alloc(size);
  buffer->pos_vec.alloc(size);

  buffer->pos = buffer->pos_vec.arrayZ;
  buffer->info = buffer->info_vec.arrayZ;
}

/**
 * hb_buffer_get_length:
 * @buffer: an #hb_buffer_t.
 *
 * Returns the number of items in the buffer.
 *
 * Return value:
 * The @buffer length.
 * The value valid as long as buffer has not been modified.
 *
 * Since: 0.9.2
 **/
unsigned int
hb_buffer_get_length (const hb_buffer_t *buffer)
{
  return buffer->info_vec.length;
}

void
hb_buffer_set_length (hb_buffer_t *buffer, unsigned int len)
{
  buffer->info_vec.length = len;
}

/**
 * hb_buffer_get_glyph_infos:
 * @buffer: an #hb_buffer_t.
 * @length: (out): output array length.
 *
 * Returns @buffer glyph information array.  Returned pointer
 * is valid as long as @buffer contents are not modified.
 *
 * Return value: (transfer none) (array length=length):
 * The @buffer glyph information array.
 * The value valid as long as buffer has not been modified.
 *
 * Since: 0.9.2
 **/
hb_glyph_info_t *
hb_buffer_get_glyph_infos (hb_buffer_t  *buffer,
                           unsigned int *length)
{
  if (length)
    *length = hb_buffer_get_length(buffer);

  return (hb_glyph_info_t *) hb_buffer_get_info(buffer);
}

/**
 * hb_buffer_get_glyph_positions:
 * @buffer: an #hb_buffer_t.
 * @length: (out): output length.
 *
 * Returns @buffer glyph position array.  Returned pointer
 * is valid as long as @buffer contents are not modified.
 *
 * Return value: (transfer none) (array length=length):
 * The @buffer glyph position array.
 * The value valid as long as buffer has not been modified.
 *
 * Since: 0.9.2
 **/
hb_glyph_position_t *
hb_buffer_get_glyph_positions (hb_buffer_t  *buffer,
                               unsigned int *length)
{
  if (!buffer->have_positions)
    hb_buffer_clear_positions(buffer);

  if (length)
    *length = hb_buffer_get_length(buffer);

  return (hb_glyph_position_t *) hb_buffer_get_pos(buffer);
}

/**
 * hb_glyph_info_get_glyph_flags:
 * @info: a #hb_glyph_info_t.
 *
 * Returns glyph flags encoded within a #hb_glyph_info_t.
 *
 * Return value:
 * The #hb_glyph_flags_t encoded within @info.
 *
 * Since: 1.5.0
 **/
hb_glyph_flags_t
(hb_glyph_info_get_glyph_flags) (const hb_glyph_info_t *info)
{
  return hb_glyph_info_get_glyph_flags (info);
}

/**
 * hb_buffer_reverse:
 * @buffer: an #hb_buffer_t.
 *
 * Reverses buffer contents.
 *
 * Since: 0.9.2
 **/
void
hb_buffer_reverse (hb_buffer_t *buffer)
{
  if (unlikely (!hb_buffer_get_length(buffer)))
    return;

  hb_buffer_reverse_range (buffer, 0, hb_buffer_get_length(buffer));
}

/**
 * hb_buffer_reverse_range:
 * @buffer: an #hb_buffer_t.
 * @start: start index.
 * @end: end index.
 *
 * Reverses buffer contents between start to end.
 *
 * Since: 0.9.41
 **/
void
hb_buffer_reverse_range (hb_buffer_t *buffer,
                         unsigned int start, unsigned int end)
{
  unsigned int i, j;

  if (end - start < 2)
    return;

  for (i = start, j = end - 1; i < j; i++, j--) {
    hb_glyph_info_t t;

    t = hb_buffer_get_info(buffer)[i];
    hb_buffer_get_info(buffer)[i] = hb_buffer_get_info(buffer)[j];
    hb_buffer_get_info(buffer)[j] = t;
  }

  if (buffer->have_positions) {
    for (i = start, j = end - 1; i < j; i++, j--) {
      hb_glyph_position_t t;

      t = hb_buffer_get_pos(buffer)[i];
      hb_buffer_get_pos(buffer)[i] = hb_buffer_get_pos(buffer)[j];
      hb_buffer_get_pos(buffer)[j] = t;
    }
  }
}

void
hb_buffer_reset_clusters (hb_buffer_t *buffer)
{
  for (uint i = 0; i < hb_buffer_get_length(buffer); i++) {
    hb_buffer_get_info(buffer)[i].cluster = i;
  }
}

/**
 * hb_buffer_guess_segment_properties:
 * @buffer: an #hb_buffer_t.
 *
 * Sets unset buffer segment properties based on buffer Unicode
 * contents.  If buffer is not empty, it must have content type
 * %HB_BUFFER_CONTENT_TYPE_UNICODE.
 *
 * If buffer script is not set (ie. is %HB_SCRIPT_INVALID), it
 * will be set to the Unicode script of the first character in
 * the buffer that has a script other than %HB_SCRIPT_COMMON,
 * %HB_SCRIPT_INHERITED, and %HB_SCRIPT_UNKNOWN.
 *
 * Next, if buffer direction is not set (ie. is %HB_DIRECTION_INVALID),
 * it will be set to the natural horizontal direction of the
 * buffer script as returned by hb_script_get_horizontal_direction().
 * If hb_script_get_horizontal_direction() returns %HB_DIRECTION_INVALID,
 * then %HB_DIRECTION_LTR is used.
 *
 * Finally, if buffer language is not set (ie. is %HB_LANGUAGE_INVALID),
 * it will be set to the process's default language as returned by
 * hb_language_get_default().  This may change in the future by
 * taking buffer script into consideration when choosing a language.
 * Note that hb_language_get_default() is NOT threadsafe the first time
 * it is called.  See documentation for that function for details.
 *
 * Since: 0.9.7
 **/
void
hb_buffer_guess_segment_properties (hb_buffer_t *buffer)
{
  assert (buffer->content_type == HB_BUFFER_CONTENT_TYPE_UNICODE ||
          (!hb_buffer_get_length(buffer) && buffer->content_type == HB_BUFFER_CONTENT_TYPE_INVALID));

  /* If script is set to INVALID, guess from buffer contents */
  if (buffer->props.script == HB_SCRIPT_INVALID) {
    for (unsigned int i = 0; i < hb_buffer_get_length(buffer); i++) {
      hb_script_t script = hb_ucd_script (hb_buffer_get_info(buffer)[i].codepoint);
      if (likely (script != HB_SCRIPT_COMMON &&
                  script != HB_SCRIPT_INHERITED &&
                  script != HB_SCRIPT_UNKNOWN)) {
        buffer->props.script = script;
        break;
      }
    }
  }

  /* If direction is set to INVALID, guess from script */
  if (buffer->props.direction == HB_DIRECTION_INVALID) {
    buffer->props.direction = hb_script_get_horizontal_direction (buffer->props.script);
    if (buffer->props.direction == HB_DIRECTION_INVALID)
      buffer->props.direction = HB_DIRECTION_LTR;
  }

  /* If language is not set, use default language from locale */
  if (buffer->props.language == HB_LANGUAGE_INVALID) {
    /* TODO get_default_for_script? using $LANGUAGE */
    buffer->props.language = hb_language_get_default ();
  }
}

template <typename utf_t>
static inline void
hb_buffer_add_utf (hb_buffer_t  *buffer,
                   const typename utf_t::codepoint_t *text,
                   int           text_length,
                   unsigned int  item_offset,
                   int           item_length)
{
  typedef typename utf_t::codepoint_t T;
  const hb_codepoint_t replacement = buffer->replacement;

  assert (buffer->content_type == HB_BUFFER_CONTENT_TYPE_UNICODE ||
          (!hb_buffer_get_length(buffer) && buffer->content_type == HB_BUFFER_CONTENT_TYPE_INVALID));

  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  if (text_length == -1)
    text_length = utf_t::strlen (text);

  if (item_length == -1)
    item_length = text_length - item_offset;

  hb_buffer_pre_allocate (buffer, hb_buffer_get_length(buffer) + item_length * sizeof (T) / 4);

  /* If buffer is empty and pre-context provided, install it.
   * This check is written this way, to make sure people can
   * provide pre-context in one add_utf() call, then provide
   * text in a follow-up call.  See:
   *
   * https://bugzilla.mozilla.org/show_bug.cgi?id=801410#c13
   */
  if (!hb_buffer_get_length(buffer) && item_offset > 0)
  {
    /* Add pre-context */
    buffer->context_len[0] = 0;
    const T *prev = text + item_offset;
    const T *start = text;
    while (start < prev && buffer->context_len[0] < buffer->CONTEXT_LENGTH)
    {
      hb_codepoint_t u;
      prev = utf_t::prev (prev, start, &u, replacement);
      buffer->context[0][buffer->context_len[0]++] = u;
    }
  }

  const T *next = text + item_offset;
  const T *end = next + item_length;
  while (next < end)
  {
    hb_codepoint_t u;
    const T *old_next = next;
    next = utf_t::next (next, end, &u, replacement);
    add (buffer, u, old_next - (const T *) text);
  }

  /* Add post-context */
  buffer->context_len[1] = 0;
  end = text + text_length;
  while (next < end && buffer->context_len[1] < buffer->CONTEXT_LENGTH)
  {
    hb_codepoint_t u;
    next = utf_t::next (next, end, &u, replacement);
    buffer->context[1][buffer->context_len[1]++] = u;
  }

  buffer->content_type = HB_BUFFER_CONTENT_TYPE_UNICODE;
}

/**
 * hb_buffer_add_utf8:
 * @buffer: an #hb_buffer_t.
 * @text: (array length=text_length) (element-type uint8_t): an array of UTF-8
 *               characters to append.
 * @text_length: the length of the @text, or -1 if it is %NULL terminated.
 * @item_offset: the offset of the first character to add to the @buffer.
 * @item_length: the number of characters to add to the @buffer, or -1 for the
 *               end of @text (assuming it is %NULL terminated).
 *
 * See hb_buffer_add_codepoints().
 *
 * Replaces invalid UTF-8 characters with the @buffer replacement code point,
 * see hb_buffer_set_replacement_codepoint().
 *
 * Since: 0.9.2
 **/
void
hb_buffer_add_utf8 (hb_buffer_t  *buffer,
                    const char   *text,
                    int           text_length,
                    unsigned int  item_offset,
                    int           item_length)
{
  hb_buffer_add_utf<hb_utf8_t> (buffer, (const uint8_t *) text, text_length, item_offset, item_length);
}

static int
compare_info_codepoint (const hb_glyph_info_t *pa,
                        const hb_glyph_info_t *pb)
{
  return (int) pb->codepoint - (int) pa->codepoint;
}

static inline void
normalize_glyphs_cluster (hb_buffer_t *buffer,
                          unsigned int start,
                          unsigned int end,
                          bool backward)
{
  hb_glyph_position_t *pos = hb_buffer_get_pos(buffer);

  /* Total cluster advance */
  hb_position_t total_x_advance = 0, total_y_advance = 0;
  for (unsigned int i = start; i < end; i++)
  {
    total_x_advance += pos[i].x_advance;
    total_y_advance += pos[i].y_advance;
  }

  hb_position_t x_advance = 0, y_advance = 0;
  for (unsigned int i = start; i < end; i++)
  {
    pos[i].x_offset += x_advance;
    pos[i].y_offset += y_advance;

    x_advance += pos[i].x_advance;
    y_advance += pos[i].y_advance;

    pos[i].x_advance = 0;
    pos[i].y_advance = 0;
  }

  if (backward)
  {
    /* Transfer all cluster advance to the last glyph. */
    pos[end - 1].x_advance = total_x_advance;
    pos[end - 1].y_advance = total_y_advance;

    hb_stable_sort (hb_buffer_get_info(buffer) + start, end - start - 1, compare_info_codepoint, hb_buffer_get_pos(buffer) + start);
  } else {
    /* Transfer all cluster advance to the first glyph. */
    pos[start].x_advance += total_x_advance;
    pos[start].y_advance += total_y_advance;
    for (unsigned int i = start + 1; i < end; i++) {
      pos[i].x_offset -= total_x_advance;
      pos[i].y_offset -= total_y_advance;
    }
    hb_stable_sort (hb_buffer_get_info(buffer) + start + 1, end - start - 1, compare_info_codepoint, hb_buffer_get_pos(buffer) + start + 1);
  }
}

/**
 * hb_buffer_normalize_glyphs:
 * @buffer: an #hb_buffer_t.
 *
 * Reorders a glyph buffer to have canonical in-cluster glyph order / position.
 * The resulting clusters should behave identical to pre-reordering clusters.
 *
 * <note>This has nothing to do with Unicode normalization.</note>
 *
 * Since: 0.9.2
 **/
void
hb_buffer_normalize_glyphs (hb_buffer_t *buffer)
{
  assert (buffer->have_positions);
  assert (buffer->content_type == HB_BUFFER_CONTENT_TYPE_GLYPHS ||
          (!hb_buffer_get_length(buffer) && buffer->content_type == HB_BUFFER_CONTENT_TYPE_INVALID));

  bool backward = HB_DIRECTION_IS_BACKWARD (buffer->props.direction);

  unsigned int count = hb_buffer_get_length(buffer);
  if (unlikely (!count)) return;
  hb_glyph_info_t *info = hb_buffer_get_info(buffer);

  unsigned int start = 0;
  unsigned int end;
  for (end = start + 1; end < count; end++)
    if (info[start].cluster != info[end].cluster) {
      normalize_glyphs_cluster (buffer, start, end, backward);
      start = end;
    }
  normalize_glyphs_cluster (buffer, start, end, backward);
}

hb_glyph_info_t*
hb_buffer_get_cur(hb_buffer_t *buffer, unsigned int i)
{
  return &hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer) + i];
}

hb_glyph_position_t*
hb_buffer_get_cur_pos(hb_buffer_t *buffer)
{
  return &hb_buffer_get_pos(buffer)[hb_buffer_get_idx(buffer)];
}

hb_glyph_info_t*
hb_buffer_get_prev(hb_buffer_t *buffer)
{
  return &hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer) ? hb_buffer_get_out_len(buffer) - 1 : 0];
}

hb_glyph_info_t*
hb_buffer_get_out_info (hb_buffer_t *buffer)
{
  if (buffer->have_separate_output) {
    return (hb_glyph_info_t *) hb_buffer_get_pos(buffer);
  } else {
    return hb_buffer_get_info(buffer);
  }
}

unsigned int
hb_buffer_backtrack_len (hb_buffer_t *buffer)
{
  return buffer->have_output ? hb_buffer_get_out_len(buffer) : hb_buffer_get_idx(buffer);
}

unsigned int
hb_buffer_lookahead_len (hb_buffer_t *buffer)
{
  return hb_buffer_get_length(buffer) - hb_buffer_get_idx(buffer);
}

unsigned int
hb_buffer_next_serial (hb_buffer_t *buffer)
{
  return buffer->serial++;
}

void
hb_buffer_set_cluster (hb_glyph_info_t *info, unsigned int cluster, unsigned int mask)
{
  if (info->cluster != cluster)
  {
    if (mask & HB_GLYPH_FLAG_UNSAFE_TO_BREAK)
      info->mask |= HB_GLYPH_FLAG_UNSAFE_TO_BREAK;
    else
      info->mask &= ~HB_GLYPH_FLAG_UNSAFE_TO_BREAK;
  }
  info->cluster = cluster;
}

bool
hb_buffer_move_to (hb_buffer_t *buffer, unsigned int i)
{
if (!buffer->have_output)
  {
    assert (i <= hb_buffer_get_length(buffer));
    hb_buffer_set_idx(buffer, i);
    return true;
  }

  assert (i <= hb_buffer_get_out_len(buffer) + (hb_buffer_get_length(buffer) - hb_buffer_get_idx(buffer)));

  if (hb_buffer_get_out_len(buffer) < i)
  {
    unsigned int count = i - hb_buffer_get_out_len(buffer);
    if (unlikely (!make_room_for (buffer, count, count))) return false;

    for (unsigned j = 0; j < count; ++j) {
      hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer) + j] = hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer) + j];
    }

    hb_buffer_set_idx(buffer, hb_buffer_get_idx(buffer) + count);
    hb_buffer_set_out_len(buffer, hb_buffer_get_out_len(buffer) + count);
  }
  else if (hb_buffer_get_out_len(buffer) > i)
  {
    /* Tricky part: rewinding... */
    unsigned int count = hb_buffer_get_out_len(buffer) - i;

    /* This will blow in our face if memory allocation fails later
     * in this same lookup...
     *
     * We used to shift with extra 32 items, instead of the 0 below.
     * But that would leave empty slots in the buffer in case of allocation
     * failures.  Setting to zero for now to avoid other problems (see
     * comments in shift_forward().  This can cause O(N^2) behavior more
     * severely than adding 32 empty slots can... */
    if (unlikely (hb_buffer_get_idx(buffer) < count && !shift_forward (buffer, count + 0))) return false;

    assert (hb_buffer_get_idx(buffer) >= count);

    hb_buffer_set_idx(buffer, hb_buffer_get_idx(buffer) - count);
    hb_buffer_set_out_len(buffer, hb_buffer_get_out_len(buffer) - count);
    for (unsigned j = 0; j < count; ++j) {
      hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer) + j] = hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer) + j];
    }
  }

  return true;
}

void
hb_buffer_swap_buffers (hb_buffer_t *buffer)
{
  assert (buffer->have_output);
  buffer->have_output = false;
  
  if (buffer->have_separate_output)
  {
    hb_glyph_info_t *tmp_string = buffer->info_vec.arrayZ;
    buffer->info_vec.arrayZ = (hb_glyph_info_t *) buffer->pos_vec.arrayZ;
    buffer->pos_vec.arrayZ = (hb_glyph_position_t *) tmp_string;
    
    buffer->info = buffer->info_vec.arrayZ;
    buffer->pos = buffer->pos_vec.arrayZ;
  }
  
  unsigned int tmp;
  tmp = hb_buffer_get_length(buffer);
  buffer->info_vec.length = hb_buffer_get_out_len(buffer);
  hb_buffer_set_out_len(buffer, tmp);
  
  hb_buffer_set_idx(buffer, 0);
}

void
hb_buffer_remove_output (hb_buffer_t *buffer)
{
if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->have_output = false;
  buffer->have_positions = false;

  hb_buffer_set_out_len(buffer, 0);
  buffer->have_separate_output = false;
}

void
hb_buffer_clear_output (hb_buffer_t *buffer)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->have_output = true;
  buffer->have_positions = false;

  hb_buffer_set_out_len(buffer, 0);
  buffer->have_separate_output = false;
}

void
hb_buffer_clear_positions (hb_buffer_t *buffer)
{
  if (unlikely (hb_object_is_immutable (buffer)))
    return;

  buffer->have_output = false;
  buffer->have_positions = true;

  hb_buffer_set_out_len(buffer, 0);
  buffer->have_separate_output = false;

  hb_memset (hb_buffer_get_pos(buffer), 0, sizeof (hb_buffer_get_pos(buffer)[0]) * hb_buffer_get_length(buffer));
}

unsigned int
hb_buffer_next_cluster (hb_buffer_t *buffer, unsigned int start)
{
  hb_glyph_info_t *info = hb_buffer_get_info(buffer);
  unsigned int count = hb_buffer_get_length(buffer);

  unsigned int cluster = info[start].cluster;
  while (++start < count && cluster == info[start].cluster)
    ;

  return start;
}

void
hb_buffer_replace_glyphs (hb_buffer_t *buffer,
                          unsigned int num_in,
                          unsigned int num_out,
                          const hb_codepoint_t *glyph_data)
{
  if (unlikely (!make_room_for (buffer, num_in, num_out))) return;

  assert (hb_buffer_get_idx(buffer) + num_in <= hb_buffer_get_length(buffer));

  hb_buffer_merge_clusters (buffer, hb_buffer_get_idx(buffer), hb_buffer_get_idx(buffer) + num_in);

  hb_glyph_info_t orig_info = hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer)];
  hb_glyph_info_t *pinfo = &hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer)];
  for (unsigned int i = 0; i < num_out; i++)
  {
    *pinfo = orig_info;
    pinfo->codepoint = glyph_data[i];
    pinfo++;
  }

  hb_buffer_set_idx(buffer, hb_buffer_get_idx(buffer) + num_in);
  hb_buffer_set_out_len(buffer, hb_buffer_get_out_len(buffer) + num_out);
}

void
hb_buffer_merge_clusters (hb_buffer_t *buffer, unsigned int start, unsigned int end)
{
  if (end - start < 2)
    return;
  hb_buffer_merge_clusters_impl (buffer, start, end);
}

void
hb_buffer_merge_clusters_impl (hb_buffer_t *buffer, unsigned int start, unsigned int end)
{
  if (buffer->cluster_level == HB_BUFFER_CLUSTER_LEVEL_CHARACTERS)
  {
    hb_buffer_unsafe_to_break (buffer, start, end);
    return;
  }

  unsigned int cluster = hb_buffer_get_info(buffer)[start].cluster;

  for (unsigned int i = start + 1; i < end; i++)
    cluster = hb_min (cluster, hb_buffer_get_info(buffer)[i].cluster);

  /* Extend end */
  while (end < hb_buffer_get_length(buffer) && hb_buffer_get_info(buffer)[end - 1].cluster == hb_buffer_get_info(buffer)[end].cluster)
    end++;

  /* Extend start */
  while (hb_buffer_get_idx(buffer) < start && hb_buffer_get_info(buffer)[start - 1].cluster == hb_buffer_get_info(buffer)[start].cluster)
    start--;

  /* If we hit the start of buffer, continue in out-buffer. */
  if (hb_buffer_get_idx(buffer) == start)
    for (unsigned int i = hb_buffer_get_out_len(buffer); i && hb_buffer_get_out_info(buffer)[i - 1].cluster == hb_buffer_get_info(buffer)[start].cluster; i--)
      hb_buffer_set_cluster (&hb_buffer_get_out_info(buffer)[i - 1], cluster, 0);

  for (unsigned int i = start; i < end; i++)
    hb_buffer_set_cluster (&hb_buffer_get_info(buffer)[i], cluster, 0);
}

void
hb_buffer_merge_out_clusters (hb_buffer_t *buffer, unsigned int start, unsigned int end)
{
  if (buffer->cluster_level == HB_BUFFER_CLUSTER_LEVEL_CHARACTERS)
    return;

  if (unlikely (end - start < 2))
    return;

  unsigned int cluster = hb_buffer_get_out_info(buffer)[start].cluster;

  for (unsigned int i = start + 1; i < end; i++)
    cluster = hb_min (cluster, hb_buffer_get_out_info(buffer)[i].cluster);

  /* Extend start */
  while (start && hb_buffer_get_out_info(buffer)[start - 1].cluster == hb_buffer_get_out_info(buffer)[start].cluster)
    start--;

  /* Extend end */
  while (end < hb_buffer_get_out_len(buffer) && hb_buffer_get_out_info(buffer)[end - 1].cluster == hb_buffer_get_out_info(buffer)[end].cluster)
    end++;

  /* If we hit the end of out-buffer, continue in buffer. */
  if (end == hb_buffer_get_out_len(buffer))
    for (unsigned int i = hb_buffer_get_idx(buffer); i < hb_buffer_get_length(buffer) && hb_buffer_get_info(buffer)[i].cluster == hb_buffer_get_out_info(buffer)[end - 1].cluster; i++)
      hb_buffer_set_cluster (&hb_buffer_get_info(buffer)[i], cluster, 0);

  for (unsigned int i = start; i < end; i++)
    hb_buffer_set_cluster (&hb_buffer_get_out_info(buffer)[i], cluster, 0);
}

void
hb_buffer_unsafe_to_break (hb_buffer_t *buffer, unsigned int start, unsigned int end)
{
  if (end - start < 2)
    return;
  hb_buffer_unsafe_to_break_impl (buffer, start, end);
}

void
hb_buffer_unsafe_to_break_impl (hb_buffer_t *buffer, unsigned int start, unsigned int end)
{
  unsigned int cluster = (unsigned int) -1;
  cluster = _unsafe_to_break_find_min_cluster (hb_buffer_get_info(buffer), start, end, cluster);
  _unsafe_to_break_set_mask (buffer, hb_buffer_get_info(buffer), start, end, cluster);
}

void
hb_buffer_unsafe_to_break_from_outbuffer (hb_buffer_t *buffer, unsigned int start, unsigned int end)
{
  if (!buffer->have_output)
  {
    hb_buffer_unsafe_to_break_impl (buffer, start, end);
    return;
  }

  assert (start <= hb_buffer_get_out_len(buffer));
  assert (hb_buffer_get_idx(buffer) <= end);

  unsigned int cluster = (unsigned int) -1;
  cluster = _unsafe_to_break_find_min_cluster (hb_buffer_get_out_info(buffer), start, hb_buffer_get_out_len(buffer), cluster);
  cluster = _unsafe_to_break_find_min_cluster (hb_buffer_get_info(buffer), hb_buffer_get_idx(buffer), end, cluster);
  _unsafe_to_break_set_mask (buffer, hb_buffer_get_out_info(buffer), start, hb_buffer_get_out_len(buffer), cluster);
  _unsafe_to_break_set_mask (buffer, hb_buffer_get_info(buffer), hb_buffer_get_idx(buffer), end, cluster);
}

void
hb_buffer_sort (hb_buffer_t *buffer, unsigned int start, unsigned int end, int(*compar)(const hb_glyph_info_t *, const hb_glyph_info_t *))
{
  assert (!buffer->have_positions);
  for (unsigned int i = start + 1; i < end; i++)
  {
    unsigned int j = i;
    while (j > start && compar (&hb_buffer_get_info(buffer)[j - 1], &hb_buffer_get_info(buffer)[i]) > 0)
      j--;
    if (i == j)
      continue;
    /* Move item i to occupy place for item j, shift what's in between. */
    hb_buffer_merge_clusters (buffer, j, i + 1);
    {
      hb_glyph_info_t t = hb_buffer_get_info(buffer)[i];
      for (int idx = (i - j - 1); idx >= 0; idx--) {
        hb_buffer_get_info(buffer)[idx + j + 1] = hb_buffer_get_info(buffer)[idx + j];
      }

      hb_buffer_get_info(buffer)[j] = t;
    }
  }
}

void
hb_buffer_replace_glyph (hb_buffer_t *buffer, hb_codepoint_t glyph_index)
{
  if (unlikely (buffer->have_separate_output || hb_buffer_get_out_len(buffer) != hb_buffer_get_idx(buffer))) {
    if (unlikely (!make_room_for (buffer, 1, 1))) return;
    hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer)] = hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer)];
  }
  hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer)].codepoint = glyph_index;
  
  hb_buffer_set_idx(buffer, hb_buffer_get_idx(buffer) + 1);
  hb_buffer_set_out_len(buffer, hb_buffer_get_out_len(buffer) + 1);
}

hb_glyph_info_t*
hb_buffer_output_glyph (hb_buffer_t *buffer, hb_codepoint_t glyph_index)
{
  if (unlikely (!make_room_for (buffer, 0, 1))) return &Crap(hb_glyph_info_t);
  
  if (unlikely (hb_buffer_get_idx(buffer) == hb_buffer_get_length(buffer) && !hb_buffer_get_out_len(buffer)))
    return &Crap(hb_glyph_info_t);
  
  hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer)] = hb_buffer_get_idx(buffer) < hb_buffer_get_length(buffer) 
    ? hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer)] : hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer) - 1];
  hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer)].codepoint = glyph_index;
  
  hb_buffer_set_out_len(buffer, hb_buffer_get_out_len(buffer) + 1);
  
  return &hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer) - 1];
}

void
hb_buffer_output_info (hb_buffer_t *buffer, const hb_glyph_info_t &glyph_info)
{
  if (unlikely (!make_room_for (buffer, 0, 1))) return;
  
  hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer)] = glyph_info;
  
  hb_buffer_set_out_len(buffer, hb_buffer_get_out_len(buffer) + 1);
}

void
hb_buffer_copy_glyph (hb_buffer_t *buffer)
{
  if (unlikely (!make_room_for (buffer, 0, 1))) return;
  
  hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer)] = hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer)];
  
  hb_buffer_set_out_len(buffer, hb_buffer_get_out_len(buffer) + 1);
}

void
hb_buffer_next_glyph (hb_buffer_t *buffer)
{
  if (buffer->have_output)
  {
    if (buffer->have_separate_output || hb_buffer_get_out_len(buffer) != hb_buffer_get_idx(buffer))
    {
      if (unlikely (!make_room_for (buffer, 1, 1))) return;
      hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer)] = hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer)];
    }
    hb_buffer_set_out_len(buffer, hb_buffer_get_out_len(buffer) + 1);
  }
  
  hb_buffer_set_idx(buffer, hb_buffer_get_idx(buffer) + 1);
}

void
hb_buffer_next_glyphs (hb_buffer_t *buffer, unsigned int n)
{
  if (buffer->have_output)
  {
    if (buffer->have_separate_output || hb_buffer_get_out_len(buffer) != hb_buffer_get_idx(buffer))
    {
      if (unlikely (!make_room_for (buffer, n, n))) return;
      for (unsigned i = 0; i < n; ++i) {
        hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer) + i] = hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer) + i];
      }
    }
    hb_buffer_set_out_len(buffer, hb_buffer_get_out_len(buffer) + n);
  }
  
  hb_buffer_set_idx(buffer, hb_buffer_get_idx(buffer) + n);
}

void
hb_buffer_skip_glyph (hb_buffer_t *buffer)
{
  hb_buffer_set_idx(buffer, hb_buffer_get_idx(buffer) + 1);
}

void
hb_buffer_reset_masks (hb_buffer_t *buffer, hb_mask_t mask)
{
  for (unsigned int j = 0; j < hb_buffer_get_length(buffer); j++)
    hb_buffer_get_info(buffer)[j].mask = mask;
}

void
hb_buffer_set_masks (hb_buffer_t *buffer, hb_mask_t value, hb_mask_t mask,
                     unsigned int cluster_start, unsigned int cluster_end)
{
  hb_mask_t not_mask = ~mask;
  value &= mask;

  if (!mask)
    return;

  if (cluster_start == 0 && cluster_end == (unsigned int)-1) {
    unsigned int count = hb_buffer_get_length(buffer);
    for (unsigned int i = 0; i < count; i++)
      hb_buffer_get_info(buffer)[i].mask = (hb_buffer_get_info(buffer)[i].mask & not_mask) | value;
    return;
  }

  unsigned int count = hb_buffer_get_length(buffer);
  for (unsigned int i = 0; i < count; i++)
    if (cluster_start <= hb_buffer_get_info(buffer)[i].cluster && hb_buffer_get_info(buffer)[i].cluster < cluster_end)
      hb_buffer_get_info(buffer)[i].mask = (hb_buffer_get_info(buffer)[i].mask & not_mask) | value;
}

void
hb_buffer_delete_glyph (hb_buffer_t *buffer)
{
  /* The logic here is duplicated in hb_ot_hide_default_ignorables(). */

  unsigned int cluster = hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer)].cluster;
  if (hb_buffer_get_idx(buffer) + 1 < hb_buffer_get_length(buffer) && cluster == hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer) + 1].cluster)
  {
    /* Cluster survives; do nothing. */
    goto done;
  }

  if (hb_buffer_get_out_len(buffer))
  {
    /* Merge cluster backward. */
    if (cluster < hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer) - 1].cluster)
    {
      unsigned int mask = hb_buffer_get_info(buffer)[hb_buffer_get_idx(buffer)].mask;
      unsigned int old_cluster = hb_buffer_get_out_info(buffer)[hb_buffer_get_out_len(buffer) - 1].cluster;
      for (unsigned i = hb_buffer_get_out_len(buffer); i && hb_buffer_get_out_info(buffer)[i - 1].cluster == old_cluster; i--)
        hb_buffer_set_cluster (&hb_buffer_get_out_info(buffer)[i - 1], cluster, mask);
    }
    goto done;
  }

  if (hb_buffer_get_idx(buffer) + 1 < hb_buffer_get_length(buffer))
  {
    /* Merge cluster forward. */
    hb_buffer_merge_clusters (buffer, hb_buffer_get_idx(buffer), hb_buffer_get_idx(buffer) + 2);
    goto done;
  }

done:
  hb_buffer_skip_glyph (buffer);
}

hb_glyph_info_t*
hb_buffer_get_info (hb_buffer_t *buffer)
{
  return buffer->info;
}

hb_glyph_position_t*
hb_buffer_get_pos (hb_buffer_t *buffer)
{
  return buffer->pos;
}

hb_buffer_scratch_flags_t*
hb_buffer_get_scratch_flags (hb_buffer_t *buffer)
{
  return &buffer->scratch_flags;
}

int
hb_buffer_get_max_ops (hb_buffer_t *buffer)
{
  return buffer->max_ops;
}

void
hb_buffer_set_max_ops (hb_buffer_t *buffer, int ops)
{
  buffer->max_ops = ops;
}

int
hb_buffer_decrement_max_ops (hb_buffer_t *buffer)
{
  return buffer->max_ops--;
}

bool
hb_buffer_have_positions (hb_buffer_t *buffer)
{
  return buffer->have_positions;
}

unsigned int
hb_buffer_get_idx (hb_buffer_t *buffer)
{
  return buffer->idx;
}

void
hb_buffer_set_idx (hb_buffer_t *buffer, unsigned int idx)
{
  buffer->idx = idx;
}

unsigned int
hb_buffer_get_out_len (hb_buffer_t *buffer)
{
  return buffer->out_len;
}

void
hb_buffer_set_out_len (hb_buffer_t *buffer, unsigned int idx)
{
  buffer->out_len = idx;
}

bool
hb_buffer_have_separate_output (hb_buffer_t *buffer)
{
  return buffer->have_separate_output;
}

hb_codepoint_t
hb_buffer_get_context (hb_buffer_t *buffer, unsigned int idx1, unsigned int idx2)
{
  return buffer->context[idx1][idx2];
}

unsigned int
hb_buffer_get_context_len (hb_buffer_t *buffer, unsigned int idx)
{
  return buffer->context_len[idx];
}