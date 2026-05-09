use std::{collections::HashMap, time::Instant};

use rash_vm::{CostumeId, RunState};

use crate::{buffers::GlobalBuffer, texture::Costume};

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

fn to_bytes<T>(s: &[T]) -> &[u8] {
    let ptr = s.as_ptr().cast();
    unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of_val(s)) }
}

pub struct Renderer {
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    sprites_buffer: wgpu::Buffer,
    global_buffer: wgpu::Buffer,

    window_size: WindowSize,
    global_state: GlobalBuffer,
    last_time: Instant,
    costumes: HashMap<CostumeId, Costume>,
    pub state: RunState,
}
