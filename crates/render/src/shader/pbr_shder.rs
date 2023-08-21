use math::{Vec2, Vec3, Vec4};
use pipeline::{
    texture_sample, BindGroup, FragmentInput, FragmentOutput, Sampler, Texture, VertexInput,
    VertexOutput,
};

use crate::{
    shader_function::{
        construct_fragment_stage_mesh_input, construct_vertex_output, contruct_fragment_output,
    },
    shader_type::{MeshVertexOutput, PbrInput},
    shader_uniform::{
        PointLightUniform, StandardMaterialFlags, StandardMaterialUniform, ViewUniform,
    },
};

pub fn vertex_main(vertex_input: VertexInput, bind_groups: &mut Vec<BindGroup>) -> VertexOutput {
    let in_position: Vec3 = vertex_input.location[0].into();
    let in_normal: Vec3 = vertex_input.location[1].into();
    let in_texture_uv: Vec2 = vertex_input.location[2].into();

    let view_uniform: ViewUniform = std::mem::take(&mut bind_groups[0][0]).into();
    let light_uniform: PointLightUniform = std::mem::take(&mut bind_groups[0][1]).into();
    let materail_uniform: StandardMaterialUniform = std::mem::take(&mut bind_groups[1][0]).into();
    let texture: Texture = std::mem::take(&mut bind_groups[1][1]).into();
    let sampler: Sampler = std::mem::take(&mut bind_groups[1][2]).into();

    println!("P{:?}", light_uniform);
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
    bind_groups[0][1] = light_uniform.into();
    bind_groups[1][0] = materail_uniform.into();
    bind_groups[1][1] = texture.into();
    bind_groups[1][2] = sampler.into();

    let a = construct_vertex_output(&out);
    construct_vertex_output(&out)
}

pub fn fragment_main(input: FragmentInput, bind_groups: &mut Vec<BindGroup>) -> FragmentOutput {
    let fragment_in = construct_fragment_stage_mesh_input(&input);

    let view_uniform: ViewUniform = std::mem::take(&mut bind_groups[0][0]).into();
    let light_uniform: PointLightUniform = std::mem::take(&mut bind_groups[0][1]).into();
    let materail_uniform: StandardMaterialUniform = std::mem::take(&mut bind_groups[1][0]).into();
    let base_color_texture: Texture = std::mem::take(&mut bind_groups[1][1]).into();
    let base_color_sampler: Sampler = std::mem::take(&mut bind_groups[1][2]).into();
    let emissive_texture: Texture = std::mem::take(&mut bind_groups[1][3]).into();
    let emissive_sampler: Sampler = std::mem::take(&mut bind_groups[1][4]).into();
    let metallic_roughness_texture: Texture = std::mem::take(&mut bind_groups[1][5]).into();
    let metallic_roughness_sampler: Sampler = std::mem::take(&mut bind_groups[1][6]).into();

    let mut output_color = materail_uniform.base_color;
    output_color =
        texture_sample(&base_color_texture, &base_color_sampler, fragment_in.uv) * output_color;
    let mut pbr_input = PbrInput::default();

    pbr_input.materail.base_color = output_color;
    pbr_input.materail.reflectance = materail_uniform.reflectance;
    pbr_input.materail.flags = materail_uniform.flags;

    let mut emissive = materail_uniform.emissive;
    let mut metallic = materail_uniform.metallic;
    let mut perceptual_roughness = materail_uniform.perceptual_roughness;
    if materail_uniform.flags & StandardMaterialFlags::EMISSIVE_TEXTURE.bits() != 0 {
        emissive = (emissive.xyz()
            * texture_sample(&emissive_texture, &emissive_sampler, fragment_in.uv).xyz())
        .extend(1.)
    }

    if materail_uniform.flags & StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE.bits() != 0 {
        let metallic_roughness = texture_sample(
            &metallic_roughness_texture,
            &metallic_roughness_sampler,
            fragment_in.uv,
        );
        metallic = metallic * metallic_roughness.z;
        perceptual_roughness = perceptual_roughness * metallic_roughness.y;
    }
    pbr_input.materail.emissive = emissive;
    pbr_input.materail.metallic = metallic;
    pbr_input.materail.perceptual_roughness = perceptual_roughness;
    // pbr_input.occlusion 暂时不写遮蔽
    pbr_input.frag_coord = fragment_in.position;
    pbr_input.world_position = fragment_in.world_position;
    pbr_input.world_normal = fragment_in.world_normal;

    bind_groups[0][0] = view_uniform.into();
    bind_groups[0][1] = light_uniform.into();
    bind_groups[1][0] = materail_uniform.into();
    bind_groups[1][1] = base_color_texture.into();
    bind_groups[1][2] = base_color_sampler.into();
    bind_groups[1][3] = emissive_texture.into();
    bind_groups[1][4] = emissive_sampler.into();
    bind_groups[1][5] = metallic_roughness_texture.into();
    bind_groups[1][6] = metallic_roughness_sampler.into();
    contruct_fragment_output(Vec4::ONE)
}
