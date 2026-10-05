struct CrtUniforms {
    curvature: f32,
    scanline_count: f32,
    scanline_strength: f32,
    rgb_mask_strength: f32, // Intensität des Subpixel-Gitters

    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
    _padding4: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: CrtUniforms;

// Hilfsfunktion: Berechnet die tonnenförmige Wölbung (Warp)
fn warp_coordinates(uv: vec2<f32>, b: f32) -> vec2<f32> {
    let dst = uv - 0.5f;
    let r2 = dst.x * dst.x + dst.y * dst.y;
    let warped_dst = dst * (1.0f + b * r2);
    return warped_dst + 0.5f;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    let original_uv = (vec2<f32>(id.xy) + 0.5f) / vec2<f32>(size);

    // 1. Krümmung anwenden
    let warped_uv = warp_coordinates(original_uv, uniforms.curvature);

    // Wenn gewölbte UV außerhalb des Monitors liegt -> Gehäuse (Schwarz) zeichnen
    if (warped_uv.x < 0.0f || warped_uv.x > 1.0f || warped_uv.y < 0.0f || warped_uv.y > 1.0f) {
        textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(0.0f, 0.0f, 0.0f, 1.0f));
        return;
    }

    // 2. Pixel aus der verzerrten Koordinate lesen
    let sample_coord = vec2<i32>(warped_uv * vec2<f32>(size));
    let clamped_coord = clamp(sample_coord, vec2<i32>(0), vec2<i32>(size) - 1i);
    var color = textureLoad(src_tex, clamped_coord, 0);

    // 3. Horizontale Scanlines berechnen
    let scanline_y = sin(warped_uv.y * uniforms.scanline_count * 3.14159265f * 2.0f);
    let scanline_factor = mix(1.0f, (scanline_y * 0.5f + 0.5f), uniforms.scanline_strength);
    var final_rgb = color.rgb * scanline_factor;

    // 4. Vertikales RGB-Subpixel-Gitter (Aperture Grille Mask)
    // Wir nutzen den echten Pixel-Thread-Index auf der X-Achse, um ein rot-grün-blaues Streifenmuster zu erzeugen.
    // Ein volles RGB-Triplett wiederholt sich alle 3 Pixel.
    let pixel_mod = id.x % 3u;
    var mask_modifier = vec3<f32>(1.0f, 1.0f, 1.0f);

    if (pixel_mod == 0u) {
        mask_modifier = vec3<f32>(1.2f, 0.7f, 0.7f); // Roter Subpixel-Fokus
    } else if (pixel_mod == 1u) {
        mask_modifier = vec3<f32>(0.7f, 1.2f, 0.7f); // Grüner Subpixel-Fokus
    } else {
        mask_modifier = vec3<f32>(0.7f, 0.7f, 1.2f); // Blauer Subpixel-Fokus
    }

    // Das Gitter linear basierend auf uniforms.rgb_mask_strength über das Bild blenden
    final_rgb = mix(final_rgb, final_rgb * mask_modifier, uniforms.rgb_mask_strength);

    // Finale Ausgabe in die Zieltextur schreiben
    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, color.a));
}
