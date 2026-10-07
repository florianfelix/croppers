struct GlitchUniforms {
    time: f32,              // Used as a static seed number (change value to roll a new glitch layout)
    line_dropouts: f32,     // intensity1: 0.0 (none) to 1.0 (tons of dead/black horizontal lines)
    block_displace: f32,    // intensity2: 0.0 (none) to 1.0 (heavy amount of horizontal block slicing)
    color_burst: f32,       // intensity3: 0.0 (none) to 1.0 (fills more screen regions with color frying)
    burst_grid_size: f32,
    tearing_micro: f32,
    tearing_macro: f32,
    tearing_blocks: f32
}

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: GlitchUniforms;

// Hash noise functions to generate hard, non-smooth random thresholds
fn hash11(p: f32) -> f32 {
    return fract(sin(p * 127.1) * 43758.5453123);
}

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) { return; }

    let uv = vec2<f32>(id.xy) / vec2<f32>(size);

    // Use the time float as a static seed offset
    let seed = uniforms.time;

    // --- 1. Macro-Block Displacement Amount ---
    // The block_displace slider directly dictates the PERCENTAGE of slices that get shifted
    var offsetUV = uv;
    let blockCount = uniforms.tearing_blocks;
    let blockId = floor(uv.y * blockCount) + seed;

    // Generate a unique value per block row between 0.0 and 1.0
    let blockRoll = hash11(blockId);

    // --- 2.1 Macro Scanline "Teeth" (Tearing) ---
    if (blockRoll < uniforms.block_displace) {
        // Generate a completely sharp, random shift distance based on this block's ID
        let shiftAmount = (hash11(blockId * 3.7) * 2.0 - 1.0) * 0.12 * uniforms.tearing_macro;
        offsetUV.x = fract(offsetUV.x + shiftAmount);
    }

    // --- 2.2 Micro Scanline "Teeth" (Tearing) ---
    // High-frequency single-pixel horizontal rips, scaled directly by the displacement slider
    let lineId = floor(uv.y * f32(size.y)) + seed * 7.1;
    if (hash11(lineId) < (uniforms.block_displace )) {
        let shiftAmount = (hash11(blockId * 1.9) * 2.0 - 1.0) * 0.12 * uniforms.tearing_micro;
        offsetUV.x = fract(offsetUV.x + shiftAmount);
    }

    // Convert our shifted coordinates back to safe integer texture indexes
    let sampleCoord = vec2<i32>(
        clamp(i32(offsetUV.x * f32(size.x)), 0, i32(size.x) - 1),
        clamp(i32(offsetUV.y * f32(size.y)), 0, i32(size.y) - 1)
    );

    var texColor = textureLoad(src_tex, sampleCoord, 0);

    // --- 3. Scanline Dropouts Amount ---
    // Individual scanlines are dropped to solid black or white noise
    // The slider directly controls how many rows out of 160 are corrupted simultaneously
    let dropoutId = floor(uv.y * 160.0) + seed * 11.3;
    let dropoutRoll = hash11(dropoutId);

    if (dropoutRoll < uniforms.line_dropouts) {
        // 80% chance to drop to pure dead analog black tape line, 20% white static strip
        if (hash11(uv.y + seed) > 0.2) {
            texColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        } else {
            let staticNoise = hash21(uv + seed);
            texColor = vec4<f32>(vec3<f32>(staticNoise), 1.0);
        }
    }

    // --- 4. Analog Color Burst Region Amount ---
    // The slider controls the density of macro regions affected by high-voltage color clipping
    // The screen is split into a grid; the higher the slider, the more grid cells are fried
    let regionGrid = floor(uv * uniforms.burst_grid_size) + seed * 4.2;
    let colorBurstRoll = hash21(regionGrid);

    if (colorBurstRoll < uniforms.color_burst) {
        // High-voltage color frying inversion matrix
        texColor = vec4<f32>(
            1.0 - texColor.g,
            clamp(texColor.b * 1.8, 0.0, 1.0),
            1.0 - texColor.r,
            texColor.a
        );
    }

    textureStore(dst_tex, vec2<i32>(id.xy), texColor);
}
