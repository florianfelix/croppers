struct HsvUniforms {
    hue: f32,          // 0.5 = Keine Änderung, < 0.5 rotierte Hues, > 0.5 rotierte Hues
    saturation: f32,   // 1.0 = Keine Änderung, 0.0 = S/W, > 1.0 = Übersättigt
    value: f32,        // 1.0 = Keine Änderung, 0.0 = Schwarz, > 1.0 = Aufgehellt
    factor: f32,       // 0.0 = Nur Originalbild, 1.0 = Voller HSV-Effekt
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: HsvUniforms;

// Hilfsfunktion: Konvertiert RGB zu HSV
fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let k = vec4<f32>(0.0f, -1.0f / 3.0f, 2.0f / 3.0f, -1.0f);

    // Vergleiche Kanäle, um Maxima und Minima zu bestimmen (Epsilon-sicher)
    let p = mix(vec4<f32>(rgb.bg, k.wz), vec4<f32>(rgb.gb, k.xy), step(rgb.b, rgb.g));
    let q = mix(vec4<f32>(p.xyw, rgb.r), vec4<f32>(rgb.r, p.yzx), step(p.x, rgb.r));

    let d = q.x - min(q.w, q.y);
    let e = 1.0e-10f;

    return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0f * d + e)), d / (q.x + e), q.x);
}

// Hilfsfunktion: Konvertiert HSV zurück zu RGB
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let k = vec4<f32>(1.0f, 2.0f / 3.0f, 1.0f / 3.0f, 3.0f);
    let p = abs(fract(hsv.xxx + k.xyz) * 6.0f - k.www);
    return hsv.z * mix(k.xxx, clamp(p - k.xxx, vec3<f32>(0.0f), vec3<f32>(1.0f)), hsv.y);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);

    // Validierung der Thread-Grenzen
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    // Originalfarbe einlesen
    let original_color = textureLoad(src_tex, vec2<i32>(id.xy), 0);

    // 1. In den HSV Farbraum wechseln
    var hsv = rgb_to_hsv(original_color.rgb);

    // 2. Modifikationen im HSV-Raum anwenden (Blender-Logik):

    // Hue-Shift: Blender nutzt 0.5 als Standardwert.
    // Der Shift berechnet sich aus (uniforms.hue - 0.5). fract() hält das Ergebnis im Kreis [0.0, 1.0]
    hsv.x = fract(hsv.x + (uniforms.hue - 0.5f));

    // Saturation & Value Modifikation via Multiplikation
    hsv.y = clamp(hsv.y * uniforms.saturation, 0.0f, 1.0f);
    hsv.z = max(hsv.z * uniforms.value, 0.0f); // Value kann theoretisch über 1.0 steigen (HDR/Brightening)

    // 3. Zurück in den RGB Farbraum konvertieren
    let hsv_modified_rgb = hsv_to_rgb(hsv);

    // 4. Blend-Factor anwenden (Blenders Mix-Faktor am Node-Ausgang)
    let final_rgb = mix(original_color.rgb, hsv_modified_rgb, uniforms.factor);

    // Alpha-Kanal beibehalten und in Zieltextur schreiben
    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, original_color.a));
}
