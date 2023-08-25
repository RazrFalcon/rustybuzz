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
use crate::complex::universal::category;
use crate::GlyphInfo;

static _use_syllable_machine_actions: [i8 ; 37] = [ 0, 1, 0, 1, 1, 1, 2, 1, 3, 1, 4, 1, 5, 1, 6, 1, 7, 1, 8, 1, 9, 1, 10, 1, 11, 1, 12, 1, 13, 1, 14, 1, 15, 1, 16, 0 , 0 ];
static _use_syllable_machine_key_offsets: [i16 ; 64] = [ 0, 1, 2, 38, 62, 86, 87, 103, 114, 120, 125, 129, 131, 132, 142, 151, 159, 160, 167, 182, 196, 209, 227, 244, 263, 286, 298, 299, 300, 326, 328, 329, 353, 369, 380, 386, 391, 395, 397, 398, 408, 417, 425, 432, 447, 461, 474, 492, 509, 528, 551, 563, 564, 565, 566, 595, 619, 621, 622, 624, 626, 629, 0 , 0 ];
static _use_syllable_machine_trans_keys: [u8 ; 633] = [ 1, 1, 0, 1, 4, 5, 11, 12, 13, 18, 19, 23, 24, 25, 26, 27, 28, 30, 31, 32, 33, 34, 35, 37, 38, 39, 41, 42, 43, 44, 45, 46, 47, 48, 49, 51, 22, 29, 11, 12, 23, 24, 25, 26, 27, 28, 30, 31, 32, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 29, 11, 12, 23, 24, 25, 26, 27, 28, 30, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 29, 31, 32, 1, 22, 23, 24, 25, 26, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 23, 24, 25, 26, 37, 38, 39, 45, 46, 47, 48, 24, 25, 26, 45, 46, 47, 25, 26, 45, 46, 47, 26, 45, 46, 47, 45, 46, 46, 24, 25, 26, 37, 38, 39, 45, 46, 47, 48, 24, 25, 26, 38, 39, 45, 46, 47, 48, 24, 25, 26, 39, 45, 46, 47, 48, 1, 24, 25, 26, 45, 46, 47, 48, 23, 24, 25, 26, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 23, 24, 25, 26, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 23, 24, 25, 26, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 23, 24, 25, 26, 28, 29, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 23, 24, 25, 26, 29, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 23, 24, 25, 26, 27, 28, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 29, 11, 12, 23, 24, 25, 26, 27, 28, 30, 32, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 29, 1, 23, 24, 25, 26, 37, 38, 39, 45, 46, 47, 48, 13, 4, 11, 12, 23, 24, 25, 26, 27, 28, 30, 31, 32, 33, 34, 35, 37, 38, 39, 41, 42, 44, 45, 46, 47, 48, 22, 29, 41, 42, 42, 11, 12, 23, 24, 25, 26, 27, 28, 30, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 29, 31, 32, 22, 23, 24, 25, 26, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 23, 24, 25, 26, 37, 38, 39, 45, 46, 47, 48, 24, 25, 26, 45, 46, 47, 25, 26, 45, 46, 47, 26, 45, 46, 47, 45, 46, 46, 24, 25, 26, 37, 38, 39, 45, 46, 47, 48, 24, 25, 26, 38, 39, 45, 46, 47, 48, 24, 25, 26, 39, 45, 46, 47, 48, 24, 25, 26, 45, 46, 47, 48, 23, 24, 25, 26, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 23, 24, 25, 26, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 23, 24, 25, 26, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 23, 24, 25, 26, 28, 29, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 23, 24, 25, 26, 29, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 23, 24, 25, 26, 27, 28, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 29, 11, 12, 23, 24, 25, 26, 27, 28, 30, 32, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 29, 1, 23, 24, 25, 26, 37, 38, 39, 45, 46, 47, 48, 1, 4, 13, 1, 5, 11, 12, 13, 23, 24, 25, 26, 27, 28, 30, 31, 32, 33, 34, 35, 37, 38, 39, 41, 42, 44, 45, 46, 47, 48, 22, 29, 11, 12, 23, 24, 25, 26, 27, 28, 30, 31, 32, 33, 34, 35, 37, 38, 39, 44, 45, 46, 47, 48, 22, 29, 41, 42, 42, 1, 5, 50, 52, 49, 50, 52, 49, 51, 0, 0 ];
static _use_syllable_machine_single_lengths: [i8 ; 64] = [ 1, 1, 34, 22, 20, 1, 16, 11, 6, 5, 4, 2, 1, 10, 9, 8, 1, 7, 15, 14, 13, 18, 17, 17, 21, 12, 1, 1, 24, 2, 1, 20, 16, 11, 6, 5, 4, 2, 1, 10, 9, 8, 7, 15, 14, 13, 18, 17, 17, 21, 12, 1, 1, 1, 27, 22, 2, 1, 2, 2, 3, 2, 0 , 0 ];
static _use_syllable_machine_range_lengths: [i8 ; 64] = [ 0, 0, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0 , 0 ];
static _use_syllable_machine_index_offsets: [i16 ; 64] = [ 0, 2, 4, 40, 64, 87, 89, 106, 118, 125, 131, 136, 139, 141, 152, 162, 171, 173, 181, 197, 212, 226, 245, 263, 282, 305, 318, 320, 322, 348, 351, 353, 376, 393, 405, 412, 418, 423, 426, 428, 439, 449, 458, 466, 482, 497, 511, 530, 548, 567, 590, 603, 605, 607, 609, 638, 662, 665, 667, 670, 673, 677, 0 , 0 ];
static _use_syllable_machine_cond_targs: [i8 ; 744] = [ 31, 2, 42, 2, 2, 3, 26, 28, 31, 51, 52, 54, 29, 33, 34, 35, 36, 46, 47, 48, 55, 49, 43, 44, 45, 39, 40, 41, 56, 57, 58, 50, 37, 38, 2, 51, 59, 61, 32, 2, 4, 5, 7, 8, 9, 10, 21, 22, 23, 3, 24, 18, 19, 20, 13, 14, 15, 25, 11, 12, 2, 5, 6, 2, 4, 5, 7, 8, 9, 10, 21, 22, 23, 18, 19, 20, 13, 14, 15, 25, 11, 12, 2, 5, 6, 24, 2, 4, 2, 6, 7, 8, 9, 10, 18, 19, 20, 13, 14, 15, 7, 11, 12, 2, 16, 2, 7, 8, 9, 10, 13, 14, 15, 11, 12, 2, 16, 2, 8, 9, 10, 11, 12, 2, 2, 9, 10, 11, 12, 2, 2, 10, 11, 12, 2, 2, 11, 12, 2, 12, 2, 8, 9, 10, 13, 14, 15, 11, 12, 2, 16, 2, 8, 9, 10, 14, 15, 11, 12, 2, 16, 2, 8, 9, 10, 15, 11, 12, 2, 16, 2, 17, 2, 8, 9, 10, 11, 12, 2, 16, 2, 7, 8, 9, 10, 18, 19, 20, 13, 14, 15, 7, 11, 12, 2, 16, 2, 7, 8, 9, 10, 19, 20, 13, 14, 15, 7, 11, 12, 2, 16, 2, 7, 8, 9, 10, 20, 13, 14, 15, 7, 11, 12, 2, 16, 2, 6, 7, 8, 9, 10, 22, 6, 18, 19, 20, 13, 14, 15, 7, 11, 12, 2, 16, 2, 6, 7, 8, 9, 10, 6, 18, 19, 20, 13, 14, 15, 7, 11, 12, 2, 16, 2, 7, 8, 9, 10, 21, 22, 18, 19, 20, 13, 14, 15, 7, 11, 12, 2, 16, 6, 2, 4, 5, 7, 8, 9, 10, 21, 22, 23, 24, 18, 19, 20, 13, 14, 15, 25, 11, 12, 2, 5, 6, 2, 4, 7, 8, 9, 10, 13, 14, 15, 11, 12, 2, 16, 2, 27, 2, 26, 2, 4, 5, 7, 8, 9, 10, 21, 22, 23, 3, 24, 18, 19, 20, 13, 14, 15, 29, 30, 25, 11, 12, 2, 5, 6, 2, 29, 30, 2, 30, 2, 31, 0, 33, 34, 35, 36, 46, 47, 48, 43, 44, 45, 39, 40, 41, 50, 37, 38, 2, 0, 32, 49, 2, 32, 33, 34, 35, 36, 43, 44, 45, 39, 40, 41, 33, 37, 38, 2, 1, 2, 33, 34, 35, 36, 39, 40, 41, 37, 38, 2, 1, 2, 34, 35, 36, 37, 38, 2, 2, 35, 36, 37, 38, 2, 2, 36, 37, 38, 2, 2, 37, 38, 2, 38, 2, 34, 35, 36, 39, 40, 41, 37, 38, 2, 1, 2, 34, 35, 36, 40, 41, 37, 38, 2, 1, 2, 34, 35, 36, 41, 37, 38, 2, 1, 2, 34, 35, 36, 37, 38, 2, 1, 2, 33, 34, 35, 36, 43, 44, 45, 39, 40, 41, 33, 37, 38, 2, 1, 2, 33, 34, 35, 36, 44, 45, 39, 40, 41, 33, 37, 38, 2, 1, 2, 33, 34, 35, 36, 45, 39, 40, 41, 33, 37, 38, 2, 1, 2, 32, 33, 34, 35, 36, 47, 32, 43, 44, 45, 39, 40, 41, 33, 37, 38, 2, 1, 2, 32, 33, 34, 35, 36, 32, 43, 44, 45, 39, 40, 41, 33, 37, 38, 2, 1, 2, 33, 34, 35, 36, 46, 47, 43, 44, 45, 39, 40, 41, 33, 37, 38, 2, 1, 32, 2, 31, 0, 33, 34, 35, 36, 46, 47, 48, 49, 43, 44, 45, 39, 40, 41, 50, 37, 38, 2, 0, 32, 2, 31, 33, 34, 35, 36, 39, 40, 41, 37, 38, 2, 1, 2, 31, 2, 53, 2, 52, 2, 3, 3, 31, 0, 52, 33, 34, 35, 36, 46, 47, 48, 55, 49, 43, 44, 45, 39, 40, 41, 56, 57, 50, 37, 38, 2, 0, 32, 2, 31, 0, 33, 34, 35, 36, 46, 47, 48, 55, 49, 43, 44, 45, 39, 40, 41, 50, 37, 38, 2, 0, 32, 2, 56, 57, 2, 57, 2, 3, 3, 2, 60, 59, 2, 59, 60, 60, 2, 59, 61, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0 , 0 ];
static _use_syllable_machine_cond_actions: [i8 ; 744] = [ 5, 33, 5, 33, 7, 0, 0, 0, 5, 0, 0, 5, 0, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 0, 5, 0, 0, 11, 0, 0, 0, 5, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 19, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 9, 19, 0, 0, 0, 0, 9, 19, 0, 0, 0, 9, 19, 0, 0, 19, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 17, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 15, 0, 23, 0, 21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 19, 0, 0, 25, 0, 25, 5, 0, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 11, 0, 5, 5, 29, 5, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 0, 0, 11, 0, 29, 5, 0, 0, 0, 5, 5, 5, 0, 0, 11, 0, 29, 0, 0, 0, 0, 0, 11, 29, 0, 0, 0, 0, 11, 29, 0, 0, 0, 11, 29, 0, 0, 29, 0, 29, 0, 0, 0, 5, 5, 5, 0, 0, 11, 0, 29, 0, 0, 0, 5, 5, 0, 0, 11, 0, 29, 0, 0, 0, 5, 0, 0, 11, 0, 29, 0, 0, 0, 0, 0, 11, 0, 29, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 0, 0, 11, 0, 29, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 0, 0, 11, 0, 29, 5, 0, 0, 0, 5, 5, 5, 5, 5, 0, 0, 11, 0, 29, 5, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 11, 0, 29, 5, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 11, 0, 29, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 11, 0, 5, 29, 5, 0, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 11, 0, 5, 29, 5, 5, 0, 0, 0, 5, 5, 5, 0, 0, 11, 0, 29, 5, 31, 0, 29, 0, 29, 0, 0, 5, 0, 0, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 5, 0, 0, 11, 0, 5, 29, 5, 0, 5, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 11, 0, 5, 29, 0, 0, 29, 0, 29, 0, 0, 31, 0, 0, 27, 0, 0, 0, 27, 0, 0, 27, 33, 33, 0, 19, 19, 15, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 17, 19, 19, 19, 19, 19, 19, 19, 19, 15, 23, 21, 19, 25, 25, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 31, 29, 29, 29, 29, 29, 29, 31, 27, 27, 27, 0 , 0 ];
static _use_syllable_machine_to_state_actions: [i8 ; 64] = [ 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0 ];
static _use_syllable_machine_from_state_actions: [i8 ; 64] = [ 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0 ];
static _use_syllable_machine_eof_trans: [i16 ; 64] = [ 681, 682, 683, 684, 685, 686, 687, 688, 689, 690, 691, 692, 693, 694, 695, 696, 697, 698, 699, 700, 701, 702, 703, 704, 705, 706, 707, 708, 709, 710, 711, 712, 713, 714, 715, 716, 717, 718, 719, 720, 721, 722, 723, 724, 725, 726, 727, 728, 729, 730, 731, 732, 733, 734, 735, 736, 737, 738, 739, 740, 741, 742, 0 , 0 ];
static use_syllable_machine_start : i32 = 2;
static use_syllable_machine_first_final : i32 = 2;
static use_syllable_machine_error : i32 = -1;
static use_syllable_machine_en_main : i32 = 2;
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

pub fn find_syllables(buffer: &mut Buffer) {
	let mut cs = 0;
	let mut ts = 0;
	let mut te = 0;
	let mut p = 0;
	let mut ixs: std::vec::Vec<_> = (0..buffer.info.len()).filter(|i| included(*i, &buffer.info)).collect();
	let pe = ixs.len();
	let eof = ixs.len();
	ixs.push(buffer.info.len());
	let mut syllable_serial = 1u8;
	
	macro_rules! found_syllable {
		($kind:expr) => {{
				found_syllable(ixs[ts], ixs[te], &mut syllable_serial, $kind, buffer);
			}}
	}
	
	
	{
		cs = ( use_syllable_machine_start ) as i32;
		ts = 0;
		te = 0;
	}
	
	{
		let mut _klen = 0;
		let mut _trans  = 0;
		let mut _keys :i32= 0;
		let mut _acts :i32= 0;
		let mut _nacts = 0;
		let mut __have = 0;
		'_resume: while ( p != pe || p == eof  ) {
			'_again: while ( true  ) {
				_acts = ( _use_syllable_machine_from_state_actions[(cs) as usize] ) as i32;
				_nacts = ( _use_syllable_machine_actions[(_acts ) as usize]
				) as u32;
				_acts += 1;
				while ( _nacts > 0  ) {
					match ( _use_syllable_machine_actions[(_acts ) as usize]
					) {
						1  => {
							{{ts = p;
								}}
							
						}
						
						_ => {}
					}
					_nacts -= 1;
					_acts += 1;
					
				}
				if ( p == eof  ) {
					{
						if ( _use_syllable_machine_eof_trans[(cs) as usize]> 0  ) {
							{
								_trans = ( _use_syllable_machine_eof_trans[(cs) as usize] ) as u32- 1;
							}
							
						}
					}
					
				}
				else {
					{
						_keys = ( _use_syllable_machine_key_offsets[(cs) as usize] ) as i32;
						_trans = ( _use_syllable_machine_index_offsets[(cs) as usize] ) as u32;
						_klen = ( _use_syllable_machine_single_lengths[(cs) as usize] ) as i32;
						__have = 0;
						if ( _klen > 0  ) {
							{
								let mut _lower  :i32= _keys;
								let mut _upper  :i32= _keys + _klen - 1;
								let mut _mid :i32= 0;
								while ( true  ) {
									if ( _upper < _lower  ) {
										{
											_keys += _klen;
											_trans += ( _klen ) as u32;
											break;
										}
										
										
									}
									_mid = _lower + ((_upper-_lower) >> 1);
									if ( ((buffer.info[ixs[p]].use_category() as u8)) < _use_syllable_machine_trans_keys[(_mid ) as usize]
									) {
										_upper = _mid - 1;
										
									}
									else if ( ((buffer.info[ixs[p]].use_category() as u8)) > _use_syllable_machine_trans_keys[(_mid ) as usize]
									) {
										_lower = _mid + 1;
										
									}
									else {
										{
											__have = 1;
											_trans += ( (_mid - _keys) ) as u32;
											break;
										}
										
									}
									
								}
							}
							
							
						}
						_klen = ( _use_syllable_machine_range_lengths[(cs) as usize] ) as i32;
						if ( __have == 0 && _klen > 0  ) {
							{
								let mut _lower  :i32= _keys;
								let mut _upper  :i32= _keys + (_klen<<1) - 2;
								let mut _mid :i32= 0;
								while ( true  ) {
									if ( _upper < _lower  ) {
										{
											_trans += ( _klen ) as u32;
											break;
										}
										
										
									}
									_mid = _lower + (((_upper-_lower) >> 1) & !1
									);
									if ( ((buffer.info[ixs[p]].use_category() as u8)) < _use_syllable_machine_trans_keys[(_mid ) as usize]
									) {
										_upper = _mid - 2;
										
									}
									else if ( ((buffer.info[ixs[p]].use_category() as u8)) > _use_syllable_machine_trans_keys[(_mid + 1 ) as usize]
									) {
										_lower = _mid + 2;
										
									}
									else {
										{
											_trans += ( ((_mid - _keys)>>1) ) as u32;
											break;
										}
										
									}
									
								}
							}
							
							
						}
					}
					
				}
				cs = ( _use_syllable_machine_cond_targs[(_trans) as usize] ) as i32;
				if ( _use_syllable_machine_cond_actions[(_trans) as usize]!= 0  ) {
					{
					
						_acts = ( _use_syllable_machine_cond_actions[(_trans) as usize] ) as i32;
						_nacts = ( _use_syllable_machine_actions[(_acts ) as usize]
						) as u32;
						_acts += 1;
						while ( _nacts > 0  ) {
							match ( _use_syllable_machine_actions[(_acts ) as usize]
							) {
								2  => {
									{{te = p+1;
										}}
									
								}
								3  => {
									{{te = p+1;
											{found_syllable!(SyllableType::IndependentCluster); }
										}}
									
								}
								4  => {
									{{te = p+1;
											{found_syllable!(SyllableType::StandardCluster); }
										}}
									
								}
								5  => {
									{{te = p+1;
											{found_syllable!(SyllableType::BrokenCluster); }
										}}
									
								}
								6  => {
									{{te = p+1;
											{found_syllable!(SyllableType::NonCluster); }
										}}
									
								}
								7  => {
									{{te = p;
											p = p - 1;
											{found_syllable!(SyllableType::ViramaTerminatedCluster); }
										}}
									
								}
								8  => {
									{{te = p;
											p = p - 1;
											{found_syllable!(SyllableType::SakotTerminatedCluster); }
										}}
									
								}
								9  => {
									{{te = p;
											p = p - 1;
											{found_syllable!(SyllableType::StandardCluster); }
										}}
									
								}
								10  => {
									{{te = p;
											p = p - 1;
											{found_syllable!(SyllableType::NumberJoinerTerminatedCluster); }
										}}
									
								}
								11  => {
									{{te = p;
											p = p - 1;
											{found_syllable!(SyllableType::NumeralCluster); }
										}}
									
								}
								12  => {
									{{te = p;
											p = p - 1;
											{found_syllable!(SyllableType::SymbolCluster); }
										}}
									
								}
								13  => {
									{{te = p;
											p = p - 1;
											{found_syllable! (SyllableType::HieroglyphCluster); }
										}}
									
								}
								14  => {
									{{te = p;
											p = p - 1;
											{found_syllable!(SyllableType::BrokenCluster); }
										}}
									
								}
								15  => {
									{{te = p;
											p = p - 1;
											{found_syllable!(SyllableType::NonCluster); }
										}}
									
								}
								16  => {
									{{p = ((te))-1;
											{found_syllable!(SyllableType::BrokenCluster); }
										}}
									
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
			if ( p == eof  ) {
				{
					if ( cs >= 2  ) {
						break '_resume;
						
					}
				}
				
			}
			else {
				{
					_acts = ( _use_syllable_machine_to_state_actions[(cs) as usize] ) as i32;
					_nacts = ( _use_syllable_machine_actions[(_acts ) as usize]
					) as u32;
					_acts += 1;
					while ( _nacts > 0  ) {
						match ( _use_syllable_machine_actions[(_acts ) as usize]
						) {
							0  => {
								{{ts = 0;
									}}
								
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

fn not_standard_default_ignorable(i: &GlyphInfo) -> bool {
	!(matches!(i.use_category(), category::O | category::RSV) && i.is_default_ignorable())
}

fn included(i: usize, infos: &[GlyphInfo]) -> bool {
	let glyph = &infos[i];
	if !not_standard_default_ignorable(glyph) {
		return false;
	}
	if glyph.use_category() == category::ZWNJ {
		for glyph2 in &infos[i + 1..] {
			if not_standard_default_ignorable(glyph2) {
				return !glyph2.is_unicode_mark();
			}
		}
	}
	true
}
