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

static _myanmar_syllable_machine_actions: [i8; 23] = [
    0, 1, 0, 1, 1, 1, 2, 1, 3, 1, 4, 1, 5, 1, 6, 1, 7, 1, 8, 1, 9, 0, 0,
];
static _myanmar_syllable_machine_key_offsets: [i16; 54] = [
    0, 24, 41, 47, 50, 55, 62, 67, 71, 81, 88, 97, 105, 108, 123, 134, 144, 153, 161, 172, 184,
    196, 210, 223, 239, 245, 248, 253, 260, 265, 269, 279, 286, 295, 303, 306, 323, 338, 349, 359,
    368, 376, 387, 399, 411, 425, 438, 454, 471, 487, 509, 514, 0, 0,
];
static _myanmar_syllable_machine_trans_keys: [u8; 517] = [
    3, 4, 8, 10, 11, 16, 18, 19, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 1, 2, 5, 6, 3, 4,
    8, 10, 18, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 5, 6, 8, 18, 25, 29, 5, 6, 8, 5, 6, 8, 25,
    29, 5, 6, 3, 8, 10, 18, 25, 5, 6, 8, 18, 25, 5, 6, 8, 25, 5, 6, 3, 8, 10, 18, 21, 25, 26, 29,
    5, 6, 3, 8, 10, 25, 29, 5, 6, 3, 8, 10, 18, 25, 26, 29, 5, 6, 3, 8, 10, 25, 26, 29, 5, 6, 16,
    1, 2, 3, 8, 10, 18, 21, 22, 23, 24, 25, 26, 27, 28, 29, 5, 6, 3, 8, 10, 18, 25, 26, 27, 28, 29,
    5, 6, 3, 8, 10, 25, 26, 27, 28, 29, 5, 6, 3, 8, 10, 25, 26, 27, 29, 5, 6, 3, 8, 10, 25, 27, 29,
    5, 6, 3, 8, 10, 25, 26, 27, 28, 29, 30, 5, 6, 3, 8, 10, 21, 23, 25, 26, 27, 28, 29, 5, 6, 3, 8,
    10, 18, 21, 25, 26, 27, 28, 29, 5, 6, 3, 8, 10, 18, 21, 22, 23, 25, 26, 27, 28, 29, 5, 6, 3, 8,
    10, 21, 22, 23, 25, 26, 27, 28, 29, 5, 6, 3, 4, 8, 10, 18, 21, 22, 23, 24, 25, 26, 27, 28, 29,
    5, 6, 8, 18, 25, 29, 5, 6, 8, 5, 6, 8, 25, 29, 5, 6, 3, 8, 10, 18, 25, 5, 6, 8, 18, 25, 5, 6,
    8, 25, 5, 6, 3, 8, 10, 18, 21, 25, 26, 29, 5, 6, 3, 8, 10, 25, 29, 5, 6, 3, 8, 10, 18, 25, 26,
    29, 5, 6, 3, 8, 10, 25, 26, 29, 5, 6, 16, 1, 2, 3, 4, 8, 10, 18, 21, 22, 23, 24, 25, 26, 27,
    28, 29, 30, 5, 6, 3, 8, 10, 18, 21, 22, 23, 24, 25, 26, 27, 28, 29, 5, 6, 3, 8, 10, 18, 25, 26,
    27, 28, 29, 5, 6, 3, 8, 10, 25, 26, 27, 28, 29, 5, 6, 3, 8, 10, 25, 26, 27, 29, 5, 6, 3, 8, 10,
    25, 27, 29, 5, 6, 3, 8, 10, 25, 26, 27, 28, 29, 30, 5, 6, 3, 8, 10, 21, 23, 25, 26, 27, 28, 29,
    5, 6, 3, 8, 10, 18, 21, 25, 26, 27, 28, 29, 5, 6, 3, 8, 10, 18, 21, 22, 23, 25, 26, 27, 28, 29,
    5, 6, 3, 8, 10, 21, 22, 23, 25, 26, 27, 28, 29, 5, 6, 3, 4, 8, 10, 18, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 5, 6, 3, 4, 8, 10, 18, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 5, 6, 3, 4, 8, 10,
    18, 21, 22, 23, 24, 25, 26, 27, 28, 29, 5, 6, 3, 4, 8, 10, 11, 16, 18, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 32, 1, 2, 5, 6, 11, 16, 32, 1, 2, 8, 0, 0,
];
static _myanmar_syllable_machine_single_lengths: [i8; 54] = [
    20, 15, 4, 1, 3, 5, 3, 2, 8, 5, 7, 6, 1, 13, 9, 8, 7, 6, 9, 10, 10, 12, 11, 14, 4, 1, 3, 5, 3,
    2, 8, 5, 7, 6, 1, 15, 13, 9, 8, 7, 6, 9, 10, 10, 12, 11, 14, 15, 14, 18, 3, 1, 0, 0,
];
static _myanmar_syllable_machine_range_lengths: [i8; 54] = [
    2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 0, 0, 0,
];
static _myanmar_syllable_machine_index_offsets: [i16; 54] = [
    0, 23, 40, 46, 49, 54, 61, 66, 70, 80, 87, 96, 104, 107, 122, 133, 143, 152, 160, 171, 183,
    195, 209, 222, 238, 244, 247, 252, 259, 264, 268, 278, 285, 294, 302, 305, 322, 337, 348, 358,
    367, 375, 386, 398, 410, 424, 437, 453, 470, 486, 507, 512, 0, 0,
];
static _myanmar_syllable_machine_cond_targs: [i8; 568] = [
    24, 34, 25, 31, 1, 47, 36, 50, 37, 42, 43, 44, 27, 39, 40, 41, 30, 46, 51, 1, 1, 0, 0, 2, 12,
    3, 9, 13, 14, 19, 20, 21, 5, 16, 17, 18, 8, 23, 0, 0, 3, 4, 5, 8, 0, 0, 3, 0, 0, 3, 5, 8, 0, 0,
    6, 3, 5, 7, 5, 0, 0, 3, 7, 5, 0, 0, 3, 5, 0, 0, 2, 3, 9, 10, 10, 5, 11, 8, 0, 0, 2, 3, 9, 5, 8,
    0, 0, 2, 3, 9, 10, 5, 11, 8, 0, 0, 2, 3, 9, 5, 11, 8, 0, 0, 1, 1, 0, 2, 3, 9, 13, 14, 19, 20,
    21, 5, 16, 17, 18, 8, 0, 0, 2, 3, 9, 15, 5, 16, 17, 18, 8, 0, 0, 2, 3, 9, 5, 16, 17, 18, 8, 0,
    0, 2, 3, 9, 5, 16, 17, 8, 0, 0, 2, 3, 9, 5, 17, 8, 0, 0, 2, 3, 9, 5, 16, 17, 18, 8, 15, 0, 0,
    2, 3, 9, 14, 20, 5, 16, 17, 18, 8, 0, 0, 2, 3, 9, 15, 14, 5, 16, 17, 18, 8, 0, 0, 2, 3, 9, 22,
    14, 19, 20, 5, 16, 17, 18, 8, 0, 0, 2, 3, 9, 14, 19, 20, 5, 16, 17, 18, 8, 0, 0, 2, 12, 3, 9,
    13, 14, 19, 20, 21, 5, 16, 17, 18, 8, 0, 0, 25, 26, 27, 30, 0, 0, 25, 0, 0, 25, 27, 30, 0, 0,
    28, 25, 27, 29, 27, 0, 0, 25, 29, 27, 0, 0, 25, 27, 0, 0, 24, 25, 31, 32, 32, 27, 33, 30, 0, 0,
    24, 25, 31, 27, 30, 0, 0, 24, 25, 31, 32, 27, 33, 30, 0, 0, 24, 25, 31, 27, 33, 30, 0, 0, 35,
    35, 0, 24, 34, 25, 31, 36, 37, 42, 43, 44, 27, 39, 40, 41, 30, 46, 0, 0, 24, 25, 31, 36, 37,
    42, 43, 44, 27, 39, 40, 41, 30, 0, 0, 24, 25, 31, 38, 27, 39, 40, 41, 30, 0, 0, 24, 25, 31, 27,
    39, 40, 41, 30, 0, 0, 24, 25, 31, 27, 39, 40, 30, 0, 0, 24, 25, 31, 27, 40, 30, 0, 0, 24, 25,
    31, 27, 39, 40, 41, 30, 38, 0, 0, 24, 25, 31, 37, 43, 27, 39, 40, 41, 30, 0, 0, 24, 25, 31, 38,
    37, 27, 39, 40, 41, 30, 0, 0, 24, 25, 31, 45, 37, 42, 43, 27, 39, 40, 41, 30, 0, 0, 24, 25, 31,
    37, 42, 43, 27, 39, 40, 41, 30, 0, 0, 24, 34, 25, 31, 36, 37, 42, 43, 44, 27, 39, 40, 41, 30,
    0, 0, 2, 12, 3, 9, 48, 14, 19, 20, 21, 5, 16, 17, 18, 8, 23, 0, 0, 2, 49, 3, 9, 13, 14, 19, 20,
    21, 5, 16, 17, 18, 8, 0, 0, 24, 34, 25, 31, 1, 1, 36, 37, 42, 43, 44, 27, 39, 40, 41, 30, 46,
    1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];
static _myanmar_syllable_machine_cond_actions: [i8; 568] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 13, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 5, 15, 0, 5, 15, 0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 5, 15,
    0, 0, 0, 5, 15, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0,
    0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5,
    15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 0, 5,
    15, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5,
    15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 11,
    17, 0, 11, 17, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 11, 17, 0, 0, 11, 17, 0, 0, 0,
    0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0,
    11, 17, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 0, 11, 17, 0, 0,
    0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 11, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 5, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 17, 0, 0, 0, 0,
    19, 9, 19, 0, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
    17, 17, 15, 15, 17, 19, 19, 0, 0,
];
static _myanmar_syllable_machine_to_state_actions: [i8; 54] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_from_state_actions: [i8; 54] = [
    3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _myanmar_syllable_machine_eof_trans: [i16; 54] = [
    515, 516, 517, 518, 519, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529, 530, 531, 532, 533,
    534, 535, 536, 537, 538, 539, 540, 541, 542, 543, 544, 545, 546, 547, 548, 549, 550, 551, 552,
    553, 554, 555, 556, 557, 558, 559, 560, 561, 562, 563, 564, 565, 566, 0, 0,
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
        let mut _klen = 0;
        let mut _trans = 0;
        let mut _keys: i32 = 0;
        let mut _acts: i32 = 0;
        let mut _nacts = 0;
        let mut __have = 0;
        '_resume: while (p != pe || p == eof) {
            '_again: while (true) {
                _acts = (_myanmar_syllable_machine_from_state_actions[(cs) as usize]) as i32;
                _nacts = (_myanmar_syllable_machine_actions[(_acts) as usize]) as u32;
                _acts += 1;
                while (_nacts > 0) {
                    match (_myanmar_syllable_machine_actions[(_acts) as usize]) {
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
                        if (_myanmar_syllable_machine_eof_trans[(cs) as usize] > 0) {
                            {
                                _trans =
                                    (_myanmar_syllable_machine_eof_trans[(cs) as usize]) as u32 - 1;
                            }
                        }
                    }
                } else {
                    {
                        _keys = (_myanmar_syllable_machine_key_offsets[(cs) as usize]) as i32;
                        _trans = (_myanmar_syllable_machine_index_offsets[(cs) as usize]) as u32;
                        _klen = (_myanmar_syllable_machine_single_lengths[(cs) as usize]) as i32;
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
                                        < _myanmar_syllable_machine_trans_keys[(_mid) as usize])
                                    {
                                        _upper = _mid - 1;
                                    } else if ((buffer.info[p].indic_category() as u8)
                                        > _myanmar_syllable_machine_trans_keys[(_mid) as usize])
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
                        _klen = (_myanmar_syllable_machine_range_lengths[(cs) as usize]) as i32;
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
                                        < _myanmar_syllable_machine_trans_keys[(_mid) as usize])
                                    {
                                        _upper = _mid - 2;
                                    } else if ((buffer.info[p].indic_category() as u8)
                                        > _myanmar_syllable_machine_trans_keys[(_mid + 1) as usize])
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
                cs = (_myanmar_syllable_machine_cond_targs[(_trans) as usize]) as i32;
                if (_myanmar_syllable_machine_cond_actions[(_trans) as usize] != 0) {
                    {
                        _acts = (_myanmar_syllable_machine_cond_actions[(_trans) as usize]) as i32;
                        _nacts = (_myanmar_syllable_machine_actions[(_acts) as usize]) as u32;
                        _acts += 1;
                        while (_nacts > 0) {
                            match (_myanmar_syllable_machine_actions[(_acts) as usize]) {
                                2 => {
                                    te = p + 1;
                                    {
                                        found_syllable!(SyllableType::ConsonantSyllable);
                                    }
                                }
                                3 => {
                                    te = p + 1;
                                    {
                                        found_syllable!(SyllableType::NonMyanmarCluster);
                                    }
                                }
                                4 => {
                                    te = p + 1;
                                    {
                                        found_syllable!(SyllableType::PunctuationCluster);
                                    }
                                }
                                5 => {
                                    te = p + 1;
                                    {
                                        found_syllable!(SyllableType::BrokenCluster);
                                    }
                                }
                                6 => {
                                    te = p + 1;
                                    {
                                        found_syllable!(SyllableType::NonMyanmarCluster);
                                    }
                                }
                                7 => {
                                    te = p;
                                    p = p - 1;
                                    {
                                        found_syllable!(SyllableType::ConsonantSyllable);
                                    }
                                }
                                8 => {
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
                            _nacts -= 1;
                            _acts += 1;
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
                    _acts = (_myanmar_syllable_machine_to_state_actions[(cs) as usize]) as i32;
                    _nacts = (_myanmar_syllable_machine_actions[(_acts) as usize]) as u32;
                    _acts += 1;
                    while (_nacts > 0) {
                        match (_myanmar_syllable_machine_actions[(_acts) as usize]) {
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
