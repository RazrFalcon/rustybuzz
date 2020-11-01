//! The Glyph Positioning Table.

use std::convert::TryFrom;

use ttf_parser::parser::{FromData, NumFrom, Offset, Offset16, Stream};
use ttf_parser::GlyphId;

use super::common::{Coverage, Device};
use super::ApplyContext;
use crate::Font;

#[derive(Clone, Copy, Debug)]
enum SinglePos<'a> {
    Format1 {
        coverage: Coverage<'a>,
        value: ValueRecord<'a>,
    },
    Format2 {
        coverage: Coverage<'a>,
        values: ValueArray<'a>,
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
                let value = ValueRecord::read(&mut s, flags, data)?;
                Self::Format1 { coverage, value }
            }
            2 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let flags = s.read::<ValueFormatFlags>()?;
                let count = s.read::<u16>()?;
                let values = ValueArray::read(&mut s, flags, count, data)?;
                Self::Format2 { coverage, values }
            }
            _ => return None,
        })
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph_id = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        let value = match self {
            Self::Format1 { coverage, value } => {
                coverage.get(glyph_id)?;
                *value
            }
            Self::Format2 { coverage, values } => {
                let index = coverage.get(glyph_id)?;
                values.get(index)?
            }
        };

        value.apply(ctx, ctx.buffer().idx);
        ctx.buffer_mut().idx += 1;

        Some(())
    }
}

#[derive(Clone, Copy, Debug)]
struct ValueArray<'a> {
    flags: ValueFormatFlags,
    data: &'a [u8],
    base: &'a [u8],
}

impl<'a> ValueArray<'a> {
    fn read(
        s: &mut Stream<'a>,
        flags: ValueFormatFlags,
        count: u16,
        base: &'a [u8],
    ) -> Option<Self> {
        let len = flags.size() * usize::from(count);
        s.read_bytes(len).map(|data| Self { flags, data, base })
    }

    fn get(&self, index: u16) -> Option<ValueRecord<'a>> {
        let size = self.flags.size();
        let start = usize::from(index) * size;
        let end = start + size;
        self.data.get(start..end).map(|data| ValueRecord {
            flags: self.flags,
            data,
            base: self.base,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct ValueRecord<'a> {
    flags: ValueFormatFlags,
    data: &'a [u8],
    base: &'a [u8],
}

impl<'a> ValueRecord<'a> {
    fn read(s: &mut Stream<'a>, flags: ValueFormatFlags, base: &'a [u8]) -> Option<Self> {
        s.read_bytes(flags.size()).map(|data| Self { flags, data, base })
    }

    fn apply(&self, ctx: &mut ApplyContext, idx: usize) -> bool {
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
                        pos.x_offset += self.device_x_delta(offset, font);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::Y_PLACEMENT_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if use_y_device && !offset.is_null() {
                        pos.y_offset += self.device_y_delta(offset, font);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::X_ADVANCE_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if horizontal && use_x_device && !offset.is_null() {
                        pos.x_advance += self.device_x_delta(offset, font);
                        worked = true;
                    }
                }
            }

            if self.flags.contains(ValueFormatFlags::Y_ADVANCE_DEVICE) {
                if let Some(offset) = s.read::<Offset16>() {
                    if !horizontal && use_y_device && !offset.is_null() {
                        // y_advance values grow downward but font-space grows upward, hence negation
                        pos.y_advance -= self.device_y_delta(offset, font);
                        worked = true;
                    }
                }
            }
        }

        ctx.buffer_mut().pos[idx] = pos;
        worked
    }

    fn device_x_delta(&self, offset: Offset16, font: &Font) -> i32 {
        self.device(offset).and_then(|device| device.get_x_delta(font)).unwrap_or(0)
    }

    fn device_y_delta(&self, offset: Offset16, font: &Font) -> i32 {
        self.device(offset).and_then(|device| device.get_y_delta(font)).unwrap_or(0)
    }

    fn device(&self, offset: Offset16) -> Option<Device> {
        self.base.get(offset.to_usize()..).and_then(Device::parse)
    }
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
        2 * usize::num_from(self.bits.count_ones())
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
    let record = ValueRecord { flags, data, base };
    record.apply(&mut ctx, idx as usize) as crate::ffi::rb_bool_t
}
