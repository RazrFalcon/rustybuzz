//! OpenType layout.

use core::ops::{Index, IndexMut};

use ttf_parser::opentype_layout::{FeatureIndex, LanguageIndex, LookupIndex, ScriptIndex};
use ttf_parser::GlyphId;

use super::buffer::{hb_buffer_t, GlyphPropsFlags, UnicodeProps};
use super::common::TagExt;
use super::ot_layout_gsubgpos::{Apply, ApplyContext};
use super::shape_plan::hb_ot_shape_plan_t;
use super::unicode::{hb_unicode_funcs_t, hb_unicode_general_category_t, GeneralCategoryExt};
use super::{hb_font_t, hb_glyph_info_t, hb_tag_t};

pub const MAX_NESTING_LEVEL: usize = 6;
pub const MAX_CONTEXT_LENGTH: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableIndex {
    GSUB = 0,
    GPOS = 1,
}

impl TableIndex {
    pub fn iter() -> impl Iterator<Item = TableIndex> {
        [Self::GSUB, Self::GPOS].iter().copied()
    }
}

impl<T> Index<TableIndex> for [T] {
    type Output = T;

    fn index(&self, table_index: TableIndex) -> &Self::Output {
        &self[table_index as usize]
    }
}

impl<T> IndexMut<TableIndex> for [T] {
    fn index_mut(&mut self, table_index: TableIndex) -> &mut Self::Output {
        &mut self[table_index as usize]
    }
}

/// A lookup-based layout table (GSUB or GPOS).
pub trait LayoutTable {
    /// The index of this table.
    const INDEX: TableIndex;

    /// Whether lookups in this table can be applied to the buffer in-place.
    const IN_PLACE: bool;

    /// The kind of lookup stored in this table.
    type Lookup: LayoutLookup;

    /// Get the lookup at the specified index.
    fn get_lookup(&self, index: LookupIndex) -> Option<&Self::Lookup>;
}

/// A lookup in a layout table.
pub trait LayoutLookup: Apply {
    /// The lookup's lookup_props.
    fn props(&self) -> u32;

    /// Whether the lookup has to be applied backwards.
    fn is_reverse(&self) -> bool;

    /// Whether any subtable of the lookup could apply at a specific glyph.
    fn covers(&self, glyph: GlyphId) -> bool;
}

pub trait LayoutTableExt {
    fn select_script(&self, script_tags: &[hb_tag_t]) -> Option<(bool, ScriptIndex, hb_tag_t)>;
    fn select_script_language(
        &self,
        script_index: ScriptIndex,
        lang_tags: &[hb_tag_t],
    ) -> Option<LanguageIndex>;
    fn get_required_language_feature(
        &self,
        script_index: ScriptIndex,
        lang_index: Option<LanguageIndex>,
    ) -> Option<(FeatureIndex, hb_tag_t)>;
    fn find_language_feature(
        &self,
        script_index: ScriptIndex,
        lang_index: Option<LanguageIndex>,
        feature_tag: hb_tag_t,
    ) -> Option<FeatureIndex>;
}

impl LayoutTableExt for ttf_parser::opentype_layout::LayoutTable<'_> {
    /// Returns true + index and tag of the first found script tag in the given GSUB or GPOS table
    /// or false + index and tag if falling back to a default script.
    fn select_script(&self, script_tags: &[hb_tag_t]) -> Option<(bool, ScriptIndex, hb_tag_t)> {
        for &tag in script_tags {
            if let Some(index) = self.scripts.index(tag) {
                return Some((true, index, tag));
            }
        }

        for &tag in &[
            // try finding 'DFLT'
            hb_tag_t::default_script(),
            // try with 'dflt'; MS site has had typos and many fonts use it now :(
            hb_tag_t::default_language(),
            // try with 'latn'; some old fonts put their features there even though
            // they're really trying to support Thai, for example :(
            hb_tag_t::from_bytes(b"latn"),
        ] {
            if let Some(index) = self.scripts.index(tag) {
                return Some((false, index, tag));
            }
        }

        None
    }

    /// Returns the index of the first found language tag in the given GSUB or GPOS table,
    /// underneath the specified script index.
    fn select_script_language(
        &self,
        script_index: ScriptIndex,
        lang_tags: &[hb_tag_t],
    ) -> Option<LanguageIndex> {
        let script = self.scripts.get(script_index)?;

        for &tag in lang_tags {
            if let Some(index) = script.languages.index(tag) {
                return Some(index);
            }
        }

        // try finding 'dflt'
        if let Some(index) = script.languages.index(hb_tag_t::default_language()) {
            return Some(index);
        }

        None
    }

    /// Returns the index and tag of a required feature in the given GSUB or GPOS table,
    /// underneath the specified script and language.
    fn get_required_language_feature(
        &self,
        script_index: ScriptIndex,
        lang_index: Option<LanguageIndex>,
    ) -> Option<(FeatureIndex, hb_tag_t)> {
        let script = self.scripts.get(script_index)?;
        let sys = match lang_index {
            Some(index) => script.languages.get(index)?,
            None => script.default_language?,
        };
        let idx = sys.required_feature?;
        let tag = self.features.get(idx)?.tag;
        Some((idx, tag))
    }

    /// Returns the index of a given feature tag in the given GSUB or GPOS table,
    /// underneath the specified script and language.
    fn find_language_feature(
        &self,
        script_index: ScriptIndex,
        lang_index: Option<LanguageIndex>,
        feature_tag: hb_tag_t,
    ) -> Option<FeatureIndex> {
        let script = self.scripts.get(script_index)?;
        let sys = match lang_index {
            Some(index) => script.languages.get(index)?,
            None => script.default_language?,
        };

        for i in 0..sys.feature_indices.len() {
            if let Some(index) = sys.feature_indices.get(i) {
                if self.features.get(index).map(|v| v.tag) == Some(feature_tag) {
                    return Some(index);
                }
            }
        }

        None
    }
}

/// Applies the lookups in the given GSUB or GPOS table.
pub fn apply_layout_table<T: LayoutTable>(
    plan: &hb_ot_shape_plan_t,
    face: &hb_font_t,
    buffer: &mut hb_buffer_t,
    table: Option<&T>,
) {
    let mut ctx = ApplyContext::new(T::INDEX, face, buffer);

    for (stage_index, stage) in plan.ot_map.stages(T::INDEX).iter().enumerate() {
        for lookup in plan.ot_map.stage_lookups(T::INDEX, stage_index) {
            ctx.lookup_index = lookup.index;
            ctx.lookup_mask = lookup.mask;
            ctx.auto_zwj = lookup.auto_zwj;
            ctx.auto_zwnj = lookup.auto_zwnj;

            ctx.random = lookup.random;

            if let Some(table) = &table {
                if let Some(lookup) = table.get_lookup(lookup.index) {
                    apply_string::<T>(&mut ctx, lookup);
                }
            }
        }

        if let Some(func) = stage.pause_func {
            func(plan, face, ctx.buffer);
        }
    }
}

fn apply_string<T: LayoutTable>(ctx: &mut ApplyContext, lookup: &T::Lookup) {
    if ctx.buffer.is_empty() || ctx.lookup_mask == 0 {
        return;
    }

    ctx.lookup_props = lookup.props();

    if !lookup.is_reverse() {
        // in/out forward substitution/positioning
        if !T::IN_PLACE {
            ctx.buffer.clear_output();
        }
        ctx.buffer.idx = 0;
        apply_forward(ctx, lookup);

        if !T::IN_PLACE {
            ctx.buffer.sync();
        }
    } else {
        // in-place backward substitution/positioning
        assert!(!ctx.buffer.have_output);

        ctx.buffer.idx = ctx.buffer.len - 1;
        apply_backward(ctx, lookup);
    }
}

fn apply_forward(ctx: &mut ApplyContext, lookup: &impl Apply) -> bool {
    let mut ret = false;
    while ctx.buffer.idx < ctx.buffer.len && ctx.buffer.successful {
        let cur = ctx.buffer.cur(0);
        if (cur.mask & ctx.lookup_mask) != 0
            && ctx.check_glyph_property(cur, ctx.lookup_props)
            && lookup.apply(ctx).is_some()
        {
            ret = true;
        } else {
            ctx.buffer.next_glyph();
        }
    }
    ret
}

fn apply_backward(ctx: &mut ApplyContext, lookup: &impl Apply) -> bool {
    let mut ret = false;
    loop {
        let cur = ctx.buffer.cur(0);
        ret |= (cur.mask & ctx.lookup_mask) != 0
            && ctx.check_glyph_property(cur, ctx.lookup_props)
            && lookup.apply(ctx).is_some();

        if ctx.buffer.idx == 0 {
            break;
        }

        ctx.buffer.idx -= 1;
    }
    ret
}

pub fn _hb_clear_syllables(_: &hb_ot_shape_plan_t, _: &hb_font_t, buffer: &mut hb_buffer_t) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        info.set_syllable(0);
    }
}

/* unicode_props */

/* Design:
 * unicode_props() is a two-byte number.  The low byte includes:
 * - General_Category: 5 bits.
 * - A bit each for:
 *   * Is it Default_Ignorable(); we have a modified Default_Ignorable().
 *   * Whether it's one of the four Mongolian Free Variation Selectors,
 *     CGJ, or other characters that are hidden but should not be ignored
 *     like most other Default_Ignorable()s do during matching.
 *   * Whether it's a grapheme continuation.
 *
 * The high-byte has different meanings, switched by the Gen-Cat:
 * - For Mn,Mc,Me: the modified Combining_Class.
 * - For Cf: whether it's ZWJ, ZWNJ, or something else.
 * - For Ws: index of which space character this is, if space fallback
 *   is needed, ie. we don't set this by default, only if asked to.
 */

//  enum hb_unicode_props_flags_t {
//     UPROPS_MASK_GEN_CAT	= 0x001Fu,
//     UPROPS_MASK_IGNORABLE	= 0x0020u,
//     UPROPS_MASK_HIDDEN	= 0x0040u, /* MONGOLIAN FREE VARIATION SELECTOR 1..4, or TAG characters */
//     UPROPS_MASK_CONTINUATION=0x0080u,

//     /* If GEN_CAT=FORMAT, top byte masks: */
//     UPROPS_MASK_Cf_ZWJ	= 0x0100u,
//     UPROPS_MASK_Cf_ZWNJ	= 0x0200u
//   };
//   HB_MARK_AS_FLAG_T (hb_unicode_props_flags_t);

//   static inline void
//   _hb_glyph_info_set_unicode_props (hb_glyph_info_t *info, hb_buffer_t *buffer)
//   {
//     hb_unicode_funcs_t *unicode = buffer->unicode;
//     unsigned int u = info->codepoint;
//     unsigned int gen_cat = (unsigned int) unicode->general_category (u);
//     unsigned int props = gen_cat;

//     if (u >= 0x80u)
//     {
//       buffer->scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_NON_ASCII;

//       if (unlikely (unicode->is_default_ignorable (u)))
//       {
//         buffer->scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_DEFAULT_IGNORABLES;
//         props |=  UPROPS_MASK_IGNORABLE;
//         if (u == 0x200Cu) props |= UPROPS_MASK_Cf_ZWNJ;
//         else if (u == 0x200Du) props |= UPROPS_MASK_Cf_ZWJ;
//         /* Mongolian Free Variation Selectors need to be remembered
//          * because although we need to hide them like default-ignorables,
//          * they need to non-ignorable during shaping.  This is similar to
//          * what we do for joiners in Indic-like shapers, but since the
//          * FVSes are GC=Mn, we have use a separate bit to remember them.
//          * Fixes:
//          * https://github.com/harfbuzz/harfbuzz/issues/234 */
//         else if (unlikely (hb_in_ranges<hb_codepoint_t> (u, 0x180Bu, 0x180Du, 0x180Fu, 0x180Fu))) props |= UPROPS_MASK_HIDDEN;
//         /* TAG characters need similar treatment. Fixes:
//          * https://github.com/harfbuzz/harfbuzz/issues/463 */
//         else if (unlikely (hb_in_range<hb_codepoint_t> (u, 0xE0020u, 0xE007Fu))) props |= UPROPS_MASK_HIDDEN;
//         /* COMBINING GRAPHEME JOINER should not be skipped; at least some times.
//          * https://github.com/harfbuzz/harfbuzz/issues/554 */
//         else if (unlikely (u == 0x034Fu))
//         {
//       buffer->scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_CGJ;
//       props |= UPROPS_MASK_HIDDEN;
//         }
//       }

//       if (unlikely (HB_UNICODE_GENERAL_CATEGORY_IS_MARK (gen_cat)))
//       {
//         props |= UPROPS_MASK_CONTINUATION;
//         props |= unicode->modified_combining_class (u)<<8;
//       }
//     }

//     info->unicode_props() = props;
//   }

#[inline]
pub fn _hb_glyph_info_set_general_category(
    info: &mut hb_glyph_info_t,
    gen_cat: hb_unicode_general_category_t,
) {
    /* Clears top-byte. */
    let gen_cat = gen_cat.to_rb();
    let n =
        (gen_cat as u16) | (info.unicode_props() & (0xFF & !UnicodeProps::GENERAL_CATEGORY.bits()));
    info.set_unicode_props(n);
}

#[inline]
pub fn _hb_glyph_info_get_general_category(
    info: &hb_glyph_info_t,
) -> hb_unicode_general_category_t {
    let n = info.unicode_props() & UnicodeProps::GENERAL_CATEGORY.bits();
    hb_unicode_general_category_t::from_rb(n as u32)
}

#[inline]
pub fn _hb_glyph_info_is_unicode_mark(info: &hb_glyph_info_t) -> bool {
    _hb_glyph_info_get_general_category(info).is_mark()
}

#[inline]
pub(crate) fn _hb_glyph_info_set_modified_combining_class(
    info: &mut hb_glyph_info_t,
    modified_class: u8,
) {
    if !_hb_glyph_info_is_unicode_mark(info) {
        return;
    }

    let n = ((modified_class as u16) << 8) | (info.unicode_props() & 0xFF);
    info.set_unicode_props(n);
}

#[inline]
pub fn _hb_glyph_info_get_modified_combining_class(info: &hb_glyph_info_t) -> u8 {
    if _hb_glyph_info_is_unicode_mark(info) {
        (info.unicode_props() >> 8) as u8
    } else {
        0
    }
}

// TODO: use
// #[inline]
// pub fn info_cc(info: &hb_glyph_info_t) -> u8 {
//     _hb_glyph_info_get_modified_combining_class(info)
// }

#[inline]
pub(crate) fn _hb_glyph_info_is_unicode_space(info: &hb_glyph_info_t) -> bool {
    _hb_glyph_info_get_general_category(info) == hb_unicode_general_category_t::SpaceSeparator
}

#[inline]
pub(crate) fn _hb_glyph_info_set_unicode_space_fallback_type(
    info: &mut hb_glyph_info_t,
    s: hb_unicode_funcs_t::space_t,
) {
    if !_hb_glyph_info_is_unicode_space(info) {
        return;
    }

    let n = ((s as u16) << 8) | (info.unicode_props() & 0xFF);
    info.set_unicode_props(n);
}

#[inline]
pub(crate) fn _hb_glyph_info_get_unicode_space_fallback_type(
    info: &hb_glyph_info_t,
) -> hb_unicode_funcs_t::space_t {
    if _hb_glyph_info_is_unicode_space(info) {
        (info.unicode_props() >> 8) as u8
    } else {
        hb_unicode_funcs_t::NOT_SPACE
    }
}

#[inline]
pub(crate) fn _hb_glyph_info_is_default_ignorable(info: &hb_glyph_info_t) -> bool {
    let n = info.unicode_props() & UnicodeProps::IGNORABLE.bits();
    n != 0 && !_hb_glyph_info_substituted(info)
}

//   static inline bool
//   _hb_glyph_info_is_default_ignorable_and_not_hidden (const hb_glyph_info_t *info)
//   {
//     return ((info->unicode_props() & (UPROPS_MASK_IGNORABLE|UPROPS_MASK_HIDDEN))
//         == UPROPS_MASK_IGNORABLE) &&
//        !_hb_glyph_info_substituted (info);
//   }

//   static inline void
//   _hb_glyph_info_unhide (hb_glyph_info_t *info)
//   {
//     info->unicode_props() &= ~ UPROPS_MASK_HIDDEN;
//   }

#[inline]
pub(crate) fn _hb_glyph_info_set_continuation(info: &mut hb_glyph_info_t) {
    let mut n = info.unicode_props();
    n |= UnicodeProps::CONTINUATION.bits();
    info.set_unicode_props(n);
}

#[inline]
pub(crate) fn _hb_glyph_info_reset_continuation(info: &mut hb_glyph_info_t) {
    let mut n = info.unicode_props();
    n &= !UnicodeProps::CONTINUATION.bits();
    info.set_unicode_props(n);
}

#[inline]
pub(crate) fn _hb_glyph_info_is_continuation(info: &hb_glyph_info_t) -> bool {
    info.unicode_props() & UnicodeProps::CONTINUATION.bits() != 0
}

pub(crate) fn _hb_grapheme_group_func(_: &hb_glyph_info_t, b: &hb_glyph_info_t) -> bool {
    _hb_glyph_info_is_continuation(b)
}

pub fn _hb_ot_layout_reverse_graphemes(buffer: &mut hb_buffer_t) {
    buffer.reverse_groups(
        _hb_grapheme_group_func,
        buffer.cluster_level == super::buffer::BufferClusterLevel::MonotoneCharacters,
    )
}

#[inline]
pub(crate) fn _hb_glyph_info_is_unicode_format(info: &hb_glyph_info_t) -> bool {
    _hb_glyph_info_get_general_category(info) == hb_unicode_general_category_t::Format
}

#[inline]
pub(crate) fn _hb_glyph_info_is_zwnj(info: &hb_glyph_info_t) -> bool {
    _hb_glyph_info_is_unicode_format(info)
        && (info.unicode_props() & UnicodeProps::CF_ZWNJ.bits() != 0)
}

#[inline]
pub(crate) fn _hb_glyph_info_is_zwj(info: &hb_glyph_info_t) -> bool {
    _hb_glyph_info_is_unicode_format(info)
        && (info.unicode_props() & UnicodeProps::CF_ZWJ.bits() != 0)
}

//   static inline bool
//   _hb_glyph_info_is_joiner (const hb_glyph_info_t *info)
//   {
//     return _hb_glyph_info_is_unicode_format (info) && (info->unicode_props() & (UPROPS_MASK_Cf_ZWNJ|UPROPS_MASK_Cf_ZWJ));
//   }

//   static inline void
//   _hb_glyph_info_flip_joiners (hb_glyph_info_t *info)
//   {
//     if (!_hb_glyph_info_is_unicode_format (info))
//       return;
//     info->unicode_props() ^= UPROPS_MASK_Cf_ZWNJ | UPROPS_MASK_Cf_ZWJ;
//   }

//   /* lig_props: aka lig_id / lig_comp
//    *
//    * When a ligature is formed:
//    *
//    *   - The ligature glyph and any marks in between all the same newly allocated
//    *     lig_id,
//    *   - The ligature glyph will get lig_num_comps set to the number of components
//    *   - The marks get lig_comp > 0, reflecting which component of the ligature
//    *     they were applied to.
//    *   - This is used in GPOS to attach marks to the right component of a ligature
//    *     in MarkLigPos,
//    *   - Note that when marks are ligated together, much of the above is skipped
//    *     and the current lig_id reused.
//    *
//    * When a multiple-substitution is done:
//    *
//    *   - All resulting glyphs will have lig_id = 0,
//    *   - The resulting glyphs will have lig_comp = 0, 1, 2, ... respectively.
//    *   - This is used in GPOS to attach marks to the first component of a
//    *     multiple substitution in MarkBasePos.
//    *
//    * The numbers are also used in GPOS to do mark-to-mark positioning only
//    * to marks that belong to the same component of the same ligature.
//    */
//   static inline void
//   _hb_glyph_info_clear_lig_props (hb_glyph_info_t *info)
//   {
//     info->lig_props() = 0;
//   }

const IS_LIG_BASE: u8 = 0x10;

#[inline]
pub(crate) fn _hb_glyph_info_set_lig_props_for_ligature(
    info: &mut hb_glyph_info_t,
    lig_id: u8,
    lig_num_comps: u8,
) {
    info.set_lig_props((lig_id << 5) | IS_LIG_BASE | (lig_num_comps & 0x0F));
}

#[inline]
pub(crate) fn _hb_glyph_info_set_lig_props_for_mark(
    info: &mut hb_glyph_info_t,
    lig_id: u8,
    lig_comp: u8,
) {
    info.set_lig_props((lig_id << 5) | (lig_comp & 0x0F));
}

#[inline]
pub(crate) fn _hb_glyph_info_set_lig_props_for_component(info: &mut hb_glyph_info_t, comp: u8) {
    _hb_glyph_info_set_lig_props_for_mark(info, 0, comp);
}

#[inline]
pub(crate) fn _hb_glyph_info_get_lig_id(info: &hb_glyph_info_t) -> u8 {
    info.lig_props() >> 5
}

#[inline]
pub(crate) fn _hb_glyph_info_ligated_internal(info: &hb_glyph_info_t) -> bool {
    info.lig_props() & IS_LIG_BASE != 0
}

#[inline]
pub(crate) fn _hb_glyph_info_get_lig_comp(info: &hb_glyph_info_t) -> u8 {
    if _hb_glyph_info_ligated_internal(info) {
        0
    } else {
        info.lig_props() & 0x0F
    }
}

#[inline]
pub(crate) fn _hb_glyph_info_get_lig_num_comps(info: &hb_glyph_info_t) -> u8 {
    if info.glyph_props() & GlyphPropsFlags::LIGATURE.bits() != 0
        && _hb_glyph_info_ligated_internal(info)
    {
        info.lig_props() & 0x0F
    } else {
        1
    }
}

//   /* glyph_props: */
//   static inline void
//   _hb_glyph_info_set_glyph_props (hb_glyph_info_t *info, unsigned int props)
//   {
//     info->glyph_props() = props;
//   }

//   static inline unsigned int
//   _hb_glyph_info_get_glyph_props (const hb_glyph_info_t *info)
//   {
//     return info->glyph_props();
//   }

#[inline]
pub(crate) fn _hb_glyph_info_is_base_glyph(info: &hb_glyph_info_t) -> bool {
    info.glyph_props() & GlyphPropsFlags::BASE_GLYPH.bits() != 0
}

#[inline]
pub(crate) fn _hb_glyph_info_is_ligature(info: &hb_glyph_info_t) -> bool {
    info.glyph_props() & GlyphPropsFlags::LIGATURE.bits() != 0
}

#[inline]
pub(crate) fn _hb_glyph_info_is_mark(info: &hb_glyph_info_t) -> bool {
    info.glyph_props() & GlyphPropsFlags::MARK.bits() != 0
}

#[inline]
pub(crate) fn _hb_glyph_info_substituted(info: &hb_glyph_info_t) -> bool {
    info.glyph_props() & GlyphPropsFlags::SUBSTITUTED.bits() != 0
}

#[inline]
pub(crate) fn _hb_glyph_info_ligated(info: &hb_glyph_info_t) -> bool {
    info.glyph_props() & GlyphPropsFlags::LIGATED.bits() != 0
}

#[inline]
pub(crate) fn _hb_glyph_info_multiplied(info: &hb_glyph_info_t) -> bool {
    info.glyph_props() & GlyphPropsFlags::MULTIPLIED.bits() != 0
}

#[inline]
pub(crate) fn _hb_glyph_info_ligated_and_didnt_multiply(info: &hb_glyph_info_t) -> bool {
    _hb_glyph_info_ligated(info) && !_hb_glyph_info_multiplied(info)
}

#[inline]
pub(crate) fn _hb_glyph_info_clear_ligated_and_multiplied(info: &mut hb_glyph_info_t) {
    let mut n = info.glyph_props();
    n &= !(GlyphPropsFlags::LIGATED | GlyphPropsFlags::MULTIPLIED).bits();
    info.set_glyph_props(n);
}

#[inline]
pub(crate) fn _hb_glyph_info_clear_substituted(info: &mut hb_glyph_info_t) {
    let mut n = info.glyph_props();
    n &= !GlyphPropsFlags::SUBSTITUTED.bits();
    info.set_glyph_props(n);
}

pub fn _hb_clear_substitution_flags(
    _: &hb_ot_shape_plan_t,
    _: &hb_font_t,
    buffer: &mut hb_buffer_t,
) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        _hb_glyph_info_clear_substituted(info);
    }
}
