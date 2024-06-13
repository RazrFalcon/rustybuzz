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

// NOTE: We need to keep this in sync with the defined keys below. In harfbuzz, they are deduplicated
// but for some reason rust ragel doesn't properly export writing exports in Rust.
pub mod myanmar_category_t {
    use crate::hb::ot_shaper_indic::indic_category_t::{OT_N, OT_PLACEHOLDER};

    pub const As: u8 = 18; /* Asat */
    #[allow(dead_code)]
    pub const D0: u8 = 20; /* Digit zero */
    #[allow(dead_code)]
    pub const DB: u8 = OT_N; /* Dot below */
    pub const GB: u8 = OT_PLACEHOLDER;
    pub const MH: u8 = 21; /* Various consonant medial types */
    pub const MR: u8 = 22; /* Various consonant medial types */
    pub const MW: u8 = 23; /* Various consonant medial types */
    pub const MY: u8 = 24; /* Various consonant medial types */
    pub const PT: u8 = 25; /* Pwo and other tones */
    //pub const VAbv: u8 = 26;
    //pub const VBlw: u8 = 27;
    //pub const VPre: u8 = 28;
    //pub const VPst: u8 = 29;
    pub const VS: u8 = 30; /* Variation selectors */
    pub const P: u8 = 31; /* Punctuation */
    pub const D: u8 = GB; /* Digits except zero */
    pub const ML: u8 = 32; /* Various consonant medial types */
}

%%{
  machine myanmar_syllable_machine;
  alphtype u8;
  write data;
}%%

%%{

C    = 1;
IV   = 2;
DB   = 3;	# Dot below	= OT_N
H    = 4;
ZWNJ = 5;
ZWJ  = 6;
V    = 8;	# Visarga and Shan tones
A    = 9;
D    = 10;	# Digits except zero = GB
GB   = 10;	# 		= OT_PLACEHOLDER
Ra   = 15;
As   = 18;	# Asat
CS   = 19;
D0   = 20;	# Digit zero
MH   = 21;	# Medial
MR   = 22;	# Medial
MW   = 23;	# Medial
MY   = 24;	# Medial
PT   = 25;	# Pwo and other tones
VAbv = 26;
VBlw = 27;
VPre = 28;
VPst = 29;
VS   = 30;	# Variation selectors
P    = 31;	# Punctuation
ML   = 32;	# Consonant medials

j = ZWJ|ZWNJ;			# Joiners
k = (Ra As H);			# Kinzi

c = C|Ra;			# is_consonant

medial_group = MY? As? MR? ((MW MH? ML? | MH ML? | ML) As?)?;
main_vowel_group = (VPre.VS?)* VAbv* VBlw* A* (DB As?)?;
post_vowel_group = VPst MH? ML? As* VAbv* A* (DB As?)?;
pwo_tone_group = PT A* DB? As?;

complex_syllable_tail = As* medial_group main_vowel_group post_vowel_group* pwo_tone_group* V* j?;
syllable_tail = (H (c|IV).VS?)* (H | complex_syllable_tail);

consonant_syllable =	(k|CS)? (c|IV|D|GB).VS? syllable_tail;
punctuation_cluster =	P V;
broken_cluster =	k? VS? syllable_tail;
other =			any;

main := |*
	consonant_syllable	=> { found_syllable!(SyllableType::ConsonantSyllable); };
	j			=> { found_syllable!(SyllableType::NonMyanmarCluster); };
	punctuation_cluster	=> { found_syllable!(SyllableType::PunctuationCluster); };
	broken_cluster		=> { found_syllable!(SyllableType::BrokenCluster); };
	other			=> { found_syllable!(SyllableType::NonMyanmarCluster); };
*|;


}%%

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
        }}
    }

    %%{
        write init;
        getkey (buffer.info[p].myanmar_category() as u8);
        write exec;
    }%%
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
