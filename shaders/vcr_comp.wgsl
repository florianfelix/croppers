struct VcrUniforms {
    time: f32,
    jitter: f32,
    glitch: f32,
    glitch_noise: f32,
    head_switch: f32,
    head_switch_noise: f32,
    tape_roll: f32,
    chroma_decay: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: VcrUniforms;

// Hash noise function producing pseudo-random values for analog static
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

    // Apply vertical tape rolling offset and detect Vertical Blanking Interval (VBI) bar
    var rolledY = initial_uv.y + (t * intensity6);
    let vbiCheck = fract(rolledY);
    let vbiHeight = 0.075 * intensity6;
    let isInsideVBI = vbiCheck < vbiHeight;

    let uvY = fract(rolledY + vbiHeight);
    let uv = vec2<f32>(initial_uv.x, uvY);

    // Compute horizontal scanline jitter and top-of-frame tape flagging
    var wiggle = sin(uv.y * 40.0 + t * 8.0) * 0.0015;
    wiggle += sin(uv.y * 150.0 - t * 25.0) * 0.0006;
    let flagging = pow(max(1.0 - uv.y, 0.0), 4.0) * 0.02;
    wiggle += flagging;

    // Moving horizontal glitch tear / tracking displacement
    let tearY = fract(t * 0.12);
    let tearRange = 0.03 * intensity2;
    var tearOffset = 0.0;
    if (uv.y > tearY && uv.y < (tearY + tearRange)) {
        tearOffset = sin(uv.x * 30.0 + t * 40.0) * 0.015 + noise(vec2<f32>(uv.y, t)) * 0.008;
    }

    // Bottom-edge VCR head-switching distortion
    let headSwitchThreshold = 1.0 - (0.025 * intensity4);
    var headSwitchXOffset = 0.0;
    let isHeadSwitchZone = uv.y > headSwitchThreshold;
    if (isHeadSwitchZone) {
        let edgeProgress = (uv.y - headSwitchThreshold) / (1.0 - headSwitchThreshold);
        headSwitchXOffset = (sin(uv.y * 500.0 + t * 10.0) * 0.03 + 0.02) * edgeProgress * intensity5;
    }

    let totalXOffset = (wiggle + tearOffset) * intensity1 + headSwitchXOffset;

    // Chrominance channel split (analog RGB color bleeding)
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

    // Overlay tracking tape static snow inside glitch tear
    if (tearOffset != 0.0) {
        let staticSnow = noise(vec2<f32>(uv.x * 4.0, uv.y + t));
        if (staticSnow > 0.5) {
            finalColor = mix(finalColor, vec4<f32>(staticSnow, staticSnow, staticSnow, 1.0), 0.3 * intensity3);
        }
    }

    // Overlay head-switching noise band at bottom of screen
    if (isHeadSwitchZone) {
        let staticCoord = vec2<f32>(uv.x * 15.0, uv.y * 90.0 + t * 60.0);
        let headSnow = noise(staticCoord);
        if (headSnow > 0.4) {
            let snowColor = vec4<f32>(headSnow, headSnow, headSnow, 1.0);
            let blendFactor = 0.75 * intensity5 * ((uv.y - headSwitchThreshold) / (1.0 - headSwitchThreshold));
            finalColor = mix(finalColor, snowColor, clamp(blendFactor, 0.0, 1.0));
        }
    }

    // Synthesize analog Vertical Blanking Interval (VBI) telemetry scanlines
    if (isInsideVBI && intensity6 > 0.0) {
        let localVbiY = vbiCheck / vbiHeight;
        let lineIndex = floor(localVbiY * 18.0);
        var vbiRGB = vec3<f32>(0.03, 0.03, 0.03);

        let signalClock = floor(initial_uv.x * 55.0 + t * 24.0);
        let binaryData = noise(vec2<f32>(signalClock, lineIndex));

        if (lineIndex < 4.0) {
            // Vertical sync pulses
            let syncPulse = floor(initial_uv.x * 4.0);
            if (syncPulse % 2.0 == 0.0) {
                vbiRGB = vec3<f32>(0.7, 0.7, 0.7);
            } else {
                vbiRGB = vec3<f32>(0.0, 0.0, 0.0);
            }
        } else if (lineIndex >= 5.0 && lineIndex <= 8.0) {
            // Closed caption and timecode data bursts
            if (binaryData > 0.45 && fract(initial_uv.x * 45.0) > 0.2) {
                vbiRGB = vec3<f32>(0.85, 0.85, 0.85);
            }
        } else if (lineIndex == 11.0 || lineIndex == 12.0) {
            // Teletext / chroma subcarrier test bursts
            if (binaryData > 0.3) {
                if (lineIndex == 11.0) {
                    vbiRGB = vec3<f32>(0.0, 0.7, 0.7);
                } else {
                    vbiRGB = vec3<f32>(0.7, 0.7, 0.0);
                }
            }
        } else if (lineIndex >= 15.0) {
            vbiRGB = vec3<f32>(0.01, 0.01, 0.01);
        }

        let vbiHum = noise(vec2<f32>(initial_uv.x * 2.0, initial_uv.y + t));
        finalColor = vec4<f32>(vbiRGB + (vbiHum * 0.02), 1.0);
    }

    // Localized color degradation and bleach fading from magnetic tape wear
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
