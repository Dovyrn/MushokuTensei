@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(invocation_id.xy);
    let dimensions = textureDimensions(texture);
    
    if (invocation_id.x >= dimensions.x || invocation_id.y >= dimensions.y) {
        return;
    }

    let uv = vec2<f32>(f32(location.x) / f32(dimensions.x), f32(location.y) / f32(dimensions.y));
    
    let color = vec4<f32>(uv.x, uv.y, 0.5 + 0.5 * sin(uv.x * 10.0), 1.0);

    textureStore(texture, location, color);
}
