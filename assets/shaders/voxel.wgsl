struct Node {
    packed_data: vec4<u32>,
};

struct Material {
    packed_color: i32,
};

struct HitInfo {
    materialid: i32,
    pos: vec3<f32>,
    normal: vec3<f32>,
    steps: i32,
};

struct DispatchParams {
    inv_view_proj: mat4x4<f32>,
    camera_origin: vec4<f32>,
};

@group(0) @binding(0) var<uniform> pc: DispatchParams;
@group(0) @binding(1) var<storage, read> nodePool: array<Node>;
@group(0) @binding(2) var<storage, read> leafData: array<u32>;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;

fn is_leaf(node: Node) -> bool {
    return (node.packed_data[0] & 1u) != 0u;
}

fn child_ptr(node: Node) -> u32 {
    return node.packed_data[0] >> 2u;
}

fn check_pop_mask(node: Node, bitIdx: u32) -> bool {
    if (bitIdx >= 32u) {
        return ((node.packed_data[2] >> (bitIdx - 32u)) & 1u) != 0u;
    }
    return ((node.packed_data[1] >> bitIdx) & 1u) != 0u;
}

const MAX_STEPS: i32 = 256;

fn viridis(t: f32) -> vec3<f32> {
    let c0 = vec3(0.2777273272234177, 0.005407344544966578, 0.3340998053353061);
    let c1 = vec3(0.1050930431085774, 1.404613529898575, 1.384590162594685);
    let c2 = vec3(-0.3308618287255563, 0.214847559468213, 0.09509516302823659);
    let c3 = vec3(-4.634230498983486, -5.799100973351585, -19.33244095627987);
    let c4 = vec3(6.228269936347081, 14.17993336680509, 56.69055260068105);
    let c5 = vec3(4.776384997670288, -13.74514537774601, -65.35303263337234);
    let c6 = vec3(-5.435455855934631, 4.645852612178535, 26.3124352495832);
    return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));
}

fn get_mirrored_pos(pos: vec3<f32>, dir: vec3<f32>, rangeCheck: bool) -> vec3<f32> {
    var mirrored = bitcast<vec3<f32>>(bitcast<vec3<u32>>(pos) ^ vec3<u32>(0x7FFFFFu));
    if (rangeCheck && (any(pos < vec3(1.0)) || any(pos >= vec3(2.0)))) {
        mirrored = vec3(3.0) - pos;
    }
    return select(pos, mirrored, dir > vec3(0.0));
}

fn get_node_cell_index(pos: vec3<f32>, scale_exp: i32) -> u32 {
    let shift = vec3<u32>(u32(scale_exp));
    let cell_pos: vec3<u32> =
        (bitcast<vec3<u32>>(pos) >> shift) & vec3<u32>(3u);

    return cell_pos.x + cell_pos.z * 4u + cell_pos.y * 16u;
}


fn floor_scale(pos: vec3<f32>, scale_exp: i32) -> vec3<f32> {
    let mask = 0xFFFFFFFFu << u32(scale_exp);
    return bitcast<vec3<f32>>(bitcast<vec3<u32>>(pos) & vec3<u32>(mask));
}

fn popcnt_var64(node: Node, width: u32) -> u32 {
    if (width >= 32u) {
        let count = countOneBits(node.packed_data[1]);
        let m = (1u << (width & 31u)) - 1u;
        return count + countOneBits(node.packed_data[2] & m);
    } else {
        let m = (1u << width) - 1u;
        return countOneBits(node.packed_data[1] & m);
    }
}

struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
};

fn get_primary_ray(screenPos: vec2<u32>) -> Ray {
    let tex_size = textureDimensions(out_tex);
    var uv = (vec2<f32>(screenPos) + 0.5) / vec2<f32>(tex_size);
    uv = uv * 2.0 - 1.0;
    let world = pc.inv_view_proj * vec4(uv.x, uv.y, 1.0, 1.0);
    let worldPos = world.xyz / world.w;

    var r: Ray;
    r.pos = pc.camera_origin.xyz;
    r.dir = normalize(worldPos - r.pos);
    return r;
}

fn raycast(origin_in: vec3<f32>, dir: vec3<f32>) -> HitInfo {
    var hit: HitInfo;
    hit.materialid = 0;
    hit.normal = vec3(0.0);
    hit.pos = vec3(0.0);

    var stack: array<u32, 11>;
    var scaleExp: i32 = 21;
    var nodeIdx: u32 = 0u;
    var node = nodePool[nodeIdx];

    let invDir = 1.0 / -abs(dir);
    var mirrorMask: u32 = 0u;

    if (dir.x > 0.0) { mirrorMask |= 3u << 0u; }
    if (dir.y > 0.0) { mirrorMask |= 3u << 4u; }
    if (dir.z > 0.0) { mirrorMask |= 3u << 2u; }

    var origin = get_mirrored_pos(origin_in, dir, true);
    var pos = clamp(origin, vec3(1.0), vec3(1.9999999));

    var sideDist: vec3<f32>;
    var childIdx: u32;
    var skipNextHit = true;

    if (any(pos != origin)) {
        let t0 = (vec3(2.0) - origin) * invDir;
        let t1 = (vec3(1.0) - origin) * invDir;
        let tmin = max(max(t0.x, t0.y), max(t0.z, 0.0));
        let tmax = min(min(t1.x, t1.y), t1.z);
        pos = clamp(origin - abs(dir) * tmin, vec3(1.0), vec3(1.9999999));
        sideDist = -t0;
        skipNextHit = tmin >= tmax;
    }

    for (var i = 0; i < MAX_STEPS; i++) {
        childIdx = get_node_cell_index(pos, scaleExp) ^ mirrorMask;

        while (check_pop_mask(node, childIdx) && !is_leaf(node) && scaleExp >= 2) {
            stack[scaleExp >> 1] = nodeIdx;
            nodeIdx = child_ptr(node) + popcnt_var64(node, childIdx);
            node = nodePool[nodeIdx];
            scaleExp -= 2;
            childIdx = get_node_cell_index(pos, scaleExp) ^ mirrorMask;
        }

        if (check_pop_mask(node, childIdx) && is_leaf(node) && !skipNextHit) {
            break;
        }

        var advScaleExp = scaleExp;
        if ((node.packed_data[1] >> (childIdx & 0x2Au) & 0x00330033u) == 0u &&
            (node.packed_data[2] >> (childIdx & 0x2Au) & 0x00330033u) == 0u) {
            advScaleExp += 1;
        }

        let edgePos = floor_scale(pos, advScaleExp);

        sideDist = (edgePos - origin) * invDir;
        let tmax = min(min(sideDist.x, sideDist.y), sideDist.z);

        let maxSiblBounds = bitcast<vec3<i32>>(edgePos) + select(vec3<i32>((1u << u32(advScaleExp)) - 1), vec3<i32>(-1), sideDist == vec3(tmax));
        pos = min(origin - abs(dir) * tmax, bitcast<vec3<f32>>(maxSiblBounds));

        let diffPos = bitcast<vec3<u32>>(pos) ^ bitcast<vec3<u32>>(edgePos);
        let diffExp = i32(firstLeadingBit((diffPos.x | diffPos.y | diffPos.z) & 0xFFAAAAAAu));

        if (diffExp > scaleExp) {
            scaleExp = diffExp;
            if (diffExp > 21) { break; }
            nodeIdx = stack[scaleExp >> 1];
            node = nodePool[nodeIdx];
        }
        skipNextHit = false;
        hit.steps = i;
    }

    if (is_leaf(node) && scaleExp <= 21) {
        pos = get_mirrored_pos(pos, dir, false);
        hit.materialid = i32(leafData[child_ptr(node) + popcnt_var64(node, childIdx)]);
        hit.pos = pos;
        let tmax = min(min(sideDist.x, sideDist.y), sideDist.z);
        hit.normal = select(vec3(0.0), -sign(dir), sideDist <= vec3(tmax));
    }
    return hit;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) screenPos: vec3<u32>) {
    let tex_size = textureDimensions(out_tex);
    if (any(screenPos.xy >= tex_size)) { return; }

    let ray = get_primary_ray(screenPos.xy);
    let scale = 1.0 / f32(1u << u32(pc.camera_origin.w));
    let origin = vec3(0.0) * scale + ray.pos * scale + 1.0;
    let hit = raycast(origin, ray.dir);
    var albedo = vec3(0.53, 0.81, 0.98);
//    if (!hit.miss) {
//        albedo = hit.normal * 0.5 + 0.5;
//    }
    albedo = viridis((f32(hit.steps) / 50));
   textureStore(out_tex, screenPos.xy, vec4(albedo, 1.0));
}