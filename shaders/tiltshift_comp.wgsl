struct TiltShiftUniforms {
    max_radius: f32,
    dir_x: i32,
    dir_y: i32,
    center: f32,     // 0.0 to 1.0 vertical position of in-focus band
    focus_size: f32, // Width of in-focus zone
    feather: f32,    // Transition width between sharp and blurred zones
    _padding1: f32,
    _padding2: f32,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: TiltShiftUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    if (global_id.x >= dims.x || global_id.y >= dims.y) {
        return;
    }

    let coords = vec2<i32>(global_id.xy);

    // Calculate normalized vertical distance from current pixel to center focus band
    let uv_y = (f32(global_id.y) + 0.5f) / f32(dims.y);
    let distance_to_center = abs(uv_y - uniforms.center);

    // Compute dynamic blur factor using smoothstep across focus size and feather zone
    let safe_focus = clamp(uniforms.focus_size, 0.0f, 1.0f);
    let safe_feather = max(uniforms.feather, 0.001f);
    let edge_start = safe_focus;
    let edge_end = safe_focus + safe_feather;
    let blur_factor = smoothstep(edge_start, edge_end, distance_to_center);

    // Scale blur radius dynamically for this pixel; pass through unchanged if zero
    let dynamic_radius = i32(uniforms.max_radius * blur_factor);
    if (dynamic_radius <= 0) {
        let color = textureLoad(input_texture, coords, 0);
        textureStore(output_texture, coords, color);
        return;
    }

    // Accumulate neighboring pixel samples along the 1D step direction
    var color_acc = vec4<f32>(0.0f);
    var count = 0.0f;
    let step_dir = vec2<i32>(uniforms.dir_x, uniforms.dir_y);

    for (var i = -dynamic_radius; i <= dynamic_radius; i++) {
        let sample_pos = coords + (step_dir * i);

        if (sample_pos.x >= 0 && sample_pos.x < i32(dims.x) && sample_pos.y >= 0 && sample_pos.y < i32(dims.y)) {
            color_acc += textureLoad(input_texture, sample_pos, 0);
            count += 1.0f;
        }
    }

    // Write normalized averaged color to output texture
    textureStore(output_texture, coords, color_acc / count);
}
