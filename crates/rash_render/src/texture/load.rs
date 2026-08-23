use rash_core::CostumeStore;
use svg_render::SvgRenderer;

use crate::texture::Texture;

pub fn load_textures(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    costumes: &mut CostumeStore,
    costume_layout: wgpu::BindGroupLayout,
) -> Vec<Texture> {
    let sampler = Texture::create_sampler(device);

    let svg_renderer = SvgRenderer::new();

    let textures = generate_textures(
        costumes,
        &svg_renderer,
        device,
        queue,
        &sampler,
        &costume_layout,
    );
    let textures = match textures {
        Ok(n) => n,
        Err(err) => {
            eprintln!("While loading costumes: {err}");
            Vec::new()
        }
    };
    costumes.free_memory();
    textures
}

fn generate_textures(
    costumes: &CostumeStore,
    svg_renderer: &SvgRenderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    sampler: &wgpu::Sampler,
    costume_layout: &wgpu::BindGroupLayout,
) -> Result<Vec<Texture>, Box<dyn std::error::Error>> {
    costumes
        .iter_by_id()
        .map(|(_, costume)| {
            if costume.is_svg
                && let Ok(svg_text) = String::from_utf8(costume.bytes.clone())
            {
                let img = svg_renderer.render(&svg_text)?;

                return Ok(Texture::from_image(
                    costume,
                    device,
                    queue,
                    &img,
                    sampler,
                    costume_layout,
                ));
            }

            Ok(Texture::from_bytes(
                costume,
                device,
                queue,
                sampler,
                costume_layout,
            )?)
        })
        .collect()
}
