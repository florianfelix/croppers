struct LinearPixelateUniforms {
    max_pixel_size: f32,
    angle: f32,
    start_x: f32,
    start_y: f32,
    length: f32,
    invert: i32,
    _padding1: f32,
    _padding2: f32,
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
    let start_point = vec2<f32>(uniforms.start_x, uniforms.start_y);
    let dir = vec2<f32>(cos(uniforms.angle), sin(uniforms.angle));
    let safe_length = max(uniforms.length, 0.001f);

    // Snap thread coordinate to coarse base grid so entire block shares gradient distance
    let base_grid_size = max(1.0f, uniforms.max_pixel_size);
    let aligned_cell = floor(pixel_coord / base_grid_size);
    let aligned_pixel = aligned_cell * base_grid_size + (base_grid_size * 0.5f);

    // Project grid position onto linear axis to compute transition factor [0.0, 1.0]
    let uv = aligned_pixel / vec2<f32>(size);
    let distance_along_path = dot(uv - start_point, dir);

    var factor = clamp(distance_along_path / safe_length, 0.0f, 1.0f);
    if (uniforms.invert == 1) {
        factor = 1.0f - factor;
    }

    // Determine target pixel size for this block
    let target_size = mix(base_grid_size, 1.0f, factor);

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
