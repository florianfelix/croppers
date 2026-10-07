struct VignetteUniforms {
    intensity: f32,
    radius: f32,
    smoothness: f32,
    scale_y: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: VignetteUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) { return; }

    // Normalize coordinates and sample original pixel color
    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(size);
    let color = textureLoad(src_tex, vec2<i32>(id.xy), 0);

    // Compute distance from center with elliptical aspect scaling
    var delta = uv - vec2<f32>(0.5, 0.5);
    delta.y *= uniforms.scale_y;
    let dist = length(delta);

    // Calculate radial falloff factor using smoothstep between inner and outer radius
    let vignette = smoothstep(uniforms.radius, uniforms.radius - uniforms.smoothness, dist);

    // Blend darkened vignette color with original image based on intensity
    let final_rgb = mix(color.rgb * vignette, color.rgb, 1.0 - uniforms.intensity);

    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, color.a));
}
