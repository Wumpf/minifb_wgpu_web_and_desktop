@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x * 0.8, y * 0.8, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // 0xFF9001, a light orange with just the right amount of blue.
    return vec4<f32>(1.0, 0.5647, 0.0039, 1.0);
}
