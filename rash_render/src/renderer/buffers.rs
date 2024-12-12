#[repr(C)]
pub struct GraphicsState {
    pub x: f32,
    pub y: f32,
    pub texture_width: f32,
    pub texture_height: f32,
    pub size: f32,
    pub _padding: [f32; 8 - 5],
}

#[repr(C)]
pub struct GlobalBuffer {
    pub resolution: [f32; 2],
}
