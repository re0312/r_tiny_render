use math::{Vec2, Vec3, Vec4};

use crate::shader_uniform::StandardMaterialUniform;

pub struct MeshVertexOutput {
    pub position: Vec4,
    pub world_position: Vec4,
    pub world_normal: Vec3,
    pub uv: Vec2,
}
pub struct PbrInput {
    pub materail: StandardMaterialUniform,
    pub occlusion: Vec3,
    pub frag_coord: Vec4,
    pub world_position: Vec4,
    pub world_normal: Vec3,
    pub N: Vec3,
    pub V: Vec3,
}

impl Default for PbrInput {
    fn default() -> Self {
        Self {
            materail: StandardMaterialUniform::default(),
            occlusion: Vec3::ONE,
            frag_coord: Vec4 {
                x: 0.,
                y: 0.,
                z: 0.,
                w: 1.,
            },
            world_position: Vec4::ZERO,
            world_normal: Vec3::new(0., 0., 1.),
            N: Vec3::new(0., 0., 1.),
            V: Vec3::new(1., 0., 0.),
        }
    }
}
