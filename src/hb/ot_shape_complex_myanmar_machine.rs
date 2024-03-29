// This file is autogenerated. Do not edit it!
//
// See docs/ragel.md for details.

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

use super::buffer::hb_buffer_t;

static _myanmar_syllable_machine_trans_keys: [u8; 114] = [
    0, 22, 1, 22, 3, 19, 3, 5, 3, 19, 1, 15, 3, 15, 3, 15, 1, 22, 1, 19, 1, 19, 1, 19, 1, 22, 0, 8,
    1, 22, 1, 22, 1, 19, 1, 19, 1, 19, 1, 20, 1, 19, 1, 22, 1, 22, 1, 22, 1, 22, 1, 22, 3, 19, 3,
    5, 3, 19, 1, 15, 3, 15, 3, 15, 1, 22, 1, 19, 1, 19, 1, 19, 1, 22, 0, 8, 1, 22, 1, 22, 1, 22, 1,
    19, 1, 19, 1, 19, 1, 20, 1, 19, 1, 22, 1, 22, 1, 22, 1, 22, 1, 22, 1, 22, 1, 22, 0, 22, 0, 8,
    5, 5, 0, 0,
];
static _myanmar_syllable_machine_char_class: [i8; 35] = [
    0, 0, 1, 2, 3, 3, 4, 5, 4, 6, 7, 4, 4, 4, 4, 8, 4, 9, 10, 4, 11, 12, 13, 14, 15, 16, 17, 18,
    19, 20, 21, 7, 22, 0, 0,
];
static _myanmar_syllable_machine_index_offsets: [i16; 58] = [
    0, 23, 45, 62, 65, 82, 97, 110, 123, 145, 164, 183, 202, 224, 233, 255, 277, 296, 315, 334,
    354, 373, 395, 417, 439, 461, 483, 500, 503, 520, 535, 548, 561, 583, 602, 621, 640, 662, 671,
    693, 715, 737, 756, 775, 794, 814, 833, 855, 877, 899, 921, 943, 965, 987, 1010, 1019, 0, 0,
];
static _myanmar_syllable_machine_indices: [i8; 1022] = [
    2, 3, 4, 5, 1, 6, 7, 2, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 24, 25, 26,
    23, 27, 28, 23, 23, 29, 23, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 23, 40, 26, 23, 27, 23, 23,
    23, 41, 23, 23, 23, 23, 23, 34, 23, 23, 23, 38, 26, 23, 27, 26, 23, 27, 23, 23, 23, 23, 23, 23,
    23, 23, 23, 34, 23, 23, 23, 38, 42, 23, 26, 23, 27, 34, 23, 23, 43, 23, 23, 23, 23, 23, 34, 26,
    23, 27, 23, 23, 23, 43, 23, 23, 23, 23, 23, 34, 26, 23, 27, 23, 23, 23, 23, 23, 23, 23, 23, 23,
    34, 24, 23, 26, 23, 27, 28, 23, 23, 44, 23, 45, 23, 23, 23, 34, 46, 23, 23, 38, 23, 23, 44, 24,
    23, 26, 23, 27, 28, 23, 23, 23, 23, 23, 23, 23, 23, 34, 23, 23, 23, 38, 24, 23, 26, 23, 27, 28,
    23, 23, 44, 23, 23, 23, 23, 23, 34, 46, 23, 23, 38, 24, 23, 26, 23, 27, 28, 23, 23, 23, 23, 23,
    23, 23, 23, 34, 46, 23, 23, 38, 24, 23, 26, 23, 27, 28, 23, 23, 44, 23, 23, 23, 23, 23, 34, 46,
    23, 23, 38, 23, 23, 44, 2, 23, 23, 23, 23, 23, 23, 23, 2, 24, 23, 26, 23, 27, 28, 23, 23, 29,
    23, 30, 31, 32, 33, 34, 35, 36, 37, 38, 23, 23, 40, 24, 23, 26, 23, 27, 28, 23, 23, 47, 23, 23,
    23, 23, 23, 34, 35, 36, 37, 38, 23, 23, 40, 24, 23, 26, 23, 27, 28, 23, 23, 23, 23, 23, 23, 23,
    23, 34, 35, 36, 37, 38, 24, 23, 26, 23, 27, 28, 23, 23, 23, 23, 23, 23, 23, 23, 34, 35, 36, 23,
    38, 24, 23, 26, 23, 27, 28, 23, 23, 23, 23, 23, 23, 23, 23, 34, 23, 36, 23, 38, 24, 23, 26, 23,
    27, 28, 23, 23, 23, 23, 23, 23, 23, 23, 34, 35, 36, 37, 38, 47, 24, 23, 26, 23, 27, 28, 23, 23,
    47, 23, 23, 23, 23, 23, 34, 35, 36, 37, 38, 24, 23, 26, 23, 27, 28, 23, 23, 23, 23, 30, 23, 32,
    23, 34, 35, 36, 37, 38, 23, 23, 40, 24, 23, 26, 23, 27, 28, 23, 23, 47, 23, 30, 23, 23, 23, 34,
    35, 36, 37, 38, 23, 23, 40, 24, 23, 26, 23, 27, 28, 23, 23, 48, 23, 30, 31, 32, 23, 34, 35, 36,
    37, 38, 23, 23, 40, 24, 23, 26, 23, 27, 28, 23, 23, 23, 23, 30, 31, 32, 23, 34, 35, 36, 37, 38,
    23, 23, 40, 24, 25, 26, 23, 27, 28, 23, 23, 29, 23, 30, 31, 32, 33, 34, 35, 36, 37, 38, 23, 23,
    40, 50, 49, 6, 49, 49, 49, 51, 49, 49, 49, 49, 49, 15, 49, 49, 49, 19, 50, 49, 6, 50, 49, 6,
    49, 49, 49, 49, 49, 49, 49, 49, 49, 15, 49, 49, 49, 19, 52, 49, 50, 49, 6, 15, 49, 49, 53, 49,
    49, 49, 49, 49, 15, 50, 49, 6, 49, 49, 49, 53, 49, 49, 49, 49, 49, 15, 50, 49, 6, 49, 49, 49,
    49, 49, 49, 49, 49, 49, 15, 3, 49, 50, 49, 6, 7, 49, 49, 54, 49, 55, 49, 49, 49, 15, 56, 49,
    49, 19, 49, 49, 54, 3, 49, 50, 49, 6, 7, 49, 49, 49, 49, 49, 49, 49, 49, 15, 49, 49, 49, 19, 3,
    49, 50, 49, 6, 7, 49, 49, 54, 49, 49, 49, 49, 49, 15, 56, 49, 49, 19, 3, 49, 50, 49, 6, 7, 49,
    49, 49, 49, 49, 49, 49, 49, 15, 56, 49, 49, 19, 3, 49, 50, 49, 6, 7, 49, 49, 54, 49, 49, 49,
    49, 49, 15, 56, 49, 49, 19, 49, 49, 54, 57, 49, 49, 49, 49, 49, 49, 49, 57, 3, 4, 50, 49, 6, 7,
    49, 49, 9, 49, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 49, 22, 3, 49, 50, 49, 6, 7, 49, 49, 9,
    49, 11, 12, 13, 14, 15, 16, 17, 18, 19, 49, 49, 22, 3, 49, 50, 49, 6, 7, 49, 49, 58, 49, 49,
    49, 49, 49, 15, 16, 17, 18, 19, 49, 49, 22, 3, 49, 50, 49, 6, 7, 49, 49, 49, 49, 49, 49, 49,
    49, 15, 16, 17, 18, 19, 3, 49, 50, 49, 6, 7, 49, 49, 49, 49, 49, 49, 49, 49, 15, 16, 17, 49,
    19, 3, 49, 50, 49, 6, 7, 49, 49, 49, 49, 49, 49, 49, 49, 15, 49, 17, 49, 19, 3, 49, 50, 49, 6,
    7, 49, 49, 49, 49, 49, 49, 49, 49, 15, 16, 17, 18, 19, 58, 3, 49, 50, 49, 6, 7, 49, 49, 58, 49,
    49, 49, 49, 49, 15, 16, 17, 18, 19, 3, 49, 50, 49, 6, 7, 49, 49, 49, 49, 11, 49, 13, 49, 15,
    16, 17, 18, 19, 49, 49, 22, 3, 49, 50, 49, 6, 7, 49, 49, 58, 49, 11, 49, 49, 49, 15, 16, 17,
    18, 19, 49, 49, 22, 3, 49, 50, 49, 6, 7, 49, 49, 59, 49, 11, 12, 13, 49, 15, 16, 17, 18, 19,
    49, 49, 22, 3, 49, 50, 49, 6, 7, 49, 49, 49, 49, 11, 12, 13, 49, 15, 16, 17, 18, 19, 49, 49,
    22, 3, 4, 50, 49, 6, 7, 49, 49, 9, 49, 11, 12, 13, 14, 15, 16, 17, 18, 19, 49, 49, 22, 24, 25,
    26, 23, 27, 28, 23, 23, 60, 23, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 23, 40, 24, 61, 26, 23,
    27, 28, 23, 23, 29, 23, 30, 31, 32, 33, 34, 35, 36, 37, 38, 23, 23, 40, 2, 3, 4, 50, 49, 6, 7,
    2, 2, 9, 49, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 49, 22, 2, 62, 62, 62, 62, 62, 62, 2, 2,
    63, 0, 0,
];
static _myanmar_syllable_machine_index_defaults: [i8; 58] = [
    1, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
    23, 23, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49,
    49, 49, 49, 23, 23, 49, 62, 62, 0, 0,
];
static _myanmar_syllable_machine_cond_targs: [i8; 66] = [
    0, 0, 1, 26, 37, 0, 27, 33, 51, 39, 54, 40, 46, 47, 48, 29, 42, 43, 44, 32, 50, 55, 45, 0, 2,
    13, 0, 3, 9, 14, 15, 21, 22, 23, 5, 17, 18, 19, 8, 25, 20, 4, 6, 7, 10, 12, 11, 16, 24, 0, 0,
    28, 30, 31, 34, 36, 35, 38, 41, 49, 52, 53, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_cond_actions: [i8; 66] = [
    0, 3, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 6, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9,
    10, 0, 0,
];
static _myanmar_syllable_machine_to_state_actions: [i8; 58] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_from_state_actions: [i8; 58] = [
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_eof_trans: [i8; 58] = [
    1, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
    24, 24, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50,
    50, 50, 50, 24, 24, 50, 63, 63, 0, 0,
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
                        if ((buffer.info[p].indic_category() as u8) <= 33
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
