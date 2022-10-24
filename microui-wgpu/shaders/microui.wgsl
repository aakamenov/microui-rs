struct VertexOutput {
    @location(1) color: vec4<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u_screen_size: vec2<f32>;

@vertex
fn vs_main(
    @location(0) pos: vec2<i32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = color;
    out.clip_position = vec4<f32>(
        2.0 * f32(pos.x) / u_screen_size.x - 1.0,
        1.0 - 2.0 * f32(pos.y) / u_screen_size.y,
        0.5,
        1.0,
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
