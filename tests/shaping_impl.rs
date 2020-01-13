use std::str::FromStr;

struct Args {
    face_index: u32,
    font_size: Option<f32>,
    font_ptem: Option<f32>,
    #[allow(dead_code)] font_funcs: Option<String>, // we don't use it, but have to parse it anyway
    variations: Vec<String>,
    #[allow(dead_code)] shaper: Vec<String>, // we don't use it, but have to parse it anyway
    #[allow(dead_code)] shapers: Vec<String>, // we don't use it, but have to parse it anyway
    direction: Option<rustybuzz::Direction>,
    language: Option<rustybuzz::Language>,
    script: Option<rustybuzz::Script>,
    #[allow(dead_code)] remove_default_ignorables: bool, // we don't use it, but have to parse it anyway
    cluster_level: u32,
    features: Vec<String>,
    no_glyph_names: bool,
    no_positions: bool,
    no_advances: bool,
    no_clusters: bool,
    show_extents: bool,
    show_flags: bool,
    ned: bool,
}

fn parse_args(args: Vec<std::ffi::OsString>) -> Result<Args, pico_args::Error> {
    let mut parser = pico_args::Arguments::from_vec(args);
    let args = Args {
        face_index: parser.opt_value_from_str("--face-index")?.unwrap_or(0),
        font_size: parser.opt_value_from_str("--font-size")?,
        font_ptem: parser.opt_value_from_str("--font-ptem")?,
        font_funcs: parser.opt_value_from_str("--font-funcs")?,
        variations: parser.opt_value_from_fn("--variations", parse_string_list)?.unwrap_or_default(),
        shaper: parser.opt_value_from_fn("--shaper", parse_string_list)?.unwrap_or_default(),
        shapers: parser.opt_value_from_fn("--shapers", parse_string_list)?.unwrap_or_default(),
        direction: parser.opt_value_from_str("--direction")?,
        language: parser.opt_value_from_str("--language")?,
        script: parser.opt_value_from_str("--script")?,
        remove_default_ignorables: parser.contains("--remove-default-ignorables"),
        cluster_level: parser.opt_value_from_str("--cluster-level")?.unwrap_or(0),
        features: parser.opt_value_from_fn("--features", parse_string_list)?.unwrap_or_default(),
        no_glyph_names: parser.contains("--no-glyph-names"),
        no_positions: parser.contains("--no-positions"),
        no_advances: parser.contains("--no-advances"),
        no_clusters: parser.contains("--no-clusters"),
        show_extents: parser.contains("--show-extents"),
        show_flags: parser.contains("--show-flags"),
        ned: parser.contains("--ned"),
    };

    parser.finish()?;

    Ok(args)
}

fn parse_string_list(s: &str) -> Result<Vec<String>, String> {
    Ok(s.split(',').map(|s| s.to_string()).collect())
}

pub fn shape(font: &str, text: &str, options: &str) -> String {
    use std::io::Read;

    let args = options.split(' ').filter(|s| !s.is_empty()).map(|s| std::ffi::OsString::from(s)).collect();
    let args = parse_args(args).unwrap();

    let font_data = std::fs::read(&format!("harfbuzz/test/shaping/data/{}", font)).unwrap();
    let face = match rustybuzz::Face::new(&font_data, args.face_index) {
        Ok(v) => v,
        Err(e) => return e.to_string(),
    };
    let mut font = rustybuzz::Font::new(face);

    if let Some(ptem) = args.font_ptem {
        font.set_ptem(ptem);
    }

    if let Some(size) = args.font_size {
        font.set_scale(size as i32, size as i32);
    }

    if !args.variations.is_empty() {
        let variations: Vec<_> = args.variations.iter()
            .map(|s| rustybuzz::Variation::from_str(s).unwrap()).collect();
        font.set_variations(&variations);
    }

    let mut buffer = rustybuzz::Buffer::new();
    buffer.add_str(&text);

    if let Some(d) = args.direction {
        buffer.set_direction(d);
    }

    if let Some(lang) = args.language {
        buffer.set_language(lang);
    }

    if let Some(script) = args.script {
        buffer.set_script(script);
    }

    buffer.set_cluster_level(args.cluster_level);
    buffer.reset_clusters();

    let mut features = Vec::new();
    for feature_str in args.features {
        let feature = rustybuzz::Feature::from_str(&feature_str).unwrap();
        features.push(feature);
    }

    let output = rustybuzz::shape(&font, buffer, &features);

    let mut format_flags = rustybuzz::SerializeFlags::default();
    if args.no_glyph_names {
        format_flags |= rustybuzz::SerializeFlags::NO_GLYPH_NAMES;
    }

    if args.no_clusters || args.ned {
        format_flags |= rustybuzz::SerializeFlags::NO_CLUSTERS;
    }

    if args.no_positions {
        format_flags |= rustybuzz::SerializeFlags::NO_POSITIONS;
    }

    if args.no_advances || args.ned {
        format_flags |= rustybuzz::SerializeFlags::NO_ADVANCES;
    }

    if args.show_extents {
        format_flags |= rustybuzz::SerializeFlags::GLYPH_EXTENTS;
    }

    if args.show_flags {
        format_flags |= rustybuzz::SerializeFlags::GLYPH_FLAGS;
    }

    let mut res = String::new();
    output.serializer(
        Some(&font),
        rustybuzz::SerializeFormat::Text,
        format_flags,
    ).read_to_string(&mut res).unwrap();
    res
}
