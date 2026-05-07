use std::time::Instant;

use crate::buffers::GlobalBuffer;

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

pub use texture::Costume;

fn to_bytes<T: ?Sized>(s: &T, size: usize) -> &[u8] {
    let ptr = s as *const T as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, size) }
}

pub struct Renderer {
    config: wgpu::SurfaceConfiguration,
    window_size: WindowSize,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    sprites_buffer: wgpu::Buffer,
    global_buffer: wgpu::Buffer,
    global_state: GlobalBuffer,
    last_time: Instant,
    pub sampler: wgpu::Sampler,
    pub costume_layout: wgpu::BindGroupLayout,
}
