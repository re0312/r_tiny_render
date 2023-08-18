use math::{Mat4, Vec2, Vec3, Vec4};
use pipeline::BindType;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewUniform {
    //视图投影
    pub view_proj: Mat4,
    pub inverse_view_porj: Mat4,
    // 视图矩阵
    pub view: Mat4,
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
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct MeshUniform {
    model: Mat4,
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
