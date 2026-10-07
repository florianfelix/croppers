struct GlitchUniforms {
    time: f32,
    line_dropouts: f32,
    block_displace: f32,
    color_burst: f32,
    burst_grid_size: f32,
    tearing_micro: f32,
    tearing_macro: f32,
    tearing_blocks: f32,
}

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: GlitchUniforms;

// 1D and 2D pseudo-random hash functions generating values in [0.0, 1.0].
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
    let seed = uniforms.time;

    // 1. Macro-block displacement: partition image into rows and test threshold.
    var offsetUV = uv;
    let blockCount = uniforms.tearing_blocks;
    let blockId = floor(uv.y * blockCount) + seed;
    let blockRoll = hash11(blockId);

    // Apply horizontal macro tearing shift to qualifying block rows.
    if (blockRoll < uniforms.block_displace) {
        let shiftAmount = (hash11(blockId * 3.7) * 2.0 - 1.0) * 0.12 * uniforms.tearing_macro;
        offsetUV.x = fract(offsetUV.x + shiftAmount);
    }

    // 2. Micro scanline tearing: apply high-frequency horizontal rips to individual lines.
    let lineId = floor(uv.y * f32(size.y)) + seed * 7.1;
    if (hash11(lineId) < uniforms.block_displace) {
        let shiftAmount = (hash11(blockId * 1.9) * 2.0 - 1.0) * 0.12 * uniforms.tearing_micro;
        offsetUV.x = fract(offsetUV.x + shiftAmount);
    }

    // Convert displaced UV coordinates to clamped pixel coordinates and sample source texture.
    let sampleCoord = vec2<i32>(
        clamp(i32(offsetUV.x * f32(size.x)), 0, i32(size.x) - 1),
        clamp(i32(offsetUV.y * f32(size.y)), 0, i32(size.y) - 1)
    );
    var texColor = textureLoad(src_tex, sampleCoord, 0);

    // 3. Scanline dropouts: randomly corrupt horizontal scanlines into black tape lines or white noise.
    let dropoutId = floor(uv.y * 160.0) + seed * 11.3;
    let dropoutRoll = hash11(dropoutId);
    if (dropoutRoll < uniforms.line_dropouts) {
        if (hash11(uv.y + seed) > 0.2) {
            texColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        } else {
            let staticNoise = hash21(uv + seed);
            texColor = vec4<f32>(vec3<f32>(staticNoise), 1.0);
        }
    }

    // 4. Analog color burst: invert and boost color channels within pseudo-random grid tiles.
    let regionGrid = floor(uv * uniforms.burst_grid_size) + seed * 4.2;
    let colorBurstRoll = hash21(regionGrid);
    if (colorBurstRoll < uniforms.color_burst) {
        texColor = vec4<f32>(
            1.0 - texColor.g,
            clamp(texColor.b * 1.8, 0.0, 1.0),
            1.0 - texColor.r,
            texColor.a
        );
    }

    // Write final glitched color to destination texture.
    textureStore(dst_tex, vec2<i32>(id.xy), texColor);
}
