use ttf_parser::colr::GradientExtend::Pad;
use ttf_parser::GlyphId;

// To make things easier, we don't have the generic parameter mask_t,
// and assume we always use u32, since this is what is also used in
// harfbuzz.
type mask_t = u32;

pub trait hb_set_digest_ext: Clone {
    type A;
    // Instead of `init()`
    fn new() -> Self;
    fn full() -> Self;
    fn union(&mut self, o: &Self::A);
    fn add(&mut self, g: GlyphId);
    fn add_array(&mut self, array: impl IntoIterator<Item = GlyphId> + Clone);
    fn add_range(&mut self, a: GlyphId, b: GlyphId) -> bool;
    fn may_have(&self, o: &Self::A) -> bool;
    fn may_have_glyph(&self, g: GlyphId) -> bool;
}

#[derive(Clone)]
pub struct hb_set_digest_bits_pattern_t<const shift: u8> {
    mask: mask_t,
}

impl<const shift: u8> hb_set_digest_bits_pattern_t<shift> {
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

impl<const shift: u8> hb_set_digest_ext for hb_set_digest_bits_pattern_t<shift> {
    type A = hb_set_digest_bits_pattern_t<shift>;

    fn new() -> Self {
        Self { mask: 0 }
    }

    fn full() -> Self {
        Self { mask: mask_t::MAX }
    }

    fn union(&mut self, o: &hb_set_digest_bits_pattern_t<shift>) {
        self.mask |= o.mask;
    }

    fn add(&mut self, g: GlyphId) {
        self.mask |= hb_set_digest_bits_pattern_t::<shift>::mask_for(g);
    }

    fn add_array(&mut self, array: impl IntoIterator<Item = GlyphId> + Clone) {
        for el in array {
            self.add(el);
        }
    }

    fn add_range(&mut self, a: GlyphId, b: GlyphId) -> bool {
        if self.mask == mask_t::MAX {
            return false;
        }

        if (b.0 as u32 >> shift) - (a.0 as u32 >> shift)
            >= hb_set_digest_bits_pattern_t::<shift>::mask_bits() - 1
        {
            self.mask = mask_t::MAX;
            false
        } else {
            let ma = hb_set_digest_bits_pattern_t::<shift>::mask_for(a);
            let mb = hb_set_digest_bits_pattern_t::<shift>::mask_for(b);
            self.mask |= mb + (mb - ma) - u32::from(mb < ma);
            true
        }
    }

    fn may_have(&self, o: &hb_set_digest_bits_pattern_t<shift>) -> bool {
        self.mask & o.mask != 0
    }

    fn may_have_glyph(&self, g: GlyphId) -> bool {
        self.mask & hb_set_digest_bits_pattern_t::<shift>::mask_for(g) != 0
    }
}

#[derive(Clone)]
pub struct hb_set_digest_combiner_t<head_t, tail_t>
where
    head_t: hb_set_digest_ext,
    tail_t: hb_set_digest_ext,
{
    head: head_t,
    tail: tail_t,
}

impl<head_t, tail_t> hb_set_digest_ext for hb_set_digest_combiner_t<head_t, tail_t>
where
    head_t: hb_set_digest_ext<A = head_t>,
    tail_t: hb_set_digest_ext<A = tail_t>,
{
    type A = hb_set_digest_combiner_t<head_t, tail_t>;

    fn new() -> Self {
        Self {
            head: head_t::new(),
            tail: tail_t::new(),
        }
    }

    fn full() -> Self {
        Self {
            head: head_t::full(),
            tail: tail_t::full(),
        }
    }

    fn union(&mut self, o: &Self::A) {
        self.head.union(&o.head);
        self.tail.union(&o.tail);
    }

    fn add(&mut self, g: GlyphId) {
        self.head.add(g);
        self.tail.add(g);
    }

    fn add_array(&mut self, array: impl IntoIterator<Item = GlyphId> + Clone) {
        // TODO: Is this expensive if someone passes e.g. a vector?
        self.head.add_array(array.clone());
        self.tail.add_array(array);
    }

    fn add_range(&mut self, a: GlyphId, b: GlyphId) -> bool {
        self.head.add_range(a, b) && self.tail.add_range(a, b)
    }

    fn may_have(&self, o: &Self::A) -> bool {
        self.head.may_have(&o.head) && self.tail.may_have(&o.tail)
    }

    fn may_have_glyph(&self, g: GlyphId) -> bool {
        self.head.may_have_glyph(g) && self.tail.may_have_glyph(g)
    }
}

#[rustfmt::skip]
pub type hb_set_digest_t = hb_set_digest_combiner_t<
    hb_set_digest_bits_pattern_t<4>,
    hb_set_digest_combiner_t<
        hb_set_digest_bits_pattern_t<0>,
        hb_set_digest_bits_pattern_t<9>
    >,
>;
