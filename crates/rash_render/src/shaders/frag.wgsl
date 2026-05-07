@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.uv);
    return color;
    // return vec4<f32>(in.uv, 0.0, 1.0);
}

fn id_to_color(id: u32) -> vec3<f32> {
    let step_size = 0.25;
    let steps_len = u32(1.0 / step_size) + 1;
    let normalized_id = id % 65u;

    let blue = step_size * f32(normalized_id % steps_len);
    let green = step_size * f32((normalized_id / steps_len) % steps_len);
    let red = step_size * f32((normalized_id / (steps_len * steps_len)) % steps_len);
    return vec3<f32>(red, green, blue);
}
