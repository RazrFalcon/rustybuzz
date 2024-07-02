use ttf_parser::GlyphId;

// To make things easier, we don't have the generic parameter mask_t,
// and assume we always use u32, since this is what is also used in
// harfbuzz.
pub type mask_t = u32;

struct hb_set_digest_bits_pattern_t<const shift: u8> {
    mask: mask_t
}

impl<const shift: u8> hb_set_digest_bits_pattern_t<shift> {
    pub fn new() -> Self {
        Self {
            mask: 0
        }
    }

    pub fn full() -> Self {
        Self {
            mask: mask_t::MAX
        }
    }

    pub fn union(&mut self, o: &hb_set_digest_bits_pattern_t<shift>) {
        self.mask |= o.mask;
    }

    pub fn add(&mut self, g: GlyphId) {
        self.mask |= hb_set_digest_bits_pattern_t::<shift>::mask_for(g);
    }

    pub fn add_range(&mut self, a: GlyphId, b: GlyphId) -> bool {
        if self.mask == mask_t::MAX {
            return false;
        }

        if (b.0 as u32 >> shift) - (a.0 as u32 >> shift) >= hb_set_digest_bits_pattern_t::<shift>::mask_bits() - 1 {
            self.mask = mask_t::MAX;
            false
        }   else {
            let ma = hb_set_digest_bits_pattern_t::<shift>::mask_for(a);
            let mb = hb_set_digest_bits_pattern_t::<shift>::mask_for(b);
            self.mask |= mb + (mb - ma) - u32::from(mb < ma);
            true
        }
    }

    const fn mask_bytes() -> u32 {
        core::mem::size_of::<mask_t>() as u32
    }

    const fn mask_bits() -> u32 {
        (core::mem::size_of::<mask_t>() * 8) as u32
    }

    fn mask_for(g: GlyphId) -> mask_t {
        1 << ((g.0 as u32 >> shift) & (hb_set_digest_bits_pattern_t::<shift>::mask_bits() - 1))
    }

    const fn num_bits() -> usize {
        let mut num = 0;

        if hb_set_digest_bits_pattern_t::<shift>::mask_bytes() >= 1 {
            num += 3;
        }

        if hb_set_digest_bits_pattern_t::<shift>::mask_bytes() >= 2 {
            num += 1;
        }

        if hb_set_digest_bits_pattern_t::<shift>::mask_bytes() >= 4 {
            num += 1;
        }

        if hb_set_digest_bits_pattern_t::<shift>::mask_bytes() >= 8 {
            num += 1;
        }

        if hb_set_digest_bits_pattern_t::<shift>::mask_bytes() >= 16 {
            num += 1;
        }

        num
    }
}