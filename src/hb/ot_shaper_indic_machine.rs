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
    clippy::comparison_chain,
    clippy::double_parens,
    clippy::unnecessary_cast,
    clippy::single_match,
    clippy::never_loop
)]

use super::buffer::hb_buffer_t;

static _indic_syllable_machine_trans_keys: [u8; 284] = [
    7, 7, 3, 7, 4, 6, 4, 7, 3, 7, 5, 5, 14, 14, 3, 7, 3, 11, 3, 7, 7, 7, 4, 6, 4, 7, 3, 7, 5, 5,
    14, 14, 3, 7, 3, 11, 3, 11, 3, 11, 7, 7, 4, 6, 4, 7, 3, 7, 5, 5, 14, 14, 3, 7, 3, 7, 3, 11, 7,
    7, 4, 6, 4, 7, 3, 7, 5, 5, 14, 14, 3, 7, 3, 7, 4, 7, 7, 7, 0, 17, 2, 15, 2, 15, 3, 15, 0, 14,
    4, 8, 4, 8, 8, 8, 4, 8, 0, 14, 0, 14, 0, 14, 2, 8, 3, 8, 4, 8, 3, 8, 4, 8, 2, 8, 4, 8, 2, 15,
    2, 15, 2, 15, 2, 15, 3, 15, 0, 14, 2, 15, 2, 15, 3, 15, 0, 14, 4, 8, 8, 8, 4, 8, 0, 14, 0, 14,
    2, 8, 3, 8, 4, 8, 3, 8, 4, 8, 4, 8, 2, 8, 4, 8, 2, 15, 2, 15, 3, 7, 2, 15, 2, 15, 3, 15, 0, 14,
    2, 15, 0, 14, 4, 8, 8, 8, 4, 8, 0, 14, 0, 14, 2, 8, 3, 8, 4, 8, 2, 15, 3, 8, 4, 8, 4, 8, 2, 8,
    4, 8, 2, 15, 3, 11, 3, 7, 2, 15, 2, 15, 3, 15, 0, 14, 2, 15, 0, 14, 4, 8, 8, 8, 4, 8, 0, 14, 0,
    14, 2, 8, 3, 8, 4, 8, 2, 15, 3, 8, 4, 8, 4, 8, 2, 8, 4, 8, 0, 15, 2, 15, 0, 15, 3, 11, 4, 8, 8,
    8, 4, 8, 0, 14, 2, 8, 4, 8, 4, 8, 8, 8, 4, 8, 0, 14, 0, 0,
];
static _indic_syllable_machine_char_class: [i8; 21] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 12, 17, 0, 0,
];
static _indic_syllable_machine_index_offsets: [i16; 143] = [
    0, 1, 6, 9, 13, 18, 19, 20, 25, 34, 39, 40, 43, 47, 52, 53, 54, 59, 68, 77, 86, 87, 90, 94, 99,
    100, 101, 106, 111, 120, 121, 124, 128, 133, 134, 135, 140, 145, 149, 150, 168, 182, 196, 209,
    224, 229, 234, 235, 240, 255, 270, 285, 292, 298, 303, 309, 314, 321, 326, 340, 354, 368, 382,
    395, 410, 424, 438, 451, 466, 471, 472, 477, 492, 507, 514, 520, 525, 531, 536, 541, 548, 553,
    567, 581, 586, 600, 614, 627, 642, 656, 671, 676, 677, 682, 697, 712, 719, 725, 730, 744, 750,
    755, 760, 767, 772, 786, 795, 800, 814, 828, 841, 856, 870, 885, 890, 891, 896, 911, 926, 933,
    939, 944, 958, 964, 969, 974, 981, 986, 1002, 1016, 1032, 1041, 1046, 1047, 1052, 1067, 1074,
    1079, 1084, 1085, 1090, 0, 0,
];
static _indic_syllable_machine_indices: [i16; 1107] = [
    1, 2, 3, 3, 4, 1, 3, 3, 4, 3, 3, 4, 1, 5, 3, 3, 4, 1, 6, 7, 8, 3, 3, 4, 1, 2, 3, 3, 4, 1, 0, 0,
    0, 9, 11, 12, 12, 13, 14, 14, 12, 12, 13, 12, 12, 13, 14, 15, 12, 12, 13, 14, 16, 17, 18, 12,
    12, 13, 14, 11, 12, 12, 13, 14, 10, 10, 10, 19, 11, 12, 12, 13, 14, 10, 10, 10, 20, 22, 23, 23,
    24, 25, 21, 21, 21, 26, 25, 23, 23, 24, 23, 23, 24, 25, 28, 23, 23, 24, 25, 29, 30, 22, 23, 23,
    24, 25, 31, 23, 23, 24, 25, 33, 34, 34, 35, 36, 32, 32, 32, 37, 36, 34, 34, 35, 34, 34, 35, 36,
    38, 34, 34, 35, 36, 39, 40, 33, 34, 34, 35, 36, 41, 34, 34, 35, 36, 23, 23, 24, 1, 43, 46, 47,
    48, 49, 50, 51, 24, 25, 52, 53, 53, 26, 45, 54, 55, 56, 57, 58, 60, 61, 62, 63, 4, 1, 64, 59,
    59, 9, 59, 59, 59, 65, 66, 61, 67, 67, 4, 1, 64, 59, 59, 59, 59, 59, 59, 65, 61, 67, 67, 4, 1,
    64, 59, 59, 59, 59, 59, 59, 65, 46, 59, 59, 59, 68, 69, 59, 1, 64, 59, 59, 59, 59, 59, 46, 70,
    70, 59, 1, 64, 64, 59, 59, 71, 64, 64, 64, 59, 59, 59, 64, 46, 59, 72, 59, 70, 70, 59, 1, 64,
    59, 59, 59, 59, 59, 46, 46, 59, 59, 59, 70, 70, 59, 1, 64, 59, 59, 59, 59, 59, 46, 46, 59, 59,
    59, 70, 69, 59, 1, 64, 59, 59, 59, 59, 59, 46, 73, 7, 74, 75, 4, 1, 64, 7, 74, 75, 4, 1, 64,
    74, 74, 4, 1, 64, 76, 77, 77, 4, 1, 64, 68, 78, 59, 1, 64, 68, 59, 70, 70, 59, 1, 64, 70, 78,
    59, 1, 64, 60, 61, 67, 67, 4, 1, 64, 59, 59, 59, 59, 59, 59, 65, 60, 61, 62, 67, 4, 1, 64, 59,
    59, 9, 59, 59, 59, 65, 80, 81, 82, 83, 13, 14, 84, 79, 79, 20, 79, 79, 79, 85, 86, 81, 87, 83,
    13, 14, 84, 79, 79, 79, 79, 79, 79, 85, 81, 87, 83, 13, 14, 84, 79, 79, 79, 79, 79, 79, 85, 88,
    79, 79, 79, 89, 90, 79, 14, 84, 79, 79, 79, 79, 79, 88, 91, 81, 92, 93, 13, 14, 84, 79, 79, 19,
    79, 79, 79, 85, 94, 81, 87, 87, 13, 14, 84, 79, 79, 79, 79, 79, 79, 85, 81, 87, 87, 13, 14, 84,
    79, 79, 79, 79, 79, 79, 85, 88, 79, 79, 79, 95, 90, 79, 14, 84, 79, 79, 79, 79, 79, 88, 84, 79,
    79, 96, 84, 84, 84, 79, 79, 79, 84, 88, 79, 97, 79, 95, 95, 79, 14, 84, 79, 79, 79, 79, 79, 88,
    88, 79, 79, 79, 95, 95, 79, 14, 84, 79, 79, 79, 79, 79, 88, 98, 17, 99, 100, 13, 14, 84, 17,
    99, 100, 13, 14, 84, 99, 99, 13, 14, 84, 101, 102, 102, 13, 14, 84, 89, 103, 79, 14, 84, 95,
    95, 79, 14, 84, 89, 79, 95, 95, 79, 14, 84, 95, 103, 79, 14, 84, 91, 81, 87, 87, 13, 14, 84,
    79, 79, 79, 79, 79, 79, 85, 91, 81, 92, 87, 13, 14, 84, 79, 79, 19, 79, 79, 79, 85, 11, 12, 12,
    13, 14, 80, 81, 87, 83, 13, 14, 84, 79, 79, 79, 79, 79, 79, 85, 105, 49, 106, 106, 24, 25, 52,
    104, 104, 104, 104, 104, 104, 56, 49, 106, 106, 24, 25, 52, 104, 104, 104, 104, 104, 104, 56,
    107, 104, 104, 104, 108, 109, 104, 25, 52, 104, 104, 104, 104, 104, 107, 48, 49, 110, 111, 24,
    25, 52, 104, 104, 26, 104, 104, 104, 56, 107, 104, 104, 104, 112, 109, 104, 25, 52, 104, 104,
    104, 104, 104, 107, 52, 104, 104, 113, 52, 52, 52, 104, 104, 104, 52, 107, 104, 114, 104, 112,
    112, 104, 25, 52, 104, 104, 104, 104, 104, 107, 107, 104, 104, 104, 112, 112, 104, 25, 52, 104,
    104, 104, 104, 104, 107, 115, 30, 116, 117, 24, 25, 52, 30, 116, 117, 24, 25, 52, 116, 116, 24,
    25, 52, 48, 49, 106, 106, 24, 25, 52, 104, 104, 104, 104, 104, 104, 56, 118, 119, 119, 24, 25,
    52, 108, 120, 104, 25, 52, 112, 112, 104, 25, 52, 108, 104, 112, 112, 104, 25, 52, 112, 120,
    104, 25, 52, 48, 49, 110, 106, 24, 25, 52, 104, 104, 26, 104, 104, 104, 56, 22, 23, 23, 24, 25,
    121, 121, 121, 26, 22, 23, 23, 24, 25, 123, 124, 125, 126, 35, 36, 127, 122, 122, 37, 122, 122,
    122, 128, 129, 124, 126, 126, 35, 36, 127, 122, 122, 122, 122, 122, 122, 128, 124, 126, 126,
    35, 36, 127, 122, 122, 122, 122, 122, 122, 128, 130, 122, 122, 122, 131, 132, 122, 36, 127,
    122, 122, 122, 122, 122, 130, 123, 124, 125, 53, 35, 36, 127, 122, 122, 37, 122, 122, 122, 128,
    130, 122, 122, 122, 133, 132, 122, 36, 127, 122, 122, 122, 122, 122, 130, 127, 122, 122, 134,
    127, 127, 127, 122, 122, 122, 127, 130, 122, 135, 122, 133, 133, 122, 36, 127, 122, 122, 122,
    122, 122, 130, 130, 122, 122, 122, 133, 133, 122, 36, 127, 122, 122, 122, 122, 122, 130, 136,
    40, 137, 138, 35, 36, 127, 40, 137, 138, 35, 36, 127, 137, 137, 35, 36, 127, 123, 124, 126,
    126, 35, 36, 127, 122, 122, 122, 122, 122, 122, 128, 139, 140, 140, 35, 36, 127, 131, 141, 122,
    36, 127, 133, 133, 122, 36, 127, 131, 122, 133, 133, 122, 36, 127, 133, 141, 122, 36, 127, 46,
    47, 48, 49, 110, 106, 24, 25, 52, 53, 53, 26, 104, 104, 46, 56, 60, 142, 62, 63, 4, 1, 64, 59,
    59, 9, 59, 59, 59, 65, 46, 47, 48, 49, 143, 144, 24, 145, 146, 59, 53, 26, 59, 59, 46, 56, 22,
    147, 147, 24, 145, 64, 59, 59, 26, 146, 59, 59, 148, 146, 146, 146, 59, 59, 59, 146, 46, 59,
    72, 22, 147, 147, 24, 145, 64, 59, 59, 59, 59, 59, 46, 150, 149, 151, 151, 149, 43, 152, 151,
    151, 149, 43, 152, 152, 149, 149, 153, 152, 152, 152, 149, 149, 149, 152, 46, 121, 121, 121,
    121, 121, 121, 121, 121, 53, 121, 121, 121, 121, 46, 0, 0,
];
static _indic_syllable_machine_index_defaults: [i16; 143] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 21, 21, 27, 21, 21, 21, 21,
    21, 21, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 42, 45, 59, 59, 59, 59, 59, 59, 59, 59, 59, 59,
    59, 59, 59, 59, 59, 59, 59, 59, 59, 59, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79,
    79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 104, 104, 104, 104, 104, 104, 104, 104, 104, 104,
    104, 104, 104, 104, 104, 104, 104, 104, 104, 104, 121, 121, 122, 122, 122, 122, 122, 122, 122,
    122, 122, 122, 122, 122, 122, 122, 122, 122, 122, 122, 122, 122, 104, 59, 59, 59, 59, 59, 59,
    59, 149, 149, 149, 149, 149, 121, 0, 0,
];
static _indic_syllable_machine_cond_targs: [i16; 156] = [
    39, 45, 50, 2, 51, 5, 6, 53, 57, 58, 39, 67, 11, 73, 68, 14, 15, 75, 80, 81, 84, 39, 89, 21,
    95, 90, 98, 39, 24, 25, 97, 103, 39, 112, 30, 118, 113, 121, 33, 34, 120, 126, 39, 137, 39, 39,
    40, 60, 85, 87, 105, 106, 91, 107, 127, 128, 99, 135, 140, 39, 41, 43, 8, 59, 46, 54, 42, 1,
    44, 48, 0, 47, 49, 52, 3, 4, 55, 7, 56, 39, 61, 63, 18, 83, 69, 76, 62, 9, 64, 78, 71, 65, 17,
    82, 66, 10, 70, 72, 74, 12, 13, 77, 16, 79, 39, 86, 26, 88, 101, 93, 19, 104, 20, 92, 94, 96,
    22, 23, 100, 27, 102, 39, 39, 108, 110, 28, 35, 114, 122, 109, 111, 124, 116, 29, 115, 117,
    119, 31, 32, 123, 36, 125, 129, 130, 134, 131, 132, 37, 133, 39, 136, 38, 138, 139, 0, 0,
];
static _indic_syllable_machine_cond_actions: [i8; 156] = [
    1, 0, 2, 0, 2, 0, 0, 2, 2, 2, 3, 2, 0, 2, 0, 0, 0, 2, 2, 2, 2, 4, 2, 0, 5, 0, 5, 6, 0, 0, 5, 2,
    7, 2, 0, 2, 0, 2, 0, 0, 2, 2, 8, 0, 0, 11, 2, 2, 5, 0, 12, 12, 0, 2, 5, 2, 5, 2, 0, 13, 2, 0,
    0, 2, 0, 2, 2, 0, 2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 2, 14, 2, 0, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 0,
    2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 2, 15, 5, 0, 5, 2, 2, 0, 5, 0, 0, 2, 5, 0, 0, 0, 0, 2, 16, 17, 2,
    0, 0, 0, 0, 2, 2, 2, 2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 2, 0, 18, 18, 0, 0, 0, 0, 19, 2, 0, 0, 0, 0,
    0,
];
static _indic_syllable_machine_to_state_actions: [i8; 143] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _indic_syllable_machine_from_state_actions: [i8; 143] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _indic_syllable_machine_eof_trans: [i16; 143] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 22, 22, 28, 22, 22, 22, 22,
    22, 22, 33, 33, 33, 33, 33, 33, 33, 33, 33, 1, 43, 45, 60, 60, 60, 60, 60, 60, 60, 60, 60, 60,
    60, 60, 60, 60, 60, 60, 60, 60, 60, 60, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80,
    80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 105, 105, 105, 105, 105, 105, 105, 105, 105, 105,
    105, 105, 105, 105, 105, 105, 105, 105, 105, 105, 122, 122, 123, 123, 123, 123, 123, 123, 123,
    123, 123, 123, 123, 123, 123, 123, 123, 123, 123, 123, 123, 123, 105, 60, 60, 60, 60, 60, 60,
    60, 150, 150, 150, 150, 150, 122, 0, 0,
];
static indic_syllable_machine_start: i32 = 39;
static indic_syllable_machine_first_final: i32 = 39;
static indic_syllable_machine_error: i32 = -1;
static indic_syllable_machine_en_main: i32 = 39;
#[derive(Clone, Copy)]
pub enum SyllableType {
    ConsonantSyllable = 0,
    VowelSyllable,
    StandaloneCluster,
    SymbolCluster,
    BrokenCluster,
    NonIndicCluster,
}

pub fn find_syllables_indic(buffer: &mut hb_buffer_t) {
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
            found_syllable(ts, te, &mut syllable_serial, $kind, buffer)
        }};
    }

    {
        cs = (indic_syllable_machine_start) as i32;
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
                match (_indic_syllable_machine_from_state_actions[(cs) as usize]) {
                    10 => {
                        ts = p;
                    }

                    _ => {}
                }
                if (p == eof) {
                    {
                        if (_indic_syllable_machine_eof_trans[(cs) as usize] > 0) {
                            {
                                _trans =
                                    (_indic_syllable_machine_eof_trans[(cs) as usize]) as u32 - 1;
                            }
                        }
                    }
                } else {
                    {
                        _keys = (cs << 1) as i32;
                        _inds = (_indic_syllable_machine_index_offsets[(cs) as usize]) as i32;
                        if ((buffer.info[p].indic_category() as u8) <= 19
                            && (buffer.info[p].indic_category() as u8) >= 1)
                        {
                            {
                                _ic = (_indic_syllable_machine_char_class
                                    [((buffer.info[p].indic_category() as u8) as i32 - 1) as usize])
                                    as i32;
                                if (_ic
                                    <= (_indic_syllable_machine_trans_keys[(_keys + 1) as usize])
                                        as i32
                                    && _ic
                                        >= (_indic_syllable_machine_trans_keys[(_keys) as usize])
                                            as i32)
                                {
                                    _trans = (_indic_syllable_machine_indices[(_inds
                                        + (_ic
                                            - (_indic_syllable_machine_trans_keys[(_keys) as usize])
                                                as i32)
                                            as i32)
                                        as usize])
                                        as u32;
                                } else {
                                    _trans = (_indic_syllable_machine_index_defaults[(cs) as usize])
                                        as u32;
                                }
                            }
                        } else {
                            {
                                _trans =
                                    (_indic_syllable_machine_index_defaults[(cs) as usize]) as u32;
                            }
                        }
                    }
                }
                cs = (_indic_syllable_machine_cond_targs[(_trans) as usize]) as i32;
                if (_indic_syllable_machine_cond_actions[(_trans) as usize] != 0) {
                    {
                        match (_indic_syllable_machine_cond_actions[(_trans) as usize]) {
                            2 => {
                                te = p + 1;
                            }
                            11 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::NonIndicCluster);
                                }
                            }
                            13 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::ConsonantSyllable);
                                }
                            }
                            14 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::VowelSyllable);
                                }
                            }
                            17 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::StandaloneCluster);
                                }
                            }
                            19 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::SymbolCluster);
                                }
                            }
                            15 => {
                                {
                                    {
                                        te = p;
                                        p = p - 1;
                                        {
                                            found_syllable!(SyllableType::BrokenCluster);
                                            /*buffer->scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;*/
                                        }
                                    }
                                }
                            }
                            16 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::NonIndicCluster);
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
                                    found_syllable!(SyllableType::VowelSyllable);
                                }
                            }
                            7 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::StandaloneCluster);
                                }
                            }
                            8 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::SymbolCluster);
                                }
                            }
                            4 => {
                                {
                                    {
                                        p = (te) - 1;
                                        {
                                            found_syllable!(SyllableType::BrokenCluster);
                                            /*buffer->scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;*/
                                        }
                                    }
                                }
                            }
                            6 => {
                                {
                                    {
                                        match (act) {
                                            1 => {
                                                p = (te) - 1;
                                                {
                                                    found_syllable!(
                                                        SyllableType::ConsonantSyllable
                                                    );
                                                }
                                            }
                                            5 => {
                                                p = (te) - 1;
                                                {
                                                    found_syllable!(SyllableType::BrokenCluster);
                                                    /*buffer->scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;*/
                                                }
                                            }
                                            6 => {
                                                p = (te) - 1;
                                                {
                                                    found_syllable!(SyllableType::NonIndicCluster);
                                                }
                                            }

                                            _ => {}
                                        }
                                    }
                                }
                            }
                            18 => {
                                {
                                    {
                                        te = p + 1;
                                    }
                                }
                                {
                                    {
                                        act = 1;
                                    }
                                }
                            }
                            5 => {
                                {
                                    {
                                        te = p + 1;
                                    }
                                }
                                {
                                    {
                                        act = 5;
                                    }
                                }
                            }
                            12 => {
                                {
                                    {
                                        te = p + 1;
                                    }
                                }
                                {
                                    {
                                        act = 6;
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
                    if (cs >= 39) {
                        break '_resume;
                    }
                }
            } else {
                {
                    match (_indic_syllable_machine_to_state_actions[(cs) as usize]) {
                        9 => {
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
