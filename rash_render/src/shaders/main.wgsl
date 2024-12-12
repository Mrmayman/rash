struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    var local_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5), // Bottom-left
        vec2<f32>( 0.5, -0.5), // Bottom-right
        vec2<f32>(-0.5,  0.5), // Top-left
        vec2<f32>(-0.5,  0.5), // Top-left
        vec2<f32>( 0.5, -0.5), // Bottom-right
        vec2<f32>( 0.5,  0.5)  // Top-right
    );

    let local_position: vec2<f32> = local_positions[in_vertex_index];
    out.clip_position = vec4<f32>(local_position, 0.0, 1.0);
    out.vert_pos = out.clip_position.xyz;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
