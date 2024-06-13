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

static _khmer_syllable_machine_trans_keys: [u8; 82] = [
    3, 10, 3, 10, 0, 0, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 0, 0, 3, 10,
    3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 0, 11, 2, 11, 0, 0, 2, 11, 11, 11, 2, 11, 2, 11, 2,
    11, 2, 11, 2, 11, 0, 0, 2, 11, 2, 11, 11, 11, 2, 11, 2, 11, 2, 11, 2, 11, 2, 11, 3, 10, 0, 0,
];
static _khmer_syllable_machine_char_class: [i8; 29] = [
    0, 0, 1, 2, 3, 3, 1, 1, 1, 4, 4, 1, 1, 1, 0, 1, 1, 1, 1, 5, 6, 7, 8, 1, 9, 10, 11, 0, 0,
];
static _khmer_syllable_machine_index_offsets: [i16; 42] = [
    0, 8, 16, 17, 25, 33, 41, 49, 57, 65, 73, 81, 89, 90, 98, 106, 114, 122, 130, 138, 146, 158,
    168, 169, 179, 180, 190, 200, 210, 220, 230, 231, 241, 251, 252, 262, 272, 282, 292, 302, 0, 0,
];
static _khmer_syllable_machine_indices: [i8; 312] = [
    1, 0, 2, 0, 0, 0, 3, 4, 1, 0, 0, 0, 0, 0, 0, 4, 5, 1, 0, 2, 0, 0, 0, 0, 4, 6, 0, 0, 0, 0, 0, 0,
    2, 7, 0, 0, 0, 0, 0, 0, 8, 9, 0, 2, 0, 0, 0, 0, 10, 9, 0, 0, 0, 0, 0, 0, 10, 11, 0, 2, 0, 0, 0,
    0, 12, 11, 0, 0, 0, 0, 0, 0, 12, 14, 13, 15, 13, 13, 13, 16, 17, 14, 18, 18, 18, 18, 18, 18,
    17, 19, 14, 13, 15, 13, 13, 13, 13, 17, 20, 13, 13, 13, 13, 13, 13, 15, 21, 13, 13, 13, 13, 13,
    13, 22, 23, 13, 15, 13, 13, 13, 13, 24, 23, 13, 13, 13, 13, 13, 13, 24, 25, 13, 15, 13, 13, 13,
    13, 26, 25, 13, 13, 13, 13, 13, 13, 26, 29, 28, 30, 31, 3, 15, 24, 26, 22, 28, 17, 19, 33, 34,
    32, 2, 10, 12, 8, 3, 4, 5, 29, 35, 36, 32, 2, 10, 12, 8, 32, 4, 5, 5, 35, 6, 32, 32, 32, 32, 8,
    32, 2, 5, 35, 7, 32, 32, 32, 32, 32, 32, 8, 5, 35, 37, 32, 2, 32, 32, 8, 32, 10, 5, 35, 38, 32,
    2, 10, 32, 8, 32, 12, 5, 33, 36, 32, 2, 10, 12, 8, 32, 4, 5, 40, 30, 41, 39, 15, 24, 26, 22,
    16, 17, 19, 42, 43, 39, 15, 24, 26, 22, 39, 17, 19, 19, 42, 20, 39, 39, 39, 39, 22, 39, 15, 19,
    42, 21, 39, 39, 39, 39, 39, 39, 22, 19, 42, 44, 39, 15, 39, 39, 22, 39, 24, 19, 42, 45, 39, 15,
    24, 39, 22, 39, 26, 19, 30, 43, 39, 15, 24, 26, 22, 39, 17, 19, 14, 46, 15, 46, 46, 46, 46, 17,
    0, 0,
];
static _khmer_syllable_machine_index_defaults: [i8; 42] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 18, 13, 13, 13, 13, 13, 13, 13, 13, 28, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 39, 39, 39, 39, 39, 39, 39, 39, 39, 46, 0, 0,
];
static _khmer_syllable_machine_cond_targs: [i8; 49] = [
    20, 1, 25, 29, 23, 24, 4, 5, 26, 7, 27, 9, 28, 20, 11, 34, 38, 32, 20, 33, 14, 15, 35, 17, 36,
    19, 37, 20, 20, 21, 30, 39, 20, 22, 0, 2, 3, 6, 8, 20, 31, 10, 12, 13, 16, 18, 20, 0, 0,
];
static _khmer_syllable_machine_cond_actions: [i8; 49] = [
    1, 0, 2, 2, 2, 0, 0, 0, 2, 0, 2, 0, 2, 3, 0, 2, 4, 4, 5, 0, 0, 0, 2, 0, 2, 0, 2, 0, 8, 2, 0, 9,
    10, 0, 0, 0, 0, 0, 0, 11, 4, 0, 0, 0, 0, 0, 12, 0, 0,
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
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 14, 19, 14, 14, 14, 14, 14, 14, 14, 14, 28, 33, 33, 33, 33, 33,
    33, 33, 33, 33, 40, 40, 40, 40, 40, 40, 40, 40, 40, 47, 0, 0,
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
                            11 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                    buffer.scratch_flags |=
                                        HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;
                                }
                            }
                            12 => {
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
                            3 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                    buffer.scratch_flags |=
                                        HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;
                                }
                            }
                            5 => match (act) {
                                2 => {
                                    p = (te) - 1;
                                    {
                                        found_syllable!(SyllableType::BrokenCluster);
                                        buffer.scratch_flags |=
                                            HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;
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
