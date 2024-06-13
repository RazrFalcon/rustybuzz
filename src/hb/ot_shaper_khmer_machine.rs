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

static _khmer_syllable_machine_trans_keys: [u8; 88] = [
    3, 10, 3, 10, 0, 0, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 0, 0,
    3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 3, 10, 0, 11, 2, 11, 0, 0, 2, 11, 2, 11, 11, 11, 2,
    11, 2, 11, 2, 11, 2, 11, 2, 11, 2, 11, 0, 0, 2, 11, 2, 11, 11, 11, 2, 11, 2, 11, 2, 11, 2, 11,
    2, 11, 3, 10, 0, 0,
];
static _khmer_syllable_machine_char_class: [i8; 29] = [
    0, 0, 1, 2, 3, 3, 1, 1, 1, 4, 4, 1, 1, 1, 0, 1, 1, 1, 1, 5, 6, 7, 8, 1, 9, 10, 11, 0, 0,
];
static _khmer_syllable_machine_index_offsets: [i16; 45] = [
    0, 8, 16, 17, 25, 33, 41, 49, 57, 65, 73, 81, 89, 97, 98, 106, 114, 122, 130, 138, 146, 154,
    166, 176, 177, 187, 197, 198, 208, 218, 228, 238, 248, 258, 259, 269, 279, 280, 290, 300, 310,
    320, 330, 0, 0,
];
static _khmer_syllable_machine_indices: [i8; 340] = [
    1, 0, 2, 0, 0, 0, 3, 4, 1, 0, 0, 0, 0, 0, 0, 4, 5, 1, 0, 2, 0, 0, 0, 0, 4, 6, 0, 0, 0, 0, 0, 0,
    2, 7, 0, 0, 0, 0, 0, 0, 8, 9, 0, 2, 0, 0, 0, 0, 10, 9, 0, 0, 0, 0, 0, 0, 10, 11, 0, 2, 0, 0, 0,
    0, 12, 11, 0, 0, 0, 0, 0, 0, 12, 1, 0, 2, 0, 0, 0, 13, 4, 15, 14, 16, 14, 14, 14, 17, 18, 15,
    19, 19, 19, 19, 19, 19, 18, 20, 15, 14, 16, 14, 14, 14, 14, 18, 21, 14, 14, 14, 14, 14, 14, 16,
    22, 14, 14, 14, 14, 14, 14, 23, 24, 14, 16, 14, 14, 14, 14, 25, 24, 14, 14, 14, 14, 14, 14, 25,
    26, 14, 16, 14, 14, 14, 14, 27, 26, 14, 14, 14, 14, 14, 14, 27, 30, 29, 31, 32, 13, 16, 25, 27,
    23, 17, 18, 20, 34, 35, 33, 2, 10, 12, 8, 13, 4, 5, 36, 34, 37, 33, 2, 10, 12, 8, 3, 4, 5, 38,
    39, 33, 2, 10, 12, 8, 33, 4, 5, 5, 38, 6, 33, 33, 33, 33, 8, 33, 2, 5, 38, 7, 33, 33, 33, 33,
    33, 33, 8, 5, 38, 40, 33, 2, 33, 33, 8, 33, 10, 5, 38, 41, 33, 2, 10, 33, 8, 33, 12, 5, 34, 39,
    33, 2, 10, 12, 8, 33, 4, 5, 34, 39, 33, 2, 10, 12, 8, 3, 4, 5, 43, 31, 44, 42, 16, 25, 27, 23,
    17, 18, 20, 45, 46, 42, 16, 25, 27, 23, 42, 18, 20, 20, 45, 21, 42, 42, 42, 42, 23, 42, 16, 20,
    45, 22, 42, 42, 42, 42, 42, 42, 23, 20, 45, 47, 42, 16, 42, 42, 23, 42, 25, 20, 45, 48, 42, 16,
    25, 42, 23, 42, 27, 20, 31, 46, 42, 16, 25, 27, 23, 42, 18, 20, 15, 49, 16, 49, 49, 49, 49, 18,
    0, 0,
];
static _khmer_syllable_machine_index_defaults: [i8; 45] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 19, 14, 14, 14, 14, 14, 14, 14, 14, 29, 33, 33, 33, 33,
    33, 33, 33, 33, 33, 33, 33, 42, 42, 42, 42, 42, 42, 42, 42, 42, 49, 0, 0,
];
static _khmer_syllable_machine_cond_targs: [i8; 52] = [
    21, 1, 27, 31, 25, 26, 4, 5, 28, 7, 29, 9, 30, 32, 21, 12, 37, 41, 35, 21, 36, 15, 16, 38, 18,
    39, 20, 40, 21, 21, 22, 33, 42, 21, 23, 10, 24, 0, 2, 3, 6, 8, 21, 34, 11, 13, 14, 17, 19, 21,
    0, 0,
];
static _khmer_syllable_machine_cond_actions: [i8; 52] = [
    1, 0, 2, 2, 2, 0, 0, 0, 2, 0, 2, 0, 2, 2, 3, 0, 2, 4, 4, 5, 0, 0, 0, 2, 0, 2, 0, 2, 0, 8, 2, 0,
    9, 10, 0, 0, 2, 0, 0, 0, 0, 0, 11, 4, 0, 0, 0, 0, 0, 12, 0, 0,
];
static _khmer_syllable_machine_to_state_actions: [i8; 45] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _khmer_syllable_machine_from_state_actions: [i8; 45] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _khmer_syllable_machine_eof_trans: [i8; 45] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 15, 20, 15, 15, 15, 15, 15, 15, 15, 15, 29, 34, 34, 34, 34,
    34, 34, 34, 34, 34, 34, 34, 43, 43, 43, 43, 43, 43, 43, 43, 43, 50, 0, 0,
];
static khmer_syllable_machine_start: i32 = 21;
static khmer_syllable_machine_first_final: i32 = 21;
static khmer_syllable_machine_error: i32 = -1;
static khmer_syllable_machine_en_main: i32 = 21;
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
                    if (cs >= 21) {
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
