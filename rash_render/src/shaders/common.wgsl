struct Sprite {
    pos: vec2<f32>,
    texture_size: vec2<f32>,
    size: f32,
    costume_id: u32,
    center_pos: vec2<f32>,
}

struct Global {
    resolution: vec2<f32>,
}

@group(0) @binding(0) var<storage, read> sprite_state: array<Sprite>;
@group(0) @binding(1) var<uniform> global_state: Global;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
    @location(1) in_vertex_index: u32,
    @location(2) uv: vec2<f32>,
};
