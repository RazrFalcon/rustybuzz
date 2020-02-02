use super::arabic::JoiningType::{self, U, L, R, D, GroupAlaph as A, GroupDalathRish as DR, T, X};

pub const JOINING_TABLE: &[JoiningType] = &[
    /* Arabic */

    /* 0600 */ U,U,U,U,U,U,X,X,U,X,X,U,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 0620 */ D,U,R,R,R,R,D,R,D,R,D,D,D,D,D,R,R,R,R,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 0640 */ D,D,D,D,D,D,D,D,R,D,D,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 0660 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,D,D,X,R,R,R,U,R,R,R,D,D,D,D,D,D,D,D,
    /* 0680 */ D,D,D,D,D,D,D,D,R,R,R,R,R,R,R,R,R,R,R,R,R,R,R,R,R,R,D,D,D,D,D,D,
    /* 06A0 */ D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 06C0 */ R,D,D,R,R,R,R,R,R,R,R,R,D,R,D,R,D,D,R,R,X,R,X,X,X,X,X,X,X,U,X,X,
    /* 06E0 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,R,R,X,X,X,X,X,X,X,X,X,X,D,D,D,X,X,D,

    /* Syriac */

    /* 0700 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,T,A,X,D,D,D,DR,DR,R,R,R,D,D,D,D,R,D,
    /* 0720 */ D,D,D,D,D,D,D,D,R,D,DR,D,R,D,D,DR,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 0740 */ X,X,X,X,X,X,X,X,X,X,X,X,X,R,D,D,

    /* Arabic Supplement */

    /* 0740 */                                 D,D,D,D,D,D,D,D,D,R,R,R,D,D,D,D,
    /* 0760 */ D,D,D,D,D,D,D,D,D,D,D,R,R,D,D,D,D,R,D,R,R,D,D,D,R,R,D,D,D,D,D,D,

    /* FILLER */

    /* 0780 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 07A0 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,

    /* NKo */

    /* 07C0 */ X,X,X,X,X,X,X,X,X,X,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 07E0 */ D,D,D,D,D,D,D,D,D,D,D,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,D,X,X,X,X,X,

    /* FILLER */

    /* 0800 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 0820 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,

    /* Mandaic */

    /* 0840 */ R,D,D,D,D,D,R,R,D,R,D,D,D,D,D,D,D,D,D,D,R,D,U,U,U,X,X,X,X,X,X,X,

    /* Syriac Supplement */

    /* 0860 */ D,U,D,D,D,D,U,R,D,R,R,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 0880 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,

    /* Arabic Extended-A */

    /* 08A0 */ D,D,D,D,D,D,D,D,D,D,R,R,R,U,R,D,D,R,R,D,D,X,D,D,D,R,D,D,D,D,X,X,
    /* 08C0 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 08E0 */ X,X,U,

    /* Mongolian */

    /* 1800 */             U,D,X,X,D,X,X,X,U,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 1820 */ D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 1840 */ D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 1860 */ D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,X,X,X,X,X,X,X,
    /* 1880 */ U,U,U,U,U,T,T,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 18A0 */ D,D,D,D,D,D,D,D,D,X,D,

    /* General Punctuation */

    /* 2000 */                         U,D,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 2020 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,U,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 2040 */ X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 2060 */ X,X,X,X,X,X,U,U,U,U,

    /* Phags-pa */

    /* A840 */ D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* A860 */ D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,L,U,

    /* Manichaean */

    /* 10AC0 */ D,D,D,D,D,R,U,R,U,R,R,U,U,L,R,R,R,R,R,D,D,D,D,L,D,D,D,D,D,R,D,D,
    /* 10AE0 */ D,R,U,U,R,X,X,X,X,X,X,D,D,D,D,R,

    /* Psalter Pahlavi */

    /* 10B80 */ D,R,D,R,R,R,D,D,D,R,D,D,R,D,R,R,D,R,X,X,X,X,X,X,X,X,X,X,X,X,X,X,
    /* 10BA0 */ X,X,X,X,X,X,X,X,X,R,R,R,R,D,D,U,

    /* Hanifi Rohingya */

    /* 10D00 */ L,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 10D20 */ D,D,R,D,

    /* Sogdian */

    /* 10F20 */                                 D,D,D,R,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 10F40 */ D,D,D,D,D,U,X,X,X,X,X,X,X,X,X,X,X,D,D,D,R,

    /* Kaithi */

    /* 110A0 */                                                           U,X,X,
    /* 110C0 */ X,X,X,X,X,X,X,X,X,X,X,X,X,U,

    /* Adlam */

    /* 1E900 */ D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 1E920 */ D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    /* 1E940 */ D,D,D,D,X,X,X,X,X,X,X,T,
];

const JOINING_OFFSET_0X0600: usize = 0;
const JOINING_OFFSET_0X1806: usize = 739;
const JOINING_OFFSET_0X200C: usize = 904;
const JOINING_OFFSET_0XA840: usize = 998;
const JOINING_OFFSET_0X10AC0: usize = 1050;
const JOINING_OFFSET_0X10B80: usize = 1098;
const JOINING_OFFSET_0X10D00: usize = 1146;
const JOINING_OFFSET_0X10F30: usize = 1182;
const JOINING_OFFSET_0X110BD: usize = 1219;
const JOINING_OFFSET_0X1E900: usize = 1236;

pub fn joining_type(u: char) -> JoiningType {
    let u = u as u32;
    match u >> 12 {
        0x0 => {
            if (0x0600..=0x08E2).contains(&u) {
                return JOINING_TABLE[u as usize - 0x0600 + JOINING_OFFSET_0X0600];
            }
        }
        0x1 => {
            if (0x1806..=0x18AA).contains(&u) {
                return JOINING_TABLE[u as usize - 0x1806 + JOINING_OFFSET_0X1806];
            }
        }
        0x2 => {
            if (0x200C..=0x2069).contains(&u) {
                return JOINING_TABLE[u as usize - 0x200C + JOINING_OFFSET_0X200C];
            }
        }
        0xA => {
            if (0xA840..=0xA873).contains(&u) {
                return JOINING_TABLE[u as usize - 0xA840 + JOINING_OFFSET_0XA840];
            }
        }
        0x10 => {
            if (0x10AC0..=0x10AEF).contains(&u) {
                return JOINING_TABLE[u as usize - 0x10AC0 + JOINING_OFFSET_0X10AC0];
            }
            if (0x10B80..=0x10BAF).contains(&u) {
                return JOINING_TABLE[u as usize - 0x10B80 + JOINING_OFFSET_0X10B80];
            }
            if (0x10D00..=0x10D23).contains(&u) {
                return JOINING_TABLE[u as usize - 0x10D00 + JOINING_OFFSET_0X10D00];
            }
            if (0x10F30..=0x10F54).contains(&u) {
                return JOINING_TABLE[u as usize - 0x10F30 + JOINING_OFFSET_0X10F30];
            }
        }
        0x11 => {
            if (0x110BD..=0x110CD).contains(&u) {
                return JOINING_TABLE[u as usize - 0x110BD + JOINING_OFFSET_0X110BD];
            }
        }
        0x1E => {
            if (0x1E900..=0x1E94B).contains(&u) {
                return JOINING_TABLE[u as usize - 0x1E900 + JOINING_OFFSET_0X1E900];
            }
        }
        _ => {}
    }

    X
}
