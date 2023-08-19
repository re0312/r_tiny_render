use math::{Vec2, Vec3};
use pipeline::{
    BindGroup, FragmentInput, FragmentOutput, Sampler, Texture, VertexInput, VertexOutput,
};

use crate::{
    shader_function::{construct_fragment_stage_mesh_input, construct_vertex_output},
    shader_type::MeshVertexOutput,
    shader_uniform::ViewUniform,
};

pub fn vertex_main(vertex_input: VertexInput, bind_groups: &mut Vec<BindGroup>) -> VertexOutput {
    let in_position: Vec3 = vertex_input.location[0].into();
    let in_normal: Vec3 = vertex_input.location[1].into();
    let in_texture_uv: Vec2 = vertex_input.location[2].into();

    let view_uniform: ViewUniform = std::mem::take(&mut bind_groups[0][0]).into();
    let texture: Texture = std::mem::take(&mut bind_groups[1][1]).into();
    let sampler: Sampler = std::mem::take(&mut bind_groups[1][2]).into();

    let clip_position = view_uniform.view_proj * in_position.extend(1.);
    let world_position = view_uniform.inverse_view * in_position.extend(1.);
    let world_normal = view_uniform.inverse_view * in_normal.extend(1.);
    let out = MeshVertexOutput {
        position: clip_position,
        world_position,
        world_normal,
        uv: in_texture_uv,
    };

    bind_groups[0][0] = view_uniform.into();
    bind_groups[1][1] = texture.into();
    bind_groups[1][2] = sampler.into();

    construct_vertex_output(&out)
}

pub fn fragment_main(input: FragmentInput, bind_groups: &mut Vec<BindGroup>) -> FragmentOutput {
    let fragment_in = construct_fragment_stage_mesh_input(&input);
    todo!();
}
