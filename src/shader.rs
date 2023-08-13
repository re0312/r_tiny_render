use bytemuck::NoUninit;

use crate::{
    bind_group::BindGroup,
    math::{Vec2, Vec3, Vec4},
};

#[derive(Clone, Copy)]
pub enum ShaderType {
    F32(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}
unsafe impl NoUninit for ShaderType {}
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

pub struct VertexOutPut {
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
pub type VertexShader = fn(VertexInput, &BindGroups) -> VertexOutPut;
pub type FragmentShader = fn(FragmentInput, &BindGroups) -> FragmentOutput;
