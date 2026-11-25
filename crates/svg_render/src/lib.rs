use image::DynamicImage;
use serde::Deserialize;
use usvg::{TreeParsing, TreePostProc};

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Medium.ttf");
const SOURCE_SERIF: &[u8] = include_bytes!("../../../assets/fonts/SourceSerifPro-Regular.ttf");
const HANDLEE: &[u8] = include_bytes!("../../../assets/fonts/Handlee-Regular.ttf");
const KNEWAVE: &[u8] = include_bytes!("../../../assets/fonts/Knewave-Regular.ttf");
const GRIFFY: &[u8] = include_bytes!("../../../assets/fonts/Griffy-Regular.ttf");
const GRAND9K_PIXEL: &[u8] = include_bytes!("../../../assets/fonts/Grand9K Pixel.ttf");

pub struct SvgRenderer {
    fonts: usvg_text_layout::fontdb::Database,
}

impl SvgRenderer {
    #[must_use]
    pub fn new() -> Self {
        let mut fonts = usvg_text_layout::fontdb::Database::new();
        fonts.load_font_data(NOTO_SANS.to_vec());
        fonts.load_font_data(SOURCE_SERIF.to_vec());
        fonts.load_font_data(HANDLEE.to_vec());
        fonts.load_font_data(KNEWAVE.to_vec());
        fonts.load_font_data(GRIFFY.to_vec());
        fonts.load_font_data(GRAND9K_PIXEL.to_vec());
        fonts.load_system_fonts();

        fonts.set_sans_serif_family("Noto Sans");
        fonts.set_serif_family("Source Serif Pro");
        fonts.set_cursive_family("Handlee");
        fonts.set_fantasy_family("Griffy");

        Self { fonts }
    }

    pub fn render(&self, input: &str) -> Result<DynamicImage, usvg::Error> {
        if let Ok(parsed) = quick_xml::de::from_str::<Svg>(input)
            && parsed.width == "0"
            && parsed.height == "0"
        {
            // SVG is empty
            let blank_image = image::DynamicImage::new_rgba8(1, 1);
            return Ok(blank_image);
        }

        let usvg_options = usvg::Options {
            resources_dir: None,
            ..Default::default()
        };

        let input = map_scratch_fonts(input);
        let mut tree = usvg::Tree::from_data(input.as_bytes(), &usvg_options)?;
        tree.postprocess(
            usvg::PostProcessingSteps {
                convert_text_into_paths: true,
            },
            &self.fonts,
        );

        let mut pixmap =
            resvg::tiny_skia::Pixmap::new(tree.size.width() as u32, tree.size.height() as u32)
                .unwrap();

        resvg::render(&tree, usvg::Transform::default(), &mut pixmap.as_mut());

        let image =
            image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_owned())
                .unwrap();

        let dyn_image = image::DynamicImage::ImageRgba8(image).resize(
            ((pixmap.width() as f32) * 2.0) as u32,
            ((pixmap.height() as f32) * 2.0) as u32,
            image::imageops::FilterType::Nearest,
        );
        Ok(dyn_image)
    }
}

#[derive(Debug, Deserialize)]
struct Svg {
    #[serde(rename = "width")]
    width: String,
    #[serde(rename = "height")]
    height: String,
}

fn map_scratch_fonts(svg: &str) -> String {
    const MAP: &[(&str, &str)] = &[
        ("Sans Serif", "Noto Sans"),
        ("Serif", "Source Serif Pro"),
        ("Handwriting", "Handlee"),
        ("Marker", "Knewave"),
        ("Curly", "Griffy"),
        ("Pixel", "Grand9K Pixel"),
    ];

    let mut out = svg.to_owned();
    for (from, to) in MAP {
        out = out.replace(
            &format!("font-family=\"{from}\""),
            &format!("font-family=\"{to}\""),
        );
        out = out.replace(
            &format!("font-family='{from}'"),
            &format!("font-family='{to}'"),
        );
    }
    out
}
