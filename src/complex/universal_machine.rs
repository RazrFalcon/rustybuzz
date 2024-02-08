#![allow(
    dead_code,
    non_upper_case_globals,
    unused_assignments,
    unused_parens,
    while_true,
    clippy::assign_op_pattern,
    clippy::collapsible_if,
    clippy::comparison_chain,
    clippy::double_parens,
    clippy::unnecessary_cast,
    clippy::single_match,
    clippy::never_loop
)]

use crate::buffer::Buffer;
use crate::complex::machine_cursor::MachineCursor;
use crate::complex::universal::category;
use crate::GlyphInfo;
use core::cell::Cell;

static _use_syllable_machine_trans_keys: [u8; 124] = [
    0, 36, 26, 27, 27, 27, 5, 33, 5, 33, 1, 1, 9, 33, 10, 33, 11, 32, 12, 32, 13, 32, 30, 31, 31,
    31, 11, 33, 11, 33, 11, 33, 1, 1, 11, 33, 10, 33, 10, 33, 10, 33, 9, 33, 9, 33, 9, 33, 5, 33,
    1, 33, 7, 7, 3, 3, 5, 33, 5, 33, 1, 1, 9, 33, 10, 33, 11, 32, 12, 32, 13, 32, 30, 31, 31, 31,
    11, 33, 11, 33, 11, 33, 1, 1, 11, 33, 10, 33, 10, 33, 10, 33, 9, 33, 9, 33, 9, 33, 5, 33, 1,
    33, 3, 3, 7, 7, 1, 33, 5, 33, 26, 27, 27, 27, 1, 4, 35, 37, 34, 37, 34, 36, 0, 0,
];
static _use_syllable_machine_char_class: [i8; 55] = [
    0, 1, 2, 2, 3, 4, 2, 2, 2, 2, 2, 5, 6, 7, 2, 2, 2, 2, 8, 2, 2, 2, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 2, 23, 24, 25, 2, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
    0, 0,
];
static _use_syllable_machine_index_offsets: [i16; 63] = [
    0, 37, 39, 40, 69, 98, 99, 124, 148, 170, 191, 211, 213, 214, 237, 260, 283, 284, 307, 331,
    355, 379, 404, 429, 454, 483, 516, 517, 518, 547, 576, 577, 602, 626, 648, 669, 689, 691, 692,
    715, 738, 761, 762, 785, 809, 833, 857, 882, 907, 932, 961, 994, 995, 996, 1029, 1058, 1060,
    1061, 1065, 1068, 1072, 0, 0,
];
static _use_syllable_machine_indices: [i8; 1077] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 10, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 7, 33, 3, 34, 1, 36, 36, 38, 39, 37, 37, 40, 41, 42, 43, 44, 45,
    46, 40, 47, 2, 48, 49, 50, 51, 52, 53, 54, 37, 37, 37, 55, 56, 57, 58, 39, 38, 39, 37, 37, 40,
    41, 42, 43, 44, 45, 46, 40, 47, 48, 48, 49, 50, 51, 52, 53, 54, 37, 37, 37, 55, 56, 57, 58, 39,
    38, 40, 41, 42, 43, 44, 37, 37, 37, 37, 37, 37, 49, 50, 51, 52, 53, 54, 37, 37, 37, 41, 56, 57,
    58, 60, 41, 42, 43, 44, 37, 37, 37, 37, 37, 37, 37, 37, 37, 52, 53, 54, 37, 37, 37, 37, 56, 57,
    58, 60, 42, 43, 44, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 56, 57, 58,
    43, 44, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 56, 57, 58, 44, 37, 37,
    37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 56, 57, 58, 56, 57, 57, 42, 43, 44, 37,
    37, 37, 37, 37, 37, 37, 37, 37, 52, 53, 54, 37, 37, 37, 37, 56, 57, 58, 60, 42, 43, 44, 37, 37,
    37, 37, 37, 37, 37, 37, 37, 37, 53, 54, 37, 37, 37, 37, 56, 57, 58, 60, 42, 43, 44, 37, 37, 37,
    37, 37, 37, 37, 37, 37, 37, 37, 54, 37, 37, 37, 37, 56, 57, 58, 60, 62, 42, 43, 44, 37, 37, 37,
    37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 56, 57, 58, 60, 41, 42, 43, 44, 37, 37, 37,
    37, 37, 37, 49, 50, 51, 52, 53, 54, 37, 37, 37, 41, 56, 57, 58, 60, 41, 42, 43, 44, 37, 37, 37,
    37, 37, 37, 37, 50, 51, 52, 53, 54, 37, 37, 37, 41, 56, 57, 58, 60, 41, 42, 43, 44, 37, 37, 37,
    37, 37, 37, 37, 37, 51, 52, 53, 54, 37, 37, 37, 41, 56, 57, 58, 60, 40, 41, 42, 43, 44, 37, 46,
    40, 37, 37, 37, 49, 50, 51, 52, 53, 54, 37, 37, 37, 41, 56, 57, 58, 60, 40, 41, 42, 43, 44, 37,
    37, 40, 37, 37, 37, 49, 50, 51, 52, 53, 54, 37, 37, 37, 41, 56, 57, 58, 60, 40, 41, 42, 43, 44,
    45, 46, 40, 37, 37, 37, 49, 50, 51, 52, 53, 54, 37, 37, 37, 41, 56, 57, 58, 60, 38, 39, 37, 37,
    40, 41, 42, 43, 44, 45, 46, 40, 47, 37, 48, 49, 50, 51, 52, 53, 54, 37, 37, 37, 55, 56, 57, 58,
    39, 38, 59, 59, 59, 59, 59, 59, 59, 59, 41, 42, 43, 44, 59, 59, 59, 59, 59, 59, 59, 59, 59, 52,
    53, 54, 59, 59, 59, 59, 56, 57, 58, 60, 64, 4, 38, 39, 37, 37, 40, 41, 42, 43, 44, 45, 46, 40,
    47, 2, 48, 49, 50, 51, 52, 53, 54, 1, 36, 37, 55, 56, 57, 58, 39, 6, 7, 66, 66, 10, 11, 12, 13,
    14, 15, 16, 10, 17, 19, 19, 20, 21, 22, 23, 24, 25, 66, 66, 66, 29, 30, 31, 32, 7, 6, 10, 11,
    12, 13, 14, 66, 66, 66, 66, 66, 66, 20, 21, 22, 23, 24, 25, 66, 66, 66, 11, 30, 31, 32, 67, 11,
    12, 13, 14, 66, 66, 66, 66, 66, 66, 66, 66, 66, 23, 24, 25, 66, 66, 66, 66, 30, 31, 32, 67, 12,
    13, 14, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 30, 31, 32, 13, 14, 66,
    66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 30, 31, 32, 14, 66, 66, 66, 66, 66,
    66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 30, 31, 32, 30, 31, 31, 12, 13, 14, 66, 66, 66, 66,
    66, 66, 66, 66, 66, 23, 24, 25, 66, 66, 66, 66, 30, 31, 32, 67, 12, 13, 14, 66, 66, 66, 66, 66,
    66, 66, 66, 66, 66, 24, 25, 66, 66, 66, 66, 30, 31, 32, 67, 12, 13, 14, 66, 66, 66, 66, 66, 66,
    66, 66, 66, 66, 66, 25, 66, 66, 66, 66, 30, 31, 32, 67, 68, 12, 13, 14, 66, 66, 66, 66, 66, 66,
    66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 30, 31, 32, 67, 11, 12, 13, 14, 66, 66, 66, 66, 66, 66,
    20, 21, 22, 23, 24, 25, 66, 66, 66, 11, 30, 31, 32, 67, 11, 12, 13, 14, 66, 66, 66, 66, 66, 66,
    66, 21, 22, 23, 24, 25, 66, 66, 66, 11, 30, 31, 32, 67, 11, 12, 13, 14, 66, 66, 66, 66, 66, 66,
    66, 66, 22, 23, 24, 25, 66, 66, 66, 11, 30, 31, 32, 67, 10, 11, 12, 13, 14, 66, 16, 10, 66, 66,
    66, 20, 21, 22, 23, 24, 25, 66, 66, 66, 11, 30, 31, 32, 67, 10, 11, 12, 13, 14, 66, 66, 10, 66,
    66, 66, 20, 21, 22, 23, 24, 25, 66, 66, 66, 11, 30, 31, 32, 67, 10, 11, 12, 13, 14, 15, 16, 10,
    66, 66, 66, 20, 21, 22, 23, 24, 25, 66, 66, 66, 11, 30, 31, 32, 67, 6, 7, 66, 66, 10, 11, 12,
    13, 14, 15, 16, 10, 17, 66, 19, 20, 21, 22, 23, 24, 25, 66, 66, 66, 29, 30, 31, 32, 7, 6, 66,
    66, 66, 66, 66, 66, 66, 66, 11, 12, 13, 14, 66, 66, 66, 66, 66, 66, 66, 66, 66, 23, 24, 25, 66,
    66, 66, 66, 30, 31, 32, 67, 69, 8, 2, 66, 66, 2, 6, 7, 8, 66, 10, 11, 12, 13, 14, 15, 16, 10,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 66, 29, 30, 31, 32, 7, 6, 7, 66, 66, 10, 11, 12,
    13, 14, 15, 16, 10, 17, 18, 19, 20, 21, 22, 23, 24, 25, 66, 66, 66, 29, 30, 31, 32, 7, 26, 27,
    27, 2, 70, 70, 2, 72, 71, 33, 33, 72, 71, 72, 33, 71, 34, 0, 0,
];
static _use_syllable_machine_index_defaults: [i8; 63] = [
    3, 35, 35, 37, 37, 59, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 61, 37, 37, 37, 37, 37, 37, 37,
    37, 59, 63, 65, 37, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
    66, 66, 66, 66, 66, 66, 66, 66, 66, 70, 71, 71, 71, 0, 0,
];
static _use_syllable_machine_cond_targs: [i8; 75] = [
    0, 1, 3, 0, 26, 28, 29, 30, 51, 53, 31, 32, 33, 34, 35, 46, 47, 48, 54, 49, 43, 44, 45, 38, 39,
    40, 55, 56, 57, 50, 36, 37, 0, 58, 60, 0, 2, 0, 4, 5, 6, 7, 8, 9, 10, 21, 22, 23, 24, 18, 19,
    20, 13, 14, 15, 25, 11, 12, 0, 0, 16, 0, 17, 0, 27, 0, 0, 41, 42, 52, 0, 0, 59, 0, 0,
];
static _use_syllable_machine_cond_actions: [i8; 75] = [
    0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    4, 0, 0, 5, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 8, 0, 9, 0,
    10, 0, 11, 12, 0, 0, 0, 13, 14, 0, 0, 0,
];
static _use_syllable_machine_to_state_actions: [i8; 63] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _use_syllable_machine_from_state_actions: [i8; 63] = [
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _use_syllable_machine_eof_trans: [i8; 63] = [
    1, 36, 36, 38, 38, 60, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 62, 38, 38, 38, 38, 38, 38, 38,
    38, 60, 64, 66, 38, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67,
    67, 67, 67, 67, 67, 67, 67, 67, 67, 71, 72, 72, 72, 0, 0,
];
static use_syllable_machine_start: i32 = 0;
static use_syllable_machine_first_final: i32 = 0;
static use_syllable_machine_error: i32 = -1;
static use_syllable_machine_en_main: i32 = 0;
#[derive(Clone, Copy)]
pub enum SyllableType {
    IndependentCluster,
    ViramaTerminatedCluster,
    SakotTerminatedCluster,
    StandardCluster,
    NumberJoinerTerminatedCluster,
    NumeralCluster,
    SymbolCluster,
    HieroglyphCluster,
    BrokenCluster,
    NonCluster,
}

pub fn find_syllables(buffer: &mut Buffer) {
    let mut cs = 0;
    let infos = Cell::as_slice_of_cells(Cell::from_mut(&mut buffer.info));
    let p0 = MachineCursor::new(infos, included);
    let mut p = p0;
    let mut ts = p0;
    let mut te = p0;
    let pe = p.end();
    let eof = p.end();
    let mut syllable_serial = 1u8;

    // Please manually replace assignments of 0 to p, ts, and te
    // to use p0 instead

    macro_rules! found_syllable {
        ($kind:expr) => {{
            found_syllable(ts.index(), te.index(), &mut syllable_serial, $kind, infos);
        }};
    }

    {
        cs = (use_syllable_machine_start) as i32;
        ts = p0;
        te = p0;
    }

    {
        let mut _trans = 0;
        let mut _keys: i32 = 0;
        let mut _inds: i32 = 0;
        let mut _ic = 0;
        '_resume: while (p != pe || p == eof) {
            '_again: while (true) {
                match (_use_syllable_machine_from_state_actions[(cs) as usize]) {
                    2 => {
                        ts = p;
                    }

                    _ => {}
                }
                if (p == eof) {
                    {
                        if (_use_syllable_machine_eof_trans[(cs) as usize] > 0) {
                            {
                                _trans =
                                    (_use_syllable_machine_eof_trans[(cs) as usize]) as u32 - 1;
                            }
                        }
                    }
                } else {
                    {
                        _keys = (cs << 1) as i32;
                        _inds = (_use_syllable_machine_index_offsets[(cs) as usize]) as i32;
                        if ((infos[p.index()].get().use_category() as u8) <= 52) {
                            {
                                _ic = (_use_syllable_machine_char_class[((infos[p.index()]
                                    .get()
                                    .use_category()
                                    as u8)
                                    as i32
                                    - 0)
                                    as usize]) as i32;
                                if (_ic
                                    <= (_use_syllable_machine_trans_keys[(_keys + 1) as usize])
                                        as i32
                                    && _ic
                                        >= (_use_syllable_machine_trans_keys[(_keys) as usize])
                                            as i32)
                                {
                                    _trans = (_use_syllable_machine_indices[(_inds
                                        + (_ic
                                            - (_use_syllable_machine_trans_keys[(_keys) as usize])
                                                as i32)
                                            as i32)
                                        as usize])
                                        as u32;
                                } else {
                                    _trans = (_use_syllable_machine_index_defaults[(cs) as usize])
                                        as u32;
                                }
                            }
                        } else {
                            {
                                _trans =
                                    (_use_syllable_machine_index_defaults[(cs) as usize]) as u32;
                            }
                        }
                    }
                }
                cs = (_use_syllable_machine_cond_targs[(_trans) as usize]) as i32;
                if (_use_syllable_machine_cond_actions[(_trans) as usize] != 0) {
                    {
                        match (_use_syllable_machine_cond_actions[(_trans) as usize]) {
                            7 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::StandardCluster);
                                }
                            }
                            4 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                }
                            }
                            3 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::NonCluster);
                                }
                            }
                            8 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::ViramaTerminatedCluster);
                                }
                            }
                            9 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::SakotTerminatedCluster);
                                }
                            }
                            6 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::StandardCluster);
                                }
                            }
                            11 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::NumberJoinerTerminatedCluster);
                                }
                            }
                            10 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::NumeralCluster);
                                }
                            }
                            5 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::SymbolCluster);
                                }
                            }
                            14 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::HieroglyphCluster);
                                }
                            }
                            12 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                }
                            }
                            13 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::NonCluster);
                                }
                            }

                            _ => {}
                        }
                    }
                }
                break '_again;
            }
            if (p == eof) {
                {
                    if (cs >= 0) {
                        break '_resume;
                    }
                }
            } else {
                {
                    match (_use_syllable_machine_to_state_actions[(cs) as usize]) {
                        1 => {
                            ts = p0;
                        }

                        _ => {}
                    }
                    p += 1;
                    continue '_resume;
                }
            }
            break '_resume;
        }
    }
}

#[inline]
fn found_syllable(
    start: usize,
    end: usize,
    syllable_serial: &mut u8,
    kind: SyllableType,
    buffer: &[Cell<GlyphInfo>],
) {
    for i in start..end {
        let mut glyph = buffer[i].get();
        glyph.set_syllable((*syllable_serial << 4) | kind as u8);
        buffer[i].set(glyph);
    }

    *syllable_serial += 1;

    if *syllable_serial == 16 {
        *syllable_serial = 1;
    }
}

fn not_ccs_default_ignorable(i: &GlyphInfo) -> bool {
    !(matches!(i.use_category(), category::CGJ | category::RSV) && i.is_default_ignorable())
}

fn included(infos: &[Cell<GlyphInfo>], i: usize) -> bool {
    let glyph = infos[i].get();
    if !not_ccs_default_ignorable(&glyph) {
        return false;
    }
    if glyph.use_category() == category::ZWNJ {
        for glyph2 in &infos[i + 1..] {
            if not_ccs_default_ignorable(&glyph2.get()) {
                return !glyph2.get().is_unicode_mark();
            }
        }
    }
    true
}
