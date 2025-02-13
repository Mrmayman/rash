use image::{GenericImageView, ImageError};

#[derive(Clone)]
pub struct IntermediateCostume {
    pub bytes: Vec<u8>,
    pub name: String,
    pub hash: String,
    pub rotation_center_x: f64,
    pub rotation_center_y: f64,
}

#[allow(unused)]
pub struct Costume {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub name: String,
    pub hash: String,
    pub rotation_center_x: f64,
    pub rotation_center_y: f64,
    pub texture_width: u32,
    pub texture_height: u32,
}

impl Costume {
    pub fn from_bytes(
        costume: &IntermediateCostume,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sampler: &wgpu::Sampler,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, ImageError> {
        let img = image::load_from_memory(&costume.bytes)?;
        Ok(Self::from_image(
            costume, device, queue, &img, sampler, layout,
        ))
    }

    pub fn from_image(
        costume: &IntermediateCostume,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        sampler: &wgpu::Sampler,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&costume.name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        Self {
            name: costume.name.clone(),
            hash: costume.hash.clone(),
            texture,
            view,
            bind_group,
            rotation_center_x: costume.rotation_center_x,
            rotation_center_y: costume.rotation_center_y,
            texture_width: dimensions.0,
            texture_height: dimensions.1,
        }
    }

    pub fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    pub fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Costume Bind Group Layout"),
        })
    }
}
