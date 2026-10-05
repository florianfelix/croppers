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

    // 1. Normalisierte UV-Koordinaten des aktuellen Threads berechnen
    let uv = (vec2<f32>(id.xy) + 0.5f) / vec2<f32>(size);
    let center = vec2<f32>(uniforms.center_x, uniforms.center_y);

    // 2. Rotation anwenden, um die Distanz zur gedrehten Fokus-Linie zu berechnen
    // Wir berechnen den Normalenvektor der Linie basierend auf dem Winkel
    let n = vec2<f32>(cos(uniforms.angle), sin(uniforms.angle));

    // Projektion des Vektors (uv - center) auf die Normale ergibt den orthogonalen Abstand
    let distance_to_line = abs(dot(uv - center, n));

    // 3. Sicherheitsnetz gegen ungültige UI-Werte
    let safe_focus = clamp(uniforms.focus_size, 0.0f, 1.0f);
    let safe_feather = max(uniforms.feather, 0.001f);

    let edge_start = safe_focus;
    let edge_end = safe_focus + safe_feather;

    // Faktor für die Verpixelung (0.0 = scharf/Original, 1.0 = maximale Kachelgröße)
    let pixelate_factor = smoothstep(edge_start, edge_end, distance_to_line);

    // 4. Dynamische Kachelgröße für diesen Pixel berechnen
    // Wenn der Faktor gegen 0 geht, wird die Pixelgröße 1.0 (Originalauflösung)
    let current_pixel_size = max(1.0f, uniforms.max_pixel_size * pixelate_factor);

    // 5. Mosaik-Effekt anwenden: Koordinaten auf das Kaster-Raster abrunden
    // Wir nehmen die Mitte der aktuellen Mosaik-Kachel als Sample-Punkt
    let pixel_coord = vec2<f32>(id.xy);
    let mosaic_cell = floor(pixel_coord / current_pixel_size);
    let sample_pixel = vec2<u32>(mosaic_cell * current_pixel_size + (current_pixel_size * 0.5f));

    // Absicherung gegen Textur-Randüberschreitung
    let clamped_sample_coord = clamp(sample_pixel, vec2<u32>(0u), size - 1u);

    // 6. Farbe aus der berechneten Kachelmitte lesen
    let final_color = textureLoad(src_tex, vec2<i32>(clamped_sample_coord), 0);

    // In die Zieltextur schreiben
    textureStore(dst_tex, vec2<i32>(id.xy), final_color);
}
