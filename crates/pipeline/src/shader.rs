use math::{Vec2, Vec3, Vec4};

use crate::bind_group::BindGroup;

#[derive(Clone, Copy)]
pub enum ShaderType {
    F32(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}
impl From<Vec4> for ShaderType {
    fn from(value: Vec4) -> Self {
        ShaderType::Vec4(value)
    }
}
impl From<ShaderType> for Vec4 {
    fn from(value: ShaderType) -> Self {
        match value {
            ShaderType::Vec4(v) => v,
            _ => Vec4::ZERO,
        }
    }
}
impl From<Vec3> for ShaderType {
    fn from(value: Vec3) -> Self {
        ShaderType::Vec3(value)
    }
}
impl From<ShaderType> for Vec3 {
    fn from(value: ShaderType) -> Self {
        match value {
            ShaderType::Vec3(v) => v,
            _ => Vec3::ZERO,
        }
    }
}
impl From<Vec2> for ShaderType {
    fn from(value: Vec2) -> Self {
        ShaderType::Vec2(value)
    }
}
impl From<ShaderType> for Vec2 {
    fn from(value: ShaderType) -> Self {
        match value {
            ShaderType::Vec2(v) => v,
            _ => Vec2::ZERO,
        }
    }
}
impl TryFrom<ShaderType> for f32 {
    type Error = ();

    fn try_from(value: ShaderType) -> Result<Self, Self::Error> {
        if let ShaderType::F32(v) = value {
            Ok(v)
        } else {
            Err(())
        }
    }
}
// 着色器输入包括 build-in input value（由上游生成，自动传递给着色器） 和 用户自定义的输入
// 按照webgpu标准实施
// https://www.w3.org/TR/WGSL/#built-in-output-value

pub struct VertexInput {
    // build-in input
    pub vertex_index: u32,   // 顶点的索引
    pub instance_index: u32, // 实例化渲染的索引，暂时未支持,这里永远设置为0
    // user-defined input
    // Each input-output location can store a value up to 16 bytes in size
    pub location: Vec<ShaderType>,
}

#[derive(Clone)]
pub struct VertexOutput {
    // build-in
    pub position: Vec4,
    //user-define
    pub location: Vec<ShaderType>,
}

pub struct FragmentInput {
    // build-in
    pub position: Vec4,
    pub sample_index: u32,
    pub sample_mask: u32,
    pub front_facing: bool,
    //user-define
    pub location: Vec<ShaderType>,
}
pub struct FragmentOutput {
    pub frag_depth: f32,
    pub sample_mask: u32,
    pub location: Vec<ShaderType>,
}

type BindGroups = Vec<BindGroup>;
pub type VertexShader = fn(VertexInput, &mut BindGroups) -> VertexOutput;
pub type FragmentShader = fn(FragmentInput, &mut BindGroups) -> FragmentOutput;
