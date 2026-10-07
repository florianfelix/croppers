struct GammaUniforms {
    gamma_r: f32,
    gamma_g: f32,
    gamma_b: f32,
    _padding: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: GammaUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);

    // Bounds check: discard invocations outside destination texture
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    // Read input pixel color
    let color = textureLoad(src_tex, vec2<i32>(id.xy), 0);

    // Clamp gamma above zero to prevent division by zero or negative exponents
    let g_r = max(uniforms.gamma_r, 0.001f);
    let g_g = max(uniforms.gamma_g, 0.001f);
    let g_b = max(uniforms.gamma_b, 0.001f);

    // Apply per-channel gamma correction: output = input^(1 / gamma)
    let final_r = pow(max(color.r, 0.0f), 1.0f / g_r);
    let final_g = pow(max(color.g, 0.0f), 1.0f / g_g);
    let final_b = pow(max(color.b, 0.0f), 1.0f / g_b);

    let final_rgb = vec3<f32>(final_r, final_g, final_b);

    // Preserve original alpha and write result to destination texture
    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, color.a));
}
