@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    var local_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // Bottom-left
        vec2<f32>( 1.0, -1.0), // Bottom-right
        vec2<f32>(-1.0,  1.0), // Top-left
        vec2<f32>(-1.0,  1.0), // Top-left
        vec2<f32>( 1.0, -1.0), // Bottom-right
        vec2<f32>( 1.0,  1.0)  // Top-right
    );

    let sprite: Sprite = sprite_state[in_vertex_index / 6];

    let local_position: vec2<f32> = local_positions[in_vertex_index] * (sprite.texture_size / global_state.resolution);

    let world_position: vec2<f32> =
        (local_position * sprite.size / 100.0)
        + (sprite.pos / global_state.resolution);

    out.clip_position = vec4<f32>(world_position, 0.0, 1.0);
    out.vert_pos = out.clip_position.xyz;
    return out;
}
