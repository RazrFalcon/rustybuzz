use core::cell::Cell;

use alloc::vec::Vec;

use crate::{buffer::Buffer, GlyphInfo};

use super::universal::category;

const MACHINE_TRANS_KEYS: &[u8] = &[
    12, 48, 1, 15, 1, 1, 12, 48, 1, 1, 0, 48, 11, 48, 11, 48, 1, 15, 1, 1, 22, 48, 23, 48, 24, 47,
    25, 47, 26, 47, 45, 46, 46, 46, 24, 48, 24, 48, 24, 48, 1, 1, 24, 48, 23, 48, 23, 48, 23, 48,
    22, 48, 22, 48, 22, 48, 22, 48, 11, 48, 1, 48, 13, 13, 4, 4, 11, 48, 41, 42, 42, 42, 11, 48,
    22, 48, 23, 48, 24, 47, 25, 47, 26, 47, 45, 46, 46, 46, 24, 48, 24, 48, 24, 48, 24, 48, 23, 48,
    23, 48, 23, 48, 22, 48, 22, 48, 22, 48, 22, 48, 11, 48, 1, 48, 1, 15, 4, 4, 13, 13, 12, 48, 1,
    48, 11, 48, 41, 42, 42, 42, 1, 5, 0,
];

const MACHINE_KEY_SPANS: &[u8] = &[
    37, 15, 1, 37, 1, 49, 38, 38, 15, 1, 27, 26, 24, 23, 22, 2, 1, 25, 25, 25, 1, 25, 26, 26, 26,
    27, 27, 27, 27, 38, 48, 1, 1, 38, 2, 1, 38, 27, 26, 24, 23, 22, 2, 1, 25, 25, 25, 25, 26, 26,
    26, 27, 27, 27, 27, 38, 48, 15, 1, 1, 37, 48, 38, 2, 1, 5,
];

const MACHINE_INDEX_OFFSETS: &[u16] = &[
    0, 38, 54, 56, 94, 96, 146, 185, 224, 240, 242, 270, 297, 322, 346, 369, 372, 374, 400, 426,
    452, 454, 480, 507, 534, 561, 589, 617, 645, 673, 712, 761, 763, 765, 804, 807, 809, 848, 876,
    903, 928, 952, 975, 978, 980, 1006, 1032, 1058, 1084, 1111, 1138, 1165, 1193, 1221, 1249, 1277,
    1316, 1365, 1381, 1383, 1385, 1423, 1472, 1511, 1514, 1516,
];

const MACHINE_INDICIES: &[u8] = &[
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 1, 0, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 4, 2, 3, 2, 6, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 6, 5, 7, 5,
    8, 9, 10, 8, 11, 12, 10, 10, 10, 10, 10, 3, 13, 14, 10, 15, 8, 8, 16, 17, 10, 10, 18, 19, 20,
    21, 22, 23, 24, 18, 25, 26, 27, 28, 29, 30, 10, 31, 32, 33, 10, 34, 35, 36, 37, 38, 39, 40, 13,
    10, 42, 1, 41, 41, 43, 41, 41, 41, 41, 41, 41, 44, 45, 46, 47, 48, 49, 50, 44, 51, 9, 52, 53,
    54, 55, 41, 56, 57, 58, 41, 41, 41, 41, 59, 60, 61, 62, 1, 41, 42, 1, 41, 41, 43, 41, 41, 41,
    41, 41, 41, 44, 45, 46, 47, 48, 49, 50, 44, 51, 52, 52, 53, 54, 55, 41, 56, 57, 58, 41, 41, 41,
    41, 59, 60, 61, 62, 1, 41, 42, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 64, 63, 42,
    63, 44, 45, 46, 47, 48, 41, 41, 41, 41, 41, 41, 53, 54, 55, 41, 56, 57, 58, 41, 41, 41, 41, 45,
    60, 61, 62, 65, 41, 45, 46, 47, 48, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 56, 57, 58, 41, 41,
    41, 41, 41, 60, 61, 62, 65, 41, 46, 47, 48, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41,
    41, 41, 41, 41, 41, 60, 61, 62, 41, 47, 48, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41,
    41, 41, 41, 41, 41, 60, 61, 62, 41, 48, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41,
    41, 41, 41, 41, 60, 61, 62, 41, 60, 61, 41, 61, 41, 46, 47, 48, 41, 41, 41, 41, 41, 41, 41, 41,
    41, 41, 56, 57, 58, 41, 41, 41, 41, 41, 60, 61, 62, 65, 41, 46, 47, 48, 41, 41, 41, 41, 41, 41,
    41, 41, 41, 41, 41, 57, 58, 41, 41, 41, 41, 41, 60, 61, 62, 65, 41, 46, 47, 48, 41, 41, 41, 41,
    41, 41, 41, 41, 41, 41, 41, 41, 58, 41, 41, 41, 41, 41, 60, 61, 62, 65, 41, 67, 66, 46, 47, 48,
    41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 60, 61, 62, 65, 41, 45,
    46, 47, 48, 41, 41, 41, 41, 41, 41, 53, 54, 55, 41, 56, 57, 58, 41, 41, 41, 41, 45, 60, 61, 62,
    65, 41, 45, 46, 47, 48, 41, 41, 41, 41, 41, 41, 41, 54, 55, 41, 56, 57, 58, 41, 41, 41, 41, 45,
    60, 61, 62, 65, 41, 45, 46, 47, 48, 41, 41, 41, 41, 41, 41, 41, 41, 55, 41, 56, 57, 58, 41, 41,
    41, 41, 45, 60, 61, 62, 65, 41, 44, 45, 46, 47, 48, 41, 50, 44, 41, 41, 41, 53, 54, 55, 41, 56,
    57, 58, 41, 41, 41, 41, 45, 60, 61, 62, 65, 41, 44, 45, 46, 47, 48, 41, 68, 44, 41, 41, 41, 53,
    54, 55, 41, 56, 57, 58, 41, 41, 41, 41, 45, 60, 61, 62, 65, 41, 44, 45, 46, 47, 48, 41, 41, 44,
    41, 41, 41, 53, 54, 55, 41, 56, 57, 58, 41, 41, 41, 41, 45, 60, 61, 62, 65, 41, 44, 45, 46, 47,
    48, 49, 50, 44, 41, 41, 41, 53, 54, 55, 41, 56, 57, 58, 41, 41, 41, 41, 45, 60, 61, 62, 65, 41,
    42, 1, 41, 41, 43, 41, 41, 41, 41, 41, 41, 44, 45, 46, 47, 48, 49, 50, 44, 51, 41, 52, 53, 54,
    55, 41, 56, 57, 58, 41, 41, 41, 41, 59, 60, 61, 62, 1, 41, 42, 63, 63, 63, 63, 63, 63, 63, 63,
    63, 63, 63, 63, 63, 64, 63, 63, 63, 63, 63, 63, 63, 45, 46, 47, 48, 63, 63, 63, 63, 63, 63, 63,
    63, 63, 63, 56, 57, 58, 63, 63, 63, 63, 63, 60, 61, 62, 65, 63, 70, 69, 11, 71, 42, 1, 41, 41,
    43, 41, 41, 41, 41, 41, 41, 44, 45, 46, 47, 48, 49, 50, 44, 51, 9, 52, 53, 54, 55, 41, 56, 57,
    58, 41, 17, 72, 41, 59, 60, 61, 62, 1, 41, 17, 72, 73, 72, 73, 3, 6, 74, 74, 75, 74, 74, 74,
    74, 74, 74, 18, 19, 20, 21, 22, 23, 24, 18, 25, 27, 27, 28, 29, 30, 74, 31, 32, 33, 74, 74, 74,
    74, 37, 38, 39, 40, 6, 74, 18, 19, 20, 21, 22, 74, 74, 74, 74, 74, 74, 28, 29, 30, 74, 31, 32,
    33, 74, 74, 74, 74, 19, 38, 39, 40, 76, 74, 19, 20, 21, 22, 74, 74, 74, 74, 74, 74, 74, 74, 74,
    74, 31, 32, 33, 74, 74, 74, 74, 74, 38, 39, 40, 76, 74, 20, 21, 22, 74, 74, 74, 74, 74, 74, 74,
    74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 38, 39, 40, 74, 21, 22, 74, 74, 74, 74, 74, 74, 74,
    74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 38, 39, 40, 74, 22, 74, 74, 74, 74, 74, 74, 74, 74,
    74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 38, 39, 40, 74, 38, 39, 74, 39, 74, 20, 21, 22, 74, 74,
    74, 74, 74, 74, 74, 74, 74, 74, 31, 32, 33, 74, 74, 74, 74, 74, 38, 39, 40, 76, 74, 20, 21, 22,
    74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 32, 33, 74, 74, 74, 74, 74, 38, 39, 40, 76, 74, 20,
    21, 22, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 33, 74, 74, 74, 74, 74, 38, 39, 40, 76,
    74, 20, 21, 22, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 38, 39,
    40, 76, 74, 19, 20, 21, 22, 74, 74, 74, 74, 74, 74, 28, 29, 30, 74, 31, 32, 33, 74, 74, 74, 74,
    19, 38, 39, 40, 76, 74, 19, 20, 21, 22, 74, 74, 74, 74, 74, 74, 74, 29, 30, 74, 31, 32, 33, 74,
    74, 74, 74, 19, 38, 39, 40, 76, 74, 19, 20, 21, 22, 74, 74, 74, 74, 74, 74, 74, 74, 30, 74, 31,
    32, 33, 74, 74, 74, 74, 19, 38, 39, 40, 76, 74, 18, 19, 20, 21, 22, 74, 24, 18, 74, 74, 74, 28,
    29, 30, 74, 31, 32, 33, 74, 74, 74, 74, 19, 38, 39, 40, 76, 74, 18, 19, 20, 21, 22, 74, 77, 18,
    74, 74, 74, 28, 29, 30, 74, 31, 32, 33, 74, 74, 74, 74, 19, 38, 39, 40, 76, 74, 18, 19, 20, 21,
    22, 74, 74, 18, 74, 74, 74, 28, 29, 30, 74, 31, 32, 33, 74, 74, 74, 74, 19, 38, 39, 40, 76, 74,
    18, 19, 20, 21, 22, 23, 24, 18, 74, 74, 74, 28, 29, 30, 74, 31, 32, 33, 74, 74, 74, 74, 19, 38,
    39, 40, 76, 74, 3, 6, 74, 74, 75, 74, 74, 74, 74, 74, 74, 18, 19, 20, 21, 22, 23, 24, 18, 25,
    74, 27, 28, 29, 30, 74, 31, 32, 33, 74, 74, 74, 74, 37, 38, 39, 40, 6, 74, 3, 74, 74, 74, 74,
    74, 74, 74, 74, 74, 74, 74, 74, 74, 4, 74, 74, 74, 74, 74, 74, 74, 19, 20, 21, 22, 74, 74, 74,
    74, 74, 74, 74, 74, 74, 74, 31, 32, 33, 74, 74, 74, 74, 74, 38, 39, 40, 76, 74, 3, 78, 78, 78,
    78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 4, 78, 79, 74, 14, 74, 6, 78, 78, 78, 78, 78, 78, 78,
    78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78,
    6, 78, 78, 78, 6, 78, 9, 74, 74, 74, 9, 74, 74, 74, 74, 74, 3, 6, 14, 74, 75, 74, 74, 74, 74,
    74, 74, 18, 19, 20, 21, 22, 23, 24, 18, 25, 26, 27, 28, 29, 30, 74, 31, 32, 33, 74, 34, 35, 74,
    37, 38, 39, 40, 6, 74, 3, 6, 74, 74, 75, 74, 74, 74, 74, 74, 74, 18, 19, 20, 21, 22, 23, 24,
    18, 25, 26, 27, 28, 29, 30, 74, 31, 32, 33, 74, 74, 74, 74, 37, 38, 39, 40, 6, 74, 34, 35, 74,
    35, 74, 9, 78, 78, 78, 9, 78, 0,
];

const MACHINE_TRANS_TARGS: &[u8] = &[
    5, 8, 5, 36, 2, 5, 1, 47, 5, 6, 5, 31, 33, 57, 58, 60, 61, 34, 37, 38, 39, 40, 41, 51, 52, 54,
    62, 55, 48, 49, 50, 44, 45, 46, 63, 64, 65, 56, 42, 43, 5, 5, 7, 0, 10, 11, 12, 13, 14, 25, 26,
    28, 29, 22, 23, 24, 17, 18, 19, 30, 15, 16, 5, 5, 9, 20, 5, 21, 27, 5, 32, 5, 35, 5, 5, 3, 4,
    53, 5, 59,
];

const MACHINE_TRANS_ACTIONS: &[u8] = &[
    1, 0, 2, 3, 0, 4, 0, 5, 8, 5, 9, 0, 5, 10, 0, 10, 3, 0, 5, 5, 0, 0, 0, 5, 5, 5, 3, 3, 5, 5, 5,
    5, 5, 5, 0, 0, 0, 3, 0, 0, 11, 12, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    13, 14, 0, 0, 15, 0, 0, 16, 0, 17, 0, 18, 19, 0, 0, 5, 20, 0,
];

const MACHINE_TO_STATE_ACTIONS: &[u8] = &[
    0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0,
];

const MACHINE_FROM_STATE_ACTIONS: &[u8] = &[
    0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0,
];

const MACHINE_EOF_TRANS: &[u8] = &[
    1, 3, 3, 6, 6, 0, 42, 42, 64, 64, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 67, 42, 42, 42, 42,
    42, 42, 42, 42, 42, 64, 70, 72, 42, 74, 74, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 79, 75, 75, 79, 75, 75, 75, 75, 79,
];

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

fn not_standard_default_ignorable(i: &GlyphInfo) -> bool {
    !(matches!(i.use_category(), category::O | category::RSV) && i.is_default_ignorable())
}

pub fn find_syllables(buffer: &mut Buffer) {
    let mut cs = 5usize;
    let mut ts = 0;
    let mut te = 0;
    let mut act = 0;
    let infos = Cell::as_slice_of_cells(Cell::from_mut(&mut buffer.info));
    let included = |i: usize| {
        let glyph = &infos[i].get();
        if !not_standard_default_ignorable(glyph) {
            return false;
        }
        if glyph.use_category() == category::ZWNJ {
            for glyph2 in &infos[i + 1..] {
                if not_standard_default_ignorable(&glyph2.get()) {
                    return !glyph2.get().is_unicode_mark();
                }
            }
        }
        true
    };
    let eof = infos.len();
    let next_glyph = |p: usize| {
        let mut q = p + 1;
        while q < eof && !included(q) {
            q += 1;
        }
        q
    };
    let first_glyph = (0..eof).find(|i| included(*i)).unwrap_or(eof);
    let prev_glyph = |p: usize| {
        let mut q = p - 1;
        while !included(q) {
            q -= 1;
        }
        q
    };
    let mut p = first_glyph;
    let pe = eof;
    let mut syllable_serial = 1u8;
    let mut reset = true;
    let mut slen;
    let mut trans = 0;
    if p == pe {
        if MACHINE_EOF_TRANS[cs] > 0 {
            trans = (MACHINE_EOF_TRANS[cs] - 1) as usize;
        }
    }

    loop {
        if reset {
            if MACHINE_FROM_STATE_ACTIONS[cs] == 7 {
                ts = p;
            }

            slen = MACHINE_KEY_SPANS[cs] as usize;
            let cs_idx = ((cs as i32) << 1) as usize;
            let glyph = &infos[p].get();
            let i = if slen > 0
                && MACHINE_TRANS_KEYS[cs_idx] <= glyph.indic_category() as u8
                && glyph.indic_category() as u8 <= MACHINE_TRANS_KEYS[cs_idx + 1]
            {
                (glyph.indic_category() as u8 - MACHINE_TRANS_KEYS[cs_idx]) as usize
            } else {
                slen
            };
            trans = MACHINE_INDICIES[MACHINE_INDEX_OFFSETS[cs] as usize + i] as usize;
        }
        reset = true;

        cs = MACHINE_TRANS_TARGS[trans] as usize;

        if MACHINE_TRANS_ACTIONS[trans] != 0 {
            match MACHINE_TRANS_ACTIONS[trans] {
                5 => {
                    te = next_glyph(p);
                }
                8 => {
                    te = next_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::IndependentCluster,
                        infos,
                    );
                }
                13 => {
                    te = next_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::StandardCluster,
                        infos,
                    );
                }
                11 => {
                    te = next_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::BrokenCluster,
                        infos,
                    );
                }
                9 => {
                    te = next_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::NonCluster,
                        infos,
                    );
                }
                14 => {
                    te = p;
                    p = prev_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::ViramaTerminatedCluster,
                        infos,
                    );
                }
                15 => {
                    te = p;
                    p = prev_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::SakotTerminatedCluster,
                        infos,
                    );
                }
                12 => {
                    te = p;
                    p = prev_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::StandardCluster,
                        infos,
                    );
                }
                17 => {
                    te = p;
                    p = prev_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::NumberJoinerTerminatedCluster,
                        infos,
                    );
                }
                16 => {
                    te = p;
                    p = prev_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::NumeralCluster,
                        infos,
                    );
                }
                18 => {
                    te = p;
                    p = prev_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::SymbolCluster,
                        infos,
                    );
                }
                19 => {
                    te = p;
                    p = prev_glyph(p);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::BrokenCluster,
                        infos,
                    );
                }
                1 => {
                    p = prev_glyph(te);
                    found_syllable(
                        ts,
                        te,
                        &mut syllable_serial,
                        SyllableType::StandardCluster,
                        infos,
                    );
                }
                2 => match act {
                    8 => {
                        p = prev_glyph(te);
                        found_syllable(
                            ts,
                            te,
                            &mut syllable_serial,
                            SyllableType::BrokenCluster,
                            infos,
                        );
                    }
                    9 => {
                        p = prev_glyph(te);
                        found_syllable(
                            ts,
                            te,
                            &mut syllable_serial,
                            SyllableType::NonCluster,
                            infos,
                        );
                    }
                    _ => {}
                },
                3 => {
                    te = next_glyph(p);
                    act = 8;
                }
                10 => {
                    te = next_glyph(p);
                    act = 9;
                }
                _ => {}
            }
        }

        if MACHINE_TO_STATE_ACTIONS[cs] == 6 {
            ts = 0;
        }

        p = next_glyph(p);
        if p != pe {
            continue;
        }

        if p == eof {
            if MACHINE_EOF_TRANS[cs] > 0 {
                trans = (MACHINE_EOF_TRANS[cs] - 1) as usize;
                reset = false;
                continue;
            }
        }

        break;
    }
}

#[inline]
fn found_syllable(
    start: usize,
    end: usize,
    syllable_serial: &mut u8,
    kind: SyllableType,
    infos: &[Cell<GlyphInfo>],
) {
    for i in start..end {
        let mut info = infos[i].get();
        info.set_syllable((*syllable_serial << 4) | kind as u8);
        infos[i].set(info);
    }

    *syllable_serial += 1;

    if *syllable_serial == 16 {
        *syllable_serial = 1;
    }
}
