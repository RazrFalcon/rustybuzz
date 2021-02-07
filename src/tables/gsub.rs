//! The Glyph Substitution Table.

use alloc::vec::Vec;

use crate::glyph_set::GlyphSet;
use super::gsubgpos::*;
use super::*;

#[derive(Clone, Debug)]
pub struct SubstTable<'a> {
    pub inner: SubstPosTable<'a>,
    pub lookups: Vec<Option<SubstLookup<'a>>>,
}

impl<'a> SubstTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let inner = SubstPosTable::parse(data)?;
        let lookups = (0..inner.lookup_count())
            .map(|i| inner.get_lookup(LookupIndex(i)).map(SubstLookup::parse))
            .collect();

        Some(Self { inner, lookups})
    }
}

#[derive(Clone, Debug)]
pub struct SubstLookup<'a> {
    pub subtables: Vec<SubstLookupSubtable<'a>>,
    pub coverage: GlyphSet,
    pub reverse: bool,
    pub props: u32,
}

impl<'a> SubstLookup<'a> {
    pub fn parse(lookup: Lookup<'a>) -> Self {
        let subtables: Vec<_> = lookup
            .subtables
            .into_iter()
            .flat_map(|data| SubstLookupSubtable::parse(data, lookup.kind))
            .collect();

        let mut coverage = GlyphSet::builder();
        let mut reverse = !subtables.is_empty();

        for subtable in &subtables {
            subtable.coverage().collect(&mut coverage);
            reverse &= subtable.is_reverse();
        }

        Self {
            subtables,
            coverage: coverage.finish(),
            reverse,
            props: lookup.props(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SubstLookupSubtable<'a> {
    Single(SingleSubst<'a>),
    Multiple(MultipleSubst<'a>),
    Alternate(AlternateSubst<'a>),
    Ligature(LigatureSubst<'a>),
    Context(ContextLookup<'a>),
    ChainContext(ChainContextLookup<'a>),
    ReverseChainSingle(ReverseChainSingleSubst<'a>),
}

impl<'a> SubstLookupSubtable<'a> {
    pub fn parse(data: &'a [u8], kind: LookupType) -> Option<Self> {
        match kind.0 {
            1 => SingleSubst::parse(data).map(Self::Single),
            2 => MultipleSubst::parse(data).map(Self::Multiple),
            3 => AlternateSubst::parse(data).map(Self::Alternate),
            4 => LigatureSubst::parse(data).map(Self::Ligature),
            5 => ContextLookup::parse(data).map(Self::Context),
            6 => ChainContextLookup::parse(data).map(Self::ChainContext),
            7 => parse_extension_lookup(data, Self::parse),
            8 => ReverseChainSingleSubst::parse(data).map(Self::ReverseChainSingle),
            _ => None,
        }
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match self {
            Self::Single(t) => t.coverage(),
            Self::Multiple(t) => t.coverage(),
            Self::Alternate(t) => t.coverage(),
            Self::Ligature(t) => t.coverage(),
            Self::Context(t) => t.coverage(),
            Self::ChainContext(t) => t.coverage(),
            Self::ReverseChainSingle(t) => t.coverage(),
        }
    }

    pub fn is_reverse(&self) -> bool {
        matches!(self, Self::ReverseChainSingle(_))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SingleSubst<'a> {
    Format1 {
        coverage: Coverage<'a>,
        delta: i16,
    },
    Format2 {
        coverage: Coverage<'a>,
        substitutes: LazyArray16<'a, GlyphId>,
    },
}

impl<'a> SingleSubst<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let delta = s.read::<i16>()?;
                Self::Format1 { coverage, delta }
            }
            2 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let count = s.read::<u16>()?;
                let substitutes = s.read_array16(count)?;
                Self::Format2 { coverage, substitutes }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { coverage, .. } => coverage,
            Self::Format2 { coverage, .. } => coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MultipleSubst<'a> {
    Format1 {
        coverage: Coverage<'a>,
        sequences: Offsets16<'a, Offset16>,
    },
}

impl<'a> MultipleSubst<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let count = s.read::<u16>()?;
                let sequences = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, sequences }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sequence<'a> {
    pub substitutes: LazyArray16<'a, GlyphId>,
}

impl<'a> Sequence<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let substitutes = s.read_array16(count)?;
        Some(Self { substitutes })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AlternateSubst<'a> {
    Format1 {
        coverage: Coverage<'a>,
        alternate_sets: Offsets16<'a, Offset16>,
    },
}

impl<'a> AlternateSubst<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let count = s.read::<u16>()?;
                let alternate_sets = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, alternate_sets }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AlternateSet<'a> {
    pub alternates: LazyArray16<'a, GlyphId>,
}

impl<'a> AlternateSet<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let alternates = s.read_array16(count)?;
        Some(Self { alternates })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LigatureSubst<'a> {
    Format1 {
        coverage: Coverage<'a>,
        ligature_sets: Offsets16<'a, Offset16>,
    },
}

impl<'a> LigatureSubst<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let count = s.read::<u16>()?;
                let ligature_sets = s.read_offsets16(count, data)?;
                Self::Format1 { coverage, ligature_sets }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LigatureSet<'a> {
    pub ligatures: Offsets16<'a, Offset16>,
}

impl<'a> LigatureSet<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let ligatures = s.read_offsets16(count, data)?;
        Some(Self { ligatures })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ligature<'a> {
    pub lig_glyph: GlyphId,
    pub components: LazyArray16<'a, u16>,
}

impl<'a> Ligature<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let lig_glyph = s.read::<GlyphId>()?;
        let count = s.read::<u16>()?;
        let components = s.read_array16(count.checked_sub(1)?)?;
        Some(Self { lig_glyph, components })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ReverseChainSingleSubst<'a> {
    Format1 {
        data: &'a [u8],
        coverage: Coverage<'a>,
        backtrack_coverages: LazyArray16<'a, u16>,
        lookahead_coverages: LazyArray16<'a, u16>,
        substitutes: LazyArray16<'a, GlyphId>,
    },
}

impl<'a> ReverseChainSingleSubst<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let coverage = Coverage::parse(s.read_at_offset16(data)?)?;
                let backtrack_count = s.read::<u16>()?;
                let backtrack_coverages = s.read_array16(backtrack_count)?;
                let lookahead_count = s.read::<u16>()?;
                let lookahead_coverages = s.read_array16(lookahead_count)?;
                let substitute_count = s.read::<u16>()?;
                let substitutes = s.read_array16(substitute_count)?;
                Self::Format1 {
                    data,
                    coverage,
                    backtrack_coverages,
                    lookahead_coverages,
                    substitutes,
                }
            }
            _ => return None,
        })
    }

    pub fn coverage(&self) -> Coverage<'a> {
        match *self {
            Self::Format1 { coverage, .. } => coverage,
        }
    }
}
