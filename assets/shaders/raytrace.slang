import generic;
import voxel;

struct DispatchParams {
    VoxelTree scene;
    float4x4 invViewProj;
    float4 cameraOrigin;
    uint treeScale;
    ImageHandle2D<float4> out;
};

[[vk::push_constant]] DispatchParams pc;

static const int MAX_STEPS = 256;

float3 viridis(float t) {

    const float3 c0 = float3(0.2777273272234177, 0.005407344544966578, 0.3340998053353061);
    const float3 c1 = float3(0.1050930431085774, 1.404613529898575, 1.384590162594685);
    const float3 c2 = float3(-0.3308618287255563, 0.214847559468213, 0.09509516302823659);
    const float3 c3 = float3(-4.634230498983486, -5.799100973351585, -19.33244095627987);
    const float3 c4 = float3(6.228269936347081, 14.17993336680509, 56.69055260068105);
    const float3 c5 = float3(4.776384997670288, -13.74514537774601, -65.35303263337234);
    const float3 c6 = float3(-5.435455855934631, 4.645852612178535, 26.3124352495832);

    return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));

}

float3 plasma(float t) {

    const float3 c0 = float3(0.05873234392399702, 0.02333670892565664, 0.5433401826748754);
    const float3 c1 = float3(2.176514634195958, 0.2383834171260182, 0.7539604599784036);
    const float3 c2 = float3(-2.689460476458034, -7.455851135738909, 3.110799939717086);
    const float3 c3 = float3(6.130348345893603, 42.3461881477227, -28.51885465332158);
    const float3 c4 = float3(-11.10743619062271, -82.66631109428045, 60.13984767418263);
    const float3 c5 = float3(10.02306557647065, 71.41361770095349, -54.07218655560067);
    const float3 c6 = float3(-3.658713842777788, -22.93153465461149, 18.19190778539828);

    return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));

}

float3 magma(float t) {

    const float3 c0 = float3(-0.002136485053939582, -0.000749655052795221, -0.005386127855323933);
    const float3 c1 = float3(0.2516605407371642, 0.6775232436837668, 2.494026599312351);
    const float3 c2 = float3(8.353717279216625, -3.577719514958484, 0.3144679030132573);
    const float3 c3 = float3(-27.66873308576866, 14.26473078096533, -13.64921318813922);
    const float3 c4 = float3(52.17613981234068, -27.94360607168351, 12.94416944238394);
    const float3 c5 = float3(-50.76852536473588, 29.04658282127291, 4.23415299384598);
    const float3 c6 = float3(18.65570506591883, -11.48977351997711, -5.601961508734096);

    return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));

}

float3 inferno(float t) {

    const float3 c0 = float3(0.0002189403691192265, 0.001651004631001012, -0.01948089843709184);
    const float3 c1 = float3(0.1065134194856116, 0.5639564367884091, 3.932712388889277);
    const float3 c2 = float3(11.60249308247187, -3.972853965665698, -15.9423941062914);
    const float3 c3 = float3(-41.70399613139459, 17.43639888205313, 44.35414519872813);
    const float3 c4 = float3(77.162935699427, -33.40235894210092, -81.80730925738993);
    const float3 c5 = float3(-71.31942824499214, 32.62606426397723, 73.20951985803202);
    const float3 c6 = float3(25.13112622477341, -12.24266895238567, -23.07032500287172);

    return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));

}



[shader("compute")]
[numthreads(8, 8)]
void main(uint2 screenPos : SV_DispatchThreadID) {
    if (any(screenPos >= pc.out.size)) return;

    float3 rayPos, rayDir;
    getPrimaryRay(screenPos, rayPos, rayDir);

    float scale = 1.0 / (1 << pc.treeScale);
    float3 origin = float3(0) * scale + rayPos * scale + 1.0; 

    HitInfo hit = pc.scene.raycast(origin, rayDir);
    float3 albedo = float3(0.53,0.81,0.98);
    if (!hit.miss) {
        albedo = hit.normal * 0.5 + 0.5;
    }


    
    pc.out.store(screenPos, float4(albedo, 1));
}

void getPrimaryRay(int2 screenPos, out float3 rayPos, out float3 rayDir) {
    float2 uv = (screenPos + 0.5) / float2(pc.out.size);
    uv = uv * 2 - 1;
    float4 world = mul(pc.invViewProj, float4(uv, 1, 1));
    float3 worldPos = world.xyz / world.w;

    rayPos = pc.cameraOrigin.xyz;
    rayDir = normalize(worldPos - rayPos);
}


public struct VoxelTree {
    Node* nodePool;
    uint* leafData;
    Material* palette;

    public HitInfo raycast(float3 origin, float3 dir) {
        HitInfo hit;
        hit.materialid = 0;
        hit.normal = float3(0);
        hit.pos = float3(0);

        uint stack[11];
        int scaleExp = 21;
        
        uint nodeIdx = 0;
        Node node = nodePool[nodeIdx];

        float3 invDir = 1.0f / -abs(dir);
        uint mirrorMask = 0;

        if (dir.x > 0) mirrorMask |= 3 << 0;
        if (dir.y > 0) mirrorMask |= 3 << 4;
        if (dir.z > 0) mirrorMask |= 3 << 2;

        origin = GetMirroredPos(origin, dir, true);
        float3 pos = clamp(origin, 1.0f, 1.9999999f);
        
        float3 sideDist;
        int childIdx;
        int i;
        bool skipNextHit = true;

        if (any(pos != origin)) {
            float3 t0 = (float3(2.0f) - origin) * invDir;
            float3 t1 = (float3(1.0f) - origin) * invDir;
            float tmin = max(max(t0.x, t0.y), max(t0.z, 0.0f));
            float tmax = min(min(t1.x, t1.y), t1.z);

            pos = clamp(origin - abs(dir) * tmin, 1.0f, 1.9999999f);
            sideDist = -t0; 

            skipNextHit = tmin >= tmax;
        }

        for (i = 0; i < MAX_STEPS; i++) {
        
            childIdx = GetNodeCellIndex(pos, scaleExp) ^ mirrorMask;

            while ((node.PopMask >> childIdx & 1) != 0 && !node.IsLeaf && scaleExp >= 2) {
                stack[scaleExp >> 1] = nodeIdx;
                nodeIdx = node.ChildPtr + popcnt_var64(node.PopMask, childIdx);
                node = nodePool[nodeIdx];

                scaleExp -= 2;
                childIdx = GetNodeCellIndex(pos, scaleExp) ^ mirrorMask;
            }
            if ((node.PopMask >> childIdx & 1) != 0 && node.IsLeaf && !skipNextHit) break;

            int advScaleExp = scaleExp;

            if ((node.PopMask >> (childIdx & 0b101010) & 0x00330033) == 0) advScaleExp++;

            float3 edgePos = FloorScale(pos, advScaleExp);
            
            sideDist = (edgePos - origin) * invDir;
            float tmax = min(min(sideDist.x, sideDist.y), sideDist.z);
            
            int3 maxSiblBounds = asint(edgePos) + select(sideDist == tmax, -1, (1 << advScaleExp) - 1);
            pos = min(origin - abs(dir) * tmax, asfloat(maxSiblBounds));

            uint3 diffPos = asuint(pos) ^ asuint(edgePos);
            int diffExp = firstbithigh((diffPos.x | diffPos.y | diffPos.z) & 0xFFAAAAAA); 
            
            if (diffExp > scaleExp) {
                scaleExp = diffExp;
                if (diffExp > 21) break;

                nodeIdx = stack[scaleExp >> 1];
                node = nodePool[nodeIdx];
            }
            skipNextHit = false;
        }
        
        if (node.IsLeaf && scaleExp <= 21) {
            pos = GetMirroredPos(pos, dir, false);
            hit.materialid = leafData[node.ChildPtr + popcnt_var64(node.PopMask, childIdx)];
            hit.pos = pos;

            float tmax = min(min(sideDist.x, sideDist.y), sideDist.z);
            bool3 sideMask = tmax >= sideDist;
            hit.normal = select(sideMask, -sign(dir), 0.0);

        }
        hit.steps = i;
        return hit;
    }

    static float3 GetMirroredPos(float3 pos, float3 dir, bool rangeCheck) {
        float3 mirrored = asfloat(asuint(pos) ^ 0x7FFFFF);
        if (rangeCheck && any(pos < 1.0 || pos >= 2.0)) mirrored = 3.0 - pos;
        return select(dir > 0, mirrored, pos);
    }
    
    static int GetNodeCellIndex(float3 pos, int scale_exp) {
        uint3 cellPos = asuint(pos) >> scale_exp & 3;
        return cellPos.x + cellPos.z * 4 + cellPos.y * 16;
    }
    
    static float3 FloorScale(float3 pos, int scale_exp) {
        uint mask = ~0u << scale_exp;
        return asfloat(asuint(pos) & mask);
    }
    
    static uint popcnt_var64(uint64_t mask, uint width) {
        uint himask = uint(mask >> 32);
        uint lomask = uint(mask);
        
        uint count = 0;
        if (width >= 32) {
            count = countbits(lomask);
            uint m = 1u << (width & 31u);
            count += countbits(himask & (m - 1u));
        } else {
             uint m = 1u << width;
             count = countbits(lomask & (m - 1u));
        }
        return count;
    }
}
