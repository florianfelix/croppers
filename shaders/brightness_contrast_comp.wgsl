struct BrightnessContrastUniforms {
    brightness: f32,
    contrast: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: BrightnessContrastUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);

    // Bounds check
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    let color = textureLoad(src_tex, vec2<i32>(id.xy), 0);

    // Apply brightness offset
    var rgb = color.rgb + vec3<f32>(uniforms.brightness);

    // Apply contrast scaling around midpoint 0.5
    rgb = (rgb - vec3<f32>(0.5)) * uniforms.contrast + vec3<f32>(0.5);

    // Clamp RGB to [0.0, 1.0] while preserving original alpha
    let final_rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));

    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, color.a));
}
