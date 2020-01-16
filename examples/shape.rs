use std::path::PathBuf;
use std::str::FromStr;

// TODO: add --font-ppem
// TODO: add --text-before
// TODO: add --text-after
// TODO: add --bot
// TODO: add --eot
// TODO: add --preserve-default-ignorables
// TODO: add --remove-default-ignorables
// TODO: add --show-text
// TODO: add --show-unicode
// TODO: add --show-line-num
// TODO: add --verify
// TODO: add --trace
// TODO: add --verbose
// TODO: add --num-iterations

const HELP: &str = "\
USAGE:
    shape [OPTIONS] [FONT-FILE] [TEXT]

OPTIONS:
    -h, --help                          Show help options
        --version                       Show version number
        --font-file FILENAME            Set font file-name
        --face-index INDEX              Set face index [default: 0]
        --font-size NUMBER              Font size [default: upem]
        --font-ptem NUMBER              Set font point-size
        --variations LIST               Set comma-separated list of font variations
        --text TEXT                     Set input text
    -u, --unicodes LIST                 Set comma-separated list of input Unicode codepoints
                                        Examples: 'U+0056,U+0057'
        --text-before TEXT              Set text context before each line
        --text-after TEXT               Set text context after each line
        --direction DIRECTION           Set text direction
                                        [possible values: ltr, rtl, ttb, btt]
        --language LANG                 Set text language [default: $LANG]
        --script TAG                    Set text script as ISO-15924 tag
        --invisible-glyph CHAR          Glyph value to replace Default-Ignorables with
        --utf8-clusters                 Use UTF-8 byte indices, not char indices
        --cluster-level N               Cluster merging level [default: 0]
                                        [possible values: 0, 1, 2]
        --normalize-glyphs              Rearrange glyph clusters in nominal order
        --features LIST                 Set comma-separated list of font features
        --output-format FORMAT          Set output format [default: text] [possible values: text, json]
        --no-glyph-names                Output glyph indices instead of names
        --no-positions                  Do not output glyph positions
        --no-advances                   Do not output glyph advances
        --no-clusters                   Do not output cluster indices
        --show-extents                  Output glyph extents
        --show-flags                    Output glyph flags
        --ned                           No Extra Data; Do not output clusters or advances

ARGS:
    [FONT-FILE]                         An optional path to font file
    [TEXT]                              An optional text
";

struct Args {
    help: bool,
    version: bool,
    font_file: Option<PathBuf>,
    face_index: u32,
    font_size: Option<f32>,
    font_ptem: Option<f32>,
    variations: Vec<rustybuzz::Variation>,
    text: Option<String>,
    unicodes: Option<String>,
    direction: Option<rustybuzz::Direction>,
    language: Option<rustybuzz::Language>,
    script: Option<rustybuzz::Script>,
    invisible_glyph: Option<u32>,
    utf8_clusters: bool,
    cluster_level: u32,
    normalize_glyphs: bool,
    features: Vec<rustybuzz::Feature>,
    format: String, // TODO: to enum
    no_glyph_names: bool,
    no_positions: bool,
    no_advances: bool,
    no_clusters: bool,
    show_extents: bool,
    show_flags: bool,
    ned: bool,
    free: Vec<String>,
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let mut args = pico_args::Arguments::from_env();
    let args = Args {
        help: args.contains(["-h", "--help"]),
        version: args.contains("--version"),
        font_file: args.opt_value_from_str("--font-file")?,
        face_index: args.opt_value_from_str("--face-index")?.unwrap_or(0),
        font_size: args.opt_value_from_str("--font-size")?,
        font_ptem: args.opt_value_from_str("--font-ptem")?,
        variations: args.opt_value_from_fn("--variations", parse_variations)?.unwrap_or_default(),
        text: args.opt_value_from_str("--text")?,
        unicodes: args.opt_value_from_fn(["-u", "--unicodes"], parse_unicodes)?,
        direction: args.opt_value_from_str("--direction")?,
        language: args.opt_value_from_str("--language")?,
        script: args.opt_value_from_str("--script")?,
        invisible_glyph: args.opt_value_from_str("--invisible-glyph")?,
        utf8_clusters: args.contains("--utf8-clusters"),
        cluster_level: args.opt_value_from_str("--cluster-level")?.unwrap_or(0),
        normalize_glyphs: args.contains("--normalize-glyphs"),
        features: args.opt_value_from_fn("--features", parse_features)?.unwrap_or_default(),
        format: args.opt_value_from_str("--output-format")?.unwrap_or("text".to_string()),
        no_glyph_names: args.contains("--no-glyph-names"),
        no_positions: args.contains("--no-positions"),
        no_advances: args.contains("--no-advances"),
        no_clusters: args.contains("--no-clusters"),
        show_extents: args.contains("--show-extents"),
        show_flags: args.contains("--show-flags"),
        ned: args.contains("--ned"),
        free: args.free()?,
    };

    Ok(args)
}

fn main() {
    use std::io::Read;

    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.help {
        print!("{}", HELP);
        return;
    }

    let mut font_set_as_free_arg = false;
    let font_path = if let Some(path) = args.font_file {
        path.clone()
    } else if !args.free.is_empty() {
        font_set_as_free_arg = true;
        PathBuf::from(&args.free[0])
    } else {
        eprintln!("Error: font is not set.");
        std::process::exit(1);
    };

    if !font_path.exists() {
        eprintln!("Error: '{}' does not exist.", font_path.display());
        std::process::exit(1);
    }

    let font_data = std::fs::read(font_path).expect("failed to load a file");
    let face = match rustybuzz::Face::new(&font_data, args.face_index) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };
    let mut font = rustybuzz::Font::new(face);

    if let Some(ptem) = args.font_ptem {
        font.set_ptem(ptem);
    }

    if let Some(size) = args.font_size {
        font.set_scale(size as i32, size as i32);
    }

    if !args.variations.is_empty() {
        font.set_variations(&args.variations);
    }

    let text = if args.free.len() == 2 && font_set_as_free_arg {
        &args.free[1]
    } else if args.free.len() == 1 && !font_set_as_free_arg {
        &args.free[0]
    } else if let Some(ref text) = args.unicodes {
        text
    } else if let Some(ref text) = args.text {
        text
    } else {
        eprintln!("Error: text is not set.");
        std::process::exit(1);
    };

    let mut buffer = rustybuzz::Buffer::new();
    buffer.add_str(text);

    if let Some(d) = args.direction {
        buffer.set_direction(d);
    }

    if let Some(lang) = args.language {
        buffer.set_language(lang);
    }

    if let Some(script) = args.script {
        buffer.set_script(script);
    }

    if args.cluster_level < 2 {
        buffer.set_cluster_level(args.cluster_level);
    }

    if let Some(g) = args.invisible_glyph {
        buffer.set_invisible_glyph(g);
    }

    if !args.utf8_clusters {
        buffer.reset_clusters();
    }

    let mut output = rustybuzz::shape(&font, buffer, &args.features);

    if args.normalize_glyphs {
        output.normalize_glyphs();
    }

    let format = if args.format == "json" {
        rustybuzz::SerializeFormat::Json
    } else {
        rustybuzz::SerializeFormat::Text
    };

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
    output.serializer(Some(&font), format, format_flags).read_to_string(&mut res).unwrap();
    println!("{}", res);
}

fn parse_unicodes(s: &str) -> Result<String, String> {
    use std::convert::TryFrom;

    let mut text = String::new();
    for u in s.split(',') {
        let u = u32::from_str_radix(&u[2..], 16)
            .map_err(|_| format!("'{}' is not a valid codepoint", u))?;

        let c = char::try_from(u).map_err(|_| format!("{} is not a valid codepoint", u))?;

        text.push(c);
    }

    Ok(text)
}

fn parse_features(s: &str) -> Result<Vec<rustybuzz::Feature>, String> {
    let mut features = Vec::new();
    for f in s.split(',') {
        features.push(rustybuzz::Feature::from_str(&f)?);
    }

    Ok(features)
}

fn parse_variations(s: &str) -> Result<Vec<rustybuzz::Variation>, String> {
    let mut variations = Vec::new();
    for v in s.split(',') {
        variations.push(rustybuzz::Variation::from_str(&v)?);
    }

    Ok(variations)
}
