//! Common tables for OpenType layout.

use std::cmp::Ordering;
use std::convert::TryFrom;

use ttf_parser::GlyphId;
use ttf_parser::parser::{FromData, LazyArray16, Offset, Offset16, Offsets16, Offset32, Stream};

use super::layout::{ApplyContext, WouldApplyContext, MAX_CONTEXT_LENGTH};
use super::matching::{
    match_glyph, match_class, match_coverage, match_input, would_match_input, Matched, MatchFunc
};

/// A type-safe wrapper for a glyph class.
#[repr(transparent)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
pub struct GlyphClass(pub u16);

impl FromData for GlyphClass {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self)
    }
}

/// A GSUB or GPOS table.
#[derive(Clone, Copy)]
pub struct SubstPosTable<'a> {
    lookups: LookupList<'a>,
}

impl<'a> SubstPosTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let major_version = s.read::<u16>()?;
        let minor_version = s.read::<u16>()?;
        if major_version != 1 {
            return None;
        }

        s.skip::<Offset16>(); // TODO: script list
        s.skip::<Offset16>(); // TODO: feature list
        let lookups = LookupList::parse(s.read_offset16_data()?)?;
        if minor_version >= 1 {
            s.skip::<Offset32>(); // TODO: feature variations
        }

        Some(Self { lookups })
    }

    pub fn lookups(&self) -> LookupList {
        self.lookups
    }
}

#[derive(Clone, Copy)]
pub struct LookupList<'a> {
    offsets: Offsets16<'a, Offset16>,
}

impl<'a> LookupList<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let offsets = s.read_offsets16(count, data)?;
        Some(LookupList { offsets })
    }

    pub fn len(&self) -> usize {
        self.offsets.len() as usize
    }

    pub fn get(&self, index: usize) -> Option<Lookup<'a>> {
        Lookup::parse(self.offsets.slice(u16::try_from(index).ok()?)?)
    }
}

#[derive(Clone, Copy)]
pub struct Lookup<'a> {
    pub type_: u16,
    pub flags: LookupFlags,
    pub offsets: Offsets16<'a, Offset16>,
    pub mark_filtering_set: Option<u16>,
}

impl<'a> Lookup<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let type_ = s.read::<u16>()?;
        let flags = LookupFlags::from_bits_truncate(s.read()?);
        let count = s.read::<u16>()?;
        let offsets = s.read_offsets16(count, data)?;

        let mut mark_filtering_set: Option<u16> = None;
        if flags.contains(LookupFlags::USE_MARK_FILTERING_SET) {
            mark_filtering_set = Some(s.read()?);
        }

        Some(Self {
            type_,
            flags,
            offsets,
            mark_filtering_set,
        })
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct LookupFlags: u16 {
        const RIGHT_TO_LEFT          = 0x0001;
        const IGNORE_BASE_GLYPHS     = 0x0002;
        const IGNORE_LIGATURES       = 0x0004;
        const IGNORE_MARKS           = 0x0008;
        const IGNORE_FLAGS           = 0x000E;
        const USE_MARK_FILTERING_SET = 0x0010;
        const MARK_ATTACHMENT_TYPE   = 0xFF00;
    }
}

#[derive(Clone, Copy, Debug)]
struct LookupRecord {
    sequence_index: u16,
    lookup_list_index: u16,
}

impl FromData for LookupRecord {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            sequence_index: s.read::<u16>()?,
            lookup_list_index: s.read::<u16>()?,
        })
    }
}

/// A table that defines which glyph ids are covered by some lookup.
///
/// https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-table
#[derive(Clone, Copy, Debug)]
pub enum Coverage<'a> {
    Format1 { glyphs: LazyArray16<'a, GlyphId> },
    Format2 { records: LazyArray16<'a, RangeRecord> },
}

impl<'a> Coverage<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let count = s.read::<u16>()?;
                Self::Format1 { glyphs: s.read_array16(count)? }
            }
            2 => {
                let count = s.read::<u16>()?;
                Self::Format2 { records: s.read_array16(count)? }
            }
            _ => return None,
        })
    }

    /// Returns the coverage index of the glyph or `None` if it is not covered.
    pub fn get(&self, glyph: GlyphId) -> Option<u16> {
        match self {
            Self::Format1 { glyphs } => glyphs.binary_search(&glyph).map(|p| p.0),
            Self::Format2 { records } => {
                let record = RangeRecord::binary_search(records, glyph)?;
                let offset = glyph.0 - record.start.0;
                record.value.checked_add(offset)
            }
        }
    }
}

/// A table that defines which classes glyph ids belong to.
///
/// https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table
#[derive(Clone, Copy, Debug)]
pub enum ClassDef<'a> {
    Format1 {
        start: GlyphId,
        classes: LazyArray16<'a, GlyphClass>,
    },
    Format2 {
        records: LazyArray16<'a, RangeRecord>,
    },
}

impl<'a> ClassDef<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let start = s.read::<GlyphId>()?;
                let count = s.read::<u16>()?;
                let classes = s.read_array16(count)?;
                Self::Format1 { start, classes }
            },
            2 => {
                let count = s.read::<u16>()?;
                Self::Format2 { records: s.read_array16(count)? }
            },
            _ => return None,
        })
    }

    /// Returns the glyph class of the glyph (zero if it is not defined).
    pub fn get(&self, glyph: GlyphId) -> GlyphClass {
        let class = match self {
            Self::Format1 { start, classes } => {
                glyph.0.checked_sub(start.0)
                    .and_then(|index| classes.get(index))
            }
            Self::Format2 { records } => {
                RangeRecord::binary_search(records, glyph)
                    .map(|record| GlyphClass(record.value))
            }
        };
        class.unwrap_or(GlyphClass(0))
    }
}

/// A record that describes a range of glyph ids.
#[derive(Clone, Copy, Debug)]
pub struct RangeRecord {
    start: GlyphId,
    end: GlyphId,
    value: u16,
}

impl RangeRecord {
    fn binary_search(records: &LazyArray16<RangeRecord>, glyph: GlyphId) -> Option<RangeRecord> {
        records.binary_search_by(|record| {
            if glyph < record.start {
                Ordering::Greater
            } else if glyph <= record.end {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }).map(|p| p.1)
    }
}

impl FromData for RangeRecord {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            start: s.read::<GlyphId>()?,
            end: s.read::<GlyphId>()?,
            value: s.read::<u16>()?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum ContextLookup<'a> {
    Format1 {
        coverage: Coverage<'a>,
        rule_sets: Offsets16<'a, Offset16>,
    },
    Format2 {
        coverage: Coverage<'a>,
        class_def: ClassDef<'a>,
        rule_sets: Offsets16<'a, Offset16>,
    },
    Format3 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        coverage_offsets: LazyArray16<'a, u16>,
        records: LazyArray16<'a, LookupRecord>,
    },
}

impl<'a> ContextLookup<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let count = s.read::<u16>()?;
                let rule_sets = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, rule_sets }
            }
            2 => {
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let class_def = ClassDef::parse(s.read_offset16_data()?)?;
                let count = s.read::<u16>()?;
                let rule_sets = s.read_offsets16(count, data)?;
                Self::Format2 { coverage, class_def, rule_sets }
            }
            3 => {
                let input_count = s.read::<u16>()?;
                let record_count = s.read::<u16>()?;
                let coverage = Coverage::parse(s.read_offset16_data()?)?;
                let coverage_offsets = s.read_array16(input_count.checked_sub(1)?)?;
                let records = s.read_array16(record_count)?;
                Self::Format3 { data, coverage, coverage_offsets, records }
            }
            _ => return None,
        })
    }

    fn coverage(&self) -> &Coverage<'a> {
        match self {
            Self::Format1 { coverage, .. } => coverage,
            Self::Format2 { coverage, .. } => coverage,
            Self::Format3 { coverage, .. } => coverage,
        }
    }

    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let glyph_id = GlyphId(u16::try_from(ctx.glyph(0)).unwrap());
        match self {
            Self::Format1 { coverage, rule_sets } => {
                coverage.get(glyph_id)
                    .and_then(|index| rule_sets.slice(index))
                    .and_then(RuleSet::parse)
                    .map(|set| set.would_apply(ctx, &match_glyph))
                    .unwrap_or(false)
            }
            Self::Format2 { class_def, rule_sets, .. } => {
                let class = class_def.get(glyph_id);
                rule_sets.get(class.0).map_or(false, |offset| !offset.is_null())
                    && rule_sets.slice(class.0)
                        .and_then(RuleSet::parse)
                        .map(|set| set.would_apply(ctx, &match_class(*class_def)))
                        .unwrap_or(false)
            }
            Self::Format3 { data, coverage_offsets, .. } => {
                would_match_input(ctx, *coverage_offsets, &match_coverage(data))
            }
        }
    }

    fn apply(&self, ctx: &mut ApplyContext) -> Option<()> {
        let glyph_id = GlyphId(u16::try_from(ctx.buffer().cur(0).codepoint).unwrap());
        match self {
            Self::Format1 { coverage, rule_sets } => {
                let index = coverage.get(glyph_id)?;
                let set = RuleSet::parse(rule_sets.slice(index)?)?;
                set.apply(ctx, &match_glyph)
            }
            Self::Format2 { coverage, class_def, rule_sets } => {
                coverage.get(glyph_id)?;
                let class = class_def.get(glyph_id);
                let offset = rule_sets.get(class.0)?;
                if !offset.is_null() {
                    let set = RuleSet::parse(rule_sets.slice(class.0)?)?;
                    set.apply(ctx, &match_class(*class_def))
                } else {
                    None
                }
            }
            Self::Format3 { data, coverage, coverage_offsets, records } => {
                coverage.get(glyph_id)?;
                apply_context_lookup(ctx, *coverage_offsets, &match_coverage(data), *records,)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RuleSet<'a> {
    rules: Offsets16<'a, Offset16>,
}

impl<'a> RuleSet<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let rules = s.read_offsets16(count, data)?;
        Some(Self { rules })
    }

    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        self.rules
            .into_iter()
            .filter_map(|data| Rule::parse(data))
            .any(|rules| rules.would_apply(ctx, match_func))
    }

    fn apply(&self, ctx: &mut ApplyContext, match_func: &MatchFunc) -> Option<()> {
        for data in self.rules {
            let rule = Rule::parse(data)?;
            if rule.apply(ctx, match_func).is_some() {
                return Some(());
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
struct Rule<'a> {
    input: LazyArray16<'a, u16>,
    records: LazyArray16<'a, LookupRecord>,
}

impl<'a> Rule<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let input_count = s.read::<u16>()?;
        let record_count = s.read::<u16>()?;
        let input = s.read_array16(input_count.checked_sub(1)?)?;
        let records = s.read_array16(record_count)?;
        Some(Self { input, records })
    }

    fn would_apply(&self, ctx: &WouldApplyContext, match_func: &MatchFunc) -> bool {
        would_match_input(ctx, self.input, match_func)
    }

    fn apply(&self, ctx: &mut ApplyContext, match_func: &MatchFunc) -> Option<()> {
        apply_context_lookup(ctx, self.input, match_func, self.records)
    }
}

fn apply_context_lookup(
    ctx: &mut ApplyContext,
    input: LazyArray16<u16>,
    match_func: &MatchFunc,
    lookups: LazyArray16<LookupRecord>,
) -> Option<()> {
    match_input(ctx, input, match_func).map(|matched| {
        let buffer = ctx.buffer_mut();
        buffer.unsafe_to_break(buffer.idx, buffer.idx + matched.len);
        apply_lookup(ctx, 1 + input.len() as usize, matched, lookups);
    })
}

fn apply_lookup(
    ctx: &mut ApplyContext,
    mut count: usize,
    mut matched: Matched,
    lookups: LazyArray16<LookupRecord>,
) {
    let this_lookup_idx = ctx.lookup_index();
    let mut buffer = ctx.buffer_mut();

    // All positions are distance from beginning of *output* buffer.
    // Adjust.
    let mut end = {
        let backtrack_len = buffer.backtrack_len();
        let delta = backtrack_len as isize - buffer.idx as isize;

        // Convert positions to new indexing.
        for j in 0..count {
            matched.positions[j] = (matched.positions[j] as isize + delta) as _;
        }

        backtrack_len + matched.len
    };

    for record in lookups {
        if !buffer.successful {
            break;
        }

        let idx = record.sequence_index as usize;
        let lookup_idx = record.lookup_list_index as usize;

        if idx >= count {
            continue;
        }

        // Don't recurse to ourself at same position.
        // Note that this test is too naive, it doesn't catch longer loops.
        if idx == 0 && lookup_idx == this_lookup_idx {
            continue;
        }

        if !buffer.move_to(matched.positions[idx]) {
            break;
        }

        if buffer.max_ops <= 0 {
            break;
        }

        let orig_len = buffer.backtrack_len() + buffer.lookahead_len();
        if !ctx.recurse(lookup_idx) {
            buffer = ctx.buffer_mut();
            continue;
        }

        buffer = ctx.buffer_mut();
        let new_len = buffer.backtrack_len() + buffer.lookahead_len();
        let mut delta = new_len as isize - orig_len as isize;
        if delta == 0 {
            continue;
        }

        // Recursed lookup changed buffer len.  Adjust.
        //
        // TODO:
        //
        // Right now, if buffer length increased by n, we assume n new glyphs
        // were added right after the current position, and if buffer length
        // was decreased by n, we assume n match positions after the current
        // one where removed.  The former (buffer length increased) case is
        // fine, but the decrease case can be improved in at least two ways,
        // both of which are significant:
        //
        //   - If recursed-to lookup is MultipleSubst and buffer length
        //     decreased, then it's current match position that was deleted,
        //     NOT the one after it.
        //
        //   - If buffer length was decreased by n, it does not necessarily
        //     mean that n match positions where removed, as there might
        //     have been marks and default-ignorables in the sequence.  We
        //     should instead drop match positions between current-position
        //     and current-position + n instead.
        //
        // It should be possible to construct tests for both of these cases.

        end = (end as isize + delta) as _;
        if end <= matched.positions[idx] {
            // End might end up being smaller than match_positions[idx] if the recursed
            // lookup ended up removing many items, more than we have had matched.
            // Just never rewind end back and get out of here.
            // https://bugs.chromium.org/p/chromium/issues/detail?id=659496
            end = matched.positions[idx];

            // There can't be any further changes.
            break;
        }

        // next now is the position after the recursed lookup.
        let mut next = idx + 1;

        if delta > 0 {
            if delta + count as isize > MAX_CONTEXT_LENGTH as isize {
                break;
            }
        } else {
            // NOTE: delta is negative.
            delta = delta.max(next as isize - count as isize);
            next = (next as isize - delta) as _;
        }

        // Shift!
        matched.positions.copy_within(next .. count, (next as isize + delta) as _);
        next = (next as isize + delta) as _;
        count = (count as isize + delta) as _;

        // Fill in new entries.
        for j in idx+1..next {
            matched.positions[j] = matched.positions[j - 1] + 1;
        }

        // And fixup the rest.
        while next < count {
            matched.positions[next] = (matched.positions[next] as isize + delta) as _;
            next += 1;
        }
    }

    buffer.move_to(end);
}

make_ffi_funcs!(ContextLookup, rb_context_lookup_would_apply, rb_context_lookup_apply);
