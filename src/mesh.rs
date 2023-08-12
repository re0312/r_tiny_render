use crate::{
    color::Color,
    material::Material,
    math::{Vec2, Vec3, Vec4},
    transform::Transform,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    // 位置坐标（齐次坐标）
    pub position: Vec4,
    // 法线向量
    pub normal: Vec3,
    // 纹理坐标
    pub texcoord: Vec2,
}
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    // 顶点数据
    pub vertices: Vec<Vertex>,
    pub material: Material,
    pub transform: Transform,
}
