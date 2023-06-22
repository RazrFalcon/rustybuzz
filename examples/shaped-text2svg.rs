use std::io::Write;
use std::path::PathBuf;

use std::convert::TryInto;
use ttf_parser as ttf;

const FONT_SIZE: f64 = 32.0;

const HELP: &str = "\
Usage:
    shaped-text2svg font.ttf out.svg 'Hello world! مرحبا بالعالم!'
    shaped-text2svg --variations 'wght:500;wdth:200' font.ttf out.svg 'Hello world! مرحبا بالعالم!'
";

struct Args {
    #[allow(dead_code)]
    variations: Vec<ttf::Variation>,
    ttf_path: PathBuf,
    svg_path: PathBuf,
    text: String,
}

fn main() {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            print!("{}", HELP);
            std::process::exit(1);
        }
    };

    if let Err(e) = process(args) {
        eprintln!("Error: {}.", e);
        std::process::exit(1);
    }
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let variations = args.opt_value_from_fn("--variations", parse_variations)?;
    let free = args.finish();
    if free.len() != 3 {
        return Err("invalid number of arguments".into());
    }

    Ok(Args {
        variations: variations.unwrap_or_default(),
        ttf_path: PathBuf::from(&free[0]),
        svg_path: PathBuf::from(&free[1]),
        text: free[2].to_str().unwrap().to_string(),
    })
}

fn parse_variations(s: &str) -> Result<Vec<ttf::Variation>, &'static str> {
    let mut variations = Vec::new();
    for part in s.split(';') {
        let mut iter = part.split(':');

        let axis = iter.next().ok_or("failed to parse a variation")?;
        let axis = ttf::Tag::from_bytes_lossy(axis.as_bytes());

        let value = iter.next().ok_or("failed to parse a variation")?;
        let value: f32 = value.parse().map_err(|_| "failed to parse a variation")?;

        variations.push(ttf::Variation { axis, value });
    }

    Ok(variations)
}

fn process(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let font_data = std::fs::read(&args.ttf_path)?;

    let mut face = rustybuzz::Face::from_slice(&font_data, 0).unwrap();
    if face.is_variable() {
        for variation in args.variations {
            face.set_variation(variation.axis, variation.value)
                .ok_or("failed to create variation coordinates")?;
        }
    }

    let units_per_em = face.units_per_em();

    // HACK(eddyb) roughly `line-height: 1.2em`, which is close to what browsers
    // do in practice *but not exactly* (`line-height: normal` is more "adaptive").
    let line_height = units_per_em * 12 / 10;

    // HACK(eddyb) because we have to emit `viewBox` before any glyphs, and we
    // need to compute `viewBox` from the glyphs, we're forced to allocate the
    // complete set of shaped and positioned glyphs.
    let mut glyphs = vec![];
    let (mut total_width, mut total_height) = (0, 0);
    let mut rtl_lines = vec![];
    {
        let mut buffer = rustybuzz::UnicodeBuffer::new();

        // This relies on the UBA ("Unicode Bidirectional Algorithm")
        // (see http://www.unicode.org/reports/tr9/#Basic_Display_Algorithm),
        // as implemented by `unicode_bidi`, to slice the text into substrings
        // that can be individually shaped, then assembled visually.
        let bidi_info = unicode_bidi::BidiInfo::new(&args.text, None);

        // Treat each paragraph as a single line (i.e. no word-wrapping) - note
        // that, while in almost all cases, the "paragraph separator" will be a
        // newline (`\n`), Unicode also has several other separator codepoints.
        for para in &bidi_info.paragraphs {
            let (mut x, mut y) = (0, total_height);

            // FIXME(eddyb) `ParagraphInfo` includes the paragraph separator,
            // which is "lossless", but we don't want a glyph for e.g. `\n`.
            let para_sep_len = bidi_info.text[para.range.clone()]
                .chars()
                .last()
                .filter(|&sep| {
                    use unicode_bidi::{BidiClass, BidiDataSource, HardcodedBidiData};

                    // Bidi Class `B` is the short name of `Paragraph_Separator`
                    // (http://www.unicode.org/reports/tr44/#Bidi_Class_Values),
                    // which is what `unicode_bidi` uses to split paragraphs.
                    HardcodedBidiData.bidi_class(sep) == BidiClass::B
                })
                .map_or(0, |sep| sep.len_utf8());
            let para_range_without_separator = para.range.start..(para.range.end - para_sep_len);

            // Split each line into "runs" (that differ in their LTR/RTL "level").
            // FIXME(eddyb) use `.has_rtl()` to bypass some of the work here.
            // FIXME(eddyb) `visual_runs` returns a modified clone of the whole
            // `Vec<Level>`, which is the size of the original text being processed.
            let (adjusted_levels, runs) = bidi_info.visual_runs(para, para_range_without_separator);

            let line_glyphs = glyphs.len()..;
            for run_range in runs {
                let run_level = adjusted_levels[run_range.start];

                // FIXME(eddyb) UBA/`unicode_bidi` only offers a LTR/RTL distinction,
                // even if `rustybuzz` has vertical `Direction`s as well.
                buffer.set_direction(if run_level.is_rtl() {
                    rustybuzz::Direction::RightToLeft
                } else {
                    rustybuzz::Direction::LeftToRight
                });
                buffer.push_str(&bidi_info.text[run_range]);
                let glyph_buffer = rustybuzz::shape(&face, &[], buffer);

                glyphs.extend(
                    glyph_buffer
                        .glyph_infos()
                        .iter()
                        .zip(glyph_buffer.glyph_positions())
                        .map(|(glyph_info, glyph_pos)| {
                            let glyph = Glyph {
                                x: x + glyph_pos.x_offset,
                                y: y - glyph_pos.y_offset,
                                glyph_id: ttf::GlyphId(glyph_info.glyph_id.try_into().unwrap()),
                            };

                            x += glyph_pos.x_advance;
                            y -= glyph_pos.y_advance;

                            glyph
                        }),
                );

                buffer = glyph_buffer.clear();
            }

            if para.level.is_rtl() {
                let line_glyphs = line_glyphs.start..glyphs.len();
                rtl_lines.push((line_glyphs, x));
            }

            total_width = total_width.max(x);
            total_height += line_height;
        }
    }

    // Align RTL.
    for (glyph_range, width) in rtl_lines {
        let dx = total_width - width;
        for g in &mut glyphs[glyph_range] {
            g.x += dx;
        }
    }

    let scale = FONT_SIZE / units_per_em as f64;

    let mut svg =
        xmlwriter::XmlWriter::with_capacity(glyphs.len() * 512, xmlwriter::Options::default());
    svg.start_element("svg");
    svg.write_attribute("xmlns", "http://www.w3.org/2000/svg");
    svg.write_attribute("xmlns:xlink", "http://www.w3.org/1999/xlink");
    svg.write_attribute_fmt("viewBox", {
        format_args!(
            "{} {} {} {}",
            // FIXME(eddyb) take the bounding box of each glyph into account,
            // instead of simply adding a 1em margin.
            -FONT_SIZE,
            -FONT_SIZE,
            (total_width as f64 * scale).ceil() + 2.0 * FONT_SIZE,
            (total_height as f64 * scale).ceil() + 2.0 * FONT_SIZE,
        )
    });

    let mut path_buf = String::with_capacity(256);
    for g in glyphs {
        g.to_svg(&face, scale, &mut svg, &mut path_buf);
    }

    std::fs::write(&args.svg_path, &svg.end_document())?;

    Ok(())
}

struct Glyph {
    x: i32,
    y: i32,
    glyph_id: ttf::GlyphId,
}

impl Glyph {
    fn to_svg(
        &self,
        face: &ttf::Face,
        scale: f64,
        svg: &mut xmlwriter::XmlWriter,
        path_buf: &mut String,
    ) {
        let (x, y) = (self.x as f64 * scale, self.y as f64 * scale);
        let glyph_id = self.glyph_id;

        if let Some(img) = face.glyph_raster_image(glyph_id, std::u16::MAX) {
            svg.start_element("image");
            svg.write_attribute("x", &(x + img.x as f64));
            svg.write_attribute("y", &(y - img.y as f64));
            svg.write_attribute("width", &img.width);
            svg.write_attribute("height", &img.height);
            svg.write_attribute_raw("xlink:href", |buf| {
                buf.extend_from_slice(b"data:image/png;base64, ");

                let mut enc = base64::write::EncoderWriter::new(buf, base64::STANDARD);
                enc.write_all(img.data).unwrap();
                enc.finish().unwrap();
            });
            svg.end_element();
            return;
        }
        if let Some(img) = face.glyph_svg_image(glyph_id) {
            let height = face.height() as f64 * scale;
            svg.start_element("image");
            svg.write_attribute("x", &x);
            svg.write_attribute("y", &(y + height));
            svg.write_attribute("width", &height);
            svg.write_attribute("height", &height);
            svg.write_attribute_raw("xlink:href", |buf| {
                buf.extend_from_slice(b"data:image/svg+xml;base64, ");

                let mut enc = base64::write::EncoderWriter::new(buf, base64::STANDARD);
                enc.write_all(img).unwrap();
                enc.finish().unwrap();
            });
            svg.end_element();
            return;
        }

        path_buf.clear();
        let mut builder = Builder(path_buf);
        let bbox = match face.outline_glyph(glyph_id, &mut builder) {
            Some(v) => v,
            None => return,
        };
        if !path_buf.is_empty() {
            path_buf.pop(); // remove trailing space
        }

        let transform = format!("matrix({} 0 0 {} {} {})", scale, -scale, x, y);

        svg.start_element("path");
        svg.write_attribute("d", path_buf);
        svg.write_attribute("transform", &transform);
        svg.end_element();

        // FIXME(eddyb) maybe add a way to enable this?
        if false {
            let bbox_w = (bbox.x_max as f64 - bbox.x_min as f64) * scale;
            let bbox_h = (bbox.y_max as f64 - bbox.y_min as f64) * scale;
            let bbox_x = x + bbox.x_min as f64 * scale;
            let bbox_y = y - bbox.y_max as f64 * scale;

            svg.start_element("rect");
            svg.write_attribute("x", &bbox_x);
            svg.write_attribute("y", &bbox_y);
            svg.write_attribute("width", &bbox_w);
            svg.write_attribute("height", &bbox_h);
            svg.write_attribute("fill", "none");
            svg.write_attribute("stroke", "green");
            svg.end_element();
        }
    }
}

struct Builder<'a>(&'a mut String);

impl ttf::OutlineBuilder for Builder<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "M {} {} ", x, y).unwrap()
    }

    fn line_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "L {} {} ", x, y).unwrap()
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap()
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap()
    }

    fn close(&mut self) {
        self.0.push_str("Z ")
    }
}
