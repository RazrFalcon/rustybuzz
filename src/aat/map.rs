use std::marker::PhantomData;

use crate::{ffi, Face, Tag};
use super::feature_mappings::FEATURE_MAPPINGS;


#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum FeatureType {
    Ligatures = 1,
    LetterCase = 3,
    VerticalSubstitution = 4,
    NumberSpacing = 6,
    VerticalPosition = 10,
    Fractions = 11,
    TypographicExtras = 14,
    MathematicalExtras = 15,
    StyleOptions = 19,
    CharacterShape = 20,
    NumberCase = 21,
    TextSpacing = 22,
    Transliteration = 23,
    RubyKana = 28,
    ItalicCjkRoman = 32,
    CaseSensitiveLayout = 33,
    AlternateKana = 34,
    StylisticAlternatives = 35,
    ContextualAlternatives = 36,
    LowerCase = 37,
    UpperCase = 38,
}


#[repr(C)]
pub struct Map {
    // This has to have the exact same memory size as the C++ variant!
    inner: ffi::rb_aat_map_t
}

impl Map {
    pub fn new() -> Self {
        // Initialized on the C++ side.
        let mut map = Map {
            inner: ffi::rb_aat_map_t {
                _chain_flags: ffi::rb_vector_t::zero(),
            }
        };

        unsafe { ffi::rb_aat_map_init(&mut map.inner); }

        map
    }

    pub fn as_ptr(&self) -> *const ffi::rb_aat_map_t {
        self as *const _ as *const ffi::rb_aat_map_t
    }

    pub fn as_ptr_mut(&mut self) -> *mut ffi::rb_aat_map_t {
        self as *mut _ as *mut ffi::rb_aat_map_t
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        unsafe { ffi::rb_aat_map_fini(&mut self.inner); }
    }
}

#[repr(C)]
pub struct MapBuilder<'a> {
    // This has to have the exact same memory size as the C++ variant!
    inner: ffi::rb_aat_map_builder_t,
    phantom: PhantomData<&'a Face<'a>>,
}

impl<'a> MapBuilder<'a> {
    pub fn new(face: &'a Face<'a>) -> Self {
        // Initialized on the C++ side.
        let mut builder = MapBuilder {
            inner: ffi::rb_aat_map_builder_t {
                _face: std::ptr::null(),
                _features: ffi::rb_vector_t::zero(),
            },
            phantom: PhantomData,
        };

        unsafe { ffi::rb_aat_map_builder_init(&mut builder.inner, face.as_ptr()); }

        builder
    }

    pub fn as_ptr(&mut self) -> *mut ffi::rb_aat_map_builder_t {
        self as *mut _ as *mut ffi::rb_aat_map_builder_t
    }

    pub fn add_feature(&mut self, tag: Tag, value: u32) -> Option<()> {
        const FEATURE_TYPE_CHARACTER_ALTERNATIVES: u16 = 17;

        let face = Face::from_ptr(self.inner._face);
        let feat = face.feat?;

        if tag == Tag::from_bytes(b"aalt") {
            if !feat.exposes_feature(FEATURE_TYPE_CHARACTER_ALTERNATIVES) {
                return Some(());
            }

            unsafe {
                ffi::rb_aat_map_builder_add_feature(
                    self.as_ptr(),
                    FEATURE_TYPE_CHARACTER_ALTERNATIVES as i32,
                    value as i32,
                    true,
                );
            }
        }

        let idx = FEATURE_MAPPINGS.binary_search_by(|map| map.ot_feature_tag.cmp(&tag)).ok()?;
        let mapping = &FEATURE_MAPPINGS[idx];

        let mut feature = feat.feature(mapping.aat_feature_type as u16);

        match feature {
            Some(feature) if feature.has_data() => {}
            _ => {
                // Special case: Chain::compile_flags will fall back to the deprecated version of
                // small-caps if necessary, so we need to check for that possibility.
                // https://github.com/harfbuzz/harfbuzz/issues/2307
                if  mapping.aat_feature_type == FeatureType::LowerCase &&
                    mapping.selector_to_enable == super::feature_selector::LOWER_CASE_SMALL_CAPS
                {
                    feature = feat.feature(FeatureType::LetterCase as u16);
                }
            }
        }

        match feature {
            Some(feature) if feature.has_data() => {
                unsafe {
                    ffi::rb_aat_map_builder_add_feature(
                        self.as_ptr(),
                        mapping.aat_feature_type as i32,
                        if value != 0 { mapping.selector_to_enable } else { mapping.selector_to_disable } as i32,
                        feature.is_exclusive(),
                    );
                }
            }
            _ => {}
        }

        Some(())
    }

    pub fn compile(&mut self) -> Map {
        let mut map = Map::new();
        unsafe { ffi::rb_aat_map_builder_compile(self.as_ptr(), map.as_ptr_mut()); }
        map
    }
}

impl Drop for MapBuilder<'_> {
    fn drop(&mut self) {
        unsafe { ffi::rb_aat_map_builder_fini(&mut self.inner); }
    }
}
