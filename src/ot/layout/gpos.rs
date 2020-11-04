//! The Glyph Positioning Table.

use std::convert::TryFrom;

use ttf_parser::parser::{FromData, LazyArray16, NumFrom, Offset, Offset16, Offsets16, Stream};
use ttf_parser::GlyphId;

use super::common::{Coverage, ClassDef, Device, LookupFlags};
use super::dyn_array::DynArray;
use super::matching::SkippyIter;
use super::ApplyContext;
use crate::buffer::{BufferScratchFlags, GlyphPosition};
use crate::common::Direction;
use crate::Font;

#[derive(Clone, Copy, Debug)]
enum SinglePos<'a> {
    Format1 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        value: ValueRecord<'a>,
    },
    Format2 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        flags: ValueFormatFlags,
        values: DynArray<'a>,
    },
}

impl<'a> SinglePos<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = s.read::<ValueFormatFlags>()?;
                let value = ValueRecord::read(&mut s, flags)?;
                Self::Format1 { data, coverage, value }
            }
            2 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = s.read::<ValueFormatFlags>()?;
                let count = s.read::<u16>()?;
                let values = DynArray::read(&mut s, usize::from(count), flags.size())?;
                Self::Format2 { data, coverage, flags, values }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
            Self::Format2 { coverage, .. } => coverage,
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph_id = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        let (base, value) = match *self {
            Self::Format1 { data, coverage, value } => {
                coverage.get(glyph_id)?;
                (data, value)
            }
            Self::Format2 { data, coverage, flags, values } => {
                let index = coverage.get(glyph_id)?;
                let record = ValueRecord::new(values.get(usize::from(index))?, flags);
                (data, record)
            }
        };

        value.apply(ctx, base, ctx.buffer().idx);
        ctx.buffer_mut().idx += 1;

        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
enum PairPos<'a> {
    Format1 {
        coverage: Coverage<'a>,
        flags: [ValueFormatFlags; 2],
        sets: Offsets16<'a, Offset16>,
    },
    Format2 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        flags: [ValueFormatFlags; 2],
        classes: [ClassDef<'a>; 2],
        counts: [u16; 2],
        matrix: DynArray<'a>,
    },
}

impl<'a> PairPos<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = [
                    s.read::<ValueFormatFlags>()?,
                    s.read::<ValueFormatFlags>()?,
                ];
                let count = s.read::<u16>()?;
                let sets = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, flags, sets }
            }
            2 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = [
                    s.read::<ValueFormatFlags>()?,
                    s.read::<ValueFormatFlags>()?,
                ];
                let classes = [
                    ClassDef::parse(s.read_offset16_data()?)?,
                    ClassDef::parse(s.read_offset16_data()?)?,
                ];
                let counts = [s.read::<u16>()?, s.read::<u16>()?];
                let count = usize::num_from(u32::from(counts[0]) * u32::from(counts[1]));
                let stride = flags[0].size() + flags[1].size();
                let matrix = DynArray::read(&mut s, count, stride)?;
                Self::Format2 { data, coverage, flags, classes, counts, matrix }
            },
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
            Self::Format2 { coverage, .. } => coverage,
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let first = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        let index = self.coverage().get(first)?;

        let mut iter = SkippyIter::new(ctx, ctx.buffer().idx, 1, false);
        if !iter.next() {
            return None;
        }

        let pos = iter.index();
        let second = GlyphId(u16::try_from(ctx.buffer().info[pos].codepoint).unwrap());

        let (base, flags, mut s) = match *self {
            Self::Format1 { flags, sets, .. } => {
                let data = sets.slice(index)?;
                let mut s = Stream::new(data);
                let count = s.read::<u16>()?;
                let stride = GlyphId::SIZE + flags[0].size() + flags[1].size();
                let records = DynArray::read(&mut s, usize::from(count), stride)?;
                let record = records.binary_search_by(|data| {
                    Stream::new(data).read::<GlyphId>().unwrap().cmp(&second)
                })?;

                let mut s = Stream::new(record);
                s.skip::<GlyphId>();
                (data, flags, s)
            }
            Self::Format2 { data, flags, classes, counts, matrix, .. } => {
                let classes = [classes[0].get(first).0, classes[1].get(second).0];
                if classes[0] >= counts[0] || classes[1] >= counts[1] {
                    return None;
                }

                let idx = usize::from(classes[0]) * usize::from(counts[1]) + usize::from(classes[1]);
                let record = matrix.get(idx)?;
                (data, flags, Stream::new(record))
            }
        };

        let records = [
            ValueRecord::read(&mut s, flags[0])?,
            ValueRecord::read(&mut s, flags[1])?,
        ];

        // Note the intentional use of "|" instead of short-circuit "||".
        if records[0].apply(ctx, base, ctx.buffer().idx) | records[1].apply(ctx, base, pos) {
            let start = ctx.buffer().idx;
            ctx.buffer_mut().unsafe_to_break(start, pos + 1);
        }

        ctx.buffer_mut().idx = pos + usize::from(!flags[1].is_empty());
        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
enum CursivePos<'a> {
    Format1 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        entry_exits: LazyArray16<'a, EntryExitRecord>,
    }
}

impl<'a> CursivePos<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let count = s.read::<u16>()?;
                let entry_exits = s.read_array16(count)?;
                Self::Format1 { data, coverage, entry_exits }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let (i, (exit_x, exit_y), (entry_x, entry_y)) = match self {
            Self::Format1 { data, coverage, entry_exits } => {
                let this = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
                let entry = entry_exits.get(coverage.get(this)?)?.entry_anchor;
                if entry.is_null() {
                    return None;
                }

                let mut iter = SkippyIter::new(ctx, ctx.buffer().idx, 1, false);
                if !iter.prev() {
                    return None;
                }

                let i = iter.index();
                let prev = GlyphId(u16::try_from(ctx.buffer().info[i].codepoint).unwrap());
                let exit = entry_exits.get(coverage.get(prev)?)?.exit_anchor;
                if exit.is_null() {
                    return None;
                }

                let exit_pos = Anchor::parse(data.get(exit.to_usize()..)?)?.get(ctx);
                let entry_pos = Anchor::parse(data.get(entry.to_usize()..)?)?.get(ctx);
                (i, exit_pos, entry_pos)
            }
        };

        let direction = ctx.direction();
        let lookup_props = ctx.lookup_props();
        let buffer = ctx.buffer_mut();
        let j = buffer.idx;
        buffer.unsafe_to_break(i, j);

        let pos = &mut buffer.pos;
        match direction {
            Direction::LeftToRight => {
                pos[i].x_advance = exit_x + pos[i].x_offset;
                let d = entry_x + pos[j].x_offset;
                pos[j].x_advance -= d;
                pos[j].x_offset -= d;
            }
            Direction::RightToLeft => {
                let d = exit_x + pos[i].x_offset;
                pos[i].x_advance -= d;
                pos[i].x_offset -= d;
                pos[j].x_advance = entry_x + pos[j].x_offset;
            }
            Direction::TopToBottom => {
                pos[i].y_advance = exit_y + pos[i].y_offset;
                let d = entry_y + pos[j].y_offset;
                pos[j].y_advance -= d;
                pos[j].y_offset -= d;
            }
            Direction::BottomToTop => {
                let d = exit_y + pos[i].y_offset;
                pos[i].y_advance -= d;
                pos[i].y_offset -= d;
                pos[j].y_advance = entry_y;
            }
            Direction::Invalid => {}
        }

        // Cross-direction adjustment

        // We attach child to parent (think graph theory and rooted trees whereas
        // the root stays on baseline and each node aligns itself against its
        // parent.
        //
        // Optimize things for the case of RightToLeft, as that's most common in
        // Arabic.
        let mut child = i;
        let mut parent = j;
        let mut x_offset = entry_x - exit_x;
        let mut y_offset = entry_y - exit_y;

        // Low bits are lookup flags, so we want to truncate.
        if lookup_props as u16 & LookupFlags::RIGHT_TO_LEFT.bits() == 0 {
            std::mem::swap(&mut child, &mut parent);
            x_offset = -x_offset;
            y_offset = -y_offset;
        }

        // If child was already connected to someone else, walk through its old
        // chain and reverse the link direction, such that the whole tree of its
        // previous connection now attaches to new parent.  Watch out for case
        // where new parent is on the path from old chain...
        reverse_cursive_minor_offset(pos, child, direction, parent);

        pos[child].set_attach_type(AttachType::Cursive as u8);
        pos[child].set_attach_chain(parent as i16 - child as i16);

        buffer.scratch_flags |= BufferScratchFlags::HAS_GPOS_ATTACHMENT;
        if direction.is_horizontal() {
            pos[child].y_offset = y_offset;
        } else {
            pos[child].x_offset = x_offset;
        }

        // If parent was attached to child, break them free.
        // https://github.com/harfbuzz/harfbuzz/issues/2469
        if pos[parent].attach_chain() == -pos[child].attach_chain() {
            pos[parent].set_attach_chain(0);
        }

        buffer.idx += 1;
        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
struct EntryExitRecord {
    entry_anchor: Offset16,
    exit_anchor: Offset16,
}

impl FromData for EntryExitRecord {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            entry_anchor: s.read()?,
            exit_anchor: s.read()?,
        })
    }
}

fn reverse_cursive_minor_offset(
    pos: &mut [GlyphPosition],
    i: usize,
    direction: Direction,
    new_parent: usize,
) {
    let chain = pos[i].attach_chain();
    let attach_type = pos[i].attach_type();
    if chain == 0 || attach_type & AttachType::Cursive as u8 == 0 {
        return;
    }

    pos[i].set_attach_chain(0);

    // Stop if we see new parent in the chain.
    let j = (i as isize + isize::from(chain)) as _;
    if j == new_parent {
        return;
    }

    reverse_cursive_minor_offset(pos, j, direction, new_parent);

    if direction.is_horizontal() {
        pos[j].y_offset = -pos[i].y_offset;
    } else {
        pos[j].x_offset = -pos[i].x_offset;
    }

    pos[j].set_attach_chain(-chain);
    pos[j].set_attach_type(attach_type);
}

#[derive(Clone, Copy, Debug)]
struct ValueRecord<'a> {
    data: &'a [u8],
    flags: ValueFormatFlags,
}

impl<'a> ValueRecord<'a> {
    fn new(data: &'a [u8], flags: ValueFormatFlags) -> Self {
        Self { data, flags }
    }

    fn read(s: &mut Stream<'a>, flags: ValueFormatFlags) -> Option<Self> {
        s.read_bytes(flags.size()).map(|data| Self { data, flags })
    }

    fn apply(&self, ctx: &mut ApplyContext, base: &[u8], idx: usize) -> bool {
        let mut s = Stream::new(self.data);

        let horizontal = ctx.direction().is_horizontal();
        let mut pos = ctx.buffer().pos[idx];
        let mut worked = false;

        if self.flags.contains(ValueFormatFlags::X_PLACEMENT) {
            if let Some(delta) = s.read::<i16>() {
                pos.x_offset += i32::from(delta);
                worked |= delta != 0;
            }
        }

        if self.flags.contains(ValueFormatFlags::Y_PLACEMENT) {
            if let Some(delta) = s.read::<i16>() {
                pos.y_offset += i32::from(delta);
                worked |= delta != 0;
            }
        }

        if self.flags.contains(ValueFormatFlags::X_ADVANCE) {
            if let Some(delta) = s.read::<i16>() {
                if horizontal {
                    pos.x_advance += i32::from(delta);
                    worked |= delta != 0;
                }
            }
        }

        if self.flags.contains(ValueFormatFlags::Y_ADVANCE) {
            if let Some(delta) = s.read::<i16>() {
                if !horizontal {
                    // y_advance values grow downward but font-space grows upward, hence negation
                    pos.y_advance -= i32::from(delta);
                    worked |= delta != 0;
                }
            }
        }

        if self.flags.intersects(ValueFormatFlags::DEVICES) {
            let font = ctx.font();
            let (ppem_x, ppem_y) = font.pixels_per_em().unwrap_or((0, 0));
            let coords = font.ttfp_face.variation_coordinates().len();
            let use_x_device = ppem_x != 0 || coords != 0;
            let use_y_device = ppem_y != 0 || coords != 0;

            if self.flags.contains(ValueFormatFlags::X_PLACEMENT_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if use_x_device && !offset.is_null() {
                        pos.x_offset += device_x_delta(base, offset, font);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::Y_PLACEMENT_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if use_y_device && !offset.is_null() {
                        pos.y_offset += device_y_delta(base, offset, font);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::X_ADVANCE_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if horizontal && use_x_device && !offset.is_null() {
                        pos.x_advance += device_x_delta(base, offset, font);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::Y_ADVANCE_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if !horizontal && use_y_device && !offset.is_null() {
                        // y_advance values grow downward but font-space grows upward, hence negation
                        pos.y_advance -= device_y_delta(base, offset, font);
                        worked = true;
                    }
                }
            }
        }

        ctx.buffer_mut().pos[idx] = pos;
        worked
    }
}

fn device_x_delta(base: &[u8], offset: Offset16, font: &Font) -> i32 {
    device(base, offset).and_then(|device| device.get_x_delta(font)).unwrap_or(0)
}

fn device_y_delta(base: &[u8], offset: Offset16, font: &Font) -> i32 {
    device(base, offset).and_then(|device| device.get_y_delta(font)).unwrap_or(0)
}

fn device(base: &[u8], offset: Offset16) -> Option<Device> {
    base.get(offset.to_usize()..).and_then(Device::parse)
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct ValueFormatFlags: u16 {
        const X_PLACEMENT        = 0x0001;
        const Y_PLACEMENT        = 0x0002;
        const X_ADVANCE          = 0x0004;
        const Y_ADVANCE          = 0x0008;
        const X_PLACEMENT_DEVICE = 0x0010;
        const Y_PLACEMENT_DEVICE = 0x0020;
        const X_ADVANCE_DEVICE   = 0x0040;
        const Y_ADVANCE_DEVICE   = 0x0080;
        const DEVICES            = Self::X_PLACEMENT_DEVICE.bits
                                 | Self::Y_PLACEMENT_DEVICE.bits
                                 | Self::X_ADVANCE_DEVICE.bits
                                 | Self::Y_ADVANCE_DEVICE.bits;
    }
}

impl ValueFormatFlags {
    fn size(self) -> usize {
        u16::SIZE * usize::num_from(self.bits.count_ones())
    }
}

impl FromData for ValueFormatFlags {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self::from_bits_truncate)
    }
}


#[derive(Clone, Copy, Debug)]
struct Anchor<'a> {
    x: i16,
    y: i16,
    x_device: Option<Device<'a>>,
    y_device: Option<Device<'a>>,
}

impl<'a> Anchor<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        if !matches!(format, 1..=3) {
            return None;
        }

        let mut table = Anchor {
            x: s.read::<i16>()?,
            y: s.read::<i16>()?,
            x_device: None,
            y_device: None,
        };

        // Note: Format 2 is not handled since there is currently no way to
        // get a glyph contour point by index.

        if format == 3 {
            table.x_device = s.read::<Option<Offset16>>()?
                .and_then(|offset| data.get(offset.to_usize()..))
                .and_then(Device::parse);

            table.y_device = s.read::<Option<Offset16>>()?
                .and_then(|offset| data.get(offset.to_usize()..))
                .and_then(Device::parse);
        }

        Some(table)
    }

    fn get(&self, ctx: &ApplyContext) -> (i32, i32) {
        let mut x = i32::from(self.x);
        let mut y = i32::from(self.y);

        if self.x_device.is_some() || self.y_device.is_some() {
            let font = ctx.font();
            let (ppem_x, ppem_y) = font.pixels_per_em().unwrap_or((0, 0));
            let coords = font.ttfp_face.variation_coordinates().len();

            if let Some(device) = self.x_device {
                if ppem_x != 0 || coords != 0 {
                    x += device.get_x_delta(font).unwrap_or(0);
                }
            }

            if let Some(device) = self.y_device {
                if ppem_y != 0 || coords != 0 {
                    y += device.get_y_delta(font).unwrap_or(0);
                }
            }
        }

        (x, y)
    }
}

enum AttachType {
    None = 0,
    Mark = 1,
    Cursive = 2,
}

make_ffi_funcs!(SinglePos, rb_single_pos_apply);
make_ffi_funcs!(PairPos, rb_pair_pos_apply);
make_ffi_funcs!(CursivePos, rb_cursive_pos_apply);

#[no_mangle]
pub extern "C" fn rb_value_format_apply(
    flags: u32,
    ctx: *mut crate::ffi::rb_ot_apply_context_t,
    base: *const u8,
    values: *const u8,
    idx: u32,
) -> crate::ffi::rb_bool_t {
    let flags = ValueFormatFlags::from_bits_truncate(flags as u16);
    let mut ctx = ApplyContext::from_ptr_mut(ctx);
    let base = unsafe { std::slice::from_raw_parts(base, isize::MAX as usize) };
    let data = unsafe { std::slice::from_raw_parts(values, isize::MAX as usize) };
    ValueRecord { data, flags }.apply(&mut ctx, base, idx as usize) as crate::ffi::rb_bool_t
}

#[no_mangle]
pub extern "C" fn rb_anchor_get(
    data: *const u8,
    ctx: *const crate::ffi::rb_ot_apply_context_t,
    x: *mut f32,
    y: *mut f32,
) {
    let data = unsafe { std::slice::from_raw_parts(data, isize::MAX as usize) };
    let ctx = ApplyContext::from_ptr(ctx);
    if let Some(anchor) = Anchor::parse(data) {
        let (vx, vy) = anchor.get(&ctx);
        unsafe {
            *x = vx as f32;
            *y = vy as f32;
        }
    }
}
