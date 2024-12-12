use std::time::Instant;

use buffers::{GlobalBuffer, GraphicsState};
use winit::window::Window;

pub mod buffers;
pub mod init;
pub mod tick;

pub struct Renderer<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub sprites_buffer: wgpu::Buffer,
    pub sprites_state: Vec<GraphicsState>,
    pub global_buffer: wgpu::Buffer,
    pub global_state: GlobalBuffer,
    pub last_time: Instant,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    pub window: &'a Window,
}
