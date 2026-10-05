struct BwUniforms {
    select: i32,
    _padding1: i32,
    _padding2: i32,
    _padding3: i32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: BwUniforms;

// Konstante für Standard-Helligkeitsberechnung (falls kein einzelner Kanal gewählt ist)
const LUMINANCE_VECTOR = vec3<f32>(0.2126f, 0.7152f, 0.0722f);

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);

    // Validierung der Thread-Grenzen gegen Texturgröße
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    // Farbe vom Quellpixel einlesen
    let color = textureLoad(src_tex, vec2<i32>(id.xy), 0);

    var grey: f32;

    // Kanal-Auswahl basierend auf dem Uniform-Wert
    switch (uniforms.select) {
        case 0: {
            grey = color.r; // R-Kanal
        }
        case 1: {
            grey = color.g; // G-Kanal
        }
        case 2: {
            grey = color.b; // B-Kanal
        }
        default: {
            // Fallback: Standard Graustufen-Konvertierung (Luminanz)
            grey = dot(color.rgb, LUMINANCE_VECTOR);
        }
    }

    // Der Grauwert wird auf R, G und B angewendet, während Alpha erhalten bleibt
    let final_rgb = vec3<f32>(grey);

    // In die Zieltextur für die Pipeline schreiben
    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, color.a));
}
