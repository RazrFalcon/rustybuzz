use std::str::FromStr;

struct Args {
    face_index: u32,
    font_ptem: Option<f32>,
    variations: Vec<String>,
    direction: Option<rustybuzz::Direction>,
    language: Option<rustybuzz::Language>,
    script: Option<rustybuzz::Script>,
    #[allow(dead_code)] remove_default_ignorables: bool, // we don't use it, but have to parse it anyway
    cluster_level: rustybuzz::BufferClusterLevel,
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
        font_ptem: parser.opt_value_from_str("--font-ptem")?,
        variations: parser.opt_value_from_fn("--variations", parse_string_list)?.unwrap_or_default(),
        direction: parser.opt_value_from_str("--direction")?,
        language: parser.opt_value_from_str("--language")?,
        script: parser.opt_value_from_str("--script")?,
        remove_default_ignorables: parser.contains("--remove-default-ignorables"),
        cluster_level: parser.opt_value_from_fn("--cluster-level", parse_cluster)?.unwrap_or_default(),
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

fn parse_cluster(s: &str) -> Result<rustybuzz::BufferClusterLevel, String> {
    match s {
        "0" => Ok(rustybuzz::BufferClusterLevel::MonotoneGraphemes),
        "1" => Ok(rustybuzz::BufferClusterLevel::MonotoneCharacters),
        "2" => Ok(rustybuzz::BufferClusterLevel::Characters),
        _ => Err(format!("invalid cluster level"))
    }
}

pub fn shape(font_path: &str, text: &str, options: &str) -> String {
    let args = options.split(' ').filter(|s| !s.is_empty()).map(|s| std::ffi::OsString::from(s)).collect();
    let args = parse_args(args).unwrap();

    let font_data = std::fs::read(font_path).unwrap();
    let mut face = rustybuzz::Face::from_slice(&font_data, args.face_index).unwrap();

    face.set_points_per_em(args.font_ptem);

    if !args.variations.is_empty() {
        let variations: Vec<_> = args.variations.iter()
            .map(|s| rustybuzz::Variation::from_str(s).unwrap()).collect();
        face.set_variations(&variations);
    }

    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(text);

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

    let glyph_buffer = rustybuzz::shape(&face, &features, buffer);

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

    glyph_buffer.serialize(&face, format_flags)
}
