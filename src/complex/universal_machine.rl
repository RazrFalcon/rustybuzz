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
  machine use_syllable_machine;
  alphtype u8;
  write data;
}%%

%%{

O	= 0; # OTHER

B	= 1; # BASE
IND	= 3; # BASE_IND
N	= 4; # BASE_NUM
GB	= 5; # BASE_OTHER
CGJ	= 6; # CGJ
#F	= 7; # CONS_FINAL
#FM	= 8; # CONS_FINAL_MOD
#M	= 9; # CONS_MED
#CM	= 10; # CONS_MOD
SUB	= 11; # CONS_SUB
H	= 12; # HALANT

HN	= 13; # HALANT_NUM
ZWNJ	= 14; # Zero width non-joiner
ZWJ	= 15; # Zero width joiner
WJ	= 16; # Word joiner
Rsv	= 17; # Reserved characters
R	= 18; # REPHA
S	= 19; # SYM
#SM	= 20; # SYM_MOD
VS	= 21; # VARIATION_SELECTOR
#V	= 36; # VOWEL
#VM	= 40; # VOWEL_MOD
CS	= 43; # CONS_WITH_STACKER
HVM	= 44; # HALANT_OR_VOWEL_MODIFIER
Sk	= 48; # SAKOT

FAbv	= 24; # CONS_FINAL_ABOVE
FBlw	= 25; # CONS_FINAL_BELOW
FPst	= 26; # CONS_FINAL_POST
MAbv	= 27; # CONS_MED_ABOVE
MBlw	= 28; # CONS_MED_BELOW
MPst	= 29; # CONS_MED_POST
MPre	= 30; # CONS_MED_PRE
CMAbv	= 31; # CONS_MOD_ABOVE
CMBlw	= 32; # CONS_MOD_BELOW
VAbv	= 33; # VOWEL_ABOVE / VOWEL_ABOVE_BELOW / VOWEL_ABOVE_BELOW_POST / VOWEL_ABOVE_POST
VBlw	= 34; # VOWEL_BELOW / VOWEL_BELOW_POST
VPst	= 35; # VOWEL_POST	UIPC = Right
VPre	= 22; # VOWEL_PRE / VOWEL_PRE_ABOVE / VOWEL_PRE_ABOVE_POST / VOWEL_PRE_POST
VMAbv	= 37; # VOWEL_MOD_ABOVE
VMBlw	= 38; # VOWEL_MOD_BELOW
VMPst	= 39; # VOWEL_MOD_POST
VMPre	= 23; # VOWEL_MOD_PRE
SMAbv	= 41; # SYM_MOD_ABOVE
SMBlw	= 42; # SYM_MOD_BELOW
FMAbv	= 45; # CONS_FINAL_MOD	UIPC = Top
FMBlw	= 46; # CONS_FINAL_MOD	UIPC = Bottom
FMPst	= 47; # CONS_FINAL_MOD	UIPC = Not_Applicable

h = H | HVM | Sk;

# Override: Adhoc ZWJ placement. https://github.com/harfbuzz/harfbuzz/issues/542#issuecomment-353169729
consonant_modifiers = CMAbv* CMBlw* ((ZWJ?.h.ZWJ? B | SUB) VS? CMAbv? CMBlw*)*;
# Override: Allow two MBlw. https://github.com/harfbuzz/harfbuzz/issues/376
medial_consonants = MPre? MAbv? MBlw?.MBlw? MPst?;
dependent_vowels = VPre* VAbv* VBlw* VPst*;
vowel_modifiers = HVM? VMPre* VMAbv* VMBlw* VMPst*;
final_consonants = FAbv* FBlw* FPst*;
final_modifiers = FMAbv* FMBlw* | FMPst?;

complex_syllable_start = (R | CS)? (B | GB) VS?;
complex_syllable_middle =
	consonant_modifiers
	medial_consonants
	dependent_vowels
	vowel_modifiers
	(Sk B)*
;
complex_syllable_tail =
	complex_syllable_middle
	final_consonants
	final_modifiers
;
number_joiner_terminated_cluster_tail = (HN N VS?)* HN;
numeral_cluster_tail = (HN N VS?)+;
symbol_cluster_tail = SMAbv+ SMBlw* | SMBlw+;

virama_terminated_cluster =
	complex_syllable_start
	consonant_modifiers
	ZWJ?.h.ZWJ?
;
sakot_terminated_cluster =
	complex_syllable_start
	complex_syllable_middle
	Sk
;
standard_cluster =
	complex_syllable_start
	complex_syllable_tail
;
broken_cluster =
	R?
	(complex_syllable_tail | number_joiner_terminated_cluster_tail | numeral_cluster_tail | symbol_cluster_tail)
;

number_joiner_terminated_cluster = N VS? number_joiner_terminated_cluster_tail;
numeral_cluster = N VS? numeral_cluster_tail?;
symbol_cluster = (S | GB) VS? symbol_cluster_tail?;
independent_cluster = (IND | O | Rsv | WJ) VS?;
other = any;

main := |*
	independent_cluster			=> { found_syllable!(SyllableType::IndependentCluster); };
	virama_terminated_cluster		=> { found_syllable!(SyllableType::ViramaTerminatedCluster); };
	sakot_terminated_cluster		=> { found_syllable!(SyllableType::SakotTerminatedCluster); };
	standard_cluster			=> { found_syllable!(SyllableType::StandardCluster); };
	number_joiner_terminated_cluster	=> { found_syllable!(SyllableType::NumberJoinerTerminatedCluster); };
	numeral_cluster				=> { found_syllable!(SyllableType::NumeralCluster); };
	symbol_cluster				=> { found_syllable!(SyllableType::SymbolCluster); };
	broken_cluster				=> { found_syllable!(SyllableType::BrokenCluster); };
	other					=> { found_syllable!(SyllableType::NonCluster); };
*|;


}%%

#[derive(Clone, Copy)]
pub enum SyllableType {
    IndependentCluster,
    ViramaTerminatedCluster,
    SakotTerminatedCluster,
    StandardCluster,
    NumberJoinerTerminatedCluster,
    NumeralCluster,
    SymbolCluster,
    BrokenCluster,
    NonCluster,
}

pub fn find_syllables(buffer: &mut Buffer) {
    let mut cs = 0;
    let mut ts = 0;
    let mut te = 0;
    let mut p = 0;
    let pe = buffer.len;
    let eof = buffer.len;
    let mut syllable_serial = 1u8;
    let mut act;

    macro_rules! found_syllable {
        ($kind:expr) => {{
            found_syllable(ts, te, &mut syllable_serial, $kind, buffer);
        }}
    }

    %%{
        write init;
        getkey (buffer.info[p].use_category() as u8);
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
