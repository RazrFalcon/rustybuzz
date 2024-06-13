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

static _khmer_syllable_machine_trans_keys: [u8; 82] = [
    2, 10, 2, 10, 2, 10, 2, 10, 2, 10, 0, 0, 2, 10, 2, 10, 2, 10, 2, 10, 2, 10, 2, 10, 2, 10, 2,
    10, 0, 0, 2, 10, 2, 10, 2, 10, 2, 10, 2, 10, 0, 11, 2, 11, 2, 11, 2, 11, 2, 11, 11, 11, 2, 11,
    2, 11, 2, 11, 0, 0, 2, 10, 2, 11, 2, 11, 2, 11, 11, 11, 2, 11, 2, 11, 0, 0, 2, 11, 2, 11, 0, 0,
];
static _khmer_syllable_machine_char_class: [i8; 29] = [
    0, 0, 1, 1, 2, 2, 1, 1, 1, 3, 3, 1, 1, 1, 0, 1, 1, 1, 1, 4, 5, 6, 7, 8, 9, 10, 11, 0, 0,
];
static _khmer_syllable_machine_index_offsets: [i16; 42] = [
    0, 9, 18, 27, 36, 45, 46, 55, 64, 73, 82, 91, 100, 109, 118, 119, 128, 137, 146, 155, 164, 176,
    186, 196, 206, 216, 217, 227, 237, 247, 248, 257, 267, 277, 287, 288, 298, 308, 309, 319, 0, 0,
];
static _khmer_syllable_machine_indices: [i8; 331] = [
    1, 0, 2, 0, 0, 0, 0, 3, 4, 1, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0, 2, 0, 0, 0, 0, 0, 4, 5, 0, 0, 0, 0,
    0, 0, 0, 2, 6, 0, 0, 0, 0, 0, 0, 0, 7, 8, 9, 0, 2, 0, 0, 0, 0, 0, 10, 9, 0, 0, 0, 0, 0, 0, 0,
    10, 11, 0, 2, 0, 0, 0, 0, 0, 12, 11, 0, 0, 0, 0, 0, 0, 0, 12, 14, 13, 13, 13, 13, 13, 13, 13,
    15, 14, 16, 17, 16, 16, 16, 16, 16, 15, 18, 16, 16, 16, 16, 16, 16, 16, 17, 19, 16, 16, 16, 16,
    16, 16, 16, 20, 21, 22, 16, 17, 16, 16, 16, 16, 16, 23, 22, 16, 16, 16, 16, 16, 16, 16, 23, 24,
    16, 17, 16, 16, 16, 16, 16, 25, 24, 16, 16, 16, 16, 16, 16, 16, 25, 14, 16, 17, 16, 16, 16, 16,
    26, 15, 29, 28, 30, 3, 17, 23, 25, 20, 31, 28, 15, 21, 33, 32, 2, 10, 12, 7, 34, 3, 4, 8, 35,
    32, 2, 10, 12, 7, 36, 32, 4, 8, 5, 32, 32, 32, 32, 7, 36, 32, 2, 8, 6, 32, 32, 32, 32, 32, 36,
    32, 7, 8, 8, 37, 32, 2, 32, 32, 7, 36, 32, 10, 8, 38, 32, 2, 10, 32, 7, 36, 32, 12, 8, 35, 32,
    2, 10, 12, 7, 34, 32, 4, 8, 29, 14, 39, 17, 39, 39, 39, 39, 39, 15, 41, 40, 17, 23, 25, 20, 42,
    40, 15, 21, 18, 40, 40, 40, 40, 20, 42, 40, 17, 21, 19, 40, 40, 40, 40, 40, 42, 40, 20, 21, 21,
    43, 40, 17, 40, 40, 20, 42, 40, 23, 21, 44, 40, 17, 23, 40, 20, 42, 40, 25, 21, 45, 46, 40, 17,
    23, 25, 20, 31, 26, 15, 21, 41, 40, 17, 23, 25, 20, 31, 40, 15, 21, 0, 0,
];
static _khmer_syllable_machine_index_defaults: [i8; 42] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 16, 16, 16, 16, 16, 16, 16, 16, 16, 28, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 39, 40, 40, 40, 40, 40, 40, 40, 40, 40, 0, 0,
];
static _khmer_syllable_machine_cond_targs: [i8; 49] = [
    20, 1, 23, 28, 22, 3, 4, 24, 25, 7, 26, 9, 27, 20, 10, 31, 20, 32, 12, 13, 33, 34, 16, 35, 18,
    36, 39, 20, 20, 21, 30, 37, 20, 0, 29, 2, 5, 6, 8, 20, 20, 11, 14, 15, 17, 38, 19, 0, 0,
];
static _khmer_syllable_machine_cond_actions: [i8; 49] = [
    1, 0, 2, 2, 2, 0, 0, 2, 0, 0, 2, 0, 2, 3, 0, 4, 5, 2, 0, 0, 2, 0, 0, 2, 0, 2, 4, 0, 8, 2, 9, 0,
    10, 0, 0, 0, 0, 0, 0, 11, 12, 0, 0, 0, 0, 4, 0, 0, 0,
];
static _khmer_syllable_machine_to_state_actions: [i8; 42] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _khmer_syllable_machine_from_state_actions: [i8; 42] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _khmer_syllable_machine_eof_trans: [i8; 42] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 14, 17, 17, 17, 17, 17, 17, 17, 17, 17, 28, 33, 33, 33, 33, 33,
    33, 33, 33, 33, 40, 41, 41, 41, 41, 41, 41, 41, 41, 41, 0, 0,
];
static khmer_syllable_machine_start: i32 = 20;
static khmer_syllable_machine_first_final: i32 = 20;
static khmer_syllable_machine_error: i32 = -1;
static khmer_syllable_machine_en_main: i32 = 20;
#[derive(Clone, Copy)]
pub enum SyllableType {
    ConsonantSyllable = 0,
    BrokenCluster,
    NonKhmerCluster,
}

pub fn find_syllables_khmer(buffer: &mut hb_buffer_t) {
    let mut cs = 0;
    let mut ts = 0;
    let mut te = 0;
    let mut act = 0;
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
        cs = (khmer_syllable_machine_start) as i32;
        ts = 0;
        te = 0;
        act = 0;
    }

    {
        let mut _trans = 0;
        let mut _keys: i32 = 0;
        let mut _inds: i32 = 0;
        let mut _ic = 0;
        '_resume: while (p != pe || p == eof) {
            '_again: while (true) {
                match (_khmer_syllable_machine_from_state_actions[(cs) as usize]) {
                    7 => {
                        ts = p;
                    }

                    _ => {}
                }
                if (p == eof) {
                    {
                        if (_khmer_syllable_machine_eof_trans[(cs) as usize] > 0) {
                            {
                                _trans =
                                    (_khmer_syllable_machine_eof_trans[(cs) as usize]) as u32 - 1;
                            }
                        }
                    }
                } else {
                    {
                        _keys = (cs << 1) as i32;
                        _inds = (_khmer_syllable_machine_index_offsets[(cs) as usize]) as i32;
                        if ((buffer.info[p].khmer_category() as u8) <= 27
                            && (buffer.info[p].khmer_category() as u8) >= 1)
                        {
                            {
                                _ic = (_khmer_syllable_machine_char_class
                                    [((buffer.info[p].khmer_category() as u8) as i32 - 1) as usize])
                                    as i32;
                                if (_ic
                                    <= (_khmer_syllable_machine_trans_keys[(_keys + 1) as usize])
                                        as i32
                                    && _ic
                                        >= (_khmer_syllable_machine_trans_keys[(_keys) as usize])
                                            as i32)
                                {
                                    _trans = (_khmer_syllable_machine_indices[(_inds
                                        + (_ic
                                            - (_khmer_syllable_machine_trans_keys[(_keys) as usize])
                                                as i32)
                                            as i32)
                                        as usize])
                                        as u32;
                                } else {
                                    _trans = (_khmer_syllable_machine_index_defaults[(cs) as usize])
                                        as u32;
                                }
                            }
                        } else {
                            {
                                _trans =
                                    (_khmer_syllable_machine_index_defaults[(cs) as usize]) as u32;
                            }
                        }
                    }
                }
                cs = (_khmer_syllable_machine_cond_targs[(_trans) as usize]) as i32;
                if (_khmer_syllable_machine_cond_actions[(_trans) as usize] != 0) {
                    {
                        match (_khmer_syllable_machine_cond_actions[(_trans) as usize]) {
                            2 => {
                                te = p + 1;
                            }
                            8 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::NonKhmerCluster);
                                }
                            }
                            10 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::ConsonantSyllable);
                                }
                            }
                            12 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                }
                            }
                            11 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::NonKhmerCluster);
                                }
                            }
                            1 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::ConsonantSyllable);
                                }
                            }
                            5 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                }
                            }
                            3 => match (act) {
                                2 => {
                                    p = (te) - 1;
                                    {
                                        found_syllable!(SyllableType::BrokenCluster);
                                    }
                                }
                                3 => {
                                    p = (te) - 1;
                                    {
                                        found_syllable!(SyllableType::NonKhmerCluster);
                                    }
                                }

                                _ => {}
                            },
                            4 => {
                                {
                                    {
                                        te = p + 1;
                                    }
                                }
                                {
                                    {
                                        act = 2;
                                    }
                                }
                            }
                            9 => {
                                {
                                    {
                                        te = p + 1;
                                    }
                                }
                                {
                                    {
                                        act = 3;
                                    }
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
                    if (cs >= 20) {
                        break '_resume;
                    }
                }
            } else {
                {
                    match (_khmer_syllable_machine_to_state_actions[(cs) as usize]) {
                        6 => {
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
