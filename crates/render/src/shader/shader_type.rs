use math::{Vec2, Vec4};

pub struct MeshVertexOutput {
    pub position: Vec4,
    pub world_position: Vec4,
    pub world_normal: Vec4,
    pub uv: Vec2,
}
