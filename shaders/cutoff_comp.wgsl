struct CutoffUniforms {
    cutoff_below: f32,
    cutoff_above: f32,
    falloff: f32,
    reintroduce: f32,
    angle: f32,
    below_enabled: u32,
    above_enabled: u32,
    mode: u32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: CutoffUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    let color = textureLoad(src_tex, vec2<i32>(id.xy), 0);

    // Determine reference value according to mode
    var val: f32;
    if (uniforms.mode == 1u) {
        // Linear spatial mode with angle
        // Center uv at (0.5, 0.5) with Y inverted so 0 is bottom and 1 is top
        let uv = vec2<f32>(f32(id.x) / f32(size.x), 1.0 - (f32(id.y) / f32(size.y)));
        let centered_uv = uv - vec2<f32>(0.5, 0.5);
        let dir = vec2<f32>(cos(uniforms.angle), sin(uniforms.angle));
        val = dot(centered_uv, dir) + 0.5;
    } else {
        // Luminance mode (ITU-R BT.709 standard)
        val = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    }

    var keep: f32 = 1.0;
    let f = max(uniforms.falloff, 0.0001);

    if (uniforms.below_enabled == 1u) {
        // Below cutoff threshold fades to outside/black
        let factor = smoothstep(uniforms.cutoff_below - f, uniforms.cutoff_below, val);
        keep = min(keep, factor);
    }

    if (uniforms.above_enabled == 1u) {
        // Above cutoff threshold fades to outside/black
        let factor = 1.0 - smoothstep(uniforms.cutoff_above, uniforms.cutoff_above + f, val);
        keep = min(keep, factor);
    }

    // Reintroduce original image outside the cutoff instead of pure black
    let outside_color = mix(vec3<f32>(0.0), color.rgb, uniforms.reintroduce);
    let final_rgb = mix(outside_color, color.rgb, keep);

    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, color.a));
}
