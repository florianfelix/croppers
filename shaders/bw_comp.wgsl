struct BwUniforms {
    select: i32,
    _padding1: i32,
    _padding2: i32,
    _padding3: i32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: BwUniforms;

// Standard Rec. 709 luminance weights for perceived brightness grayscale
const LUMINANCE_VECTOR = vec3<f32>(0.2126f, 0.7152f, 0.0722f);

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);

    // Bounds check: discard invocations outside destination texture
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    // Read input pixel color
    let color = textureLoad(src_tex, vec2<i32>(id.xy), 0);

    var grey: f32;

    // Extract selected color channel (0=R, 1=G, 2=B) or compute luminance as fallback
    switch (uniforms.select) {
        case 0: {
            grey = color.r;
        }
        case 1: {
            grey = color.g;
        }
        case 2: {
            grey = color.b;
        }
        default: {
            grey = dot(color.rgb, LUMINANCE_VECTOR);
        }
    }

    // Assign grayscale value across RGB channels while preserving original alpha
    let final_rgb = vec3<f32>(grey);
    textureStore(dst_tex, vec2<i32>(id.xy), vec4<f32>(final_rgb, color.a));
}
