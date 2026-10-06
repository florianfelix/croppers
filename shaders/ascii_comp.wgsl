struct AsciiUniforms {
    scale: f32,          // Dynamic pixel size for each text block (e.g., 8.0, 16.0)
    charset: u32,        // Selects character set: 0, 1, or 2
    inset: f32,          // Inset padding factor: 0.0 (full block) to 0.45 (tiny character)
    mult: f32
}
@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: AsciiUniforms;

const MASTER_GLYPHS = array<vec2<u32>, 16>(
    vec2<u32>(0x00000000u, 0x00000000u), // 0: ' ' (Empty space)
    vec2<u32>(0x00000000u, 0x00181800u), // 1: '.'
    vec2<u32>(0x00181800u, 0x00181800u), // 2: ':'
    vec2<u32>(0x00181800u, 0x00183000u), // 3: ';'
    vec2<u32>(0x00000000u, 0x003c3c00u), // 4: '-'
    vec2<u32>(0x003c3c00u, 0x003c3c00u), // 5: '='
    vec2<u32>(0x0018187eu, 0x7e181800u), // 6: '+'
    vec2<u32>(0x0000663cu, 0x3c660000u), // 7: 'x'
    vec2<u32>(0x00c66c38u, 0x386cc600u), // 8: 'X'
    vec2<u32>(0x3c66663cu, 0x3c66663cu), // 9: '8'
    vec2<u32>(0x3c666666u, 0x6666663cu), // 10: '0'
    vec2<u32>(0x3e603c02u, 0x023c067cu), // 11: 'S'
    vec2<u32>(0x00663cffu, 0xff3c6600u), // 12: '*'
    vec2<u32>(0x62666c70u, 0x0e366646u), // 13: '%'
    vec2<u32>(0x6c6cff6cu, 0x6cff6c6cu), // 14: '#'
    vec2<u32>(0x3c66667eu, 0x66763c00u)  // 15: '@'
);

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    let cellSize = max(uniforms.scale, 1.0);

    // 1. Calculate cell grid indices using floating-point math
    let currentPixel = vec2<f32>(id.xy);
    let cellIndices = floor(currentPixel / cellSize);

    // 2. Find the center pixel of the dynamic block to sample image colors accurately
    let src_size = textureDimensions(src_tex);
    let halfCell = cellSize * 0.5;
    let cellCenter = vec2<i32>(
        clamp(i32(cellIndices.x * cellSize + halfCell), 0, i32(src_size.x) - 1),
        clamp(i32(cellIndices.y * cellSize + halfCell), 0, i32(src_size.y) - 1)
    );
    let cellColor = textureLoad(src_tex, cellCenter, 0);

    // Calculate luminance (0.0 to 1.0)
    let luminance = 0.299 * cellColor.r + 0.587 * cellColor.g + 0.114 * cellColor.b;
    let densityFactor = 1.0 - luminance;

    // 3. Map brightness to the selected charset via switch routing
    var glyphPacked: vec2<u32>;
    switch (uniforms.charset) {
        case 0u: {
            let ramp = array<u32, 14>(15, 14, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
            let charIndex = u32(clamp(densityFactor * 13.0, 0.0, 13.0));
            glyphPacked = MASTER_GLYPHS[ramp[charIndex]];
        }
        case 1u: {
            let ramp = array<u32, 10>(15, 13, 14, 12, 6, 5, 4, 2, 1, 0);
            let charIndex = u32(clamp(densityFactor * 9.0, 0.0, 9.0));
            glyphPacked = MASTER_GLYPHS[ramp[charIndex]];
        }
        default: {
            let ramp = array<u32, 5>(15, 8, 2, 1, 0);
            let charIndex = u32(clamp(densityFactor * 4.0, 0.0, 4.0));
            glyphPacked = MASTER_GLYPHS[ramp[charIndex]];
        }
    }

    // 4. Determine the normalized local coordinate (0.0 to 1.0) inside this specific box
    let localUV = (currentPixel - (cellIndices * cellSize)) / cellSize;

    // 5. Apply dynamic inset adjustment
    // Clamp the layout boundaries to prevent division errors or inside out mirrors
    let clampedInset = clamp(uniforms.inset, 0.0, 0.45);

    var bitState: u32 = 0u;

    // Check if the current pixel falls inside the padding/inset boundary zone
    if (localUV.x >= clampedInset && localUV.x <= (1.0 - clampedInset) &&
        localUV.y >= clampedInset && localUV.y <= (1.0 - clampedInset)) {

        // Re-scale the coordinates back to a clean 0.0 - 1.0 inner canvas window
        let insetUV = (localUV - clampedInset) / (1.0 - 2.0 * clampedInset);

        // Map the new inner space onto our 8x8 font grid bits
        let localX = u32(clamp(insetUV.x * 8.0, 0.0, 7.0));
        let localY = u32(clamp(insetUV.y * 8.0, 0.0, 7.0));

        // 6. Extract the individual bit state
        if (localY < 4u) {
            let bitIndex = 31u - ((localY * 8u) + localX);
            bitState = (glyphPacked.x >> bitIndex) & 1u;
        } else {
            let bitIndex = 31u - (((localY - 4u) * 8u) + localX);
            bitState = (glyphPacked.y >> bitIndex) & 1u;
        }
    }

    // 7. Rasterize the finalized output
    var finalColor = textureLoad(src_tex, vec2<i32>(id.xy), 0);
    finalColor = finalColor * uniforms.mult;
    // var finalColor = vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black background
    if (bitState == 1u) {
        finalColor = vec4<f32>(cellColor.rgb, 1.0);
    }

    textureStore(dst_tex, vec2<i32>(id.xy), finalColor);
}
