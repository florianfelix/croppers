struct PixelateTiltUniforms {
    max_pixel_size: f32,
    angle: f32,
    center_x: f32,
    center_y: f32,

    focus_size: f32,
    feather: f32,
    _padding1: f32,
    _padding2: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: PixelateTiltUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    let pixel_coord = vec2<f32>(id.xy);
    let center = vec2<f32>(uniforms.center_x, uniforms.center_y);

    // Korrigierter Richtungsvektor ohne den unbekannten Bezeichner
    let n_vector = vec2<f32>(cos(uniforms.angle), sin(uniforms.angle));

    // 1. HARD GRID ALIGNMENT
    // Wir runden die Thread-Koordinate auf das gröbste Raster ab.
    // Das zwingt die gesamte Kachel, dieselbe Distanz zum Fokus-Streifen zu berechnen.
    let base_grid_size = max(1.0f, uniforms.max_pixel_size);
    let aligned_cell = floor(pixel_coord / base_grid_size);
    let aligned_pixel = aligned_cell * base_grid_size + (base_grid_size * 0.5f);

    // 2. Normalisierte UV-Koordinaten des ausgerichteten Rasters berechnen
    let uv = aligned_pixel / vec2<f32>(size);

    // 3. Orthogonalen Abstand zur rotierten Fokus-Linie berechnen
    let distance_to_line = abs(dot(uv - center, n_vector));

    // 4. Sicherheitsnetz für den Smoothstep Übergang
    let safe_focus = clamp(uniforms.focus_size, 0.0f, 1.0f);
    let safe_feather = max(uniforms.feather, 0.001f);

    let edge_start = safe_focus;
    let edge_end = safe_focus + safe_feather;

    // Faktor berechnen (0.0 = im Fokus/scharf, 1.0 = maximale Verpixelung)
    let pixelate_factor = smoothstep(edge_start, edge_end, distance_to_line);

    // 5. Kontinuierliche Ziel-Pixelgröße bestimmen
    let target_size = mix(1.0f, base_grid_size, pixelate_factor);

    // 6. BUCKET-SELEKTION (Feste Power-of-Two Stufen)
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
        current_pixel_size = 1.0f; // Gestochen scharf im Fokuszentrum
    }

    // 7. Mosaik-Kachelung anwenden
    let mosaic_cell = floor(pixel_coord / current_pixel_size);
    let sample_pixel = vec2<u32>(mosaic_cell * current_pixel_size + (current_pixel_size * 0.5f));

    let clamped_sample_coord = clamp(sample_pixel, vec2<u32>(0u), size - 1u);

    // 8. Farbe auslesen und speichern
    let final_color = textureLoad(src_tex, vec2<i32>(clamped_sample_coord), 0);
    textureStore(dst_tex, vec2<i32>(id.xy), final_color);
}
