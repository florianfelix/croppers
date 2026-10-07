struct HsvUniforms {
    hue: f32,          // 0.5 = neutral, < 0.5 rotates hue backwards, > 0.5 rotates hue forward
    saturation: f32,   // 1.0 = neutral, 0.0 = greyscale, > 1.0 = oversaturated
    value: f32,        // 1.0 = neutral, 0.0 = black, > 1.0 = brightened
    factor: f32,       // 0.0 = original image only, 1.0 = full HSV adjustment
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: HsvUniforms;

// Converts an RGB color to HSV color space.
fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let k = vec4<f32>(0.0f, -1.0f / 3.0f, 2.0f / 3.0f, -1.0f);

    // Channel comparison to determine max and min components with epsilon safety.
    let p = mix(vec4<f32>(rgb.bg, k.wz), vec4<f32>(rgb.gb, k.xy), step(rgb.b, rgb.g));
    let q = mix(vec4<f32>(p.xyw, rgb.r), vec4<f32>(rgb.r, p.yzx), step(p.x, rgb.r));

    let d = q.x - min(q.w, q.y);
    let e = 1.0e-10f;

    return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0f * d + e)), d / (q.x + e), q.x);
}

// Converts an HSV color back to RGB color space.
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let k = vec4<f32>(1.0f, 2.0f / 3.0f, 1.0f / 3.0f, 3.0f);
    let p = abs(fract(hsv.xxx + k.xyz) * 6.0f - k.www);
    return hsv.z * mix(k.xxx, clamp(p - k.xxx, vec3<f32>(0.0f), vec3<f32>(1.0f)), hsv.y);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);

    // Bounds check to discard invocations outside destination texture.
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    // Load original color and convert RGB to HSV color space.
    let original_color = textureLoad(src_tex, vec2<i32>(id.xy), 0);
    var hsv = rgb_to_hsv(original_color.rgb);

    // Apply Blender-style hue rotation where 0.5 is neutral and fract() keeps hue in [0.0, 1.0].
    hsv.x = fract(hsv.x + (uniforms.hue - 0.5f));

    // Scale saturation and value (brightness).
    hsv.y = clamp(hsv.y * uniforms.saturation, 0.0f, 1.0f);
    hsv.z = max(hsv.z * uniforms.value, 0.0f);

    // Convert modified HSV back to RGB and blend with original color using mix factor.
    let hsv_modified_rgb = hsv_to_rgb(hsv);
    let final_rgb = mix(original_color.rgb, hsv_modified_rgb, uniforms.factor);

    // Preserve original alpha and store resulting pixel.
    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, original_color.a));
}
