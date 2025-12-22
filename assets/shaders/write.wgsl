@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let size = textureDimensions(output_texture);
    let coords = vec2<i32>(invocation_id.xy);

    if (coords.x >= i32(size.x) || coords.y >= i32(size.y)) {
        return;
    }

    let color = vec4<f32>(
        f32(coords.x) / f32(size.x),
        f32(coords.y) / f32(size.y),
        1.0,
        1.0
    );

    textureStore(output_texture, coords, color);
}