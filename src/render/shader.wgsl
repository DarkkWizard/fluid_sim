@group(0) @binding(0)
var<uniform> screen: vec2<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    let zero_to_one = model.position / screen;
    let zero_to_two = zero_to_one * 2.0;
    let clip_space = zero_to_two - 1.0;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_space.x, -clip_space.y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 0.0, 0.0); // Bright Yellow
}
