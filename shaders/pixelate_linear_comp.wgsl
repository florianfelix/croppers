struct LinearPixelateUniforms {
    max_pixel_size: f32,
    angle: f32,
    start: f32,
    end: f32,
    invert: i32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: LinearPixelateUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    let pixel_coord = vec2<f32>(id.xy);

    // Snap thread coordinate to coarse base grid so entire block shares gradient distance
    let base_grid_size = max(1.0f, uniforms.max_pixel_size);
    let aligned_cell = floor(pixel_coord / base_grid_size);
    let aligned_pixel = aligned_cell * base_grid_size + (base_grid_size * 0.5f);

    // Project grid position onto linear axis rotating around center (0.5, 0.5)
    let uv = vec2<f32>(aligned_pixel.x / f32(size.x), 1.0 - (aligned_pixel.y / f32(size.y)));
    let centered_uv = uv - vec2<f32>(0.5, 0.5);
    let dir = vec2<f32>(cos(uniforms.angle), sin(uniforms.angle));
    let pos = dot(centered_uv, dir) + 0.5;

    let span = max(uniforms.end - uniforms.start, 0.0001);
    var factor = clamp((pos - uniforms.start) / span, 0.0f, 1.0f);
    if (uniforms.invert == 1) {
        factor = 1.0f - factor;
    }

    // Determine target pixel size for this block (1.0 = clear, base_grid_size = full pixelate)
    let target_size = mix(1.0f, base_grid_size, factor);

    // Quantize pixel size to power-of-two steps so smaller sub-blocks tile cleanly
    var current_pixel_size: f32 = 1.0f;
    if (target_size >= 96.0f) {
        current_pixel_size = 128.0f;
    } else if (target_size >= 48.0f) {
        current_pixel_size = 64.0f;
    } else if (target_size >= 24.0f) {
        current_pixel_size = 32.0f;
    } else if (target_size >= 12.0f) {
        current_pixel_size = 16.0f;
    } else if (target_size >= 6.0f) {
        current_pixel_size = 8.0f;
    } else if (target_size >= 3.0f) {
        current_pixel_size = 4.0f;
    } else if (target_size >= 1.5f) {
        current_pixel_size = 2.0f;
    } else {
        current_pixel_size = 1.0f;
    }

    // Sample color at center of quantized mosaic tile
    let mosaic_cell = floor(pixel_coord / current_pixel_size);
    let sample_pixel = vec2<u32>(mosaic_cell * current_pixel_size + (current_pixel_size * 0.5f));
    let clamped_sample_coord = clamp(sample_pixel, vec2<u32>(0u), size - 1u);

    let final_color = textureLoad(src_tex, vec2<i32>(clamped_sample_coord), 0);

    textureStore(dst_tex, vec2<i32>(id.xy), final_color);
}
