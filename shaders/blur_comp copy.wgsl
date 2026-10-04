struct BlurUniforms {
    radius: f32,
    _padding0: f32,
    _padding1: f32,
    _padding2: f32,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: BlurUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    if (global_id.x >= dims.x || global_id.y >= dims.y) {
        return;
    }

    let coords = vec2<i32>(global_id.xy);
    let r = i32(uniforms.radius);

    if (r <= 0) {
        let color = textureLoad(input_texture, coords, 0);
        textureStore(output_texture, coords, color);
        return;
    }

    var color_acc = vec4<f32>(0.0);
    var count = 0.0;

    for (var x = -r; x <= r; x++) {
        for (var y = -r; y <= r; y++) {
            let sample_pos = coords + vec2<i32>(x, y);
            if (sample_pos.x >= 0 && sample_pos.x < i32(dims.x) && sample_pos.y >= 0 && sample_pos.y < i32(dims.y)) {
                color_acc += textureLoad(input_texture, sample_pos, 0);
                count += 1.0;
            }
        }
    }

    textureStore(output_texture, coords, color_acc / count);
}
