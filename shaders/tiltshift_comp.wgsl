struct TiltShiftUniforms {
    max_radius: f32,
    dir_x: i32,
    dir_y: i32,
    center: f32,     // 0.0 bis 1.0 (Top-Down Position des scharfen Streifens)
    focus_size: f32, // Breite der Zone, die komplett scharf bleibt
    feather: f32,    // Radius/Breite der Weichzeichnungskante (Übergangszone)
    _padding1: f32,
    _padding2: f32,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: TiltShiftUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    if (global_id.x >= dims.x || global_id.y >= dims.y) {
        return;
    }

    let coords = vec2<i32>(global_id.xy);

    // 1. Normalisierte Y-Koordinate des aktuellen Pixels berechnen (0.0 bis 1.0)
    let uv_y = (f32(global_id.y) + 0.5f) / f32(dims.y);

    // 2. Absoluten Abstand zum Fokus-Zentrum ermitteln
    let distance_to_center = abs(uv_y - uniforms.center);

    // 3. Sicherheitsnetz gegen Werte außerhalb von [0.0, 1.0] und Division durch 0
    // focus_size wird auf den maximalen UV-Bereich gedeckelt
    let safe_focus = clamp(uniforms.focus_size, 0.0f, 1.0f);

    // feather darf nicht negativ sein; wir erzwingen zudem eine minimale Breite
    let safe_feather = max(uniforms.feather, 0.001f);

    let edge_start = safe_focus;
    let edge_end = safe_focus + safe_feather;

    // Dynamischen Unschärfe-Faktor (0.0 bis 1.0) berechnen
    let blur_factor = smoothstep(edge_start, edge_end, distance_to_center);


    // 4. Den finalen Radius für DIESEN spezifischen Pixel skalieren
    let dynamic_radius = i32(uniforms.max_radius * blur_factor);

    // Falls der berechnete Radius für diesen Pixel 0 ist, das Originalpixel direkt kopieren
    if (dynamic_radius <= 0) {
        let color = textureLoad(input_texture, coords, 0);
        textureStore(output_texture, coords, color);
        return;
    }

    // 5. Klassische 1D-Akkumulation entlang der Schrittrichtung (Ihre bestehende Logik)
    var color_acc = vec4<f32>(0.0f);
    var count = 0.0f;
    let step_dir = vec2<i32>(uniforms.dir_x, uniforms.dir_y);

    for (var i = -dynamic_radius; i <= dynamic_radius; i++) {
        let sample_pos = coords + (step_dir * i);

        // Boundaries-Check
        if (sample_pos.x >= 0 && sample_pos.x < i32(dims.x) && sample_pos.y >= 0 && sample_pos.y < i32(dims.y)) {
            color_acc += textureLoad(input_texture, sample_pos, 0);
            count += 1.0f;
        }
    }

    // Ergebnis in den Ping-Pong-Buffer schreiben
    textureStore(output_texture, coords, color_acc / count);
}
