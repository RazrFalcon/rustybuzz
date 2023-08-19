use crate::buffer::Buffer;
use crate::scan::scan;

use alloc::vec::Vec;

use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::combinator::{opt, value, map};
use nom::multi::fold_many0;
use nom::sequence::{pair, tuple};

use nom::IResult;

macro_rules! define_tags {
    ($($tagname:ident = $tagval:expr;)*) => {$(
        #[allow(non_snake_case, dead_code)]
        fn $tagname(c: &[u8]) -> IResult<&[u8], ()> {
            value((), tag([$tagval]))(c)
        }
    )*};
}

define_tags! {
    X = 0;
    C = 1;
    V = 2;
    N = 3;
    H = 4;
    ZWNJ = 5;
    ZWJ = 6;
    M = 7;
    SM = 8;
    A = 9;
    VD = 9;
    PLACEHOLDER = 10;
    DOTTEDCIRCLE = 11;
    RS = 12;
    MPst = 13;
    Repha = 14;
    Ra = 15;
    CM = 16;
    Symbol = 17;
    CS = 18;
}

// c = (C | Ra);
fn is_consonant(c: &[u8]) -> IResult<&[u8], ()> {
    value((), alt((C, Ra)))(c)
}

// n = ((ZWNJ?.RS)? (N.N?)?);
fn is_consonant_modifier(c: &[u8]) -> IResult<&[u8], ()> {
    value((), pair(opt(pair(opt(ZWNJ), RS)), opt(pair(N, opt(N)))))(c)
}

// z = ZWJ|ZWNJ;
fn is_joiner(c: &[u8]) -> IResult<&[u8], ()> {
    value((), alt((ZWJ, ZWNJ)))(c)
}

// reph = (Ra H | Repha);
fn possible_reph(c: &[u8]) -> IResult<&[u8], ()> {
    value((), alt((
        value((), pair(Ra, H)),
        Repha
    )))(c)
}

// cn = c.ZWJ?.n?;
fn cn(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        is_consonant,
        opt(ZWJ),
        opt(is_consonant_modifier)
    )))(c)
}

// symbol = Symbol.N?;
fn symbol(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        Symbol,
        opt(N)
    )))(c)
}

// matra_group = z*.(M | SM? MPst).N?.H?
fn matra_group(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        fold_many0(is_joiner, || (), |(), ()| ()),
        alt((
            M,
            value((), pair(opt(SM), MPst))
        )),
        opt(N),
        opt(H)
    )))(c)
}

// syllable_tail = (z?.SM.SM?.ZWNJ?)? (A | VD)*;
fn syllable_tail(c: &[u8]) -> IResult<&[u8], ()> {
    value((), pair(
        opt(tuple((
            opt(is_joiner),
            SM,
            opt(SM),
            opt(ZWNJ)
        ))),
        fold_many0(
            alt((A, VD)),
            || (),
            |(), ()| ()
        )
    ))(c)
}

// halant_group = (z?.H.(ZWJ.N?)?);
fn halant_group(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        opt(is_joiner),
        H,
        opt(pair(ZWJ, opt(N)))
    )))(c)
}

// final_halant_group = halant_group | H.ZWNJ;
fn final_halant_group(c: &[u8]) -> IResult<&[u8], ()> {
    value((), alt((
        halant_group,
        value((), pair(H, ZWNJ))
    )))(c)
}

// medial_group = CM?;
fn medial_group(c: &[u8]) -> IResult<&[u8], ()> {
    value((), opt(CM))(c)
}

// halant_or_matra_group = (final_halant_group | matra_group*);
fn halant_or_matra_group(c: &[u8]) -> IResult<&[u8], ()> {
    value((), alt((
        final_halant_group,
        fold_many0(matra_group, || (), |(), ()| ())
    )))(c)
}

// complex_syllable_tail = (halant_group.cn)* medial_group halant_or_matra_group syllable_tail;
fn complex_syllable_tail(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        fold_many0(
            pair(halant_group, cn),
            || (),
            |(), _| ()
        ),
        medial_group,
        halant_or_matra_group,
        syllable_tail
    )))(c)
}

// consonant_syllable =	(Repha|CS)? cn complex_syllable_tail;
fn consonant_syllable(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        opt(alt((Repha, CS))),
        cn,
        complex_syllable_tail
    )))(c)
}

// vowel_syllable =	reph? V.n? (ZWJ | complex_syllable_tail);
fn vowel_syllable(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        opt(possible_reph),
        V,
        opt(N),
        alt((ZWJ, complex_syllable_tail))
    )))(c)
}

// standalone_cluster =	((Repha|CS)? PLACEHOLDER | reph? DOTTEDCIRCLE).n? complex_syllable_tail;
fn standalone_cluster(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        alt((
            value((), pair(opt(alt((Repha, CS))), PLACEHOLDER)),
            value((), pair(opt(possible_reph), DOTTEDCIRCLE))
        )),
        opt(N),
        complex_syllable_tail
    )))(c)
}

// symbol_cluster =	symbol syllable_tail;
fn symbol_cluster(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        symbol,
        syllable_tail
    )))(c)
}

// broken_cluster =	reph? n? complex_syllable_tail;
fn broken_cluster(c: &[u8]) -> IResult<&[u8], ()> {
    value((), tuple((
        opt(possible_reph),
        opt(is_consonant_modifier),
        complex_syllable_tail
    )))(c)
}

/// Find a syllable in the buffer.
fn get_syllable(c: &[u8]) -> IResult<&[u8], SyllableType> {
    scan((
        map(consonant_syllable, |()| SyllableType::ConsonantSyllable),
        map(vowel_syllable, |()| SyllableType::VowelSyllable),
        map(standalone_cluster, |()| SyllableType::StandaloneCluster),
        map(symbol_cluster, |()| SyllableType::SymbolCluster),
        map(broken_cluster, |()| SyllableType::BrokenCluster),
        map(take(1u32), |_| SyllableType::NonIndicCluster)
    ))(c)
}

pub fn find_syllables_indic(buffer: &mut Buffer) {
    // Collect all of the indic categories.
    let indic_categories = buffer.info.iter().map(|c| c.indic_category()).collect::<Vec<_>>();

    // Begin iterating over the indic categories.
    let mut start = 0;
    let mut end = 0;
    let mut serial = 0;
    let mut slice = &*indic_categories;

    while !slice.is_empty() {
        let (rest, syllable) = match get_syllable(slice) {
            Ok(t) => t,
            Err(_) => {
                // TODO: Handle this error in greater depth.
                break;
            }
        };

        // Update the buffer.
        let length = slice.len() - rest.len();
        end += length;
        found_syllable(start, end, &mut serial, syllable, buffer);

        // Update our state.
        start += length;
        slice = rest;
    }
}

#[derive(Clone, Copy)]
pub enum SyllableType {
    ConsonantSyllable = 0,
    VowelSyllable,
    StandaloneCluster,
    SymbolCluster,
    BrokenCluster,
    NonIndicCluster,
}

#[inline]
fn found_syllable(
    start: usize,
    end: usize,
    syllable_serial: &mut u8,
    kind: SyllableType,
    buffer: &mut Buffer,
) {
    for i in start..end {
        buffer.info[i].set_syllable((*syllable_serial << 4) | kind as u8);
    }

    *syllable_serial += 1;

    if *syllable_serial == 16 {
        *syllable_serial = 1;
    }
}
