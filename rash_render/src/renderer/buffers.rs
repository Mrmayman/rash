use crate::CostumeId;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GraphicsState {
    pub x: f32,
    pub y: f32,
    pub texture_width: f32,
    pub texture_height: f32,
    pub size: f32,
    pub costume: CostumeId,
    pub center_x: f32,
    pub center_y: f32,
}

#[repr(C)]
pub struct GlobalBuffer {
    pub resolution: [f32; 2],
}
