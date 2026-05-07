const screen_width = 360.0;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.in_vertex_index = in_vertex_index;

    var local_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // Bottom-left
        vec2<f32>( 1.0, -1.0), // Bottom-right
        vec2<f32>(-1.0,  1.0), // Top-left
        vec2<f32>(-1.0,  1.0), // Top-left
        vec2<f32>( 1.0, -1.0), // Bottom-right
        vec2<f32>( 1.0,  1.0)  // Top-right
    );

    out.uv = (local_positions[in_vertex_index % 6] + 1.0) / 2.0;
    out.uv.y = 1.0 - out.uv.y;

    let global_resolution = vec2<f32>(screen_width * (global_state.resolution.x / global_state.resolution.y), screen_width);

    let sprite: Sprite = sprite_state[in_vertex_index / 6];
    let sprite_center_pos_ = ((sprite.center_pos - (sprite.texture_size * 0.5)) / global_resolution);
    let sprite_center_pos = vec2<f32>(-sprite_center_pos_.x, sprite_center_pos_.y);

    let local_position: vec2<f32> = (local_positions[in_vertex_index % 6]
        * ((sprite.texture_size * 0.5) / global_resolution))
        + sprite_center_pos;

    let world_position: vec2<f32> =
        (local_position * sprite.size / 100.0)
        + (sprite.pos * 2.0 / global_resolution);

    out.clip_position = vec4<f32>(world_position, 0.0, 1.0);
    out.vert_pos = out.clip_position.xyz;
    return out;
}
