use std::time::Instant;

use rash_core::RunState;

use crate::{buffers::GlobalBuffer, texture::Texture};

#[derive(Clone, Copy)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

// Code taken from tutorial: https://sotrh.github.io/learn-wgpu/

mod buffers;
mod init;
mod texture;
mod tick;

pub struct Renderer {
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    sprites_buffer: wgpu::Buffer,
    global_buffer: wgpu::Buffer,

    window_size: WindowSize,
    global_state: GlobalBuffer,
    last_time: Instant,
    textures: Vec<Texture>,
    pub state: RunState,
}
