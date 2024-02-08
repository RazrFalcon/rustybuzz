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

%%{
  machine myanmar_syllable_machine;
  alphtype u8;
  write data;
}%%

%%{

A    = 10;
As   = 18;
C    = 1;
D    = 32;
D0   = 20;
DB   = 3;
GB   = 11;
H    = 4;
IV   = 2;
MH   = 21;
ML   = 33;
MR   = 22;
MW   = 23;
MY   = 24;
PT   = 25;
V    = 8;
VAbv = 26;
VBlw = 27;
VPre = 28;
VPst = 29;
VS   = 30;
ZWJ  = 6;
ZWNJ = 5;
Ra   = 16;
P    = 31;
CS   = 19;

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
        }}
    }

    %%{
        write init;
        getkey (buffer.info[p].indic_category() as u8);
        write exec;
    }%%
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
