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

use super::buffer::{hb_buffer_t, HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE};

static _myanmar_syllable_machine_trans_keys: [u8; 112] = [
    0, 21, 1, 21, 3, 19, 3, 5, 1, 21, 1, 19, 1, 19, 3, 19, 3, 19, 1, 19, 1, 19, 1, 21, 3, 19, 0, 8,
    1, 19, 1, 19, 1, 20, 1, 19, 1, 21, 1, 21, 1, 19, 1, 21, 1, 21, 1, 21, 1, 21, 1, 21, 3, 19, 3,
    5, 1, 21, 1, 19, 1, 19, 3, 19, 3, 19, 1, 19, 1, 19, 1, 21, 3, 19, 0, 8, 1, 21, 1, 19, 1, 19, 1,
    20, 1, 19, 1, 21, 1, 21, 1, 19, 1, 21, 1, 21, 1, 21, 1, 21, 1, 21, 1, 21, 1, 21, 0, 21, 0, 8,
    0, 0,
];
static _myanmar_syllable_machine_char_class: [i8; 43] = [
    0, 0, 1, 2, 3, 3, 4, 5, 6, 7, 7, 4, 4, 4, 8, 4, 4, 9, 4, 10, 11, 12, 13, 4, 4, 4, 4, 4, 4, 4,
    4, 14, 4, 4, 15, 16, 17, 18, 19, 20, 21, 0, 0,
];
static _myanmar_syllable_machine_index_offsets: [i16; 57] = [
    0, 22, 43, 60, 63, 84, 103, 122, 139, 156, 175, 194, 215, 232, 241, 260, 279, 299, 318, 339,
    360, 379, 400, 421, 442, 463, 484, 501, 504, 525, 544, 563, 580, 597, 616, 635, 656, 673, 682,
    703, 722, 741, 761, 780, 801, 822, 841, 862, 883, 904, 925, 946, 967, 988, 1010, 0, 0,
];
static _myanmar_syllable_machine_indices: [i8; 1021] = [
    2, 3, 4, 5, 1, 6, 7, 2, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25, 22,
    26, 27, 22, 22, 22, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 25, 22, 26, 22, 22, 22, 22,
    22, 22, 22, 31, 40, 22, 22, 22, 22, 37, 25, 22, 26, 23, 22, 25, 22, 26, 27, 22, 22, 22, 41, 22,
    22, 31, 42, 43, 22, 22, 22, 37, 22, 42, 23, 22, 25, 22, 26, 27, 22, 22, 22, 22, 22, 22, 31, 22,
    22, 22, 22, 22, 37, 44, 22, 25, 22, 26, 37, 22, 22, 22, 22, 22, 22, 22, 45, 22, 22, 22, 22, 37,
    25, 22, 26, 22, 22, 22, 22, 22, 22, 22, 22, 45, 22, 22, 22, 22, 37, 25, 22, 26, 22, 22, 22, 22,
    22, 22, 22, 22, 22, 22, 22, 22, 22, 37, 23, 22, 25, 22, 26, 27, 22, 22, 22, 41, 22, 22, 31, 22,
    22, 22, 22, 22, 37, 23, 22, 25, 22, 26, 27, 22, 22, 22, 41, 22, 22, 31, 42, 22, 22, 22, 22, 37,
    23, 22, 25, 22, 26, 27, 22, 22, 22, 41, 22, 22, 31, 42, 22, 22, 22, 22, 37, 22, 42, 25, 22, 26,
    22, 22, 22, 22, 22, 22, 22, 31, 22, 22, 22, 22, 22, 37, 2, 22, 22, 22, 22, 22, 22, 22, 2, 23,
    22, 25, 22, 26, 27, 22, 22, 22, 28, 29, 22, 31, 22, 22, 22, 22, 22, 37, 23, 22, 25, 22, 26, 27,
    22, 22, 22, 22, 29, 22, 31, 22, 22, 22, 22, 22, 37, 23, 22, 25, 22, 26, 27, 22, 22, 22, 28, 29,
    30, 31, 22, 22, 22, 22, 22, 37, 46, 23, 22, 25, 22, 26, 27, 22, 22, 22, 28, 29, 30, 31, 22, 22,
    22, 22, 22, 37, 23, 22, 25, 22, 26, 27, 22, 22, 22, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 22,
    39, 23, 22, 25, 22, 26, 27, 22, 22, 22, 28, 29, 30, 31, 46, 22, 22, 22, 22, 37, 22, 39, 23, 22,
    25, 22, 26, 27, 22, 22, 22, 28, 29, 30, 31, 46, 22, 22, 22, 22, 37, 23, 22, 25, 22, 26, 27, 22,
    22, 22, 28, 29, 30, 31, 22, 33, 22, 35, 22, 37, 22, 39, 23, 22, 25, 22, 26, 27, 22, 22, 22, 28,
    29, 30, 31, 46, 33, 22, 22, 22, 37, 22, 39, 23, 22, 25, 22, 26, 27, 22, 22, 22, 28, 29, 30, 31,
    47, 33, 34, 35, 22, 37, 22, 39, 23, 22, 25, 22, 26, 27, 22, 22, 22, 28, 29, 30, 31, 22, 33, 34,
    35, 22, 37, 22, 39, 23, 24, 25, 22, 26, 27, 22, 22, 22, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
    22, 39, 49, 48, 6, 48, 48, 48, 48, 48, 48, 48, 13, 50, 48, 48, 48, 48, 19, 49, 48, 6, 3, 48,
    49, 48, 6, 7, 48, 48, 48, 51, 48, 48, 13, 52, 53, 48, 48, 48, 19, 48, 52, 3, 48, 49, 48, 6, 7,
    48, 48, 48, 48, 48, 48, 13, 48, 48, 48, 48, 48, 19, 54, 48, 49, 48, 6, 19, 48, 48, 48, 48, 48,
    48, 48, 55, 48, 48, 48, 48, 19, 49, 48, 6, 48, 48, 48, 48, 48, 48, 48, 48, 55, 48, 48, 48, 48,
    19, 49, 48, 6, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 19, 3, 48, 49, 48, 6, 7, 48,
    48, 48, 51, 48, 48, 13, 48, 48, 48, 48, 48, 19, 3, 48, 49, 48, 6, 7, 48, 48, 48, 51, 48, 48,
    13, 52, 48, 48, 48, 48, 19, 3, 48, 49, 48, 6, 7, 48, 48, 48, 51, 48, 48, 13, 52, 48, 48, 48,
    48, 19, 48, 52, 49, 48, 6, 48, 48, 48, 48, 48, 48, 48, 13, 48, 48, 48, 48, 48, 19, 56, 48, 48,
    48, 48, 48, 48, 48, 56, 3, 4, 49, 48, 6, 7, 48, 48, 48, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 3, 48, 49, 48, 6, 7, 48, 48, 48, 10, 11, 48, 13, 48, 48, 48, 48, 48, 19, 3, 48, 49, 48,
    6, 7, 48, 48, 48, 48, 11, 48, 13, 48, 48, 48, 48, 48, 19, 3, 48, 49, 48, 6, 7, 48, 48, 48, 10,
    11, 12, 13, 48, 48, 48, 48, 48, 19, 57, 3, 48, 49, 48, 6, 7, 48, 48, 48, 10, 11, 12, 13, 48,
    48, 48, 48, 48, 19, 3, 48, 49, 48, 6, 7, 48, 48, 48, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    48, 21, 3, 48, 49, 48, 6, 7, 48, 48, 48, 10, 11, 12, 13, 57, 48, 48, 48, 48, 19, 48, 21, 3, 48,
    49, 48, 6, 7, 48, 48, 48, 10, 11, 12, 13, 57, 48, 48, 48, 48, 19, 3, 48, 49, 48, 6, 7, 48, 48,
    48, 10, 11, 12, 13, 48, 15, 48, 17, 48, 19, 48, 21, 3, 48, 49, 48, 6, 7, 48, 48, 48, 10, 11,
    12, 13, 57, 15, 48, 48, 48, 19, 48, 21, 3, 48, 49, 48, 6, 7, 48, 48, 48, 10, 11, 12, 13, 58,
    15, 16, 17, 48, 19, 48, 21, 3, 48, 49, 48, 6, 7, 48, 48, 48, 10, 11, 12, 13, 48, 15, 16, 17,
    48, 19, 48, 21, 3, 4, 49, 48, 6, 7, 48, 48, 48, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 48, 21,
    23, 24, 25, 22, 26, 27, 22, 22, 22, 28, 29, 30, 31, 59, 33, 34, 35, 36, 37, 38, 39, 23, 60, 25,
    22, 26, 27, 22, 22, 22, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 22, 39, 2, 3, 4, 49, 48, 6, 7,
    2, 2, 48, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 2, 61, 61, 61, 61, 61, 61, 2, 2, 0,
    0,
];
static _myanmar_syllable_machine_index_defaults: [i8; 57] = [
    1, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
    22, 22, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
    48, 48, 48, 22, 22, 48, 61, 0, 0,
];
static _myanmar_syllable_machine_cond_targs: [i8; 64] = [
    0, 0, 1, 26, 37, 0, 27, 29, 51, 54, 39, 40, 41, 28, 43, 44, 46, 47, 48, 30, 50, 45, 0, 2, 13,
    0, 3, 5, 14, 15, 16, 4, 18, 19, 21, 22, 23, 6, 25, 20, 12, 9, 10, 11, 7, 8, 17, 24, 0, 0, 36,
    33, 34, 35, 31, 32, 38, 42, 49, 52, 53, 0, 0, 0,
];
static _myanmar_syllable_machine_cond_actions: [i8; 64] = [
    0, 3, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 6, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0,
];
static _myanmar_syllable_machine_to_state_actions: [i8; 57] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_from_state_actions: [i8; 57] = [
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_eof_trans: [i8; 57] = [
    1, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
    23, 23, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49,
    49, 49, 49, 23, 23, 49, 62, 0, 0,
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

pub fn find_syllables_myanmar(buffer: &mut hb_buffer_t) {
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
                        if ((buffer.info[p].myanmar_category() as u8) <= 41
                            && (buffer.info[p].myanmar_category() as u8) >= 1)
                        {
                            {
                                _ic = (_myanmar_syllable_machine_char_class[((buffer.info[p]
                                    .myanmar_category()
                                    as u8)
                                    as i32
                                    - 1)
                                    as usize]) as i32;
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
                            8 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                    buffer.scratch_flags |=
                                        HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;
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
                                    buffer.scratch_flags |=
                                        HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;
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
    buffer: &mut hb_buffer_t,
) {
    for i in start..end {
        buffer.info[i].set_syllable((*syllable_serial << 4) | kind as u8);
    }

    *syllable_serial += 1;

    if *syllable_serial == 16 {
        *syllable_serial = 1;
    }
}
