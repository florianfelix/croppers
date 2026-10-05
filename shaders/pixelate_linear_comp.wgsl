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

    // 1. HARD GRID ALIGNMENT (Die wichtigste Änderung)
    // Wir runden die Thread-Koordinate auf das gröbste Raster (max_pixel_size) ab.
    // Dadurch wird die Verlaufsposition für einen kompletten Block vereinheitlicht.
    let base_grid_size = max(1.0f, uniforms.max_pixel_size);
    let aligned_cell = floor(pixel_coord / base_grid_size);
    let aligned_pixel = aligned_cell * base_grid_size + (base_grid_size * 0.5f);

    // 2. Verlaufstest mit der am Raster ausgerichteten Koordinate durchführen
    let uv = aligned_pixel / vec2<f32>(size);
    let distance_along_path = dot(uv - start_point, dir);

    var factor = clamp(distance_along_path / safe_length, 0.0f, 1.0f);
    if (uniforms.invert == 1) {
        factor = 1.0f - factor;
    }

    // 3. Ziel-Pixelgröße für diesen gesamten Block ermitteln
    let target_size = mix(base_grid_size, 1.0f, factor);

    // 4. BUCKET-SELEKTION (Power of Two)
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

    // 5. Mosaik-Kachelung anwenden
    // Da current_pixel_size ein sauberer Teiler oder ein Vielfaches von base_grid_size ist,
    // rasten die kleineren Pixel perfekt und nahtlos innerhalb des übergeordneten Gitters ein.
    let mosaic_cell = floor(pixel_coord / current_pixel_size);
    let sample_pixel = vec2<u32>(mosaic_cell * current_pixel_size + (current_pixel_size * 0.5f));

    let clamped_sample_coord = clamp(sample_pixel, vec2<u32>(0u), size - 1u);

    // 6. Farbe auslesen und speichern
    let final_color = textureLoad(src_tex, vec2<i32>(clamped_sample_coord), 0);
    textureStore(dst_tex, vec2<i32>(id.xy), final_color);
}
