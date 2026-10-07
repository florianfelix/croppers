struct AsciiUniforms {
    scale: f32,          // Pixel size for each text block (e.g., 8.0, 16.0)
    charset: u32,        // Character set selector: 0, 1, or 2
    inset: f32,          // Inset padding factor: 0.0 (full block) to 0.45 (small glyph)
    mult: f32,           // Multiplier for background image brightness
    glyph_selection: i32,
    scale_x: f32,
    rain_amount: f32,
    rain_length_random: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: AsciiUniforms;

// 8x8 font bitmap glyph table encoded into pairs of 32-bit integers
const MASTER_GLYPHS = array<vec2<u32>, 16>(
    vec2<u32>(0x00000000u, 0x00000000u), // 0: ' ' (Space)
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

// --- Option 1: 8x16 Glyphs (vec4<u32>) ---
const GLYPHS_8X16 = array<vec4<u32>, 16>(
    vec4<u32>(0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u), // 0: ' '
    vec4<u32>(0x00000000u, 0x00001818u, 0x18000000u, 0x00000000u), // 1: '.'
    vec4<u32>(0x00000000u, 0x18181800u, 0x00181818u, 0x00000000u), // 2: ':'
    vec4<u32>(0x00000018u, 0x18180000u, 0x18181838u, 0x30000000u), // 3: ';'
    vec4<u32>(0x00000000u, 0x003c3c3cu, 0x00000000u, 0x00000000u), // 4: '-'
    vec4<u32>(0x00000000u, 0xffff00ffu, 0xff000000u, 0x00000000u), // 5: '='
    vec4<u32>(0x00000018u, 0x181818ffu, 0xff181818u, 0x00000000u), // 6: '+'
    vec4<u32>(0x00000000u, 0x667e3c18u, 0x3c3c66e7u, 0x00000000u), // 7: 'x'
    vec4<u32>(0x0000c3e7u, 0x667e3c18u, 0x3c3c7e66u, 0xe7c30000u), // 8: 'X'
    vec4<u32>(0x00003c7eu, 0x6666663cu, 0x7e67e3e7u, 0x7e3c0000u), // 9: '8'
    vec4<u32>(0x00003c7eu, 0x6667e7ffu, 0xffe76766u, 0x7e3c0000u), // 10: '0'
    vec4<u32>(0x00003c7eu, 0xe6e0f07cu, 0x1e0707e7u, 0xfe3c0000u), // 11: 'S'
    vec4<u32>(0x00001818u, 0x7e3c7e5au, 0x18000000u, 0x00000000u), // 12: '*'
    vec4<u32>(0x000000f0u, 0x9090f16eu, 0x768f0909u, 0x0f000000u), // 13: '%'
    vec4<u32>(0x0000121bu, 0x32ffff36u, 0x64fffe6cu, 0xc8000000u), // 14: '#'
    vec4<u32>(0x00003e77u, 0xc3df9fb3u, 0xb39fdfc0u, 0x733f0000u) // 15: '@'
);

// --- Option 2: 16x16 Glyphs (array<u32, 128>) ---
const GLYPHS_16X16 = array<u32, 128>(
    // Glyph 0: ' '
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 1: '.'
    0x00000000u, 0x00000000u, 0x00000000u, 0x01800180u,
    0x01800000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 2: ':'
    0x00000000u, 0x00000000u, 0x01800180u, 0x01800000u,
    0x00000180u, 0x01800180u, 0x00000000u, 0x00000000u,
    // Glyph 3: ';'
    0x00000000u, 0x00000180u, 0x01800180u, 0x00000000u,
    0x01800180u, 0x01800380u, 0x03000000u, 0x00000000u,
    // Glyph 4: '-'
    0x00000000u, 0x00000000u, 0x000003c0u, 0x03c003c0u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 5: '='
    0x00000000u, 0x00000000u, 0x0ff00ff0u, 0x00000ff0u,
    0x0ff00000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 6: '+'
    0x00000000u, 0x00000180u, 0x01800180u, 0x01800ff0u,
    0x0ff00180u, 0x01800180u, 0x00000000u, 0x00000000u,
    // Glyph 7: 'x'
    0x00000000u, 0x00000000u, 0x066007e0u, 0x03c00180u,
    0x03c003c0u, 0x06600e70u, 0x00000000u, 0x00000000u,
    // Glyph 8: 'X'
    0x00000000u, 0x0c300e70u, 0x066007e0u, 0x03c00180u,
    0x03c003c0u, 0x07e00660u, 0x0e700c30u, 0x00000000u,
    // Glyph 9: '8'
    0x00000000u, 0x03c007e0u, 0x06600660u, 0x066003c0u,
    0x07e00670u, 0x0e300e70u, 0x07e003c0u, 0x00000000u,
    // Glyph 10: '0'
    0x00000000u, 0x03c007e0u, 0x06600670u, 0x0e700ff0u,
    0x0ff00e70u, 0x06700660u, 0x07e003c0u, 0x00000000u,
    // Glyph 11: 'S'
    0x00000000u, 0x03c007e0u, 0x0e600e00u, 0x0f0007c0u,
    0x01e00070u, 0x00700e70u, 0x0fe003c0u, 0x00000000u,
    // Glyph 12: '*'
    0x00000000u, 0x01800180u, 0x07e003c0u, 0x07e005a0u,
    0x01800000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 13: '%'
    0x00000000u, 0x00000f00u, 0x09000900u, 0x0f1006e0u,
    0x076008f0u, 0x00900090u, 0x00f00000u, 0x00000000u,
    // Glyph 14: '#'
    0x00000000u, 0x012001b0u, 0x03200ff0u, 0x0ff00360u,
    0x06401ff0u, 0x0fe006c0u, 0x0c800000u, 0x00000000u,
    // Glyph 15: '@'
    0x00000000u, 0x03e00770u, 0x0c300df0u, 0x09f01b30u,
    0x1b3009f0u, 0x0df00c00u, 0x073003f0u, 0x00000000u
);

// --- Option 3: 16x32 Glyphs (array<u32, 256>) ---
const GLYPHS_16X32 = array<u32, 256>(
    // Glyph 0: ' '
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 1: '.'
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x00000000u, 0x000007c0u, 0x07c007c0u,
    0x07c007c0u, 0x07c00000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 2: ':'
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0x07c007c0u, 0x07c007c0u, 0x07c003c0u, 0x00000000u,
    0x00000000u, 0x07c007c0u, 0x07c007c0u, 0x07c007c0u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 3: ';'
    0x00000000u, 0x00000000u, 0x00000000u, 0x07c007c0u,
    0x07c007c0u, 0x07c003c0u, 0x00000000u, 0x00000000u,
    0x07c007c0u, 0x07c007c0u, 0x07c007c0u, 0x07800780u,
    0x0f000f00u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 4: '-'
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x1ff01ff8u, 0x1ff81ff8u, 0x1ff00000u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 5: '='
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0x0000ffffu, 0xffffffffu, 0x00000000u, 0x0000fffeu,
    0xffffffffu, 0xffff0000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 6: '+'
    0x00000000u, 0x00000000u, 0x00000000u, 0x000003c0u,
    0x03c003c0u, 0x03c003c0u, 0x03c0ffffu, 0xffffffffu,
    0xffff03c0u, 0x03c003c0u, 0x03c003c0u, 0x03c00380u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 7: 'x'
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    0xfc3e7c3eu, 0x3e7c1ef8u, 0x1ff00ff0u, 0x07e007c0u,
    0x07e00ff0u, 0x0ff01ef8u, 0x3e7c7c7cu, 0x7c3ef81fu,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 8: 'X'
    0x00000000u, 0x00000000u, 0x0000f01fu, 0xf81f783eu,
    0x7c3c3e7cu, 0x3e781ff8u, 0x0ff00ff0u, 0x07e007c0u,
    0x07e007e0u, 0x0ff01ff0u, 0x1ff83e78u, 0x3c7c7c3eu,
    0xf81ef81fu, 0xf00f0000u, 0x00000000u, 0x00000000u,
    // Glyph 9: '8'
    0x00000000u, 0x00000000u, 0x03800ff0u, 0x1ff83ffcu,
    0x7c7c783eu, 0x781e783eu, 0x7c3c3e7cu, 0x1ff80ff0u,
    0x3ff83e7cu, 0x783e781eu, 0x781e781eu, 0x781e7c7eu,
    0x7ffc3ffcu, 0x1ff00380u, 0x00000000u, 0x00000000u,
    // Glyph 10: '0'
    0x00000000u, 0x00000000u, 0x01800ff0u, 0x1ff83ff8u,
    0x3e7c7c3cu, 0x7c3e783eu, 0x783e781eu, 0xf99efbdeu,
    0xfbdef99eu, 0x781e783eu, 0x783e7c3eu, 0x7c3c3e7cu,
    0x3ff81ff8u, 0x0ff00180u, 0x00000000u, 0x00000000u,
    // Glyph 11: 'S'
    0x00000000u, 0x00000000u, 0x03801ff8u, 0x3ffc7ffcu,
    0x7e3c780cu, 0x78007800u, 0x7c007f00u, 0x3fe03ff8u,
    0x0ffc03fcu, 0x007e003eu, 0x001e001eu, 0x603e7c7eu,
    0x7ffc7ffcu, 0x3ff00380u, 0x00000000u, 0x00000000u,
    // Glyph 12: '*'
    0x00000000u, 0x00000000u, 0x00000380u, 0x03806384u,
    0x7b9e7ffeu, 0x1ff807e0u, 0x0ff03ffcu, 0x7bbe738eu,
    0x03800380u, 0x01800000u, 0x00000000u, 0x00000000u,
    0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u,
    // Glyph 13: '%'
    0x00000000u, 0x00000000u, 0x00003c00u, 0x7e00ff00u,
    0xe780c380u, 0xc380e780u, 0xff037e1fu, 0x3c7c03e0u,
    0x0f807c3cu, 0xf0ff00ffu, 0x01e701c3u, 0x01c300e7u,
    0x00ff007eu, 0x00180000u, 0x00000000u, 0x00000000u,
    // Glyph 14: '#'
    0x00000000u, 0x00000000u, 0x0000038eu, 0x078e079eu,
    0x071e071cu, 0x7fffffffu, 0xffffffffu, 0x1e381e78u,
    0x1c781c70u, 0xfffefffeu, 0xfffe78f0u, 0x78e079e0u,
    0x71e071c0u, 0x71c00000u, 0x00000000u, 0x00000000u,
    // Glyph 15: '@'
    0x00000000u, 0x000003e0u, 0x0ff81ffcu, 0x3e1e780eu,
    0xf007e067u, 0xe1ffc3ffu, 0xc79fc70fu, 0xcf07cf07u,
    0xcf07cf07u, 0xc70fc7ffu, 0xe3ffe1f7u, 0xf0007800u,
    0x7c023f1eu, 0x1fff07feu, 0x00f00000u, 0x00000000u
);

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    // Calculate 2D cell dimensions from scale and scale_x multiplier (safely defaulting to 1.0)
    let scaleX = select(uniforms.scale_x, 1.0, uniforms.scale_x <= 0.0);
    let cellSize = vec2<f32>(
        max(uniforms.scale * scaleX, 1.0),
        max(uniforms.scale, 1.0)
    );

    // Map pixel coordinate to cell grid indices
    let currentPixel = vec2<f32>(id.xy);
    let cellIndices = floor(currentPixel / cellSize);

    // Sample source image color and luminance at the center of the cell block
    let src_size = textureDimensions(src_tex);
    let halfCell = cellSize * 0.5;
    let cellCenter = vec2<i32>(
        clamp(i32(cellIndices.x * cellSize.x + halfCell.x), 0, i32(src_size.x) - 1),
        clamp(i32(cellIndices.y * cellSize.y + halfCell.y), 0, i32(src_size.y) - 1)
    );
    let cellColor = textureLoad(src_tex, cellCenter, 0);

    let luminance = 0.299 * cellColor.r + 0.587 * cellColor.g + 0.114 * cellColor.b;
    let densityFactor = 1.0 - luminance;

    // Select character glyph index based on brightness ramp and charset
    var glyphIndex: u32 = 0u;
    switch (uniforms.charset) {
        case 0u: {
            let ramp = array<u32, 14>(15, 14, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
            let charIndex = u32(clamp(densityFactor * 13.0, 0.0, 13.0));
            glyphIndex = ramp[charIndex];
        }
        case 1u: {
            let ramp = array<u32, 10>(15, 13, 14, 12, 6, 5, 4, 2, 1, 0);
            let charIndex = u32(clamp(densityFactor * 9.0, 0.0, 9.0));
            glyphIndex = ramp[charIndex];
        }
        default: {
            let ramp = array<u32, 5>(15, 8, 2, 1, 0);
            let charIndex = u32(clamp(densityFactor * 4.0, 0.0, 4.0));
            glyphIndex = ramp[charIndex];
        }
    }

    // Determine normalized local coordinate inside cell and apply inset padding
    let localUV = (currentPixel - (cellIndices * cellSize)) / cellSize;
    let clampedInset = clamp(uniforms.inset, 0.0, 0.45);

    var bitState: u32 = 0u;

    // Extract active bitmap bit according to selected glyph resolution
    if (localUV.x >= clampedInset && localUV.x <= (1.0 - clampedInset) &&
        localUV.y >= clampedInset && localUV.y <= (1.0 - clampedInset)) {

        let insetUV = (localUV - clampedInset) / (1.0 - 2.0 * clampedInset);

        switch (uniforms.glyph_selection) {
            // Option 0: 8x8 bitmap (vec2<u32>)
            case 0: {
                let lx = u32(clamp(insetUV.x * 8.0, 0.0, 7.0));
                let ly = u32(clamp(insetUV.y * 8.0, 0.0, 7.0));
                let packed = MASTER_GLYPHS[glyphIndex];
                let word = select(packed.y, packed.x, ly < 4u);
                let bitIdx = 31u - ((ly % 4u) * 8u + lx);
                bitState = (word >> bitIdx) & 1u;
            }
            // Option 1: 8x16 bitmap (vec4<u32>)
            case 1: {
                let lx = u32(clamp(insetUV.x * 8.0, 0.0, 7.0));
                let ly = u32(clamp(insetUV.y * 16.0, 0.0, 15.0));
                let packed = GLYPHS_8X16[glyphIndex];
                let chunk = ly / 4u;
                var word = packed.x;
                if (chunk == 1u) { word = packed.y; }
                else if (chunk == 2u) { word = packed.z; }
                else if (chunk == 3u) { word = packed.w; }
                let bitIdx = 31u - ((ly % 4u) * 8u + lx);
                bitState = (word >> bitIdx) & 1u;
            }
            // Option 2: 16x16 bitmap (array<u32, 128>)
            case 2: {
                let lx = u32(clamp(insetUV.x * 16.0, 0.0, 15.0));
                let ly = u32(clamp(insetUV.y * 16.0, 0.0, 15.0));
                let word = GLYPHS_16X16[glyphIndex * 8u + (ly / 2u)];
                let bitIdx = select(15u - lx, 31u - lx, (ly % 2u) == 0u);
                bitState = (word >> bitIdx) & 1u;
            }
            // Option 3: 16x32 bitmap (array<u32, 256>)
            case 3: {
                let lx = u32(clamp(insetUV.x * 16.0, 0.0, 15.0));
                let ly = u32(clamp(insetUV.y * 32.0, 0.0, 31.0));
                let word = GLYPHS_16X32[glyphIndex * 16u + (ly / 2u)];
                let bitIdx = select(15u - lx, 31u - lx, (ly % 2u) == 0u);
                bitState = (word >> bitIdx) & 1u;
            }
            default: {}
        }
    }

    // Blend background image with cell character color based on font bitmask
    var finalColor = textureLoad(src_tex, vec2<i32>(id.xy), 0);
    finalColor = finalColor * uniforms.mult;
    if (bitState == 1u) {
        finalColor = vec4<f32>(cellColor.rgb, 1.0);
    }

    textureStore(dst_tex, vec2<i32>(id.xy), finalColor);
}
