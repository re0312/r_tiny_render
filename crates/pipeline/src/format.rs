#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum VertexFormat {
    // One single-precision float (f32). `float` in shaders.
    Float32,
    /// Two single-precision floats (f32). `vec2` in shaders.
    Float32x2,
    // Three single-precision floats (f32). `vec3` in shaders.
    Float32x3,
    // Four single-precision floats (f32). `vec4` in shaders.
    Float32x4,
    /// One unsigned int (u32). `uint` in shaders.
    Uint32,
}

impl VertexFormat {
    #[inline]
    pub fn size(&self) -> usize {
        match self {
            VertexFormat::Float32 => 4,
            VertexFormat::Float32x2 => 8,
            VertexFormat::Float32x3 => 12,
            VertexFormat::Float32x4 => 16,
            _ => 0,
        }
    }
}

// 按照webgpu标准实施，目前只支持rgba norm 格式
/// Corresponds to [WebGPU `GPUTextureFormat`](
/// https://gpuweb.github.io/gpuweb/#enumdef-gputextureformat).
#[derive(Clone, Copy, Debug, Default)]
pub enum TextureFormat {
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    #[default]
    Rgba8Unorm,
    Rgb8Unorm,
    R8Unorm,
}
impl TextureFormat {
    pub fn size(&self) -> usize {
        match self {
            TextureFormat::Rgba8Unorm => 4,
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rgb8Unorm => 3,
        }
    }
}
