module generic;

public interface IImageDescriptor {
    static const bool isWriteable;
}

public struct ReadableSelector : IImageDescriptor {
    static const bool isWriteable = false;
}

public struct WritableSelector : IImageDescriptor {
    static const bool isWriteable = true;
}

[vk::binding(0, 0)] public __DynamicResource TextureHeap[];
[vk::binding(1, 0)] public __DynamicResource RWTextureHeap[];

public struct ImageHandle<T : __IDynamicResourceCastable<__DynamicResourceKind.General>, S : IImageDescriptor> {
    public uint DescriptorIndex;

    public property bool isNull { 
        get { return DescriptorIndex == ~0; } 
    }

    public property T instance {
        [ForceInline] get { 
            if (S.isWriteable) {
                return RWTextureHeap[DescriptorIndex].as<T>();
            } else {
                return TextureHeap[DescriptorIndex].as<T>();
            }
        }
    }
}

public typealias ImageHandle2D<F: ITexelElement = float4> = ImageHandle<RWTexture2D<F>, WritableSelector>;

public extension ImageHandle<RWTexture2D<float4>, WritableSelector> {
    [ForceInline] 
    public void store(uint2 pos, float4 value) { 
        this.instance[pos] = value; 
    }
    
    public property uint2 size {
        get {
            uint w, h;
            this.instance.GetDimensions(w, h);
            return uint2(w, h);
        }
    }
}