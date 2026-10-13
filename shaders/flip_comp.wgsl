struct FlipUniforms {
    flip_horizontal: u32,
    flip_vertical: u32,
    _padding1: u32,
    _padding2: u32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: FlipUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(dst_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    var sample_x = id.x;
    if (uniforms.flip_horizontal == 1u) {
        sample_x = size.x - 1u - id.x;
    }

    var sample_y = id.y;
    if (uniforms.flip_vertical == 1u) {
        sample_y = size.y - 1u - id.y;
    }

    let color = textureLoad(src_tex, vec2<i32>(i32(sample_x), i32(sample_y)), 0);
    textureStore(dst_tex, vec2<i32>(id.xy), color);
}
