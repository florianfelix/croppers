struct VcrUniforms {
    time: f32,
    jitter: f32,
    glitch: f32,
    glitch_noise: f32,
    head_switch: f32,
    head_switch_noise: f32,
    tape_roll: f32,
    chroma_decay: f32
}

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: VcrUniforms;

fn noise(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    let initial_uv = vec2<f32>(id.xy) / vec2<f32>(size);
    let t = uniforms.time;
    let intensity1 = uniforms.jitter;
    let intensity2 = uniforms.glitch;
    let intensity3 = uniforms.glitch_noise;
    let intensity4 = uniforms.head_switch;
    let intensity5 = uniforms.head_switch_noise;
    let intensity6 = uniforms.tape_roll;
    let intensity7 = uniforms.chroma_decay;

    // --- 0. Vertikales Bildrollen & Austastbalken (VBI) ---
    var rolledY = initial_uv.y + (t * intensity6);
    let vbiCheck = fract(rolledY);

    // We keep the VBI size stable or scale it up slightly during tracking loss
    let vbiHeight = 0.075 * intensity6;
    let isInsideVBI = vbiCheck < vbiHeight;

    let uvY = fract(rolledY + vbiHeight);
    let uv = vec2<f32>(initial_uv.x, uvY);

    // --- 1. Jitter & Horizontal Wiggling ---
    var wiggle = sin(uv.y * 40.0 + t * 8.0) * 0.0015;
    wiggle += sin(uv.y * 150.0 - t * 25.0) * 0.0006;

    let flagging = pow(max(1.0 - uv.y, 0.0), 4.0) * 0.02;
    wiggle += flagging;

    // --- 2. Moving Line Tear / Tracking Glitch ---
    let tearY = fract(t * 0.12);
    let tearRange = 0.03 * intensity2;
    var tearOffset = 0.0;

    if (uv.y > tearY && uv.y < (tearY + tearRange)) {
        tearOffset = sin(uv.x * 30.0 + t * 40.0) * 0.015 + noise(vec2<f32>(uv.y, t)) * 0.008;
    }

    // --- 5. Head-Switching Noise Preparation ---
    let headSwitchThreshold = 1.0 - (0.025 * intensity4);
    var headSwitchXOffset = 0.0;
    let isHeadSwitchZone = uv.y > headSwitchThreshold;

    if (isHeadSwitchZone) {
        let edgeProgress = (uv.y - headSwitchThreshold) / (1.0 - headSwitchThreshold);
        headSwitchXOffset = (sin(uv.y * 500.0 + t * 10.0) * 0.03 + 0.02) * edgeProgress * intensity5;
    }

    let totalXOffset = (wiggle + tearOffset) * intensity1 + headSwitchXOffset;

    // --- 3. Chrominance Shift (RGB Split) ---
    let chromaShift = (0.003 + sin(t * 1.5) * 0.001) * intensity1;

    let uvRed   = vec2<f32>(clamp(uv.x + totalXOffset - chromaShift, 0.0, 1.0), uv.y);
    let uvGreen = vec2<f32>(clamp(uv.x + totalXOffset, 0.0, 1.0), uv.y);
    let uvBlue  = vec2<f32>(clamp(uv.x + totalXOffset + chromaShift, 0.0, 1.0), uv.y);

    let coordR = vec2<i32>(i32(uvRed.x * f32(size.x)),   i32(uvRed.y * f32(size.y)));
    let coordG = vec2<i32>(i32(uvGreen.x * f32(size.x)), i32(uvGreen.y * f32(size.y)));
    let coordB = vec2<i32>(i32(uvBlue.x * f32(size.x)),  i32(uvBlue.y * f32(size.y)));

    let colorR = textureLoad(src_tex, coordR, 0).r;
    let colorG = textureLoad(src_tex, coordG, 0).g;
    let colorB = textureLoad(src_tex, coordB, 0).b;
    let alpha  = textureLoad(src_tex, coordG, 0).a;

    var finalColor = vec4<f32>(colorR, colorG, colorB, alpha);

    // --- 4. Snowy Tracking Tape Static ---
    if (tearOffset != 0.0) {
        let staticSnow = noise(vec2<f32>(uv.x * 4.0, uv.y + t));
        if (staticSnow > 0.5) {
            finalColor = mix(finalColor, vec4<f32>(staticSnow, staticSnow, staticSnow, 1.0), 0.3 * intensity3);
        }
    }

    // --- 6. Head-Switching Static Overlay ---
    if (isHeadSwitchZone) {
        let staticCoord = vec2<f32>(uv.x * 15.0, uv.y * 90.0 + t * 60.0);
        let headSnow = noise(staticCoord);
        if (headSnow > 0.4) {
            let snowColor = vec4<f32>(headSnow, headSnow, headSnow, 1.0);
            let blendFactor = 0.75 * intensity5 * ((uv.y - headSwitchThreshold) / (1.0 - headSwitchThreshold));
            finalColor = mix(finalColor, snowColor, clamp(blendFactor, 0.0, 1.0));
        }
    }

    // --- 7. Upgraded, High-Fidelity Austastbalken (VBI) Pass ---
    if (isInsideVBI && intensity6 > 0.0) {
        // Normalize VBI height from 0.0 (top of bar) to 1.0 (bottom of bar)
        let localVbiY = vbiCheck / vbiHeight;

        // Split our VBI into 18 discrete analog data scanlines
        let lineIndex = floor(localVbiY * 18.0);

        // Default base blanking color (Very dark gray analog pedestal)
        var vbiRGB = vec3<f32>(0.03, 0.03, 0.03);

        // Rhythmic ticking clock generation for signals
        let signalClock = floor(initial_uv.x * 55.0 + t * 24.0);
        let binaryData = noise(vec2<f32>(signalClock, lineIndex));

        if (lineIndex < 4.0) {
            // Lines 0-3: Vertical Sync / Equalizing Pulses (Thick alternating black/white blocks)
            let syncPulse = floor(initial_uv.x * 4.0);
            if (syncPulse % 2.0 == 0.0) {
                vbiRGB = vec3<f32>(0.7, 0.7, 0.7);
            } else {
                vbiRGB = vec3<f32>(0.0, 0.0, 0.0);
            }
        } else if (lineIndex >= 5.0 && lineIndex <= 8.0) {
            // Lines 5-8: Closed Caption / VITC Timecode (Flickering binary squares)
            if (binaryData > 0.45 && fract(initial_uv.x * 45.0) > 0.2) {
                vbiRGB = vec3<f32>(0.85, 0.85, 0.85);
            }
        } else if (lineIndex == 11.0 || lineIndex == 12.0) {
            // Lines 11-12: Teletext Data burst / Test Pattern
            if (binaryData > 0.3) {
                // Alternating color burst lines (simulating chrominance subcarrier calibration)
                if (lineIndex == 11.0) {
                    vbiRGB = vec3<f32>(0.0, 0.7, 0.7); // Cyan calibration strip
                } else {
                    vbiRGB = vec3<f32>(0.7, 0.7, 0.0); // Yellow calibration strip
                }
            }
        } else if (lineIndex >= 15.0) {
            // Lines 15-18: Vertical Sync termination guard bands (Fades smoothly back to video signal)
            vbiRGB = vec3<f32>(0.01, 0.01, 0.01);
        }

        // Add subtle horizontal magnetic hum noise across the entire VBI bar block
        let vbiHum = noise(vec2<f32>(initial_uv.x * 2.0, initial_uv.y + t));
        finalColor = vec4<f32>(vbiRGB + (vbiHum * 0.02), 1.0);
    }

    // --- 8. Fleckiger Farbverlust / Bleiche-Effekt ---
    let luminance = 0.299 * finalColor.r + 0.587 * finalColor.g + 0.114 * finalColor.b;
    let grayColor = vec3<f32>(luminance);

    let blotchCoord = vec2<f32>(uv.x * 3.0, uv.y * 2.0 + t * 0.1);
    var blotchNoise = noise(floor(blotchCoord * 4.0) / 4.0);
    blotchNoise += sin(uv.x * 5.0 + t) * 0.2;
    blotchNoise += cos(uv.y * 4.0 - t * 0.5) * 0.2;

    let localizedBleach = clamp(blotchNoise, 0.0, 1.0);
    let finalBleachFactor = clamp(intensity7 * localizedBleach * 1.5, 0.0, 1.0);

    finalColor = vec4<f32>(mix(finalColor.rgb, grayColor, finalBleachFactor), finalColor.a);

    textureStore(dst_tex, vec2<i32>(id.xy), finalColor);
}
