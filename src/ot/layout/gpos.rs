//! The Glyph Positioning Table.

use std::convert::TryFrom;

use ttf_parser::parser::{FromData, NumFrom, Offset, Offset16, Offsets16, Stream};
use ttf_parser::GlyphId;

use super::common::{Coverage, ClassDef, Device};
use super::dyn_array::DynArray;
use super::matching::SkippyIter;
use super::ApplyContext;
use crate::Font;

#[derive(Clone, Copy, Debug)]
enum SinglePos<'a> {
    Format1 {
        base: &'a [u8],
        coverage: Coverage<'a>,
        value: ValueRecord<'a>,
    },
    Format2 {
        base: &'a [u8],
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
                Self::Format1 { base: data, coverage, value }
            }
            2 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = s.read::<ValueFormatFlags>()?;
                let count = s.read::<u16>()?;
                let values = DynArray::read(&mut s, usize::from(count), flags.size())?;
                Self::Format2 { base: data, coverage, flags, values }
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
            Self::Format1 { base, coverage, value } => {
                coverage.get(glyph_id)?;
                (base, value)
            }
            Self::Format2 { base, coverage, flags, values } => {
                let index = coverage.get(glyph_id)?;
                let data = values.get(usize::from(index))?;
                (base, ValueRecord::new(data, flags))
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
        base: &'a [u8],
        coverage: Coverage<'a>,
        flags: [ValueFormatFlags; 2],
        sets: Offsets16<'a, Offset16>,
    },
    Format2 {
        base: &'a [u8],
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
                Self::Format1 { base: data, coverage, flags, sets }
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
                Self::Format2 { base: data, coverage, flags, classes, counts, matrix }
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
            Self::Format1 { base, flags, sets, .. } => {
                let mut s = Stream::new(sets.slice(index)?);
                let count = s.read::<u16>()?;
                let stride = GlyphId::SIZE + flags[0].size() + flags[1].size();
                let records = DynArray::read(&mut s, usize::from(count), stride)?;
                let record = records.binary_search_by(|data| {
                    Stream::new(data).read::<GlyphId>().unwrap().cmp(&second)
                })?;

                let mut s = Stream::new(record);
                s.skip::<GlyphId>();
                (base, flags, s)
            }
            Self::Format2 { base, flags, classes, counts, matrix, .. } => {
                let classes = [classes[0].get(first).0, classes[1].get(second).0];
                if classes[0] >= counts[0] || classes[1] >= counts[1] {
                    return None;
                }

                let idx = usize::from(classes[0]) * usize::from(counts[1]) + usize::from(classes[1]);
                let record = matrix.get(idx)?;
                (base, flags, Stream::new(record))
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

make_ffi_funcs!(SinglePos, rb_single_pos_apply);
make_ffi_funcs!(PairPos, rb_pair_pos_apply);

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
