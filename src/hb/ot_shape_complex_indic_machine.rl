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

use crate::buffer::Buffer;

%%{
  machine indic_syllable_machine;
  alphtype u8;
  write data;
}%%

%%{

C    = 1;
V    = 2;
N    = 3;
H    = 4;
ZWNJ = 5;
ZWJ  = 6;
M    = 7;
SM   = 8;
A    = 10;
PLACEHOLDER = 11;
DOTTEDCIRCLE = 12;
RS    = 13;
Repha = 15;
Ra    = 16;
CM    = 17;
Symbol= 18;
CS    = 19;

c = (C | Ra);			# is_consonant
n = ((ZWNJ?.RS)? (N.N?)?);	# is_consonant_modifier
z = ZWJ|ZWNJ;			# is_joiner
reph = (Ra H | Repha);		# possible reph

cn = c.ZWJ?.n?;
forced_rakar = ZWJ H ZWJ Ra;
symbol = Symbol.N?;
matra_group = z*.M.N?.(H | forced_rakar)?;
syllable_tail = (z?.SM.SM?.ZWNJ?)? A*;
halant_group = (z?.H.(ZWJ.N?)?);
final_halant_group = halant_group | H.ZWNJ;
medial_group = CM?;
halant_or_matra_group = (final_halant_group | matra_group*);

complex_syllable_tail = (halant_group.cn)* medial_group halant_or_matra_group syllable_tail;

consonant_syllable =	(Repha|CS)? cn complex_syllable_tail;
vowel_syllable =	reph? V.n? (ZWJ | complex_syllable_tail);
standalone_cluster =	((Repha|CS)? PLACEHOLDER | reph? DOTTEDCIRCLE).n? complex_syllable_tail;
symbol_cluster =	symbol syllable_tail;
broken_cluster =	reph? n? complex_syllable_tail;
other =			any;

main := |*
	consonant_syllable	=> { found_syllable!(SyllableType::ConsonantSyllable); };
	vowel_syllable		=> { found_syllable!(SyllableType::VowelSyllable); };
	standalone_cluster	=> { found_syllable!(SyllableType::StandaloneCluster); };
	symbol_cluster		=> { found_syllable!(SyllableType::SymbolCluster); };
	broken_cluster		=> { found_syllable!(SyllableType::BrokenCluster); /*buffer->scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;*/ };
	other			=> { found_syllable!(SyllableType::NonIndicCluster); };
*|;


}%%

#[derive(Clone, Copy)]
pub enum SyllableType {
    ConsonantSyllable = 0,
    VowelSyllable,
    StandaloneCluster,
    SymbolCluster,
    BrokenCluster,
    NonIndicCluster,
}

pub fn find_syllables_indic(buffer: &mut Buffer) {
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
