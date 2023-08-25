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

static _myanmar_syllable_machine_trans_keys: [u8; 106] = [
    0, 21, 1, 20, 3, 19, 3, 5, 3, 19, 1, 15, 3, 15, 3, 15, 1, 19, 1, 19, 1, 19, 1, 19, 0, 8, 1, 19,
    1, 19, 1, 19, 1, 19, 1, 19, 1, 20, 1, 19, 1, 19, 1, 19, 1, 19, 1, 19, 3, 19, 3, 5, 3, 19, 1,
    15, 3, 15, 3, 15, 1, 19, 1, 19, 1, 19, 1, 19, 0, 8, 1, 20, 1, 19, 1, 19, 1, 19, 1, 19, 1, 19,
    1, 20, 1, 19, 1, 19, 1, 19, 1, 19, 1, 19, 1, 20, 1, 19, 0, 20, 0, 8, 5, 5, 0, 0,
];
static _myanmar_syllable_machine_char_class: [i8; 34] = [
    0, 0, 1, 2, 3, 3, 4, 5, 4, 6, 7, 4, 4, 4, 4, 8, 4, 9, 10, 4, 11, 12, 13, 14, 15, 16, 17, 18,
    19, 20, 21, 7, 0, 0,
];
static _myanmar_syllable_machine_index_offsets: [i16; 54] = [
    0, 22, 42, 59, 62, 79, 94, 107, 120, 139, 158, 177, 196, 205, 224, 243, 262, 281, 300, 320,
    339, 358, 377, 396, 415, 432, 435, 452, 467, 480, 493, 512, 531, 550, 569, 578, 598, 617, 636,
    655, 674, 693, 713, 732, 751, 770, 789, 808, 828, 847, 868, 877, 0, 0,
];
static _myanmar_syllable_machine_indices: [i8; 880] = [
    2, 3, 4, 5, 1, 6, 7, 2, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25, 22,
    26, 27, 22, 22, 28, 22, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 25, 22, 26, 22, 22, 22, 39, 22,
    22, 22, 22, 22, 33, 22, 22, 22, 37, 25, 22, 26, 25, 22, 26, 22, 22, 22, 22, 22, 22, 22, 22, 22,
    33, 22, 22, 22, 37, 40, 22, 25, 22, 26, 33, 22, 22, 41, 22, 22, 22, 22, 22, 33, 25, 22, 26, 22,
    22, 22, 41, 22, 22, 22, 22, 22, 33, 25, 22, 26, 22, 22, 22, 22, 22, 22, 22, 22, 22, 33, 23, 22,
    25, 22, 26, 27, 22, 22, 42, 22, 42, 22, 22, 22, 33, 43, 22, 22, 37, 23, 22, 25, 22, 26, 27, 22,
    22, 22, 22, 22, 22, 22, 22, 33, 22, 22, 22, 37, 23, 22, 25, 22, 26, 27, 22, 22, 42, 22, 22, 22,
    22, 22, 33, 43, 22, 22, 37, 23, 22, 25, 22, 26, 27, 22, 22, 22, 22, 22, 22, 22, 22, 33, 43, 22,
    22, 37, 2, 22, 22, 22, 22, 22, 22, 22, 2, 23, 22, 25, 22, 26, 27, 22, 22, 28, 22, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 23, 22, 25, 22, 26, 27, 22, 22, 44, 22, 22, 22, 22, 22, 33, 34, 35, 36,
    37, 23, 22, 25, 22, 26, 27, 22, 22, 22, 22, 22, 22, 22, 22, 33, 34, 35, 36, 37, 23, 22, 25, 22,
    26, 27, 22, 22, 22, 22, 22, 22, 22, 22, 33, 34, 35, 22, 37, 23, 22, 25, 22, 26, 27, 22, 22, 22,
    22, 22, 22, 22, 22, 33, 22, 35, 22, 37, 23, 22, 25, 22, 26, 27, 22, 22, 22, 22, 22, 22, 22, 22,
    33, 34, 35, 36, 37, 44, 23, 22, 25, 22, 26, 27, 22, 22, 22, 22, 29, 22, 31, 22, 33, 34, 35, 36,
    37, 23, 22, 25, 22, 26, 27, 22, 22, 44, 22, 29, 22, 22, 22, 33, 34, 35, 36, 37, 23, 22, 25, 22,
    26, 27, 22, 22, 45, 22, 29, 30, 31, 22, 33, 34, 35, 36, 37, 23, 22, 25, 22, 26, 27, 22, 22, 22,
    22, 29, 30, 31, 22, 33, 34, 35, 36, 37, 23, 24, 25, 22, 26, 27, 22, 22, 28, 22, 29, 30, 31, 32,
    33, 34, 35, 36, 37, 47, 46, 6, 46, 46, 46, 48, 46, 46, 46, 46, 46, 15, 46, 46, 46, 19, 47, 46,
    6, 47, 46, 6, 46, 46, 46, 46, 46, 46, 46, 46, 46, 15, 46, 46, 46, 19, 49, 46, 47, 46, 6, 15,
    46, 46, 50, 46, 46, 46, 46, 46, 15, 47, 46, 6, 46, 46, 46, 50, 46, 46, 46, 46, 46, 15, 47, 46,
    6, 46, 46, 46, 46, 46, 46, 46, 46, 46, 15, 3, 46, 47, 46, 6, 7, 46, 46, 51, 46, 51, 46, 46, 46,
    15, 52, 46, 46, 19, 3, 46, 47, 46, 6, 7, 46, 46, 46, 46, 46, 46, 46, 46, 15, 46, 46, 46, 19, 3,
    46, 47, 46, 6, 7, 46, 46, 51, 46, 46, 46, 46, 46, 15, 52, 46, 46, 19, 3, 46, 47, 46, 6, 7, 46,
    46, 46, 46, 46, 46, 46, 46, 15, 52, 46, 46, 19, 53, 46, 46, 46, 46, 46, 46, 46, 53, 3, 4, 47,
    46, 6, 7, 46, 46, 9, 46, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 3, 46, 47, 46, 6, 7, 46, 46,
    9, 46, 11, 12, 13, 14, 15, 16, 17, 18, 19, 3, 46, 47, 46, 6, 7, 46, 46, 54, 46, 46, 46, 46, 46,
    15, 16, 17, 18, 19, 3, 46, 47, 46, 6, 7, 46, 46, 46, 46, 46, 46, 46, 46, 15, 16, 17, 18, 19, 3,
    46, 47, 46, 6, 7, 46, 46, 46, 46, 46, 46, 46, 46, 15, 16, 17, 46, 19, 3, 46, 47, 46, 6, 7, 46,
    46, 46, 46, 46, 46, 46, 46, 15, 46, 17, 46, 19, 3, 46, 47, 46, 6, 7, 46, 46, 46, 46, 46, 46,
    46, 46, 15, 16, 17, 18, 19, 54, 3, 46, 47, 46, 6, 7, 46, 46, 46, 46, 11, 46, 13, 46, 15, 16,
    17, 18, 19, 3, 46, 47, 46, 6, 7, 46, 46, 54, 46, 11, 46, 46, 46, 15, 16, 17, 18, 19, 3, 46, 47,
    46, 6, 7, 46, 46, 55, 46, 11, 12, 13, 46, 15, 16, 17, 18, 19, 3, 46, 47, 46, 6, 7, 46, 46, 46,
    46, 11, 12, 13, 46, 15, 16, 17, 18, 19, 3, 4, 47, 46, 6, 7, 46, 46, 9, 46, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 23, 24, 25, 22, 26, 27, 22, 22, 56, 22, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38,
    23, 57, 25, 22, 26, 27, 22, 22, 28, 22, 29, 30, 31, 32, 33, 34, 35, 36, 37, 2, 3, 4, 47, 46, 6,
    7, 2, 2, 9, 46, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 2, 58, 58, 58, 58, 58, 58, 2, 2, 59, 0,
    0,
];
static _myanmar_syllable_machine_index_defaults: [i8; 54] = [
    1, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
    46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 22,
    22, 46, 58, 58, 0, 0,
];
static _myanmar_syllable_machine_cond_targs: [i8; 62] = [
    0, 0, 1, 24, 34, 0, 25, 31, 47, 36, 50, 37, 42, 43, 44, 27, 39, 40, 41, 30, 46, 51, 0, 2, 12,
    0, 3, 9, 13, 14, 19, 20, 21, 5, 16, 17, 18, 8, 23, 4, 6, 7, 10, 11, 15, 22, 0, 0, 26, 28, 29,
    32, 33, 35, 38, 45, 48, 49, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_cond_actions: [i8; 62] = [
    0, 3, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 6, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 10, 0, 0,
];
static _myanmar_syllable_machine_to_state_actions: [i8; 54] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_from_state_actions: [i8; 54] = [
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_eof_trans: [i8; 54] = [
    1, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
    47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 23,
    23, 47, 59, 59, 0, 0,
];
static myanmar_syllable_machine_start: i32 = 0;
static myanmar_syllable_machine_first_final: i32 = 0;
static myanmar_syllable_machine_error: i32 = -1;
static myanmar_syllable_machine_en_main: i32 = 0;
#[derive(Clone, Copy)]
pub enum SyllableType {
    ConsonantSyllable = 0,
    PunctuationCluster,
    BrokenCluster,
    NonMyanmarCluster,
}

pub fn find_syllables_myanmar(buffer: &mut Buffer) {
    let mut cs = 0;
    let mut ts = 0;
    let mut te;
    let mut p = 0;
    let pe = buffer.len;
    let eof = buffer.len;
    let mut syllable_serial = 1u8;

    macro_rules! found_syllable {
        ($kind:expr) => {{
            found_syllable(ts, te, &mut syllable_serial, $kind, buffer);
        }};
    }

    {
        cs = (myanmar_syllable_machine_start) as i32;
        ts = 0;
        te = 0;
    }

    {
        let mut _trans = 0;
        let mut _keys: i32 = 0;
        let mut _inds: i32 = 0;
        let mut _ic = 0;
        '_resume: while (p != pe || p == eof) {
            '_again: while (true) {
                match (_myanmar_syllable_machine_from_state_actions[(cs) as usize]) {
                    2 => {
                        ts = p;
                    }

                    _ => {}
                }
                if (p == eof) {
                    {
                        if (_myanmar_syllable_machine_eof_trans[(cs) as usize] > 0) {
                            {
                                _trans =
                                    (_myanmar_syllable_machine_eof_trans[(cs) as usize]) as u32 - 1;
                            }
                        }
                    }
                } else {
                    {
                        _keys = (cs << 1) as i32;
                        _inds = (_myanmar_syllable_machine_index_offsets[(cs) as usize]) as i32;
                        if ((buffer.info[p].indic_category() as u8) <= 32
                            && (buffer.info[p].indic_category() as u8) >= 1)
                        {
                            {
                                _ic = (_myanmar_syllable_machine_char_class
                                    [((buffer.info[p].indic_category() as u8) as i32 - 1) as usize])
                                    as i32;
                                if (_ic
                                    <= (_myanmar_syllable_machine_trans_keys[(_keys + 1) as usize])
                                        as i32
                                    && _ic
                                        >= (_myanmar_syllable_machine_trans_keys[(_keys) as usize])
                                            as i32)
                                {
                                    _trans = (_myanmar_syllable_machine_indices[(_inds
                                        + (_ic
                                            - (_myanmar_syllable_machine_trans_keys
                                                [(_keys) as usize])
                                                as i32)
                                            as i32)
                                        as usize])
                                        as u32;
                                } else {
                                    _trans = (_myanmar_syllable_machine_index_defaults
                                        [(cs) as usize])
                                        as u32;
                                }
                            }
                        } else {
                            {
                                _trans = (_myanmar_syllable_machine_index_defaults[(cs) as usize])
                                    as u32;
                            }
                        }
                    }
                }
                cs = (_myanmar_syllable_machine_cond_targs[(_trans) as usize]) as i32;
                if (_myanmar_syllable_machine_cond_actions[(_trans) as usize] != 0) {
                    {
                        match (_myanmar_syllable_machine_cond_actions[(_trans) as usize]) {
                            6 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::ConsonantSyllable);
                                }
                            }
                            4 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::NonMyanmarCluster);
                                }
                            }
                            10 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::PunctuationCluster);
                                }
                            }
                            8 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                }
                            }
                            3 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::NonMyanmarCluster);
                                }
                            }
                            5 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::ConsonantSyllable);
                                }
                            }
                            7 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                }
                            }
                            9 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::NonMyanmarCluster);
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
                    match (_myanmar_syllable_machine_to_state_actions[(cs) as usize]) {
                        1 => {
                            ts = 0;
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
