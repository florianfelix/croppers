@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

struct CanvasUniforms {
    image_bounds: vec4<f32>, // min_x, min_y, max_x, max_y in viewport/screen pixels
    bg_color: vec4<f32>,
    shadow_color: vec4<f32>,
    radius: f32,
    shadow_offset: f32,
    _padding2: f32,
    _padding3: f32,
};

@group(0) @binding(2) var<uniform> uniforms: CanvasUniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>, // ignored
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) dummy: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.dummy = vec2<f32>(0.0);
    return out;
}

// Distance from a point p to a rect [b.xy, b.zw]
fn sdRect(p: vec2<f32>, b: vec4<f32>) -> f32 {
    let d = max(b.xy - p, p - b.zw);
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = in.clip_position.xy;
    let bounds = uniforms.image_bounds;

    // Check if inside image
    if (px.x >= bounds.x && px.x <= bounds.z && px.y >= bounds.y && px.y <= bounds.w) {
        let uv_x = (px.x - bounds.x) / (bounds.z - bounds.x);
        let uv_y = (px.y - bounds.y) / (bounds.w - bounds.y);
        return textureSample(t_diffuse, s_diffuse, vec2<f32>(uv_x, uv_y));
    }

    // Outside image: background + shadow
    // Let's compute a simple soft drop shadow.
    // Shift the bounds downwards for the shadow offset
    let shadow_offset = vec2<f32>(0.0, uniforms.shadow_offset);
    let shadow_bounds = vec4<f32>(bounds.xy + shadow_offset, bounds.zw + shadow_offset);

    let dist = sdRect(px, shadow_bounds);
    let blur_radius = uniforms.radius;

    // If distance is less than blur radius, compute shadow alpha
    var shadow_alpha = 0.0;
    if (dist <= blur_radius) {
        // Simple exponential/smooth drop off
        shadow_alpha = (1.0 - smoothstep(0.0, blur_radius, dist)) * uniforms.shadow_color.a;
    }

    // Blend background with shadow
    let final_color = mix(uniforms.bg_color.rgb, uniforms.shadow_color.rgb, shadow_alpha);
    return vec4<f32>(final_color, 1.0);
}
