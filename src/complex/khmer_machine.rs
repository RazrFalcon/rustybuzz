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

static _khmer_syllable_machine_actions: [i8; 29] = [
    0, 1, 0, 1, 1, 1, 2, 1, 5, 1, 6, 1, 7, 1, 8, 1, 9, 1, 10, 1, 11, 2, 2, 3, 2, 2, 4, 0, 0,
];
static _khmer_syllable_machine_key_offsets: [i16; 42] = [
    0, 5, 8, 12, 15, 18, 21, 25, 28, 32, 35, 38, 42, 45, 48, 51, 55, 58, 62, 65, 70, 84, 94, 103,
    109, 110, 115, 122, 130, 139, 142, 146, 155, 161, 162, 167, 174, 182, 185, 195, 0, 0,
];
static _khmer_syllable_machine_trans_keys: [u8; 206] = [
    20, 21, 26, 5, 6, 21, 5, 6, 21, 26, 5, 6, 21, 5, 6, 16, 1, 2, 21, 5, 6, 21, 26, 5, 6, 21, 5, 6,
    21, 26, 5, 6, 21, 5, 6, 21, 5, 6, 21, 26, 5, 6, 21, 5, 6, 16, 1, 2, 21, 5, 6, 21, 26, 5, 6, 21,
    5, 6, 21, 26, 5, 6, 21, 5, 6, 20, 21, 26, 5, 6, 14, 16, 21, 22, 26, 27, 28, 29, 1, 2, 5, 6, 11,
    12, 14, 20, 21, 22, 26, 27, 28, 29, 5, 6, 14, 21, 22, 26, 27, 28, 29, 5, 6, 14, 21, 22, 29, 5,
    6, 22, 14, 21, 22, 5, 6, 14, 21, 22, 26, 29, 5, 6, 14, 21, 22, 26, 27, 29, 5, 6, 14, 21, 22,
    26, 27, 28, 29, 5, 6, 16, 1, 2, 21, 26, 5, 6, 14, 21, 22, 26, 27, 28, 29, 5, 6, 14, 21, 22, 29,
    5, 6, 22, 14, 21, 22, 5, 6, 14, 21, 22, 26, 29, 5, 6, 14, 21, 22, 26, 27, 29, 5, 6, 16, 1, 2,
    14, 20, 21, 22, 26, 27, 28, 29, 5, 6, 14, 21, 22, 26, 27, 28, 29, 5, 6, 0, 0,
];
static _khmer_syllable_machine_single_lengths: [i8; 42] = [
    3, 1, 2, 1, 1, 1, 2, 1, 2, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 3, 8, 8, 7, 4, 1, 3, 5, 6, 7, 1, 2, 7,
    4, 1, 3, 5, 6, 1, 8, 7, 0, 0,
];
static _khmer_syllable_machine_range_lengths: [i8; 42] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1,
    1, 0, 1, 1, 1, 1, 1, 1, 0, 0,
];
static _khmer_syllable_machine_index_offsets: [i16; 42] = [
    0, 5, 8, 12, 15, 18, 21, 25, 28, 32, 35, 38, 42, 45, 48, 51, 55, 58, 62, 65, 70, 82, 92, 101,
    107, 109, 114, 121, 129, 138, 141, 145, 154, 160, 162, 167, 174, 182, 185, 195, 0, 0,
];
static _khmer_syllable_machine_cond_targs: [i8; 246] = [
    28, 22, 23, 1, 20, 22, 1, 20, 22, 23, 1, 20, 23, 3, 20, 24, 24, 20, 25, 5, 20, 26, 23, 7, 20,
    26, 7, 20, 27, 23, 9, 20, 27, 9, 20, 31, 10, 20, 31, 32, 10, 20, 32, 12, 20, 33, 33, 20, 34,
    14, 20, 35, 32, 16, 20, 35, 16, 20, 36, 32, 18, 20, 36, 18, 20, 39, 31, 32, 10, 20, 37, 21, 31,
    33, 32, 35, 36, 34, 21, 30, 28, 20, 29, 28, 22, 24, 23, 26, 27, 25, 0, 20, 4, 22, 24, 23, 26,
    27, 25, 2, 20, 4, 23, 24, 25, 3, 20, 24, 20, 4, 25, 24, 5, 20, 4, 26, 24, 23, 25, 6, 20, 4, 27,
    24, 23, 26, 25, 8, 20, 29, 22, 24, 23, 26, 27, 25, 2, 20, 21, 21, 20, 31, 32, 10, 20, 13, 31,
    33, 32, 35, 36, 34, 11, 20, 13, 32, 33, 34, 12, 20, 33, 20, 13, 34, 33, 14, 20, 13, 35, 33, 32,
    34, 15, 20, 13, 36, 33, 32, 35, 34, 17, 20, 38, 38, 20, 37, 39, 31, 33, 32, 35, 36, 34, 19, 20,
    37, 31, 33, 32, 35, 36, 34, 11, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20,
    20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20,
    20, 0, 0,
];
static _khmer_syllable_machine_cond_actions: [i8; 246] = [
    5, 5, 5, 0, 15, 5, 0, 15, 5, 5, 0, 15, 5, 0, 15, 0, 0, 15, 5, 0, 15, 5, 5, 0, 15, 5, 0, 15, 5,
    5, 0, 15, 5, 0, 15, 21, 0, 19, 21, 5, 0, 17, 5, 0, 17, 0, 0, 17, 5, 0, 17, 5, 5, 0, 17, 5, 0,
    17, 5, 5, 0, 17, 5, 0, 17, 21, 21, 5, 0, 17, 0, 5, 21, 0, 5, 5, 5, 5, 5, 24, 5, 7, 0, 5, 5, 0,
    5, 5, 5, 5, 0, 9, 0, 5, 0, 5, 5, 5, 5, 0, 9, 0, 5, 0, 5, 0, 9, 0, 9, 0, 5, 0, 0, 9, 0, 5, 0, 5,
    5, 0, 9, 0, 5, 0, 5, 5, 5, 0, 9, 0, 5, 0, 5, 5, 5, 5, 0, 9, 5, 5, 9, 21, 5, 0, 13, 0, 21, 0, 5,
    5, 5, 5, 0, 11, 0, 5, 0, 5, 0, 11, 0, 11, 0, 5, 0, 0, 11, 0, 5, 0, 5, 5, 0, 11, 0, 5, 0, 5, 5,
    5, 0, 11, 21, 21, 11, 0, 21, 21, 0, 5, 5, 5, 5, 0, 11, 0, 21, 0, 5, 5, 5, 5, 0, 11, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 19, 17, 17, 17, 17, 17, 17, 17, 17, 17, 0, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 13, 11, 11, 11, 11, 11, 11, 11, 11, 11, 0, 0,
];
static _khmer_syllable_machine_to_state_actions: [i8; 42] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _khmer_syllable_machine_from_state_actions: [i8; 42] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _khmer_syllable_machine_eof_trans: [i16; 42] = [
    205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223,
    224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242,
    243, 244, 0, 0,
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

pub fn find_syllables_khmer(buffer: &mut Buffer) {
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
        let mut _klen = 0;
        let mut _trans = 0;
        let mut _keys: i32 = 0;
        let mut _acts: i32 = 0;
        let mut _nacts = 0;
        let mut __have = 0;
        '_resume: while (p != pe || p == eof) {
            '_again: while (true) {
                _acts = (_khmer_syllable_machine_from_state_actions[(cs) as usize]) as i32;
                _nacts = (_khmer_syllable_machine_actions[(_acts) as usize]) as u32;
                _acts += 1;
                while (_nacts > 0) {
                    match (_khmer_syllable_machine_actions[(_acts) as usize]) {
                        1 => {
                            ts = p;
                        }

                        _ => {}
                    }
                    _nacts -= 1;
                    _acts += 1;
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
                        _keys = (_khmer_syllable_machine_key_offsets[(cs) as usize]) as i32;
                        _trans = (_khmer_syllable_machine_index_offsets[(cs) as usize]) as u32;
                        _klen = (_khmer_syllable_machine_single_lengths[(cs) as usize]) as i32;
                        __have = 0;
                        if (_klen > 0) {
                            {
                                let mut _lower: i32 = _keys;
                                let mut _upper: i32 = _keys + _klen - 1;
                                let mut _mid: i32 = 0;
                                while (true) {
                                    if (_upper < _lower) {
                                        {
                                            _keys += _klen;
                                            _trans += (_klen) as u32;
                                            break;
                                        }
                                    }
                                    _mid = _lower + ((_upper - _lower) >> 1);
                                    if ((buffer.info[p].indic_category() as u8)
                                        < _khmer_syllable_machine_trans_keys[(_mid) as usize])
                                    {
                                        _upper = _mid - 1;
                                    } else if ((buffer.info[p].indic_category() as u8)
                                        > _khmer_syllable_machine_trans_keys[(_mid) as usize])
                                    {
                                        _lower = _mid + 1;
                                    } else {
                                        {
                                            __have = 1;
                                            _trans += (_mid - _keys) as u32;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        _klen = (_khmer_syllable_machine_range_lengths[(cs) as usize]) as i32;
                        if (__have == 0 && _klen > 0) {
                            {
                                let mut _lower: i32 = _keys;
                                let mut _upper: i32 = _keys + (_klen << 1) - 2;
                                let mut _mid: i32 = 0;
                                while (true) {
                                    if (_upper < _lower) {
                                        {
                                            _trans += (_klen) as u32;
                                            break;
                                        }
                                    }
                                    _mid = _lower + (((_upper - _lower) >> 1) & !1);
                                    if ((buffer.info[p].indic_category() as u8)
                                        < _khmer_syllable_machine_trans_keys[(_mid) as usize])
                                    {
                                        _upper = _mid - 2;
                                    } else if ((buffer.info[p].indic_category() as u8)
                                        > _khmer_syllable_machine_trans_keys[(_mid + 1) as usize])
                                    {
                                        _lower = _mid + 2;
                                    } else {
                                        {
                                            _trans += ((_mid - _keys) >> 1) as u32;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                cs = (_khmer_syllable_machine_cond_targs[(_trans) as usize]) as i32;
                if (_khmer_syllable_machine_cond_actions[(_trans) as usize] != 0) {
                    {
                        _acts = (_khmer_syllable_machine_cond_actions[(_trans) as usize]) as i32;
                        _nacts = (_khmer_syllable_machine_actions[(_acts) as usize]) as u32;
                        _acts += 1;
                        while (_nacts > 0) {
                            match (_khmer_syllable_machine_actions[(_acts) as usize]) {
                                2 => {
                                    te = p + 1;
                                }
                                3 => {
                                    act = 2;
                                }
                                4 => {
                                    act = 3;
                                }
                                5 => {
                                    te = p + 1;
                                    {
                                        found_syllable!(SyllableType::NonKhmerCluster);
                                    }
                                }
                                6 => {
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
                                8 => {
                                    te = p;
                                    p = p - 1;
                                    {
                                        found_syllable!(SyllableType::NonKhmerCluster);
                                    }
                                }
                                9 => {
                                    p = (te) - 1;
                                    {
                                        found_syllable!(SyllableType::ConsonantSyllable);
                                    }
                                }
                                10 => {
                                    p = (te) - 1;
                                    {
                                        found_syllable!(SyllableType::BrokenCluster);
                                    }
                                }
                                11 => match (act) {
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

                                _ => {}
                            }
                            _nacts -= 1;
                            _acts += 1;
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
                    _acts = (_khmer_syllable_machine_to_state_actions[(cs) as usize]) as i32;
                    _nacts = (_khmer_syllable_machine_actions[(_acts) as usize]) as u32;
                    _acts += 1;
                    while (_nacts > 0) {
                        match (_khmer_syllable_machine_actions[(_acts) as usize]) {
                            0 => {
                                ts = 0;
                            }

                            _ => {}
                        }
                        _nacts -= 1;
                        _acts += 1;
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
