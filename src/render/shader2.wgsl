// Uniforms to control the canvas and point size
@group(0) @binding(0)
var<uniform> screen_and_size: vec3<f32>; // screen.x, screen.y, point_size

// Your list of points, now in a storage buffer
@group(0) @binding(1)
var<storage, read> point_data: array<vec2<f32>>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>, // To draw a circle in the fragment shader
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Deconstruct uniforms
    let screen = vec2<f32>(screen_and_size.x, screen_and_size.y);
    let point_size = screen_and_size.z;

    // Define the 4 corners of a square, which will form two triangles.
    // (0,0), (1,0), (0,1) and (1,0), (1,1), (0,1) in a triangle list.
    let quad_offsets = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );

    // Figure out which point and which corner we're drawing
    let point_index = vertex_index / 6u;
    let corner_index = vertex_index % 6u;

    // Get the corner offset and the center position of the point
    let corner_offset = quad_offsets[corner_index];
    let point_center = point_data[point_index];

    // Calculate the vertex position in pixel space
    let offset = (corner_offset - 0.5) * point_size;
    let final_position = point_center + offset;

    // Convert from pixel space to clip space
    let zero_to_one = final_position / screen;
    let zero_to_two = zero_to_one * 2.0;
    let clip_space = zero_to_two - 1.0;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_space.x, -clip_space.y, 0.0, 1.0);
    out.uv = corner_offset; // Pass the corner offset as UV coordinates
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate the distance from the center of the point (0.5, 0.5)
    let dist = distance(in.uv, vec2(0.5));

    // Discard fragment if it's outside the circle's radius (0.5)
    if (dist > 0.5) {
        discard;
    }

    // Return a solid color for the circle
    return vec4<f32>(0.384313, 0.17254901, 0.65882352, 1.0); // Purple?
}

