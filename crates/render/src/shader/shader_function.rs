use math::{Vec2, Vec3, Vec4};
use pipeline::{FragmentInput, FragmentOutput, ShaderType, VertexInput, VertexOutput};

use super::shader_type::MeshVertexOutput;

//  struct MeshVertexOutput {
//   // this is `clip position` when the struct is used as a vertex stage output
//   // and `frag coord` when used as a fragment stage input
//   @builtin(position) position: vec4<f32>,
//   @location(0) world_position: vec4<f32>,
//   @location(1) world_normal: vec3<f32>,
//   #ifdef VERTEX_UVS
//   @location(2) uv: vec2<f32>,
//   #endif
//   #ifdef VERTEX_TANGENTS
//   @location(3) world_tangent: vec4<f32>,
//   #endif
//   #ifdef VERTEX_COLORS
//   @location(4) color: vec4<f32>,
//   #endif
//   #ifdef VERTEX_OUTPUT_INSTANCE_INDEX
//   @location(5) instance_index: u32,
//   #endif
// }
pub fn construct_vertex_output(mesh_vertex_output: &MeshVertexOutput) -> VertexOutput {
    let mut out = VertexOutput {
        position: Vec4::ONE,
        location: vec![ShaderType::Vec4(Vec4::ZERO); 5],
    };
    out.position = mesh_vertex_output.position;
    out.location[0] = mesh_vertex_output.world_position.into();
    out.location[1] = mesh_vertex_output.world_normal.into();
    out.location[2] = mesh_vertex_output.uv.into();

    out
}

pub fn construct_fragment_stage_mesh_input(input: &FragmentInput) -> MeshVertexOutput {
    MeshVertexOutput {
        position: input.position,
        world_position: input.location[0].into(),
        world_normal: input.location[1].into(),
        uv: input.location[2].into(),
        world_tangent: input.location[3].into(),
    }
}

pub fn contruct_fragment_output(in_color: Vec4) -> FragmentOutput {
    FragmentOutput {
        frag_depth: None,
        sample_mask: 0,
        location: vec![ShaderType::Vec4(in_color)],
    }
}
