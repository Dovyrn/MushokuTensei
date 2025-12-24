module voxel;

public struct Node {
    uint packedData[4];

    public property bool IsLeaf {
        get {
            return (packedData[0] & 1) != 0;
        }
    }

    public property uint ChildPtr {
        get {
            return packedData[0] >> 2;
        }
    }

    public property uint64_t PopMask {
        get {
            return uint64_t(packedData[1]) | (uint64_t(packedData[2]) << 32);
        }
    }
}



public struct Material {
    public int packedColor;

    public property float3 color {
        get {
            float r = float((packedColor >> 16) & 0xFF) / 255.0;
            float g = float((packedColor >> 8) & 0xFF) / 255.0;
            float b = float(packedColor & 0xFF) / 255.0;
            return float3(r, g, b);
        }
    }
}

public struct HitInfo {
    public int materialid;
    public float3 pos;
    public float3 normal;
    public int steps;

    public property bool miss {
        get {
            return materialid == 0;
        }
    }
}