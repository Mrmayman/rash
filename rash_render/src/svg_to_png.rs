use anyhow::Context;
use image::DynamicImage;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Svg {
    #[serde(rename = "width")]
    width: String,
    #[serde(rename = "height")]
    height: String,
}

use usvg_text_layout::TreeTextToPath;

pub fn render(
    input: &str,
    fontdb: &usvg_text_layout::fontdb::Database,
) -> anyhow::Result<DynamicImage> {
    // Check if the SVG is empty.
    let parsed: Svg = serde_xml_rs::from_str(&input).unwrap();
    if parsed.width == "0" && parsed.height == "0" {
        let blank_image = image::DynamicImage::new_rgba8(1, 1);
        return Ok(blank_image);
    }

    // Setup USVG Options.
    let usvg_options = usvg::Options {
        // Get file's absolute directory.
        resources_dir: std::fs::canonicalize(input)
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())),
        ..Default::default()
    };

    // Build SVG Tree.
    let mut tree = usvg::Tree::from_data(input.as_bytes(), &usvg_options)?;
    // Render text if needed.
    tree.convert_text(fontdb);

    // Create Pixel Map to draw SVG to.
    let mut pixmap =
        tiny_skia::Pixmap::new(tree.size.width() as u32, tree.size.height() as u32).unwrap();

    // Draw to Pixel Map.
    resvg::render(
        &tree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    );

    let image =
        image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_owned())
            .context("Could not construct RGBA image from converted buffer (SVG)")?;

    let dyn_image = image::DynamicImage::ImageRgba8(image).resize(
        ((pixmap.width() as f32) * 2.0) as u32,
        ((pixmap.height() as f32) * 2.0) as u32,
        image::imageops::FilterType::Nearest,
    );
    Ok(dyn_image)
}
