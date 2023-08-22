use math::{Mat4, Vec2, Vec3, Vec4};
use pipeline::BindType;

// NOTE: These must match the bit flags in bevy_pbr/src/render/pbr_types.wgsl!
bitflags::bitflags! {
  /// Bitflags info about the material a shader is currently rendering.
  /// This is accessible in the shader in the [`StandardMaterialUniform`]
  #[repr(transparent)]
  pub struct StandardMaterialFlags: u32 {
      const BASE_COLOR_TEXTURE         = (1 << 0);
      const EMISSIVE_TEXTURE           = (1 << 1);
      const METALLIC_ROUGHNESS_TEXTURE = (1 << 2);
      const OCCLUSION_TEXTURE          = (1 << 3);
      const DOUBLE_SIDED               = (1 << 4);
      const UNLIT                      = (1 << 5);
      const TWO_COMPONENT_NORMAL_MAP   = (1 << 6);
      const FLIP_NORMAL_MAP_Y          = (1 << 7);
      const FOG_ENABLED                = (1 << 8);
      const DEPTH_MAP                  = (1 << 9); // Used for parallax mapping
      const ALPHA_MODE_RESERVED_BITS   = (Self::ALPHA_MODE_MASK_BITS << Self::ALPHA_MODE_SHIFT_BITS); // ← Bitmask reserving bits for the `AlphaMode`
      const ALPHA_MODE_OPAQUE          = (0 << Self::ALPHA_MODE_SHIFT_BITS);                          // ← Values are just sequential values bitshifted into
      const ALPHA_MODE_MASK            = (1 << Self::ALPHA_MODE_SHIFT_BITS);                          //   the bitmask, and can range from 0 to 7.
      const ALPHA_MODE_BLEND           = (2 << Self::ALPHA_MODE_SHIFT_BITS);                          //
      const ALPHA_MODE_PREMULTIPLIED   = (3 << Self::ALPHA_MODE_SHIFT_BITS);                          //
      const ALPHA_MODE_ADD             = (4 << Self::ALPHA_MODE_SHIFT_BITS);                          //   Right now only values 0–5 are used, which still gives
      const ALPHA_MODE_MULTIPLY        = (5 << Self::ALPHA_MODE_SHIFT_BITS);                          // ← us "room" for two more modes without adding more bits
      const NONE                       = 0;
      const UNINITIALIZED              = 0xFFFF;
  }
}

impl StandardMaterialFlags {
    const ALPHA_MODE_MASK_BITS: u32 = 0b111;
    const ALPHA_MODE_SHIFT_BITS: u32 = 32 - Self::ALPHA_MODE_MASK_BITS.count_ones();
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewUniform {
    //视图投影
    pub view_proj: Mat4,
    pub inverse_view_porj: Mat4,
    // 视图矩阵
    pub view: Mat4,
    // 视图变换矩阵
    pub inverse_view: Mat4,
    // 投影矩阵
    pub projectiton: Mat4,
    pub inverse_projection: Mat4,
    // 相机的世界坐标
    pub world_position: Vec3,
    // 相机视窗
    pub viewport: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StandardMaterialUniform {
    /// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything
    /// in between.
    pub base_color: Vec4,
    // Use a color for user friendliness even though we technically don't use the alpha channel
    // Might be used in the future for exposure correction in HDR
    pub emissive: Vec4,
    /// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
    /// Defaults to `0.5`.
    pub perceptual_roughness: f32,
    /// From [0.0, 1.0], dielectric to pure metallic
    pub metallic: f32,
    /// Specular intensity for non-metals on a linear scale of [0.0, 1.0]
    /// defaults to 0.5 which is mapped to 4% reflectance in the shader
    pub reflectance: f32,
    /// The [`StandardMaterialFlags`] accessible in the `wgsl` shader.
    pub flags: u32,
}
impl Default for StandardMaterialUniform {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
            emissive: Vec4::new(0., 0., 0., 1.),
            perceptual_roughness: 0.5,
            metallic: 0.0,
            reflectance: 0.5,
            flags: StandardMaterialFlags::ALPHA_MODE_OPAQUE.bits(),
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshUniform {
    pub model: Mat4,
    pub inverse_transpose_model: Mat4,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniform {
    pub color_inverse_square_range: Vec4,
    pub position_radius: Vec4,
}

impl From<BindType> for PointLightUniform {
    fn from(value: BindType) -> Self {
        match value {
            BindType::Uniform(v) => unsafe { *(v.as_ptr() as *const PointLightUniform) },
            _ => panic!("wrong format corresponding"),
        }
    }
}
impl From<PointLightUniform> for BindType {
    fn from(value: PointLightUniform) -> Self {
        BindType::Uniform(bytemuck::cast_slice(&[value]).to_vec())
    }
}

impl From<BindType> for ViewUniform {
    fn from(value: BindType) -> Self {
        match value {
            BindType::Uniform(v) => unsafe { *(v.as_ptr() as *const ViewUniform) },
            _ => panic!("wrong format corresponding"),
        }
    }
}
impl From<ViewUniform> for BindType {
    fn from(value: ViewUniform) -> Self {
        BindType::Uniform(bytemuck::cast_slice(&[value]).to_vec())
    }
}
impl From<BindType> for StandardMaterialUniform {
    fn from(value: BindType) -> Self {
        match value {
            BindType::Uniform(v) => unsafe { *(v.as_ptr() as *const StandardMaterialUniform) },
            _ => panic!("wrong format corresponding"),
        }
    }
}
impl From<StandardMaterialUniform> for BindType {
    fn from(value: StandardMaterialUniform) -> Self {
        BindType::Uniform(bytemuck::cast_slice(&[value]).to_vec())
    }
}

impl From<BindType> for MeshUniform {
    fn from(value: BindType) -> Self {
        match value {
            BindType::Uniform(v) => unsafe { *(v.as_ptr() as *const MeshUniform) },
            _ => panic!("wrong format corresponding"),
        }
    }
}
impl From<MeshUniform> for BindType {
    fn from(value: MeshUniform) -> Self {
        BindType::Uniform(bytemuck::cast_slice(&[value]).to_vec())
    }
}
