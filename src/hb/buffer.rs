use alloc::{string::String, vec::Vec};
use core::convert::TryFrom;
use core::cmp::min;

use ttf_parser::GlyphId;

use super::buffer::glyph_flag::{UNSAFE_TO_BREAK, UNSAFE_TO_CONCAT};
use super::face::GlyphExtents;
use super::unicode::{CharExt, GeneralCategoryExt};
use super::{hb_font_t, hb_mask_t};
use crate::{script, BufferClusterLevel, BufferFlags, Direction, Language, Script, SerializeFlags};

const CONTEXT_LENGTH: usize = 5;

pub mod glyph_flag {
    /// Indicates that if input text is broken at the
    /// beginning of the cluster this glyph is part of,
    /// then both sides need to be re-shaped, as the
    /// result might be different.
    ///
    /// On the flip side, it means that when
    /// this flag is not present, then it is safe
    /// to break the glyph-run at the beginning of
    /// this cluster, and the two sides will represent
    /// the exact same result one would get if breaking
    /// input text at the beginning of this cluster and
    /// shaping the two sides separately.
    ///
    /// This can be used to optimize paragraph layout,
    /// by avoiding re-shaping of each line after line-breaking.
    pub const UNSAFE_TO_BREAK: u32 = 0x00000001;
    /// Indicates that if input text is changed on one side
    /// of the beginning of the cluster this glyph is part
    /// of, then the shaping results for the other side
    /// might change.
    ///
    /// Note that the absence of this flag will NOT by
    /// itself mean that it IS safe to concat text. Only
    /// two pieces of text both of which clear of this
    /// flag can be concatenated safely.
    ///
    /// This can be used to optimize paragraph layout,
    /// by avoiding re-shaping of each line after
    /// line-breaking, by limiting the reshaping to a
    /// small piece around the breaking position only,
    /// even if the breaking position carries the
    /// UNSAFE_TO_BREAK or when hyphenation or
    /// other text transformation happens at
    /// line-break position, in the following way:
    ///
    /// 1. Iterate back from the line-break
    /// position until the first cluster
    /// start position that is NOT unsafe-to-concat,
    /// 2. shape the segment from there till the
    /// end of line, 3. check whether the resulting
    /// glyph-run also is clear of the unsafe-to-concat
    /// at its start-of-text position; if it is, just
    /// splice it into place and the line is shaped;
    /// If not, move on to a position further back that
    /// is clear of unsafe-to-concat and retry from
    /// there, and repeat.
    ///
    /// At the start of next line a similar
    /// algorithm can be implemented.
    /// That is: 1. Iterate forward from
    /// the line-break position until the first cluster
    /// start position that is NOT unsafe-to-concat, 2.
    /// shape the segment from beginning of the line to
    /// that position, 3. check whether the resulting
    /// glyph-run also is clear of the unsafe-to-concat
    /// at its end-of-text position; if it is, just splice
    /// it into place and the beginning is shaped; If not,
    /// move on to a position further forward that is clear
    /// of unsafe-to-concat and retry up to there, and repeat.
    ///
    /// A slight complication will arise in the
    /// implementation of the algorithm above,
    /// because while
    /// our buffer API has a way to return flags
    /// for position corresponding to
    /// start-of-text, there is currently no
    /// position corresponding to end-of-text.
    /// This limitation can be alleviated by
    /// shaping more text than needed and
    /// looking for unsafe-to-concat flag
    /// within text clusters.
    ///
    /// The UNSAFE_TO_BREAK flag will always imply this flag.
    /// To use this flag, you must enable the buffer flag
    ///	PRODUCE_UNSAFE_TO_CONCAT during shaping, otherwise
    /// the buffer flag will not be reliably produced.
    pub const UNSAFE_TO_CONCAT: u32 = 0x00000002;

    /// All the currently defined flags.
    pub const DEFINED: u32 = 0x00000003; // OR of all defined flags
}

/// Holds the positions of the glyph in both horizontal and vertical directions.
///
/// All positions are relative to the current point.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct GlyphPosition {
    /// How much the line advances after drawing this glyph when setting text in
    /// horizontal direction.
    pub x_advance: i32,
    /// How much the line advances after drawing this glyph when setting text in
    /// vertical direction.
    pub y_advance: i32,
    /// How much the glyph moves on the X-axis before drawing it, this should
    /// not affect how much the line advances.
    pub x_offset: i32,
    /// How much the glyph moves on the Y-axis before drawing it, this should
    /// not affect how much the line advances.
    pub y_offset: i32,
    var: u32,
}

unsafe impl bytemuck::Zeroable for GlyphPosition {}
unsafe impl bytemuck::Pod for GlyphPosition {}

impl GlyphPosition {
    #[inline]
    pub(crate) fn attach_chain(&self) -> i16 {
        // glyph to which this attaches to, relative to current glyphs;
        // negative for going back, positive for forward.
        let v: &[i16; 2] = bytemuck::cast_ref(&self.var);
        v[0]
    }

    #[inline]
    pub(crate) fn set_attach_chain(&mut self, n: i16) {
        let v: &mut [i16; 2] = bytemuck::cast_mut(&mut self.var);
        v[0] = n;
    }

    #[inline]
    pub(crate) fn attach_type(&self) -> u8 {
        // attachment type
        // Note! if attach_chain() is zero, the value of attach_type() is irrelevant.
        let v: &[u8; 4] = bytemuck::cast_ref(&self.var);
        v[2]
    }

    #[inline]
    pub(crate) fn set_attach_type(&mut self, n: u8) {
        let v: &mut [u8; 4] = bytemuck::cast_mut(&mut self.var);
        v[2] = n;
    }
}

/// A glyph info.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct hb_glyph_info_t {
    // NOTE: Stores a Unicode codepoint before shaping and a glyph ID after.
    //       Just like harfbuzz, we are using the same variable for two purposes.
    //       Occupies u32 as a codepoint and u16 as a glyph id.
    /// A selected glyph.
    ///
    /// Guarantee to be <= `u16::MAX`.
    pub glyph_id: u32,
    pub(crate) mask: hb_mask_t,
    /// An index to the start of the grapheme cluster in the original string.
    ///
    /// [Read more on clusters](https://harfbuzz.github.io/clusters.html).
    pub cluster: u32,
    pub(crate) var1: u32,
    pub(crate) var2: u32,
}

unsafe impl bytemuck::Zeroable for hb_glyph_info_t {}
unsafe impl bytemuck::Pod for hb_glyph_info_t {}

impl hb_glyph_info_t {
    /// Indicates that if input text is broken at the beginning of the cluster this glyph
    /// is part of, then both sides need to be re-shaped, as the result might be different.
    ///
    /// On the flip side, it means that when this flag is not present,
    /// then it's safe to break the glyph-run at the beginning of this cluster,
    /// and the two sides represent the exact same result one would get if breaking input text
    /// at the beginning of this cluster and shaping the two sides separately.
    /// This can be used to optimize paragraph layout, by avoiding re-shaping of each line
    /// after line-breaking, or limiting the reshaping to a small piece around
    /// the breaking point only.
    pub fn unsafe_to_break(&self) -> bool {
        self.mask & glyph_flag::UNSAFE_TO_BREAK != 0
    }

    #[inline]
    pub(crate) fn as_char(&self) -> char {
        char::try_from(self.glyph_id).unwrap()
    }

    #[inline]
    pub(crate) fn as_glyph(&self) -> GlyphId {
        debug_assert!(self.glyph_id <= u32::from(u16::MAX));
        GlyphId(self.glyph_id as u16)
    }

    // Var allocation: unicode_props
    // Used during the entire shaping process to store unicode properties

    #[inline]
    pub(crate) fn unicode_props(&self) -> u16 {
        let v: &[u16; 2] = bytemuck::cast_ref(&self.var2);
        v[0]
    }

    #[inline]
    pub(crate) fn set_unicode_props(&mut self, n: u16) {
        let v: &mut [u16; 2] = bytemuck::cast_mut(&mut self.var2);
        v[0] = n;
    }

    pub(crate) fn init_unicode_props(&mut self, scratch_flags: &mut hb_buffer_scratch_flags_t) {
        let u = self.as_char();
        let gc = u.general_category();
        let mut props = gc.to_rb() as u16;

        if u as u32 >= 0x80 {
            *scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_NON_ASCII;

            if u.is_default_ignorable() {
                props |= UnicodeProps::IGNORABLE.bits();
                *scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_DEFAULT_IGNORABLES;

                match u as u32 {
                    0x200C => props |= UnicodeProps::CF_ZWNJ.bits(),
                    0x200D => props |= UnicodeProps::CF_ZWJ.bits(),

                    // Mongolian Free Variation Selectors need to be remembered
                    // because although we need to hide them like default-ignorables,
                    // they need to non-ignorable during shaping.  This is similar to
                    // what we do for joiners in Indic-like shapers, but since the
                    // FVSes are GC=Mn, we have use a separate bit to remember them.
                    // Fixes:
                    // https://github.com/harfbuzz/harfbuzz/issues/234
                    0x180B..=0x180D | 0x180F => props |= UnicodeProps::HIDDEN.bits(),

                    // TAG characters need similar treatment. Fixes:
                    // https://github.com/harfbuzz/harfbuzz/issues/463
                    0xE0020..=0xE007F => props |= UnicodeProps::HIDDEN.bits(),

                    // COMBINING GRAPHEME JOINER should not be skipped; at least some times.
                    // https://github.com/harfbuzz/harfbuzz/issues/554
                    0x034F => {
                        props |= UnicodeProps::HIDDEN.bits();
                        *scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_CGJ;
                    }

                    _ => {}
                }
            }

            if gc.is_mark() {
                props |= UnicodeProps::CONTINUATION.bits();
                props |= (u.modified_combining_class() as u16) << 8;
            }
        }

        self.set_unicode_props(props);
    }

    #[inline]
    pub(crate) fn is_hidden(&self) -> bool {
        self.unicode_props() & UnicodeProps::HIDDEN.bits() != 0
    }

    #[inline]
    pub(crate) fn unhide(&mut self) {
        let mut n = self.unicode_props();
        n &= !UnicodeProps::HIDDEN.bits();
        self.set_unicode_props(n);
    }

    #[inline]
    pub(crate) fn lig_props(&self) -> u8 {
        let v: &[u8; 4] = bytemuck::cast_ref(&self.var1);
        v[2]
    }

    #[inline]
    pub(crate) fn set_lig_props(&mut self, n: u8) {
        let v: &mut [u8; 4] = bytemuck::cast_mut(&mut self.var1);
        v[2] = n;
    }

    #[inline]
    pub(crate) fn glyph_props(&self) -> u16 {
        let v: &[u16; 2] = bytemuck::cast_ref(&self.var1);
        v[0]
    }

    #[inline]
    pub(crate) fn set_glyph_props(&mut self, n: u16) {
        let v: &mut [u16; 2] = bytemuck::cast_mut(&mut self.var1);
        v[0] = n;
    }

    #[inline]
    pub(crate) fn syllable(&self) -> u8 {
        let v: &[u8; 4] = bytemuck::cast_ref(&self.var1);
        v[3]
    }

    #[inline]
    pub(crate) fn set_syllable(&mut self, n: u8) {
        let v: &mut [u8; 4] = bytemuck::cast_mut(&mut self.var1);
        v[3] = n;
    }

    // Var allocation: glyph_index
    // Used during the normalization process to store glyph indices

    #[inline]
    pub(crate) fn glyph_index(&mut self) -> u32 {
        self.var1
    }

    #[inline]
    pub(crate) fn set_glyph_index(&mut self, n: u32) {
        self.var1 = n;
    }
}

pub type hb_buffer_cluster_level_t = u32;
pub const HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES: u32 = 0;
pub const HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS: u32 = 1;
pub const HB_BUFFER_CLUSTER_LEVEL_CHARACTERS: u32 = 2;
pub const HB_BUFFER_CLUSTER_LEVEL_DEFAULT: u32 = HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES;

pub struct hb_buffer_t {
    // Information about how the text in the buffer should be treated.
    pub flags: BufferFlags,
    pub cluster_level: hb_buffer_cluster_level_t,
    pub invisible: Option<GlyphId>,

    // Buffer contents.
    pub direction: Direction,
    pub script: Option<Script>,
    pub language: Option<Language>,

    /// Allocations successful.
    pub successful: bool,
    /// Whether we have an output buffer going on.
    pub(crate) have_output: bool,
    pub have_separate_output: bool,
    /// Whether we have positions
    pub have_positions: bool,

    pub idx: usize,
    pub len: usize,
    pub out_len: usize,

    pub info: Vec<hb_glyph_info_t>,
    pub pos: Vec<GlyphPosition>,

    // Text before / after the main buffer contents.
    // Always in Unicode, and ordered outward.
    // Index 0 is for "pre-context", 1 for "post-context".
    pub context: [[char; CONTEXT_LENGTH]; 2],
    pub context_len: [usize; 2],

    // Managed by enter / leave
    pub serial: u8,
    pub scratch_flags: hb_buffer_scratch_flags_t,
    /// Maximum allowed len.
    pub max_len: usize,
    /// Maximum allowed operations.
    pub max_ops: i32,
}

impl hb_buffer_t {
    pub const MAX_LEN_FACTOR: usize = 64;
    pub const MAX_LEN_MIN: usize = 16384;
    // Shaping more than a billion chars? Let us know!
    pub const MAX_LEN_DEFAULT: usize = 0x3FFFFFFF;

    pub const MAX_OPS_FACTOR: i32 = 1024;
    pub const MAX_OPS_MIN: i32 = 16384;
    // Shaping more than a billion operations? Let us know!
    pub const MAX_OPS_DEFAULT: i32 = 0x1FFFFFFF;

    /// Creates a new `Buffer`.
    pub fn new() -> Self {
        hb_buffer_t {
            flags: BufferFlags::empty(),
            cluster_level: HB_BUFFER_CLUSTER_LEVEL_DEFAULT,
            invisible: None,
            scratch_flags: HB_BUFFER_SCRATCH_FLAG_DEFAULT,
            max_len: Self::MAX_LEN_DEFAULT,
            max_ops: Self::MAX_OPS_DEFAULT,
            direction: Direction::Invalid,
            script: None,
            language: None,
            successful: true,
            have_output: false,
            have_positions: false,
            idx: 0,
            len: 0,
            out_len: 0,
            info: Vec::new(),
            pos: Vec::new(),
            have_separate_output: false,
            serial: 0,
            context: [
                ['\0', '\0', '\0', '\0', '\0'],
                ['\0', '\0', '\0', '\0', '\0'],
            ],
            context_len: [0, 0],
        }
    }

    #[inline]
    pub fn info_slice(&self) -> &[hb_glyph_info_t] {
        &self.info[..self.len]
    }

    #[inline]
    pub fn info_slice_mut(&mut self) -> &mut [hb_glyph_info_t] {
        &mut self.info[..self.len]
    }

    #[inline]
    pub fn out_info(&self) -> &[hb_glyph_info_t] {
        if self.have_separate_output {
            bytemuck::cast_slice(self.pos.as_slice())
        } else {
            &self.info
        }
    }

    #[inline]
    pub fn out_info_mut(&mut self) -> &mut [hb_glyph_info_t] {
        if self.have_separate_output {
            bytemuck::cast_slice_mut(self.pos.as_mut_slice())
        } else {
            &mut self.info
        }
    }

    #[inline]
    fn set_out_info(&mut self, i: usize, info: hb_glyph_info_t) {
        self.out_info_mut()[i] = info;
    }

    #[inline]
    pub fn cur(&self, i: usize) -> &hb_glyph_info_t {
        &self.info[self.idx + i]
    }

    #[inline]
    pub fn cur_mut(&mut self, i: usize) -> &mut hb_glyph_info_t {
        let idx = self.idx + i;
        &mut self.info[idx]
    }

    #[inline]
    pub fn cur_pos_mut(&mut self) -> &mut GlyphPosition {
        let i = self.idx;
        &mut self.pos[i]
    }

    #[inline]
    pub fn prev(&self) -> &hb_glyph_info_t {
        let idx = self.out_len.saturating_sub(1);
        &self.out_info()[idx]
    }

    #[inline]
    pub fn prev_mut(&mut self) -> &mut hb_glyph_info_t {
        let idx = self.out_len.saturating_sub(1);
        &mut self.out_info_mut()[idx]
    }

    fn clear(&mut self) {
        self.direction = Direction::Invalid;
        self.script = None;
        self.language = None;

        self.successful = true;
        self.have_output = false;
        self.have_positions = false;

        self.idx = 0;
        self.info.clear();
        self.pos.clear();
        self.len = 0;
        self.out_len = 0;
        self.have_separate_output = false;

        self.context = [
            ['\0', '\0', '\0', '\0', '\0'],
            ['\0', '\0', '\0', '\0', '\0'],
        ];
        self.context_len = [0, 0];

        self.serial = 0;
        self.scratch_flags = HB_BUFFER_SCRATCH_FLAG_DEFAULT;
        self.cluster_level = HB_BUFFER_CLUSTER_LEVEL_DEFAULT;
    }

    #[inline]
    pub fn backtrack_len(&self) -> usize {
        if self.have_output {
            self.out_len
        } else {
            self.idx
        }
    }

    #[inline]
    pub fn lookahead_len(&self) -> usize {
        self.len - self.idx
    }

    #[inline]
    fn next_serial(&mut self) -> u8 {
        self.serial += 1;

        if self.serial == 0 {
            self.serial += 1;
        }

        self.serial
    }

    fn add(&mut self, codepoint: u32, cluster: u32) {
        self.ensure(self.len + 1);

        let i = self.len;
        self.info[i] = hb_glyph_info_t {
            glyph_id: codepoint,
            mask: 0,
            cluster,
            var1: 0,
            var2: 0,
        };

        self.len += 1;
    }

    #[inline]
    pub fn reverse(&mut self) {
        if self.is_empty() {
            return;
        }

        self.reverse_range(0, self.len);
    }

    pub fn reverse_range(&mut self, start: usize, end: usize) {
        if end - start < 2 {
            return;
        }

        self.info[start..end].reverse();
        if self.have_positions {
            self.pos[start..end].reverse();
        }
    }

    pub fn reverse_groups<F>(&mut self, group: F, merge_clusters: bool)
    where
        F: Fn(&hb_glyph_info_t, &hb_glyph_info_t) -> bool,
    {
        if self.is_empty() {
            return;
        }

        let mut start = 0;

        for i in 1..self.len {
            if !group(&self.info[i - 1], &self.info[i]) {
                if merge_clusters {
                    self.merge_clusters(start, i);
                }

                self.reverse_range(start, i);
                start = i;
            }

            if merge_clusters {
                self.merge_clusters(start, i);
            }

            self.reverse_range(start, i);

            self.reverse();
        }
    }

    pub fn group_end<F>(&self, mut start: usize, group: F) -> usize
    where
        F: Fn(&hb_glyph_info_t, &hb_glyph_info_t) -> bool,
    {
        start += 1;

        while start < self.len && group(&self.info[start - 1], &self.info[start]) {
            start += 1;
        }

        start
    }

    #[inline]
    fn reset_clusters(&mut self) {
        for (i, info) in self.info.iter_mut().enumerate() {
            info.cluster = i as u32;
        }
    }

    pub fn guess_segment_properties(&mut self) {
        if self.script.is_none() {
            for info in &self.info {
                match info.as_char().script() {
                    crate::script::COMMON | crate::script::INHERITED | crate::script::UNKNOWN => {}
                    s => {
                        self.script = Some(s);
                        break;
                    }
                }
            }
        }

        if self.direction == Direction::Invalid {
            if let Some(script) = self.script {
                self.direction = Direction::from_script(script).unwrap_or_default();
            }

            if self.direction == Direction::Invalid {
                self.direction = Direction::LeftToRight;
            }
        }

        // TODO: language must be set
    }

    pub fn sync(&mut self) {
        assert!(self.have_output);

        assert!(self.idx <= self.len);
        if !self.successful {
            self.have_output = false;
            self.out_len = 0;
            self.idx = 0;
            return;
        }

        self.next_glyphs(self.len - self.idx);

        if self.have_separate_output {
            // Swap info and pos buffers.
            let info: Vec<GlyphPosition> = bytemuck::cast_vec(core::mem::take(&mut self.info));
            let pos: Vec<hb_glyph_info_t> = bytemuck::cast_vec(core::mem::take(&mut self.pos));
            self.pos = info;
            self.info = pos;
        }

        self.len = self.out_len;

        self.have_output = false;
        self.out_len = 0;
        self.idx = 0;
    }

    pub fn clear_output(&mut self) {
        self.have_output = true;
        self.have_positions = false;

        self.idx = 0;
        self.out_len = 0;
        self.have_separate_output = false;
    }

    pub fn clear_positions(&mut self) {
        self.have_output = false;
        self.have_positions = true;

        self.out_len = 0;
        self.have_separate_output = false;

        for pos in &mut self.pos {
            *pos = GlyphPosition::default();
        }
    }

    pub fn replace_glyphs(&mut self, num_in: usize, num_out: usize, glyph_data: &[u32]) {
        if !self.make_room_for(num_in, num_out) {
            return;
        }

        assert!(self.idx + num_in <= self.len);

        self.merge_clusters(self.idx, self.idx + num_in);

        let orig_info = self.info[self.idx];
        for i in 0..num_out {
            let ii = self.out_len + i;
            self.set_out_info(ii, orig_info);
            self.out_info_mut()[ii].glyph_id = glyph_data[i];
        }

        self.idx += num_in;
        self.out_len += num_out;
    }

    pub fn replace_glyph(&mut self, glyph_index: u32) {
        if self.have_separate_output || self.out_len != self.idx {
            if !self.make_room_for(1, 1) {
                return;
            }

            self.set_out_info(self.out_len, self.info[self.idx]);
        }

        let out_len = self.out_len;
        self.out_info_mut()[out_len].glyph_id = glyph_index;

        self.idx += 1;
        self.out_len += 1;
    }

    pub fn output_glyph(&mut self, glyph_index: u32) {
        if !self.make_room_for(0, 1) {
            return;
        }

        if self.idx == self.len && self.out_len == 0 {
            return;
        }

        let out_len = self.out_len;
        if self.idx < self.len {
            self.set_out_info(out_len, self.info[self.idx]);
        } else {
            let info = self.out_info()[out_len - 1];
            self.set_out_info(out_len, info);
        }

        self.out_info_mut()[out_len].glyph_id = glyph_index;

        self.out_len += 1;
    }

    pub fn output_info(&mut self, glyph_info: hb_glyph_info_t) {
        if !self.make_room_for(0, 1) {
            return;
        }

        self.set_out_info(self.out_len, glyph_info);
        self.out_len += 1;
    }

    /// Copies glyph at idx to output but doesn't advance idx.
    pub fn copy_glyph(&mut self) {
        if !self.make_room_for(0, 1) {
            return;
        }

        self.set_out_info(self.out_len, self.info[self.idx]);
        self.out_len += 1;
    }

    /// Copies glyph at idx to output and advance idx.
    ///
    /// If there's no output, just advance idx.
    pub fn next_glyph(&mut self) {
        if self.have_output {
            if self.have_separate_output || self.out_len != self.idx {
                if !self.make_room_for(1, 1) {
                    return;
                }

                self.set_out_info(self.out_len, self.info[self.idx]);
            }

            self.out_len += 1;
        }

        self.idx += 1;
    }

    /// Copies n glyphs at idx to output and advance idx.
    ///
    /// If there's no output, just advance idx.
    pub fn next_glyphs(&mut self, n: usize) {
        if self.have_output {
            if self.have_separate_output || self.out_len != self.idx {
                if !self.make_room_for(n, n) {
                    return;
                }

                for i in 0..n {
                    self.set_out_info(self.out_len + i, self.info[self.idx + i]);
                }
            }

            self.out_len += n;
        }

        self.idx += n;
    }

    /// Advance idx without copying to output.
    pub fn skip_glyph(&mut self) {
        self.idx += 1;
    }

    pub fn reset_masks(&mut self, mask: hb_mask_t) {
        for info in &mut self.info[..self.len] {
            info.mask = mask;
        }
    }

    pub fn set_masks(
        &mut self,
        mut value: hb_mask_t,
        mask: hb_mask_t,
        cluster_start: u32,
        cluster_end: u32,
    ) {
        let not_mask = !mask;
        value &= mask;

        if mask == 0 {
            return;
        }

        if cluster_start == 0 && cluster_end == core::u32::MAX {
            for info in &mut self.info[..self.len] {
                info.mask = (info.mask & not_mask) | value;
            }

            return;
        }

        for info in &mut self.info[..self.len] {
            if cluster_start <= info.cluster && info.cluster < cluster_end {
                info.mask = (info.mask & not_mask) | value;
            }
        }
    }

    pub fn merge_clusters(&mut self, start: usize, end: usize) {
        if end - start < 2 {
            return;
        }

        self.merge_clusters_impl(start, end)
    }

    fn merge_clusters_impl(&mut self, mut start: usize, mut end: usize) {
        if self.cluster_level == HB_BUFFER_CLUSTER_LEVEL_CHARACTERS {
            self.unsafe_to_break(Some(start), Some(end));
            return;
        }

        let mut cluster = self.info[start].cluster;

        for i in start + 1..end {
            cluster = core::cmp::min(cluster, self.info[i].cluster);
        }

        // Extend end
        while end < self.len && self.info[end - 1].cluster == self.info[end].cluster {
            end += 1;
        }

        // Extend start
        while end < start && self.info[start - 1].cluster == self.info[start].cluster {
            start -= 1;
        }

        // If we hit the start of buffer, continue in out-buffer.
        if self.idx == start {
            let mut i = self.out_len;
            while i != 0 && self.out_info()[i - 1].cluster == self.info[start].cluster {
                Self::set_cluster(&mut self.out_info_mut()[i - 1], cluster, 0);
                i -= 1;
            }
        }

        for i in start..end {
            Self::set_cluster(&mut self.info[i], cluster, 0);
        }
    }

    pub fn merge_out_clusters(&mut self, mut start: usize, mut end: usize) {
        if self.cluster_level == HB_BUFFER_CLUSTER_LEVEL_CHARACTERS {
            return;
        }

        if end - start < 2 {
            return;
        }

        let mut cluster = self.out_info()[start].cluster;

        for i in start + 1..end {
            cluster = core::cmp::min(cluster, self.out_info()[i].cluster);
        }

        // Extend start
        while start != 0 && self.out_info()[start - 1].cluster == self.out_info()[start].cluster {
            start -= 1;
        }

        // Extend end
        while end < self.out_len && self.out_info()[end - 1].cluster == self.out_info()[end].cluster
        {
            end += 1;
        }

        // If we hit the start of buffer, continue in out-buffer.
        if end == self.out_len {
            let mut i = self.idx;
            while i < self.len && self.info[i].cluster == self.out_info()[end - 1].cluster {
                Self::set_cluster(&mut self.info[i], cluster, 0);
                i += 1;
            }
        }

        for i in start..end {
            Self::set_cluster(&mut self.out_info_mut()[i], cluster, 0);
        }
    }

    /// Merge clusters for deleting current glyph, and skip it.
    pub fn delete_glyph(&mut self) {
        let cluster = self.info[self.idx].cluster;

        if self.idx + 1 < self.len && cluster == self.info[self.idx + 1].cluster {
            // Cluster survives; do nothing.
            self.skip_glyph();
            return;
        }

        if self.out_len != 0 {
            // Merge cluster backward.
            if cluster < self.out_info()[self.out_len - 1].cluster {
                let mask = self.info[self.idx].mask;
                let old_cluster = self.out_info()[self.out_len - 1].cluster;

                let mut i = self.out_len;
                while i != 0 && self.out_info()[i - 1].cluster == old_cluster {
                    Self::set_cluster(&mut self.out_info_mut()[i - 1], cluster, mask);
                    i -= 1;
                }
            }

            self.skip_glyph();
            return;
        }

        if self.idx + 1 < self.len {
            // Merge cluster forward.
            self.merge_clusters(self.idx, self.idx + 2);
        }

        self.skip_glyph();
    }

    pub fn delete_glyphs_inplace(&mut self, filter: impl Fn(&hb_glyph_info_t) -> bool) {
        // Merge clusters and delete filtered glyphs.
        // NOTE! We can't use out-buffer as we have positioning data.
        let mut j = 0;

        for i in 0..self.len {
            if filter(&self.info[i]) {
                // Merge clusters.
                // Same logic as delete_glyph(), but for in-place removal

                let cluster = self.info[i].cluster;
                if i + 1 < self.len && cluster == self.info[i + 1].cluster {
                    // Cluster survives; do nothing.
                    continue;
                }

                if j != 0 {
                    // Merge cluster backward.
                    if cluster < self.info[j - 1].cluster {
                        let mask = self.info[i].mask;
                        let old_cluster = self.info[j - 1].cluster;

                        let mut k = j;
                        while k > 0 && self.info[k - 1].cluster == old_cluster {
                            Self::set_cluster(&mut self.info[k - 1], cluster, mask);
                            k -= 1;
                        }
                    }
                    continue;
                }

                if i + 1 < self.len {
                    // Merge cluster forward.
                    self.merge_clusters(i, i + 2);
                }

                continue;
            }

            if j != i {
                self.info[j] = self.info[i];
                self.pos[j] = self.pos[i];
            }

            j += 1;
        }

        self.len = j;
    }

    pub fn unsafe_to_break(&mut self, start: Option<usize>, end: Option<usize>) {
        self._set_glyph_flags(
            UNSAFE_TO_BREAK | UNSAFE_TO_CONCAT,
            start,
            end,
            Some(true),
            None,
        );
    }

    /// Adds glyph flags in mask to infos with clusters between start and end.
    /// The start index will be from out-buffer if from_out_buffer is true.
    /// If interior is true, then the cluster having the minimum value is skipped. */
    fn _set_glyph_flags(
        &mut self,
        mask: hb_mask_t,
        start: Option<usize>,
        end: Option<usize>,
        interior: Option<bool>,
        from_out_buffer: Option<bool>,
    ) {
        let start = start.unwrap_or(0);
        let end = min(end.unwrap_or(self.len), self.len);
        let interior = interior.unwrap_or(false);
        let from_out_buffer = from_out_buffer.unwrap_or(false);

        if interior && !from_out_buffer && end - start < 2 {
            return;
        }

        self.scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_GLYPH_FLAGS;

        if !from_out_buffer || !self.have_output {
            if !interior {
                for i in start..end {
                    self.info[i].mask |= mask;
                }
            } else {
                let cluster = Self::_infos_find_min_cluster(&self.info, start, end, None);
                if Self::_infos_set_glyph_flags(&mut self.info, start, end, cluster, mask) {
                    self.scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_GLYPH_FLAGS;
                }
            }
        } else {
            assert!(start <= self.out_len);
            assert!(self.idx <= end);

            if !interior {
                for i in start..self.out_len {
                    self.out_info_mut()[i].mask |= mask;
                }

                for i in self.idx..end {
                    self.info[i].mask |= mask;
                }
            } else {
                let mut cluster = Self::_infos_find_min_cluster(&self.info, self.idx, end, None);
                cluster = Self::_infos_find_min_cluster(
                    &self.out_info(),
                    start,
                    self.out_len,
                    Some(cluster),
                );

                let out_len = self.out_len;
                let first = Self::_infos_set_glyph_flags(
                    &mut self.out_info_mut(),
                    start,
                    out_len,
                    cluster,
                    mask,
                );
                let second =
                    Self::_infos_set_glyph_flags(&mut self.info, self.idx, end, cluster, mask);

                if first || second {
                    self.scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_GLYPH_FLAGS;
                }
            }
        }
    }

    pub fn unsafe_to_concat(&mut self, start: Option<usize>, end: Option<usize>) {
        if !self.flags.contains(BufferFlags::PRODUCE_UNSAFE_TO_CONCAT) {
            return;
        }

        self._set_glyph_flags(UNSAFE_TO_CONCAT, start, end, Some(true), None);
    }

    pub fn unsafe_to_break_from_outbuffer(&mut self, start: Option<usize>, end: Option<usize>) {
        if !self.flags.contains(BufferFlags::PRODUCE_UNSAFE_TO_CONCAT) {
            return;
        }

        self._set_glyph_flags(
            UNSAFE_TO_BREAK | UNSAFE_TO_CONCAT,
            start,
            end,
            Some(true),
            Some(true),
        );
    }

    pub fn unsafe_to_concat_from_outbuffer(&mut self, start: Option<usize>, end: Option<usize>) {
        self._set_glyph_flags(UNSAFE_TO_CONCAT, start, end, Some(false), Some(true));
    }

    pub fn move_to(&mut self, i: usize) -> bool {
        if !self.have_output {
            assert!(i <= self.len);
            self.idx = i;
            return true;
        }

        if !self.successful {
            return false;
        }

        assert!(i <= self.out_len + (self.len - self.idx));

        if self.out_len < i {
            let count = i - self.out_len;
            if !self.make_room_for(count, count) {
                return false;
            }

            for j in 0..count {
                self.set_out_info(self.out_len + j, self.info[self.idx + j]);
            }

            self.idx += count;
            self.out_len += count;
        } else if self.out_len > i {
            // Tricky part: rewinding...
            let count = self.out_len - i;

            // This will blow in our face if memory allocation fails later
            // in this same lookup...
            //
            // We used to shift with extra 32 items.
            // But that would leave empty slots in the buffer in case of allocation
            // failures.  See comments in shift_forward().  This can cause O(N^2)
            // behavior more severely than adding 32 empty slots can...
            if self.idx < count {
                self.shift_forward(count - self.idx);
            }

            assert!(self.idx >= count);

            self.idx -= count;
            self.out_len -= count;

            for j in 0..count {
                self.info[self.idx + j] = self.out_info()[self.out_len + j];
            }
        }

        true
    }

    pub fn ensure(&mut self, size: usize) -> bool {
        if size < self.len {
            return true;
        }

        if size > self.max_len {
            self.successful = false;
            return false;
        }

        self.info.resize(size, hb_glyph_info_t::default());
        self.pos.resize(size, GlyphPosition::default());
        true
    }

    pub fn set_len(&mut self, len: usize) {
        self.ensure(len);
        self.len = len;
    }

    fn make_room_for(&mut self, num_in: usize, num_out: usize) -> bool {
        if !self.ensure(self.out_len + num_out) {
            return false;
        }

        if !self.have_separate_output && self.out_len + num_out > self.idx + num_in {
            assert!(self.have_output);

            self.have_separate_output = true;
            for i in 0..self.out_len {
                self.set_out_info(i, self.info[i]);
            }
        }

        true
    }

    fn shift_forward(&mut self, count: usize) {
        assert!(self.have_output);
        self.ensure(self.len + count);

        for i in (0..(self.len - self.idx)).rev() {
            self.info[self.idx + count + i] = self.info[self.idx + i];
        }

        if self.idx + count > self.len {
            for info in &mut self.info[self.len..self.idx + count] {
                *info = hb_glyph_info_t::default();
            }
        }

        self.len += count;
        self.idx += count;
    }

    fn clear_context(&mut self, side: usize) {
        self.context_len[side] = 0;
    }

    pub fn sort(
        &mut self,
        start: usize,
        end: usize,
        cmp: impl Fn(&hb_glyph_info_t, &hb_glyph_info_t) -> bool,
    ) {
        assert!(!self.have_positions);

        for i in start + 1..end {
            let mut j = i;
            while j > start && cmp(&self.info[j - 1], &self.info[i]) {
                j -= 1;
            }

            if i == j {
                continue;
            }

            // Move item i to occupy place for item j, shift what's in between.
            self.merge_clusters(j, i + 1);

            {
                let t = self.info[i];
                for idx in (0..i - j).rev() {
                    self.info[idx + j + 1] = self.info[idx + j];
                }

                self.info[j] = t;
            }
        }
    }

    pub fn set_cluster(info: &mut hb_glyph_info_t, cluster: u32, mask: hb_mask_t) {
        if info.cluster != cluster {
            info.mask = (info.mask & !glyph_flag::DEFINED) | (mask & glyph_flag::DEFINED);
        }

        info.cluster = cluster;
    }

    // Called around shape()
    pub(crate) fn enter(&mut self) {
        self.serial = 0;
        self.scratch_flags = HB_BUFFER_SCRATCH_FLAG_DEFAULT;

        if let Some(len) = self.len.checked_mul(hb_buffer_t::MAX_LEN_FACTOR) {
            self.max_len = len.max(hb_buffer_t::MAX_LEN_MIN);
        }

        if let Ok(len) = i32::try_from(self.len) {
            if let Some(ops) = len.checked_mul(hb_buffer_t::MAX_OPS_FACTOR) {
                self.max_ops = ops.max(hb_buffer_t::MAX_OPS_MIN);
            }
        }
    }

    // Called around shape()
    pub(crate) fn leave(&mut self) {
        self.max_len = hb_buffer_t::MAX_LEN_DEFAULT;
        self.max_ops = hb_buffer_t::MAX_OPS_DEFAULT;
        self.serial = 0;
    }

    fn _infos_find_min_cluster(
        info: &[hb_glyph_info_t],
        start: usize,
        end: usize,
        cluster: Option<u32>,
    ) -> u32 {
        let mut cluster = cluster.unwrap_or(core::u32::MAX);

        for glyph_info in &info[start..end] {
            cluster = core::cmp::min(cluster, glyph_info.cluster);
        }

        cluster
    }

    #[must_use]
    fn _infos_set_glyph_flags(
        info: &mut [hb_glyph_info_t],
        start: usize,
        end: usize,
        cluster: u32,
        mask: hb_mask_t,
    ) -> bool {
        // NOTE: Because of problems with ownership, we don't pass the scratch flags to this
        // function, unlike in harfbuzz. Because of this, each time you call this function you
        // the caller needs to set the "BufferScratchFlags::HAS_GLYPH_FLAGS" scratch flag
        // themselves if the function returns true.
        let mut unsafe_to_break = false;
        for glyph_info in &mut info[start..end] {
            if glyph_info.cluster != cluster {
                glyph_info.mask |= mask;
                unsafe_to_break = true;
            }
        }

        unsafe_to_break
    }

    /// Checks that buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn push_str(&mut self, text: &str) {
        self.ensure(self.len + text.chars().count());

        for (i, c) in text.char_indices() {
            self.add(c as u32, i as u32);
        }
    }

    fn set_pre_context(&mut self, text: &str) {
        self.clear_context(0);
        for (i, c) in text.chars().rev().enumerate().take(CONTEXT_LENGTH) {
            self.context[0][i] = c;
            self.context_len[0] += 1;
        }
    }

    fn set_post_context(&mut self, text: &str) {
        self.clear_context(1);
        for (i, c) in text.chars().enumerate().take(CONTEXT_LENGTH) {
            self.context[1][i] = c;
            self.context_len[1] += 1;
        }
    }

    pub fn next_syllable(&self, mut start: usize) -> usize {
        if start >= self.len {
            return start;
        }

        let syllable = self.info[start].syllable();
        start += 1;
        while start < self.len && syllable == self.info[start].syllable() {
            start += 1;
        }

        start
    }

    #[inline]
    pub fn allocate_lig_id(&mut self) -> u8 {
        let mut lig_id = self.next_serial() & 0x07;

        if lig_id == 0 {
            lig_id = self.allocate_lig_id();
        }

        lig_id
    }
}

pub(crate) fn _cluster_group_func(a: &hb_glyph_info_t, b: &hb_glyph_info_t) -> bool {
    a.cluster == b.cluster
}

// TODO: to iter if possible

macro_rules! foreach_cluster {
    ($buffer:expr, $start:ident, $end:ident, $($body:tt)*) => {
        foreach_group!($buffer, $start, $end, crate::hb::buffer::_cluster_group_func, $($body)*)
    };
}

macro_rules! foreach_group {
    ($buffer:expr, $start:ident, $end:ident, $group_func:expr, $($body:tt)*) => {{
        let count = $buffer.len;
        let mut $start = 0;
        let mut $end = if count > 0 { $buffer.group_end(0, $group_func) } else { 0 };

        while $start < count {
            $($body)*;
            $start = $end;
            $end = $buffer.group_end($start, $group_func);
        }
    }};
}

macro_rules! foreach_syllable {
    ($buffer:expr, $start:ident, $end:ident, $($body:tt)*) => {{
        let mut $start = 0;
        let mut $end = $buffer.next_syllable(0);
        while $start < $buffer.len {
            $($body)*;
            $start = $end;
            $end = $buffer.next_syllable($start);
        }
    }};
}

macro_rules! foreach_grapheme {
    ($buffer:expr, $start:ident, $end:ident, $($body:tt)*) => {
        foreach_group!($buffer, $start, $end, crate::hb::ot_layout::_hb_grapheme_group_func, $($body)*)
    };
}

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy)]
    pub struct UnicodeProps: u16 {
        const GENERAL_CATEGORY  = 0x001F;
        const IGNORABLE         = 0x0020;
        // MONGOLIAN FREE VARIATION SELECTOR 1..4, or TAG characters
        const HIDDEN            = 0x0040;
        const CONTINUATION      = 0x0080;

        // If GEN_CAT=FORMAT, top byte masks:
        const CF_ZWJ            = 0x0100;
        const CF_ZWNJ           = 0x0200;
    }
}

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy)]
    pub struct GlyphPropsFlags: u16 {
        // The following three match LookupFlags::Ignore* numbers.
        const BASE_GLYPH    = 0x02;
        const LIGATURE      = 0x04;
        const MARK          = 0x08;
        const CLASS_MASK    = Self::BASE_GLYPH.bits() | Self::LIGATURE.bits() | Self::MARK.bits();

        // The following are used internally; not derived from GDEF.
        const SUBSTITUTED   = 0x10;
        const LIGATED       = 0x20;
        const MULTIPLIED    = 0x40;

        const PRESERVE      = Self::SUBSTITUTED.bits() | Self::LIGATED.bits() | Self::MULTIPLIED.bits();
    }
}

pub type hb_buffer_scratch_flags_t = u32;
pub const HB_BUFFER_SCRATCH_FLAG_DEFAULT: u32 = 0x00000000;
pub const HB_BUFFER_SCRATCH_FLAG_HAS_NON_ASCII: u32 = 0x00000001;
pub const HB_BUFFER_SCRATCH_FLAG_HAS_DEFAULT_IGNORABLES: u32 = 0x00000002;
pub const HB_BUFFER_SCRATCH_FLAG_HAS_SPACE_FALLBACK: u32 = 0x00000004;
pub const HB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT: u32 = 0x00000008;
pub const HB_BUFFER_SCRATCH_FLAG_HAS_CGJ: u32 = 0x00000010;
pub const HB_BUFFER_SCRATCH_FLAG_HAS_GLYPH_FLAGS: u32 = 0x00000020;

/* Reserved for complex shapers' internal use. */
pub const HB_BUFFER_SCRATCH_FLAG_COMPLEX0: u32 = 0x01000000;
// pub const HB_BUFFER_SCRATCH_FLAG_COMPLEX1: u32 = 0x02000000;
// pub const HB_BUFFER_SCRATCH_FLAG_COMPLEX2: u32 = 0x04000000;
// pub const HB_BUFFER_SCRATCH_FLAG_COMPLEX3: u32 = 0x08000000;

/// A buffer that contains an input string ready for shaping.
pub struct UnicodeBuffer(pub(crate) hb_buffer_t);

impl UnicodeBuffer {
    /// Create a new `UnicodeBuffer`.
    #[inline]
    pub fn new() -> UnicodeBuffer {
        UnicodeBuffer(hb_buffer_t::new())
    }

    /// Returns the length of the data of the buffer.
    ///
    /// This corresponds to the number of unicode codepoints contained in the
    /// buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len
    }

    /// Returns `true` if the buffer contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Pushes a string to a buffer.
    #[inline]
    pub fn push_str(&mut self, str: &str) {
        self.0.push_str(str);
    }

    /// Sets the pre-context for this buffer.
    #[inline]
    pub fn set_pre_context(&mut self, str: &str) {
        self.0.set_pre_context(str)
    }

    /// Sets the post-context for this buffer.
    #[inline]
    pub fn set_post_context(&mut self, str: &str) {
        self.0.set_post_context(str)
    }

    /// Appends a character to a buffer with the given cluster value.
    #[inline]
    pub fn add(&mut self, codepoint: char, cluster: u32) {
        self.0.add(codepoint as u32, cluster);
        self.0.context_len[1] = 0;
    }

    /// Set the text direction of the `Buffer`'s contents.
    #[inline]
    pub fn set_direction(&mut self, direction: Direction) {
        self.0.direction = direction;
    }

    /// Returns the `Buffer`'s text direction.
    #[inline]
    pub fn direction(&self) -> Direction {
        self.0.direction
    }

    /// Set the script from an ISO15924 tag.
    #[inline]
    pub fn set_script(&mut self, script: Script) {
        self.0.script = Some(script);
    }

    /// Get the ISO15924 script tag.
    pub fn script(&self) -> Script {
        self.0.script.unwrap_or(script::UNKNOWN)
    }

    /// Set the buffer language.
    #[inline]
    pub fn set_language(&mut self, lang: Language) {
        self.0.language = Some(lang);
    }

    /// Get the buffer language.
    #[inline]
    pub fn language(&self) -> Option<Language> {
        self.0.language.clone()
    }

    /// Guess the segment properties (direction, language, script) for the
    /// current buffer.
    #[inline]
    pub fn guess_segment_properties(&mut self) {
        self.0.guess_segment_properties()
    }

    /// Set the flags for this buffer.
    #[inline]
    pub fn set_flags(&mut self, flags: BufferFlags) {
        self.0.flags = flags;
    }

    /// Get the flags for this buffer.
    #[inline]
    pub fn flags(&self) -> BufferFlags {
        self.0.flags
    }

    /// Set the cluster level of the buffer.
    #[inline]
    pub fn set_cluster_level(&mut self, cluster_level: BufferClusterLevel) {
        self.0.cluster_level = match cluster_level {
            BufferClusterLevel::MonotoneGraphemes => HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES,
            BufferClusterLevel::MonotoneCharacters => HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS,
            BufferClusterLevel::Characters => HB_BUFFER_CLUSTER_LEVEL_CHARACTERS,
        }
    }

    /// Retrieve the cluster level of the buffer.
    #[inline]
    pub fn cluster_level(&self) -> BufferClusterLevel {
        match self.0.cluster_level {
            HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES => BufferClusterLevel::MonotoneGraphemes,
            HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS => BufferClusterLevel::MonotoneCharacters,
            HB_BUFFER_CLUSTER_LEVEL_CHARACTERS => BufferClusterLevel::Characters,
            _ => BufferClusterLevel::MonotoneGraphemes,
        }
    }

    /// Resets clusters.
    #[inline]
    pub fn reset_clusters(&mut self) {
        self.0.reset_clusters();
    }

    /// Clear the contents of the buffer.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl core::fmt::Debug for UnicodeBuffer {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.debug_struct("UnicodeBuffer")
            .field("direction", &self.direction())
            .field("language", &self.language())
            .field("script", &self.script())
            .field("cluster_level", &self.cluster_level())
            .finish()
    }
}

impl Default for UnicodeBuffer {
    fn default() -> UnicodeBuffer {
        UnicodeBuffer::new()
    }
}

/// A buffer that contains the results of the shaping process.
pub struct GlyphBuffer(pub(crate) hb_buffer_t);

impl GlyphBuffer {
    /// Returns the length of the data of the buffer.
    ///
    /// When called before shaping this is the number of unicode codepoints
    /// contained in the buffer. When called after shaping it returns the number
    /// of glyphs stored.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len
    }

    /// Returns `true` if the buffer contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the glyph infos.
    #[inline]
    pub fn glyph_infos(&self) -> &[hb_glyph_info_t] {
        &self.0.info[0..self.0.len]
    }

    /// Get the glyph positions.
    #[inline]
    pub fn glyph_positions(&self) -> &[GlyphPosition] {
        &self.0.pos[0..self.0.len]
    }

    /// Clears the content of the glyph buffer and returns an empty
    /// `UnicodeBuffer` reusing the existing allocation.
    #[inline]
    pub fn clear(mut self) -> UnicodeBuffer {
        self.0.clear();
        UnicodeBuffer(self.0)
    }

    /// Converts the glyph buffer content into a string.
    pub fn serialize(&self, face: &hb_font_t, flags: SerializeFlags) -> String {
        self.serialize_impl(face, flags).unwrap_or_default()
    }

    fn serialize_impl(
        &self,
        face: &hb_font_t,
        flags: SerializeFlags,
    ) -> Result<String, core::fmt::Error> {
        use core::fmt::Write;

        let mut s = String::with_capacity(64);

        let info = self.glyph_infos();
        let pos = self.glyph_positions();
        let mut x = 0;
        let mut y = 0;
        for (info, pos) in info.iter().zip(pos) {
            if !flags.contains(SerializeFlags::NO_GLYPH_NAMES) {
                match face.glyph_name(info.as_glyph()) {
                    Some(name) => s.push_str(name),
                    None => write!(&mut s, "gid{}", info.glyph_id)?,
                }
            } else {
                write!(&mut s, "{}", info.glyph_id)?;
            }

            if !flags.contains(SerializeFlags::NO_CLUSTERS) {
                write!(&mut s, "={}", info.cluster)?;
            }

            if !flags.contains(SerializeFlags::NO_POSITIONS) {
                if x + pos.x_offset != 0 || y + pos.y_offset != 0 {
                    write!(&mut s, "@{},{}", x + pos.x_offset, y + pos.y_offset)?;
                }

                if !flags.contains(SerializeFlags::NO_ADVANCES) {
                    write!(&mut s, "+{}", pos.x_advance)?;
                    if pos.y_advance != 0 {
                        write!(&mut s, ",{}", pos.y_advance)?;
                    }
                }
            }

            if flags.contains(SerializeFlags::GLYPH_FLAGS) {
                if info.mask & glyph_flag::DEFINED != 0 {
                    write!(&mut s, "#{:X}", info.mask & glyph_flag::DEFINED)?;
                }
            }

            if flags.contains(SerializeFlags::GLYPH_EXTENTS) {
                let mut extents = GlyphExtents::default();
                face.glyph_extents(info.as_glyph(), &mut extents);
                write!(
                    &mut s,
                    "<{},{},{},{}>",
                    extents.x_bearing, extents.y_bearing, extents.width, extents.height
                )?;
            }

            if flags.contains(SerializeFlags::NO_ADVANCES) {
                x += pos.x_advance;
                y += pos.y_advance;
            }

            s.push('|');
        }

        // Remove last `|`.
        if !s.is_empty() {
            s.pop();
        }

        Ok(s)
    }
}

impl core::fmt::Debug for GlyphBuffer {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.debug_struct("GlyphBuffer")
            .field("glyph_positions", &self.glyph_positions())
            .field("glyph_infos", &self.glyph_infos())
            .finish()
    }
}
