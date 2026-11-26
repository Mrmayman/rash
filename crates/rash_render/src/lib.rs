use rash_core::RunState;

use crate::{buffers::GlobalBuffer, texture::Texture};

#[derive(Clone, Copy)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub struct RenderTarget {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: WindowSize,
}

impl RenderTarget {
    pub fn new(device: &wgpu::Device, size: WindowSize) -> Self {
        assert!(size.width > 0 && size.height > 0);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Rash Render Target"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            _texture: texture,
            view,
            size,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub const fn size(&self) -> WindowSize {
        self.size
    }
}

impl WindowSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

// Code taken from tutorial: https://sotrh.github.io/learn-wgpu/

mod buffers;
mod init;
mod texture;
mod tick;

pub struct Renderer {
    config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    sprites_buffer: wgpu::Buffer,
    global_buffer: wgpu::Buffer,

    window_size: WindowSize,
    global_state: GlobalBuffer,
    textures: Vec<Texture>,
    pub state: RunState,
}
