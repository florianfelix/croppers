// Ikonische 4x4 Bayer-Matrix
const BAYER_4X4 = mat4x4<f32>(
    vec4<f32>(0.0f / 16.0f,  8.0f / 16.0f,  2.0f / 16.0f, 10.0f / 16.0f),
    vec4<f32>(12.0f / 16.0f,  4.0f / 16.0f, 14.0f / 16.0f,  6.0f / 16.0f),
    vec4<f32>(3.0f / 16.0f, 11.0f / 16.0f,  1.0f / 16.0f,  9.0f / 16.0f),
    vec4<f32>(15.0f / 16.0f,  7.0f / 16.0f, 13.0f / 16.0f,  5.0f / 16.0f)
);

// Feine 8x8 Bayer-Matrix (Aufgeteilt in ein flaches Array für kompromissloses Alignment)
const BAYER_8X8 = array<f32, 64>(
     0.0f/64.0f, 32.0f/64.0f,  8.0f/64.0f, 40.0f/64.0f,  2.0f/64.0f, 34.0f/64.0f, 10.0f/64.0f, 42.0f/64.0f,
    48.0f/64.0f, 16.0f/64.0f, 56.0f/64.0f, 24.0f/64.0f, 50.0f/64.0f, 18.0f/64.0f, 58.0f/64.0f, 26.0f/64.0f,
    12.0f/64.0f, 44.0f/64.0f,  4.0f/64.0f, 36.0f/64.0f, 14.0f/64.0f, 46.0f/64.0f,  6.0f/64.0f, 38.0f/64.0f,
    60.0f/64.0f, 28.0f/64.0f, 52.0f/64.0f, 20.0f/64.0f, 62.0f/64.0f, 30.0f/64.0f, 54.0f/64.0f, 22.0f/64.0f,
     3.0f/64.0f, 35.0f/64.0f, 11.0f/64.0f, 43.0f/64.0f,  1.0f/64.0f, 33.0f/64.0f,  9.0f/64.0f, 41.0f/64.0f,
    51.0f/64.0f, 19.0f/64.0f, 59.0f/64.0f, 27.0f/64.0f, 49.0f/64.0f, 17.0f/64.0f, 57.0f/64.0f, 25.0f/64.0f,
    15.0f/64.0f, 47.0f/64.0f,  7.0f/64.0f, 39.0f/64.0f, 13.0f/64.0f, 45.0f/64.0f,  5.0f/64.0f, 37.0f/64.0f,
    63.0f/64.0f, 31.0f/64.0f, 55.0f/64.0f, 23.0f/64.0f, 61.0f/64.0f, 29.0f/64.0f, 53.0f/64.0f, 21.0f/64.0f
);

// --- PALETTEN DEFINITIONEN ---
const PAL_GAMEBOY = array<vec3<f32>, 4>(
    vec3<f32>(0.612f, 0.737f, 0.059f), vec3<f32>(0.545f, 0.675f, 0.059f),
    vec3<f32>(0.188f, 0.384f, 0.188f), vec3<f32>(0.059f, 0.220f, 0.059f)
);
const PAL_CGA = array<vec3<f32>, 4>(
    vec3<f32>(0.000f, 0.000f, 0.000f), vec3<f32>(0.333f, 1.000f, 1.000f),
    vec3<f32>(1.000f, 0.333f, 1.000f), vec3<f32>(1.000f, 1.000f, 1.000f)
);
const PAL_PICO8 = array<vec3<f32>, 16>(
    vec3<f32>(0.000f, 0.000f, 0.000f), vec3<f32>(0.114f, 0.169f, 0.325f), vec3<f32>(0.494f, 0.145f, 0.325f), vec3<f32>(0.000f, 0.522f, 0.341f),
    vec3<f32>(0.671f, 0.322f, 0.212f), vec3<f32>(0.373f, 0.341f, 0.306f), vec3<f32>(0.761f, 0.765f, 0.780f), vec3<f32>(1.000f, 0.925f, 0.843f),
    vec3<f32>(1.000f, 0.000f, 0.302f), vec3<f32>(1.000f, 0.639f, 0.000f), vec3<f32>(1.000f, 0.925f, 0.000f), vec3<f32>(0.000f, 0.894f, 0.212f),
    vec3<f32>(0.114f, 0.675f, 1.000f), vec3<f32>(0.514f, 0.463f, 0.612f), vec3<f32>(1.000f, 0.467f, 0.659f), vec3<f32>(1.000f, 0.804f, 0.678f)
);
const PAL_C64 = array<vec3<f32>, 16>(
    vec3<f32>(0.000f, 0.000f, 0.000f), vec3<f32>(1.000f, 1.000f, 1.000f), vec3<f32>(0.533f, 0.200f, 0.133f), vec3<f32>(0.467f, 0.733f, 0.800f),
    vec3<f32>(0.533f, 0.267f, 0.600f), vec3<f32>(0.400f, 0.667f, 0.267f), vec3<f32>(0.133f, 0.133f, 0.667f), vec3<f32>(0.867f, 0.933f, 0.467f),
    vec3<f32>(0.533f, 0.400f, 0.000f), vec3<f32>(0.333f, 0.267f, 0.000f), vec3<f32>(0.800f, 0.467f, 0.400f), vec3<f32>(0.333f, 0.333f, 0.333f),
    vec3<f32>(0.467f, 0.467f, 0.467f), vec3<f32>(0.667f, 1.000f, 0.467f), vec3<f32>(0.667f, 0.667f, 1.000f), vec3<f32>(0.667f, 0.667f, 0.667f)
);
const PAL_NES = array<vec3<f32>, 16>(
    vec3<f32>(0.000f, 0.000f, 0.000f), vec3<f32>(0.992f, 0.992f, 0.992f), vec3<f32>(0.741f, 0.000f, 0.000f), vec3<f32>(0.000f, 0.643f, 0.898f),
    vec3<f32>(0.000f, 0.000f, 0.737f), vec3<f32>(0.000f, 0.533f, 0.000f), vec3<f32>(0.592f, 0.000f, 0.584f), vec3<f32>(0.925f, 0.710f, 0.000f),
    vec3<f32>(0.965f, 0.439f, 0.024f), vec3<f32>(0.443f, 0.259f, 0.000f), vec3<f32>(0.969f, 0.475f, 0.467f), vec3<f32>(0.329f, 0.329f, 0.329f),
    vec3<f32>(0.655f, 0.655f, 0.655f), vec3<f32>(0.000f, 0.902f, 0.000f), vec3<f32>(0.384f, 0.541f, 0.992f), vec3<f32>(0.455f, 0.000f, 0.208f)
);
const PAL_CATPPUCCIN = array<vec3<f32>, 16>(
    vec3<f32>(0.118f, 0.118f, 0.180f), vec3<f32>(0.937f, 0.890f, 0.933f), vec3<f32>(0.965f, 0.608f, 0.667f), vec3<f32>(0.651f, 0.890f, 0.631f),
    vec3<f32>(0.976f, 0.812f, 0.616f), vec3<f32>(0.541f, 0.706f, 0.973f), vec3<f32>(0.796f, 0.651f, 0.969f), vec3<f32>(0.561f, 0.863f, 0.843f),
    vec3<f32>(0.690f, 0.710f, 0.776f), vec3<f32>(0.427f, 0.435f, 0.529f), vec3<f32>(0.902f, 0.545f, 0.408f), vec3<f32>(0.149f, 0.149f, 0.227f),
    vec3<f32>(0.180f, 0.180f, 0.259f), vec3<f32>(0.227f, 0.227f, 0.310f), vec3<f32>(0.345f, 0.357f, 0.443f), vec3<f32>(0.490f, 0.502f, 0.592f)
);
const PAL_TOKYONIGHT = array<vec3<f32>, 16>(
    vec3<f32>(0.094f, 0.102f, 0.149f), vec3<f32>(0.780f, 0.816f, 0.933f), vec3<f32>(0.980f, 0.443f, 0.533f), vec3<f32>(0.478f, 0.784f, 0.420f),
    vec3<f32>(0.878f, 0.745f, 0.439f), vec3<f32>(0.482f, 0.655f, 0.984f), vec3<f32>(0.733f, 0.557f, 0.984f), vec3<f32>(0.463f, 0.839f, 0.871f),
    vec3<f32>(0.643f, 0.698f, 0.882f), vec3<f32>(0.337f, 0.376f, 0.529f), vec3<f32>(1.000f, 0.612f, 0.408f), vec3<f32>(0.125f, 0.133f, 0.192f),
    vec3<f32>(0.165f, 0.176f, 0.243f), vec3<f32>(0.204f, 0.220f, 0.318f), vec3<f32>(0.267f, 0.290f, 0.412f), vec3<f32>(0.447f, 0.494f, 0.675f)
);
const PAL_GRUVBOX = array<vec3<f32>, 16>(
    vec3<f32>(0.157f, 0.157f, 0.157f), vec3<f32>(0.925f, 0.859f, 0.698f), vec3<f32>(0.804f, 0.141f, 0.114f), vec3<f32>(0.596f, 0.635f, 0.118f),
    vec3<f32>(0.843f, 0.600f, 0.129f), vec3<f32>(0.271f, 0.522f, 0.533f), vec3<f32>(0.694f, 0.384f, 0.525f), vec3<f32>(0.424f, 0.612f, 0.443f),
    vec3<f32>(0.651f, 0.600f, 0.518f), vec3<f32>(0.557f, 0.494f, 0.443f), vec3<f32>(0.984f, 0.286f, 0.204f), vec3<f32>(0.722f, 0.722f, 0.149f),
    vec3<f32>(0.980f, 0.741f, 0.184f), vec3<f32>(0.522f, 0.718f, 0.718f), vec3<f32>(0.827f, 0.533f, 0.706f), vec3<f32>(0.667f, 0.812f, 0.557f)
);
// Scheme 1: Warm Muted / Rosé Pine Dawn style (16 colors)
const PAL_WARM_MUTED = array<vec3<f32>, 16>(
    // Normal: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.173f, 0.173f, 0.188f), vec3<f32>(0.663f, 0.439f, 0.439f), vec3<f32>(0.467f, 0.549f, 0.451f), vec3<f32>(0.800f, 0.569f, 0.400f),
    vec3<f32>(0.424f, 0.498f, 0.576f), vec3<f32>(0.722f, 0.580f, 0.686f), vec3<f32>(0.557f, 0.643f, 0.635f), vec3<f32>(0.686f, 0.655f, 0.616f),
    // Bright: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.647f, 0.647f, 0.612f), vec3<f32>(0.737f, 0.561f, 0.561f), vec3<f32>(0.573f, 0.639f, 0.561f), vec3<f32>(0.851f, 0.675f, 0.549f),
    vec3<f32>(0.541f, 0.600f, 0.659f), vec3<f32>(0.800f, 0.702f, 0.776f), vec3<f32>(0.667f, 0.733f, 0.729f), vec3<f32>(0.776f, 0.753f, 0.725f)
);

// Scheme 2: Earth / Forest Vintage (16 colors)
const PAL_EARTH_VINTAGE = array<vec3<f32>, 16>(
    // Normal: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.000f, 0.000f, 0.000f), vec3<f32>(0.408f, 0.341f, 0.259f), vec3<f32>(0.373f, 0.529f, 0.373f), vec3<f32>(0.702f, 0.427f, 0.263f),
    vec3<f32>(0.471f, 0.510f, 0.294f), vec3<f32>(0.733f, 0.467f, 0.267f), vec3<f32>(0.788f, 0.647f, 0.329f), vec3<f32>(0.843f, 0.769f, 0.514f),
    // Bright: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.400f, 0.400f, 0.400f), vec3<f32>(0.408f, 0.341f, 0.259f), vec3<f32>(0.373f, 0.529f, 0.373f), vec3<f32>(0.702f, 0.427f, 0.263f),
    vec3<f32>(0.471f, 0.510f, 0.294f), vec3<f32>(0.733f, 0.467f, 0.267f), vec3<f32>(0.788f, 0.647f, 0.329f), vec3<f32>(0.843f, 0.769f, 0.514f)
);

// Scheme 3: Everforest Dark (16 colors)
const PAL_EVERFOREST = array<vec3<f32>, 16>(
    // Normal: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.204f, 0.247f, 0.267f), vec3<f32>(0.902f, 0.494f, 0.502f), vec3<f32>(0.655f, 0.753f, 0.502f), vec3<f32>(0.859f, 0.737f, 0.498f),
    vec3<f32>(0.498f, 0.733f, 0.702f), vec3<f32>(0.839f, 0.600f, 0.714f), vec3<f32>(0.514f, 0.753f, 0.573f), vec3<f32>(0.827f, 0.776f, 0.667f),
    // Bright: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.522f, 0.573f, 0.537f), vec3<f32>(0.902f, 0.494f, 0.502f), vec3<f32>(0.655f, 0.753f, 0.502f), vec3<f32>(0.859f, 0.737f, 0.498f),
    vec3<f32>(0.498f, 0.733f, 0.702f), vec3<f32>(0.839f, 0.600f, 0.714f), vec3<f32>(0.514f, 0.753f, 0.573f), vec3<f32>(0.827f, 0.776f, 0.667f)
);

// Scheme 4: Deep Ocean / Teal (16 colors)
const PAL_DEEP_OCEAN = array<vec3<f32>, 16>(
    // Normal: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.047f, 0.063f, 0.078f), vec3<f32>(0.761f, 0.192f, 0.153f), vec3<f32>(0.165f, 0.659f, 0.537f), vec3<f32>(0.929f, 0.706f, 0.263f),
    vec3<f32>(0.098f, 0.329f, 0.400f), vec3<f32>(0.306f, 0.318f, 0.400f), vec3<f32>(0.200f, 0.522f, 0.620f), vec3<f32>(0.600f, 0.820f, 0.808f),
    // Bright: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.047f, 0.063f, 0.078f), vec3<f32>(0.761f, 0.192f, 0.153f), vec3<f32>(0.165f, 0.659f, 0.537f), vec3<f32>(0.929f, 0.706f, 0.263f),
    vec3<f32>(0.098f, 0.329f, 0.400f), vec3<f32>(0.306f, 0.318f, 0.400f), vec3<f32>(0.200f, 0.522f, 0.620f), vec3<f32>(0.600f, 0.820f, 0.808f)
);

// Scheme 5: Ayu Mirage (16 colors)
const PAL_AYU_MIRAGE = array<vec3<f32>, 16>(
    // Normal: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.090f, 0.106f, 0.141f), vec3<f32>(0.929f, 0.510f, 0.455f), vec3<f32>(0.529f, 0.851f, 0.424f), vec3<f32>(0.980f, 0.800f, 0.431f),
    vec3<f32>(0.427f, 0.796f, 0.980f), vec3<f32>(0.855f, 0.729f, 0.980f), vec3<f32>(0.565f, 0.882f, 0.776f), vec3<f32>(0.780f, 0.780f, 0.780f),
    // Bright: black, red, green, yellow, blue, magenta, cyan, white
    vec3<f32>(0.408f, 0.408f, 0.408f), vec3<f32>(0.949f, 0.529f, 0.475f), vec3<f32>(0.835f, 1.000f, 0.502f), vec3<f32>(1.000f, 0.820f, 0.451f),
    vec3<f32>(0.451f, 0.816f, 1.000f), vec3<f32>(0.875f, 0.749f, 1.000f), vec3<f32>(0.584f, 0.902f, 0.796f), vec3<f32>(1.000f, 1.000f, 1.000f)
);

struct ColorDitherUniforms {
    color_steps: f32,
    spread: f32,
    pattern_scale: f32,
    palette_index: i32,

    matrix_type: i32, // 0 = 4x4, 1 = 8x8
    _padding4: f32,
    _padding5: f32,
    _padding6: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: ColorDitherUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    let raw_color = textureLoad(src_tex, vec2<i32>(id.xy), 0);

    // 1. Skaliertes Gitter berechnen
    let scale = max(1.0f, uniforms.pattern_scale);
    let dither_coord = floor(vec2<f32>(id.xy) / scale);

    var bayer_value: f32 = 0.0f;

    // 2. MATRIX-SWITCH (Wählt dynamisch das Berechnungsverfahren)
    if (uniforms.matrix_type == 1) {
        // 8x8 Modulo-Auswertung
        let x = u32(dither_coord.x) % 8u;
        let y = u32(dither_coord.y) % 8u;
        bayer_value = BAYER_8X8[y * 8u + x];
    } else {
        // Standard: 4x4 Matrix-Auslese
        let x = u32(dither_coord.x) % 4u;
        let y = u32(dither_coord.y) % 4u;
        bayer_value = BAYER_4X4[y][x];
    }

    let steps = max(2.0f, uniforms.color_steps);
    let dither_shift = (bayer_value - 0.5f) * uniforms.spread / steps;

    var dithered_rgb: vec3<f32>;
    let r_stepped = clamp(raw_color.r + dither_shift, 0.0f, 1.0f) * (steps - 1.0f);
    dithered_rgb.r = floor(r_stepped + 0.5f) / (steps - 1.0f);

    let g_stepped = clamp(raw_color.g + dither_shift, 0.0f, 1.0f) * (steps - 1.0f);
    dithered_rgb.g = floor(g_stepped + 0.5f) / (steps - 1.0f);

    let b_stepped = clamp(raw_color.b + dither_shift, 0.0f, 1.0f) * (steps - 1.0f);
    dithered_rgb.b = floor(b_stepped + 0.5f) / (steps - 1.0f);

    var final_rgb = dithered_rgb;

    // 3. PALETTEN-MAPPING
    switch (uniforms.palette_index) {
        case 1: { // Game Boy
            var min_dist = 999.0f;
            for (var i = 0u; i < 4u; i = i + 1u) {
                let dist = distance(dithered_rgb, PAL_GAMEBOY[i]);
                if (dist < min_dist) { min_dist = dist; final_rgb = PAL_GAMEBOY[i]; }
            }
        }
        case 2: { // CGA
            var min_dist = 999.0f;
            for (var i = 0u; i < 4u; i = i + 1u) {
                let dist = distance(dithered_rgb, PAL_CGA[i]);
                if (dist < min_dist) { min_dist = dist; final_rgb = PAL_CGA[i]; }
            }
        }
        case 3: { // PICO-8
            var min_dist = 999.0f;
            for (var i = 0u; i < 16u; i = i + 1u) {
                let dist = distance(dithered_rgb, PAL_PICO8[i]);
                if (dist < min_dist) { min_dist = dist; final_rgb = PAL_PICO8[i]; }
            }
        }
        case 4: { // Commodore 64
            var min_dist = 999.0f;
            for (var i = 0u; i < 16u; i = i + 1u) {
                let dist = distance(dithered_rgb, PAL_C64[i]);
                if (dist < min_dist) { min_dist = dist; final_rgb = PAL_C64[i]; }
            }
        }
        case 5: { // NES
            var min_dist = 999.0f;
            for (var i = 0u; i < 16u; i = i + 1u) {
                let dist = distance(dithered_rgb, PAL_NES[i]);
                if (dist < min_dist) { min_dist = dist; final_rgb = PAL_NES[i]; }
            }
        }
        case 6: { // Catppuccin Mocha
            var min_dist = 999.0f;
            for (var i = 0u; i < 16u; i = i + 1u) {
                let dist = distance(dithered_rgb, PAL_CATPPUCCIN[i]);
                if (dist < min_dist) { min_dist = dist; final_rgb = PAL_CATPPUCCIN[i]; }
            }
        }
        case 7: { // Tokyo Night
            var min_dist = 999.0f;
            for (var i = 0u; i < 16u; i = i + 1u) {
                let dist = distance(dithered_rgb, PAL_TOKYONIGHT[i]);
                if (dist < min_dist) { min_dist = dist; final_rgb = PAL_TOKYONIGHT[i]; }
            }
        }
        case 8: { // Gruvbox Dark
            var min_dist = 999.0f;
            for (var i = 0u; i < 16u; i = i + 1u) {
                let dist = distance(dithered_rgb, PAL_GRUVBOX[i]);
                if (dist < min_dist) { min_dist = dist; final_rgb = PAL_GRUVBOX[i]; }
            }
        }
        case 9: { // Warm Muted
                var min_dist = 999.0f;
                for (var i = 0u; i < 16u; i = i + 1u) {
                    let dist = distance(dithered_rgb, PAL_WARM_MUTED[i]);
                    if (dist < min_dist) { min_dist = dist; final_rgb = PAL_WARM_MUTED[i]; }
                }
            }
            case 10: { // Earth Vintage
                var min_dist = 999.0f;
                for (var i = 0u; i < 16u; i = i + 1u) {
                    let dist = distance(dithered_rgb, PAL_EARTH_VINTAGE[i]);
                    if (dist < min_dist) { min_dist = dist; final_rgb = PAL_EARTH_VINTAGE[i]; }
                }
            }
            case 11: { // Everforest Dark
                var min_dist = 999.0f;
                for (var i = 0u; i < 16u; i = i + 1u) {
                    let dist = distance(dithered_rgb, PAL_EVERFOREST[i]);
                    if (dist < min_dist) { min_dist = dist; final_rgb = PAL_EVERFOREST[i]; }
                }
            }
            case 12: { // Deep Ocean
                var min_dist = 999.0f;
                for (var i = 0u; i < 16u; i = i + 1u) {
                    let dist = distance(dithered_rgb, PAL_DEEP_OCEAN[i]);
                    if (dist < min_dist) { min_dist = dist; final_rgb = PAL_DEEP_OCEAN[i]; }
                }
            }
            case 13: { // Ayu Mirage
                var min_dist = 999.0f;
                for (var i = 0u; i < 16u; i = i + 1u) {
                    let dist = distance(dithered_rgb, PAL_AYU_MIRAGE[i]);
                    if (dist < min_dist) { min_dist = dist; final_rgb = PAL_AYU_MIRAGE[i]; }
                }
            }
        default: {
            final_rgb = dithered_rgb;
        }
    }

    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, raw_color.a));
}
