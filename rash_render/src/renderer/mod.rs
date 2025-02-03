use std::{sync::Arc, time::Instant};

use buffers::GlobalBuffer;
use winit::window::Window;

pub mod buffers;
pub mod init;
pub mod tick;

pub fn to_bytes<T: ?Sized>(s: &T, size: usize) -> &[u8] {
    let ptr = s as *const T as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, size) }
}

pub struct InnerRenderer<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub sprites_buffer: wgpu::Buffer,
    pub global_buffer: wgpu::Buffer,
    pub global_state: GlobalBuffer,
    pub last_time: Instant,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    pub window: Arc<Window>,
}
