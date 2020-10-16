use std::convert::TryFrom;
use std::fmt;
use std::mem;

use crate::{script, Font, Mask};
use crate::common::{Direction, Language, Script};
use crate::ffi;
use crate::unicode::{CharExt, GeneralCategory, GeneralCategoryExt, Space};

const CONTEXT_LENGTH: usize = 5;

pub(crate) mod glyph_flag {
    /// Indicates that if input text is broken at the
    /// beginning of the cluster this glyph is part of,
    /// then both sides need to be re-shaped, as the
    /// result might be different.  On the flip side,
    /// it means that when this flag is not present,
    /// then it's safe to break the glyph-run at the
    /// beginning of this cluster, and the two sides
    /// represent the exact same result one would get
    /// if breaking input text at the beginning of
    /// this cluster and shaping the two sides
    /// separately.  This can be used to optimize
    /// paragraph layout, by avoiding re-shaping
    /// of each line after line-breaking, or limiting
    /// the reshaping to a small piece around the
    /// breaking point only.
    #[allow(dead_code)]
    pub const UNSAFE_TO_BREAK: u32 = 0x00000001;

    /// All the currently defined flags.
    pub const DEFINED: u32 = 0x00000001; // OR of all defined flags
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


/// A glyph info.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct GlyphInfo {
    /// A selected glyph.
    pub codepoint: u32,
    pub(crate) mask: ffi::rb_mask_t,
    /// An original cluster index.
    pub cluster: u32,
    pub(crate) var1: u32,
    pub(crate) var2: u32,
}

impl GlyphInfo {
    #[inline]
    pub(crate) fn as_char(&self) -> char {
        char::try_from(self.codepoint).unwrap()
    }

    #[inline]
    fn glyph_props(&self) -> u16 {
        unsafe {
            let v: ffi::rb_var_int_t = std::mem::transmute(self.var1);
            v.var_u16[0]
        }
    }

    #[inline]
    fn set_glyph_props(&mut self, n: u16) {
        unsafe {
            let v: &mut ffi::rb_var_int_t = std::mem::transmute(&mut self.var1);
            v.var_u16[0] = n;
        }
    }

    #[inline]
    fn unicode_props(&self) -> u16 {
        unsafe {
            let v: ffi::rb_var_int_t = std::mem::transmute(self.var2);
            v.var_u16[0]
        }
    }

    #[inline]
    fn set_unicode_props(&mut self, n: u16) {
        unsafe {
            let v: &mut ffi::rb_var_int_t = std::mem::transmute(&mut self.var2);
            v.var_u16[0] = n;
        }
    }

    #[inline]
    pub(crate) fn lig_props(&self) -> u8 {
        unsafe {
            let v: ffi::rb_var_int_t = std::mem::transmute(self.var1);
            v.var_u8[2]
        }
    }

    #[inline]
    pub(crate) fn general_category(&self) -> GeneralCategory {
        let n = self.unicode_props() & UnicodeProps::GENERAL_CATEGORY.bits;
        GeneralCategory::from_rb(n as u32)
    }

    #[inline]
    pub(crate) fn set_general_category(&mut self, gc: GeneralCategory) {
        let gc = gc.to_rb();
        let n = (gc as u16) | (self.unicode_props() & (0xFF & !UnicodeProps::GENERAL_CATEGORY.bits));
        self.set_unicode_props(n);
    }

    #[inline]
    pub(crate) fn space_fallback(&self) -> Option<Space> {
        if self.general_category() == GeneralCategory::SpaceSeparator {
            unsafe {
                let n = (self.unicode_props() >> 8) as u8;
                Some(std::mem::transmute(n))
            }
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn is_default_ignorable(&self) -> bool {
        let n = self.unicode_props() & UnicodeProps::IGNORABLE.bits;
        n != 0 && !self.is_ligated()
    }

    #[inline]
    pub(crate) fn is_ligated(&self) -> bool {
        self.glyph_props() & GlyphPropsFlags::LIGATED.bits != 0
    }

    #[inline]
    pub(crate) fn is_unicode_mark(&self) -> bool {
        self.general_category().is_mark()
    }

    #[inline]
    pub(crate) fn modified_combining_class(&self) -> u8 {
        if self.is_unicode_mark() {
            (self.unicode_props() >> 8) as u8
        } else {
            0
        }
    }

    #[inline]
    pub(crate) fn set_modified_combining_class(&mut self, mcc: u8) {
        if !self.is_unicode_mark() {
            return;
        }

        let n = ((mcc as u16) << 8) | (self.unicode_props() & 0xFF);
        self.set_unicode_props(n);
    }

    #[inline]
    pub(crate) fn is_multiplied(&self) -> bool {
        self.glyph_props() & GlyphPropsFlags::MULTIPLIED.bits != 0
    }

    #[inline]
    pub(crate) fn clear_ligated_and_multiplied(&mut self) {
        let mut n = self.glyph_props();
        n &= !(GlyphPropsFlags::LIGATED | GlyphPropsFlags::MULTIPLIED).bits;
        self.set_glyph_props(n);
    }

    #[inline]
    pub(crate) fn is_substituted(&self) -> bool {
        self.glyph_props() & GlyphPropsFlags::SUBSTITUTED.bits != 0
    }

    #[inline]
    pub(crate) fn is_ligated_and_didnt_multiply(&self) -> bool {
        self.is_ligated() && !self.is_multiplied()
    }

    #[inline]
    pub(crate) fn lig_id(&self) -> u8 {
        self.lig_props() >> 5
    }

    #[inline]
    pub(crate) fn is_ligated_internal(&self) -> bool {
        const IS_LIG_BASE: u8 = 0x10;
        self.lig_props() & IS_LIG_BASE != 0
    }

    #[inline]
    pub(crate) fn lig_comp(&self) -> u8 {
        if self.is_ligated_internal() {
            0
        } else {
            self.lig_props() & 0x0F
        }
    }

    #[inline]
    pub(crate) fn lig_num_comps(&self) -> u8 {
        if self.glyph_props() & GlyphPropsFlags::LIGATURE.bits != 0 && self.is_ligated_internal() {
            self.lig_props() & 0x0F
        } else {
            1
        }
    }

    #[inline]
    pub(crate) fn set_continuation(&mut self) {
        let mut n = self.unicode_props();
        n |= UnicodeProps::CONTINUATION.bits;
        self.set_unicode_props(n);
    }

    #[inline]
    pub(crate) fn reset_continuation(&mut self) {
        let mut n = self.unicode_props();
        n &= !UnicodeProps::CONTINUATION.bits;
        self.set_unicode_props(n);
    }

    #[inline]
    pub(crate) fn syllable(&self) -> u8 {
        unsafe {
            let v: &ffi::rb_var_int_t = std::mem::transmute(&self.var1);
            v.var_u8[3]
        }
    }

    #[inline]
    pub(crate) fn set_syllable(&mut self, n: u8) {
        unsafe {
            let v: &mut ffi::rb_var_int_t = std::mem::transmute(&mut self.var1);
            v.var_u8[3] = n;
        }
    }
}


/// A cluster level.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BufferClusterLevel {
    MonotoneGraphemes,
    MonotoneCharacters,
    Characters,
}

impl Default for BufferClusterLevel {
    #[inline]
    fn default() -> Self {
        BufferClusterLevel::MonotoneGraphemes
    }
}


pub(crate) struct Buffer {
    // Information about how the text in the buffer should be treated.
    pub(crate) flags: BufferFlags,
    pub(crate) cluster_level: BufferClusterLevel,
    invisible: Option<char>,
    pub(crate) scratch_flags: BufferScratchFlags,
    // Maximum allowed len.
    max_len: u32,
    /// Maximum allowed operations.
    pub(crate) max_ops: i32,

    // Buffer contents.
    pub(crate) direction: Direction,
    pub(crate) script: Option<Script>,
    pub(crate) language: Option<Language>,

    /// Allocations successful.
    successful: bool,
    /// Whether we have an output buffer going on.
    have_output: bool,
    have_separate_output: bool,
    /// Whether we have positions
    have_positions: bool,

    pub(crate) idx: usize,
    pub(crate) len: usize,
    pub(crate) out_len: usize,

    pub(crate) info: Vec<GlyphInfo>,
    pub(crate) pos: Vec<GlyphPosition>,

    serial: u32,

    // Text before / after the main buffer contents.
    // Always in Unicode, and ordered outward.
    // Index 0 is for "pre-context", 1 for "post-context".
    pub(crate) context: [[char; CONTEXT_LENGTH]; 2],
    pub(crate) context_len: [usize; 2],
}

impl Buffer {
    /// Creates a new `Buffer`.
    pub fn new() -> Self {
        Buffer {
            flags: BufferFlags::empty(),
            cluster_level: BufferClusterLevel::default(),
            invisible: None,
            scratch_flags: BufferScratchFlags::default(),
            max_len: 0x3FFFFFFF,
            max_ops: 0x1FFFFFFF,
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
            context: [['\0', '\0', '\0', '\0', '\0'], ['\0', '\0', '\0', '\0', '\0']],
            context_len: [0, 0],
        }
    }

    #[inline]
    fn from_ptr(buffer: *const ffi::rb_buffer_t) -> &'static Buffer {
        unsafe { &*(buffer as *const Buffer) }
    }

    #[inline]
    pub(crate) fn from_ptr_mut(buffer: *mut ffi::rb_buffer_t) -> &'static mut Buffer {
        unsafe { &mut *(buffer as *mut Buffer) }
    }

    #[inline]
    pub(crate) fn as_ptr(&mut self) -> *mut ffi::rb_buffer_t {
        self as *mut _ as *mut ffi::rb_buffer_t
    }

    #[inline]
    pub fn info_slice(&self) -> &[GlyphInfo] {
        &self.info[..self.len]
    }

    #[inline]
    pub fn info_slice_mut(&mut self) -> &mut [GlyphInfo] {
        &mut self.info[..self.len]
    }

    #[inline]
    pub(crate) fn out_info(&self) -> &[GlyphInfo] {
        if self.have_separate_output {
            unsafe { mem::transmute(self.pos.as_slice()) }
        } else {
            &self.info
        }
    }

    #[inline]
    pub(crate) fn out_info_mut(&mut self) -> &mut [GlyphInfo] {
        if self.have_separate_output {
            unsafe { mem::transmute(self.pos.as_mut_slice()) }
        } else {
            &mut self.info
        }
    }

    #[inline]
    fn set_out_info(&mut self, i: usize, info: GlyphInfo) {
        self.out_info_mut()[i] = info;
    }

    #[inline]
    pub(crate) fn cur(&self, i: usize) -> &GlyphInfo {
        &self.info[self.idx + i]
    }

    #[inline]
    pub(crate) fn cur_mut(&mut self, i: usize) -> &mut GlyphInfo {
        let idx = self.idx + i;
        &mut self.info[idx]
    }

    #[inline]
    fn cur_pos_mut(&mut self) -> &mut GlyphPosition {
        let i = self.idx;
        &mut self.pos[i]
    }

    #[inline]
    fn prev_mut(&mut self) -> &mut GlyphInfo {
        let idx = if self.out_len != 0 { self.out_len - 1 } else { 0 };
        &mut self.out_info_mut()[idx]
    }

    fn clear(&mut self) {
        self.direction = Direction::Invalid;
        self.script = None;
        self.language = None;
        self.scratch_flags = BufferScratchFlags::default();

        self.successful = true;
        self.have_output = false;
        self.have_positions = false;

        self.idx = 0;
        self.info.clear();
        self.pos.clear();
        self.len = 0;
        self.out_len = 0;
        self.have_separate_output = false;

        self.serial = 0;

        self.context = [['\0', '\0', '\0', '\0', '\0'], ['\0', '\0', '\0', '\0', '\0']];
        self.context_len = [0, 0];
    }

    #[inline]
    fn backtrack_len(&self) -> usize {
        if self.have_output { self.out_len } else { self.idx }
    }

    #[inline]
    fn lookahead_len(&self) -> usize {
        self.len - self.idx
    }

    #[inline]
    fn next_serial(&mut self) -> u32 {
        self.serial += 1;
        self.serial
    }

    fn add(&mut self, codepoint: u32, cluster: u32) {
        self.ensure(self.len + 1);

        let i = self.len;
        self.info[i] = GlyphInfo {
            codepoint,
            mask: 0,
            cluster,
            var1: 0,
            var2: 0,
        };

        self.len += 1;
    }

    #[inline]
    fn reverse(&mut self) {
        if self.is_empty() {
            return;
        }

        self.reverse_range(0, self.len);
    }

    fn reverse_range(&mut self, start: usize, end: usize) {
        if end - start < 2 {
            return;
        }

        let mut i = start;
        let mut j = end - 1;
        while i < j {
            self.info.swap(i, j);
            i += 1;
            j -= 1;
        }

        if self.have_positions {
            i = start;
            j = end - 1;
            while i < j {
                self.pos.swap(i, j);
                i += 1;
                j -= 1;
            }
        }
    }

    #[inline]
    fn reset_clusters(&mut self) {
        for (i, info) in self.info.iter_mut().enumerate() {
            info.cluster = i as u32;
        }
    }

    fn guess_segment_properties(&mut self) {
        if self.script.is_none() {
            for info in &self.info {
                let c = char::try_from(info.codepoint).unwrap();
                match c.script() {
                      crate::script::COMMON
                    | crate::script::INHERITED
                    | crate::script::UNKNOWN => {}
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

    pub(crate) fn swap_buffers(&mut self) {
        if !self.successful {
            return;
        }

        assert!(self.have_output);
        self.have_output = false;

        if self.have_separate_output {
            unsafe {
                mem::swap(&mut self.info, mem::transmute(&mut self.pos));
            }
        }

        mem::swap(&mut self.len, &mut self.out_len);

        self.idx = 0;
    }

    fn remove_output(&mut self) {
        self.have_output = false;
        self.have_positions = false;

        self.out_len = 0;
        self.have_separate_output = false;
    }

    pub(crate) fn clear_output(&mut self) {
        self.have_output = true;
        self.have_positions = false;

        self.out_len = 0;
        self.have_separate_output = false;
    }

    fn clear_positions(&mut self) {
        self.have_output = false;
        self.have_positions = true;

        self.out_len = 0;
        self.have_separate_output = false;

        for pos in &mut self.pos {
            *pos = GlyphPosition::default();
        }
    }

    pub(crate) fn replace_glyphs(&mut self, num_in: usize, num_out: usize, glyph_data: &[u32]) {
        if !self.make_room_for(num_in, num_out) {
            return;
        }

        assert!(self.idx + num_in <= self.len);

        self.merge_clusters(self.idx, self.idx + num_in);

        let orig_info = self.info[self.idx];
        for i in 0..num_out {
            let ii = self.out_len + i;
            self.set_out_info(ii, orig_info);
            self.out_info_mut()[ii].codepoint = glyph_data[i];
        }

        self.idx += num_in;
        self.out_len += num_out;
    }

    pub(crate) fn replace_glyph(&mut self, glyph_index: u32) {
        if self.have_separate_output || self.out_len != self.idx {
            if !self.make_room_for(1, 1) {
                return;
            }

            self.set_out_info(self.out_len, self.info[self.idx]);
        }

        let out_len = self.out_len;
        self.out_info_mut()[out_len].codepoint = glyph_index;

        self.idx += 1;
        self.out_len += 1;
    }

    pub(crate) fn output_glyph(&mut self, glyph_index: u32) {
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

        self.out_info_mut()[out_len].codepoint = glyph_index;

        self.out_len += 1;
    }

    pub(crate) fn output_info(&mut self, glyph_info: GlyphInfo) {
        if !self.make_room_for(0, 1) {
            return;
        }

        self.set_out_info(self.out_len, glyph_info);
        self.out_len += 1;
    }

    /// Copies glyph at idx to output but doesn't advance idx.
    fn copy_glyph(&mut self) {
        if !self.make_room_for(0, 1) {
            return;
        }

        self.set_out_info(self.out_len, self.info[self.idx]);
        self.out_len += 1;
    }

    /// Copies glyph at idx to output and advance idx.
    ///
    /// If there's no output, just advance idx.
    pub(crate) fn next_glyph(&mut self) {
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
    fn next_glyphs(&mut self, n: usize) {
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
    fn skip_glyph(&mut self) {
        self.idx += 1;
    }

    fn reset_masks(&mut self, mask: Mask) {
        for info in &mut self.info[..self.len] {
            info.mask = mask;
        }
    }

    fn set_masks(
        &mut self,
        mut value: Mask,
        mask: Mask,
        cluster_start: u32,
        cluster_end: u32,
    ) {
        let not_mask = !mask;
        value &= mask;

        if mask == 0 {
            return;
        }

        if cluster_start == 0 && cluster_end == std::u32::MAX {
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

    pub(crate) fn merge_clusters(&mut self, start: usize, end: usize) {
        if end - start < 2 {
            return;
        }

        self.merge_clusters_impl(start, end)
    }

    fn merge_clusters_impl(&mut self, mut start: usize, mut end: usize) {
        if self.cluster_level == BufferClusterLevel::Characters {
            self.unsafe_to_break(start, end);
            return;
        }

        let mut cluster = self.info[start].cluster;

        for i in start+1..end {
            cluster = std::cmp::min(cluster, self.info[i].cluster);
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

    pub(crate) fn merge_out_clusters(&mut self, mut start: usize, mut end: usize) {
        if self.cluster_level == BufferClusterLevel::Characters {
            return;
        }

        if end - start < 2 {
            return;
        }

        let mut cluster = self.out_info()[start].cluster;

        for i in start+1..end {
            cluster = std::cmp::min(cluster, self.out_info()[i].cluster);
        }

        // Extend start
        while start != 0 && self.out_info()[start - 1].cluster == self.out_info()[start].cluster {
            start -= 1;
        }

        // Extend end
        while end < self.out_len && self.out_info()[end - 1].cluster == self.out_info()[end].cluster {
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
    fn delete_glyph(&mut self) {
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

    pub(crate) fn unsafe_to_break(&mut self, start: usize, end: usize) {
        if end - start < 2 {
            return;
        }

        self.unsafe_to_break_impl(start, end);
    }

    fn unsafe_to_break_impl(&mut self, start: usize, end: usize) {
        let mut cluster = std::u32::MAX;
        cluster = Self::_unsafe_to_break_find_min_cluster(&self.info, start, end, cluster);
        let unsafe_to_break = Self::_unsafe_to_break_set_mask(&mut self.info, start, end, cluster);
        if unsafe_to_break {
            self.scratch_flags |= BufferScratchFlags::HAS_UNSAFE_TO_BREAK;
        }
    }

    pub(crate) fn unsafe_to_break_from_outbuffer(&mut self, start: usize, end: usize) {
        if !self.have_output {
            self.unsafe_to_break_impl(start, end);
            return;
        }

        assert!(start <= self.out_len);
        assert!(self.idx <= end);

        let mut cluster = std::u32::MAX;
        cluster = Self::_unsafe_to_break_find_min_cluster(self.out_info(), start, self.out_len, cluster);
        cluster = Self::_unsafe_to_break_find_min_cluster(&self.info, self.idx, end, cluster);
        let idx = self.idx;
        let out_len = self.out_len;
        let unsafe_to_break1 = Self::_unsafe_to_break_set_mask(self.out_info_mut(), start, out_len, cluster);
        let unsafe_to_break2 = Self::_unsafe_to_break_set_mask(&mut self.info, idx, end, cluster);

        if unsafe_to_break1 || unsafe_to_break2 {
            self.scratch_flags |= BufferScratchFlags::HAS_UNSAFE_TO_BREAK;
        }
    }

    fn move_to(&mut self, i: usize) -> bool {
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
            // We used to shift with extra 32 items, instead of the 0 below.
            // But that would leave empty slots in the buffer in case of allocation
            // failures.  Setting to zero for now to avoid other problems (see
            // comments in shift_forward().  This can cause O(N^2) behavior more
            // severely than adding 32 empty slots can...
            if self.idx < count {
                self.shift_forward(count);
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

    pub(crate) fn ensure(&mut self, size: usize) -> bool {
        if size < self.len {
            return true;
        }

        if size > self.max_len as usize {
            self.successful = false;
            return false;
        }

        self.info.resize(size, GlyphInfo::default());
        self.pos.resize(size, GlyphPosition::default());
        true
    }

    pub(crate) fn set_len(&mut self, len: usize) {
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

        for i in 0..(self.len - self.idx) {
            self.info[self.idx + count + i] = self.info[self.idx + i];
        }

        if self.idx + count > self.len {
            for info in &mut self.info[self.len..self.idx+count] {
                *info = GlyphInfo::default();
            }
        }

        self.len += count;
        self.idx += count;
    }

    pub(crate) fn sort(&mut self, start: usize, end: usize, cmp: fn(&GlyphInfo, &GlyphInfo) -> bool) {
        assert!(!self.have_positions);

        for i in start+1..end {
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
                for idx in (0..i-j).rev() {
                    self.info[idx + j + 1] = self.info[idx + j];
                }

                self.info[j] = t;
            }
        }
    }

    fn set_cluster(info: &mut GlyphInfo, cluster: u32, mask: Mask) {
        if info.cluster != cluster {
            if mask & glyph_flag::UNSAFE_TO_BREAK != 0 {
                info.mask |= glyph_flag::UNSAFE_TO_BREAK;
            } else {
                info.mask &= !glyph_flag::UNSAFE_TO_BREAK;
            }
        }

        info.cluster = cluster;
    }

    fn _unsafe_to_break_find_min_cluster(info: &[GlyphInfo], start: usize, end: usize, mut cluster: u32) -> u32 {
        for i in start..end {
            cluster = std::cmp::min(cluster, info[i].cluster);
        }

        cluster
    }

    fn _unsafe_to_break_set_mask(info: &mut [GlyphInfo], start: usize, end: usize, cluster: u32) -> bool {
        let mut unsafe_to_break = false;
        for i in start..end {
            if info[i].cluster != cluster {
                unsafe_to_break = true;
                info[i].mask |= glyph_flag::UNSAFE_TO_BREAK;
            }
        }

        unsafe_to_break
    }

    /// Checks that buffer contains no elements.
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn push_str(&mut self, text: &str) {
        self.ensure(self.len + text.chars().count());

        for (i, c) in text.char_indices() {
            self.add(c as u32, i as u32);
        }
    }

    // fn next_cluster(&self, mut start: usize) -> usize {
    //     if start >= self.len {
    //         return start;
    //     }
    //
    //     // TODO: to iter
    //     let cluster = self.info[start].cluster;
    //     start += 1;
    //     while start < self.len && cluster == self.info[start].cluster {
    //         start += 1;
    //     }
    //
    //     start
    // }

    pub(crate) fn next_syllable(&self, mut start: usize) -> usize {
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
}


bitflags::bitflags! {
    #[derive(Default)]
    pub struct UnicodeProps: u16 {
        const GENERAL_CATEGORY  = 0x001F;
        const IGNORABLE         = 0x0020;
        // MONGOLIAN FREE VARIATION SELECTOR 1..3, or TAG characters
        const HIDDEN            = 0x0040;
        const CONTINUATION      = 0x0080;

        // If GEN_CAT=FORMAT, top byte masks:
        const CF_ZWJ            = 0x0100;
        const CF_ZWNJ           = 0x0200;
    }
}


bitflags::bitflags! {
    #[derive(Default)]
    pub struct GlyphPropsFlags: u16 {
        // The following three match LookupFlags::Ignore* numbers.
        const BASE_GLYPH    = 0x02;
        const LIGATURE      = 0x04;
        const MARK          = 0x08;

        // The following are used internally; not derived from GDEF.
        const SUBSTITUTED   = 0x10;
        const LIGATED       = 0x20;
        const MULTIPLIED    = 0x40;

        const PRESERVE      = Self::SUBSTITUTED.bits | Self::LIGATED.bits | Self::MULTIPLIED.bits;
    }
}


bitflags::bitflags! {
    #[derive(Default)]
    pub struct BufferFlags: u32 {
        const BEGINNING_OF_TEXT             = 1 << 1;
        const END_OF_TEXT                   = 1 << 2;
        const PRESERVE_DEFAULT_IGNORABLES   = 1 << 3;
        const REMOVE_DEFAULT_IGNORABLES     = 1 << 4;
        const DO_NOT_INSERT_DOTTED_CIRCLE   = 1 << 5;
    }
}


bitflags::bitflags! {
    #[derive(Default)]
    pub struct BufferScratchFlags: u32 {
        const HAS_NON_ASCII             = 0x00000001;
        const HAS_DEFAULT_IGNORABLES    = 0x00000002;
        const HAS_SPACE_FALLBACK        = 0x00000004;
        const HAS_GPOS_ATTACHMENT       = 0x00000008;
        const HAS_UNSAFE_TO_BREAK       = 0x00000010;
        const HAS_CGJ                   = 0x00000020;

        // Reserved for complex shapers' internal use.
        const COMPLEX0                  = 0x01000000;
        const COMPLEX1                  = 0x02000000;
        const COMPLEX2                  = 0x04000000;
        const COMPLEX3                  = 0x08000000;
    }
}


bitflags::bitflags! {
    /// Flags used for serialization with a `BufferSerializer`.
    #[derive(Default)]
    pub struct SerializeFlags: u8 {
        /// Do not serialize glyph cluster.
        const NO_CLUSTERS       = 0b00000001;
        /// Do not serialize glyph position information.
        const NO_POSITIONS      = 0b00000010;
        /// Do no serialize glyph name.
        const NO_GLYPH_NAMES    = 0b00000100;
        /// Serialize glyph extents.
        const GLYPH_EXTENTS     = 0b00001000;
        /// Serialize glyph flags.
        const GLYPH_FLAGS       = 0b00010000;
        /// Do not serialize glyph advances, glyph offsets will reflect absolute
        /// glyph positions.
        const NO_ADVANCES       = 0b00100000;
    }
}


/// A buffer that contains an input string ready for shaping.
pub struct UnicodeBuffer(pub(crate) Buffer);

impl UnicodeBuffer {
    /// Create a new `UnicodeBuffer`.
    #[inline]
    pub fn new() -> UnicodeBuffer {
        UnicodeBuffer(Buffer::new())
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

    /// Set the cluster level of the buffer.
    #[inline]
    pub fn set_cluster_level(&mut self, cluster_level: BufferClusterLevel) {
        self.0.cluster_level = cluster_level
    }

    /// Retrieve the cluster level of the buffer.
    #[inline]
    pub fn cluster_level(&self) -> BufferClusterLevel {
        self.0.cluster_level
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

impl std::fmt::Debug for UnicodeBuffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
pub struct GlyphBuffer(pub(crate) Buffer);

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
    pub fn glyph_infos(&self) -> &[GlyphInfo] {
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
    pub fn serialize(&self, font: &Font, flags: SerializeFlags) -> String {
        use std::fmt::Write;

        let mut s = String::with_capacity(64);

        let info = self.glyph_infos();
        let pos = self.glyph_positions();
        let mut x = 0;
        let mut y = 0;
        for (info, pos) in info.iter().zip(pos) {
            if !flags.contains(SerializeFlags::NO_GLYPH_NAMES) {
                match font.glyph_name(info.codepoint) {
                    Some(name) => s.push_str(name),
                    None => write!(&mut s, "gid{}", info.codepoint).unwrap(),
                }
            } else {
                write!(&mut s, "{}", info.codepoint).unwrap();
            }

            if !flags.contains(SerializeFlags::NO_CLUSTERS) {
                write!(&mut s, "={}", info.cluster).unwrap();
            }

            if !flags.contains(SerializeFlags::NO_POSITIONS) {
                if x + pos.x_offset != 0 || y + pos.y_offset != 0 {
                    write!(&mut s, "@{},{}", x + pos.x_offset, y + pos.y_offset).unwrap();
                }

                if !flags.contains(SerializeFlags::NO_ADVANCES) {
                    write!(&mut s, "+{}", pos.x_advance).unwrap();
                    if pos.y_advance != 0 {
                        write!(&mut s, ",{}", pos.y_advance).unwrap();
                    }
                }
            }

            if flags.contains(SerializeFlags::GLYPH_FLAGS) {
                if info.mask & glyph_flag::DEFINED != 0 {
                    write!(&mut s, "#{:X}", info.mask & glyph_flag::DEFINED).unwrap();
                }
            }

            if flags.contains(SerializeFlags::GLYPH_EXTENTS) {
                let extents = font.glyph_extents(info.codepoint).unwrap_or_default();
                write!(&mut s, "<{},{},{},{}>", extents.x_bearing, extents.y_bearing, extents.width, extents.height).unwrap();
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

        s
    }
}

impl fmt::Debug for GlyphBuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("GlyphBuffer")
            .field("glyph_positions", &self.glyph_positions())
            .field("glyph_infos", &self.glyph_infos())
            .finish()
    }
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_cluster_level(buffer: *const ffi::rb_buffer_t) -> u32 {
    Buffer::from_ptr(buffer).cluster_level as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_direction(buffer: *const ffi::rb_buffer_t) -> ffi::rb_direction_t {
    Buffer::from_ptr(buffer).direction.to_raw()
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_invisible_glyph(buffer: *const ffi::rb_buffer_t) -> ffi::rb_codepoint_t {
    Buffer::from_ptr(buffer).invisible.unwrap_or('\0') as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_pre_allocate(buffer: *mut ffi::rb_buffer_t, size: u32) {
    Buffer::from_ptr_mut(buffer).ensure(size as usize);
}

#[no_mangle]
pub extern "C" fn rb_buffer_reverse(buffer: *mut ffi::rb_buffer_t) {
    Buffer::from_ptr_mut(buffer).reverse();
}

#[no_mangle]
pub extern "C" fn rb_buffer_reverse_range(buffer: *mut ffi::rb_buffer_t, start: u32, end: u32) {
    Buffer::from_ptr_mut(buffer).reverse_range(start as usize, end as usize);
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_length(buffer: *const ffi::rb_buffer_t) -> u32 {
    Buffer::from_ptr(buffer).len as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_set_length(buffer: *mut ffi::rb_buffer_t, len: u32) {
    Buffer::from_ptr_mut(buffer).set_len(len as usize);
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_cur(buffer: *mut ffi::rb_buffer_t, i: u32) -> *mut GlyphInfo {
    let buffer = Buffer::from_ptr_mut(buffer);
    buffer.cur_mut(i as usize) as *mut _ as *mut GlyphInfo
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_cur_pos(buffer: *mut ffi::rb_buffer_t) -> *mut GlyphPosition {
    let buffer = Buffer::from_ptr_mut(buffer);
    buffer.cur_pos_mut() as *mut _ as *mut GlyphPosition
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_prev(buffer: *mut ffi::rb_buffer_t) -> *mut GlyphInfo {
    let buffer = Buffer::from_ptr_mut(buffer);
    buffer.prev_mut() as *mut _ as *mut GlyphInfo
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_out_infos(buffer: *mut ffi::rb_buffer_t) -> *mut GlyphInfo {
    let buffer = Buffer::from_ptr_mut(buffer);
    buffer.out_info_mut().as_mut_ptr() as *mut _
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_backtrack_len(buffer: *const ffi::rb_buffer_t) -> u32 {
    Buffer::from_ptr(buffer).backtrack_len() as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_lookahead_len(buffer: *const ffi::rb_buffer_t) -> u32 {
    Buffer::from_ptr(buffer).lookahead_len() as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_next_serial(buffer: *mut ffi::rb_buffer_t) -> u32 {
    Buffer::from_ptr_mut(buffer).next_serial() as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_set_cluster(info: *mut GlyphInfo, cluster: u32, mask: u32) {
    let info = unsafe { &mut *(info as *mut GlyphInfo) };
    Buffer::set_cluster(info, cluster, mask)
}

#[no_mangle]
pub extern "C" fn rb_buffer_move_to(buffer: *mut ffi::rb_buffer_t, i: u32) -> bool {
    Buffer::from_ptr_mut(buffer).move_to(i as usize)
}

#[no_mangle]
pub extern "C" fn rb_buffer_swap_buffers(buffer: *mut ffi::rb_buffer_t) {
    Buffer::from_ptr_mut(buffer).swap_buffers()
}

#[no_mangle]
pub extern "C" fn rb_buffer_remove_output(buffer: *mut ffi::rb_buffer_t) {
    Buffer::from_ptr_mut(buffer).remove_output()
}

#[no_mangle]
pub extern "C" fn rb_buffer_clear_output(buffer: *mut ffi::rb_buffer_t) {
    Buffer::from_ptr_mut(buffer).clear_output()
}

#[no_mangle]
pub extern "C" fn rb_buffer_clear_positions(buffer: *mut ffi::rb_buffer_t) {
    Buffer::from_ptr_mut(buffer).clear_positions()
}

#[no_mangle]
pub extern "C" fn rb_buffer_replace_glyphs(buffer: *mut ffi::rb_buffer_t, num_in: u32, num_out: u32, glyph_data: *const u32) {
    let glyph_data = unsafe { std::slice::from_raw_parts(glyph_data as *const _, num_out as usize) };
    Buffer::from_ptr_mut(buffer).replace_glyphs(num_in as usize, num_out as usize, glyph_data);
}

#[no_mangle]
pub extern "C" fn rb_buffer_merge_clusters(buffer: *mut ffi::rb_buffer_t, start: u32, end: u32) {
    Buffer::from_ptr_mut(buffer).merge_clusters(start as usize, end as usize);
}

#[no_mangle]
pub extern "C" fn rb_buffer_merge_out_clusters(buffer: *mut ffi::rb_buffer_t, start: u32, end: u32) {
    Buffer::from_ptr_mut(buffer).merge_out_clusters(start as usize, end as usize);
}

#[no_mangle]
pub extern "C" fn rb_buffer_unsafe_to_break(buffer: *mut ffi::rb_buffer_t, start: u32, end: u32) {
    Buffer::from_ptr_mut(buffer).unsafe_to_break(start as usize, end as usize);
}

#[no_mangle]
pub extern "C" fn rb_buffer_unsafe_to_break_from_outbuffer(buffer: *mut ffi::rb_buffer_t, start: u32, end: u32) {
    Buffer::from_ptr_mut(buffer).unsafe_to_break_from_outbuffer(start as usize, end as usize);
}

#[no_mangle]
pub extern "C" fn rb_buffer_unsafe_to_break_all(buffer: *mut ffi::rb_buffer_t) {
    let buffer = Buffer::from_ptr_mut(buffer);
    buffer.unsafe_to_break_impl(0, buffer.len);
}

#[no_mangle]
pub extern "C" fn rb_buffer_replace_glyph(buffer: *mut ffi::rb_buffer_t, glyph_index: ffi::rb_codepoint_t) {
    Buffer::from_ptr_mut(buffer).replace_glyph(glyph_index);
}

#[no_mangle]
pub extern "C" fn rb_buffer_output_glyph(buffer: *mut ffi::rb_buffer_t, glyph_index: ffi::rb_codepoint_t) {
    Buffer::from_ptr_mut(buffer).output_glyph(glyph_index)
}

#[no_mangle]
pub extern "C" fn rb_buffer_output_info(buffer: *mut ffi::rb_buffer_t, glyph_info: GlyphInfo) {
    Buffer::from_ptr_mut(buffer).output_info(glyph_info);
}

#[no_mangle]
pub extern "C" fn rb_buffer_copy_glyph(buffer: *mut ffi::rb_buffer_t) {
    Buffer::from_ptr_mut(buffer).copy_glyph();
}

#[no_mangle]
pub extern "C" fn rb_buffer_next_glyph(buffer: *mut ffi::rb_buffer_t) {
    Buffer::from_ptr_mut(buffer).next_glyph();
}

#[no_mangle]
pub extern "C" fn rb_buffer_next_glyphs(buffer: *mut ffi::rb_buffer_t, n: u32) {
    Buffer::from_ptr_mut(buffer).next_glyphs(n as usize);
}

#[no_mangle]
pub extern "C" fn rb_buffer_skip_glyph(buffer: *mut ffi::rb_buffer_t) {
    Buffer::from_ptr_mut(buffer).skip_glyph();
}

#[no_mangle]
pub extern "C" fn rb_buffer_reset_masks(buffer: *mut ffi::rb_buffer_t, mask: Mask) {
    Buffer::from_ptr_mut(buffer).reset_masks(mask);
}

#[no_mangle]
pub extern "C" fn rb_buffer_set_masks(buffer: *mut ffi::rb_buffer_t, value: Mask, mask: Mask, start: u32, end: u32) {
    Buffer::from_ptr_mut(buffer).set_masks(value, mask, start, end);
}

#[no_mangle]
pub extern "C" fn rb_buffer_delete_glyph(buffer: *mut ffi::rb_buffer_t) {
    Buffer::from_ptr_mut(buffer).delete_glyph();
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_glyph_infos(buffer: *mut ffi::rb_buffer_t) -> *mut GlyphInfo {
    Buffer::from_ptr_mut(buffer).info.as_mut_ptr() as *mut _
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_pos(buffer: *mut ffi::rb_buffer_t) -> *mut GlyphPosition{
    Buffer::from_ptr_mut(buffer).pos.as_mut_ptr() as *mut _
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_max_ops(buffer: *const ffi::rb_buffer_t) -> i32 {
    Buffer::from_ptr(buffer).max_ops
}

#[no_mangle]
pub extern "C" fn rb_buffer_set_max_ops(buffer: *mut ffi::rb_buffer_t, ops: i32) {
    Buffer::from_ptr_mut(buffer).max_ops = ops;
}

#[no_mangle]
pub extern "C" fn rb_buffer_decrement_max_ops(buffer: *mut ffi::rb_buffer_t, count: i32) -> i32 {
    let buffer = Buffer::from_ptr_mut(buffer);
    buffer.max_ops -= count;
    buffer.max_ops
}

#[no_mangle]
pub extern "C" fn rb_buffer_set_max_len(buffer: *mut ffi::rb_buffer_t, len: u32) {
    Buffer::from_ptr_mut(buffer).max_len = len;
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_index(buffer: *const ffi::rb_buffer_t) -> u32 {
    Buffer::from_ptr(buffer).idx as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_set_index(buffer: *mut ffi::rb_buffer_t, idx: u32) {
    Buffer::from_ptr_mut(buffer).idx = idx as usize;
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_out_len(buffer: *const ffi::rb_buffer_t) -> u32 {
    Buffer::from_ptr(buffer).out_len as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_set_out_len(buffer: *mut ffi::rb_buffer_t, idx: u32) {
    Buffer::from_ptr_mut(buffer).out_len = idx as usize;
}

#[no_mangle]
pub extern "C" fn rb_buffer_has_separate_output(buffer: *const ffi::rb_buffer_t) -> bool {
    Buffer::from_ptr(buffer).have_separate_output
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_context(buffer: *const ffi::rb_buffer_t, idx1: u32, idx2: u32) -> ffi::rb_codepoint_t {
    Buffer::from_ptr(buffer).context[idx1 as usize][idx2 as usize] as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_context_len(buffer: *const ffi::rb_buffer_t, idx: u32) -> u32 {
    Buffer::from_ptr(buffer).context_len[idx as usize] as u32
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_glyph_positions(buffer: *mut ffi::rb_buffer_t) -> *mut GlyphPosition {
    let buffer = Buffer::from_ptr_mut(buffer);
    if !buffer.have_positions {
        buffer.clear_positions();
    }

    buffer.pos.as_mut_ptr() as *mut _
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_script(buffer: *const ffi::rb_buffer_t) -> ffi::rb_script_t {
    Buffer::from_ptr(buffer).script.map(|s| (s.0).0).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_flags(buffer: *const ffi::rb_buffer_t) -> u32 {
    Buffer::from_ptr(buffer).flags.bits
}

#[no_mangle]
pub extern "C" fn rb_buffer_set_direction(buffer: *mut ffi::rb_buffer_t, direction: ffi::rb_direction_t) {
    Buffer::from_ptr_mut(buffer).direction = Direction::from_raw(direction);
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_segment_properties(
    buffer: *const ffi::rb_buffer_t,
    props: *mut ffi::rb_segment_properties_t,
) {
    let buffer = Buffer::from_ptr(buffer);
    unsafe {
        *props = ffi::rb_segment_properties_t {
            direction: buffer.direction.to_raw(),
            script: buffer.script.map(|s| (s.0).0).unwrap_or(0),
            language: buffer.language.as_ref().map(|s| s.0.as_ptr()).unwrap_or(std::ptr::null()),
        };
    }
}

#[no_mangle]
pub extern "C" fn rb_buffer_get_scratch_flags(buffer: *const ffi::rb_buffer_t) -> u32 {
    Buffer::from_ptr(buffer).scratch_flags.bits
}

#[no_mangle]
pub extern "C" fn rb_buffer_set_scratch_flags(buffer: *mut ffi::rb_buffer_t, flags: u32) {
    Buffer::from_ptr_mut(buffer).scratch_flags.bits = flags;
}

#[no_mangle]
pub extern "C" fn rb_buffer_is_allocation_successful(buffer: *const ffi::rb_buffer_t) -> bool {
    Buffer::from_ptr(buffer).successful
}

#[no_mangle]
pub extern "C" fn rb_buffer_sort(buffer: *mut ffi::rb_buffer_t, start: u32, end: u32,
                                 cmp: extern "C" fn(*const GlyphInfo, *const GlyphInfo) -> i32
) {
    let buffer = Buffer::from_ptr_mut(buffer);
    let start = start as usize;
    let end = end as usize;

    assert!(!buffer.have_positions);

    for i in start+1..end {
        let mut j = i;
        while j > start && cmp(&GlyphInfo::from(buffer.info[j - 1]) as *const _,
                               &GlyphInfo::from(buffer.info[i]) as *const _) > 0 {
            j -= 1;
        }

        if i == j {
            continue;
        }

        // Move item i to occupy place for item j, shift what's in between.
        buffer.merge_clusters(j, i + 1);

        {
            let t = buffer.info[i];
            for idx in (0..i-j).rev() {
                buffer.info[idx + j + 1] = buffer.info[idx + j];
            }

            buffer.info[j] = t;
        }
    }
}
