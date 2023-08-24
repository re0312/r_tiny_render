use crate::{
    pbr_function::{apply_normal_mapping, pbr},
    shader_function::{
        construct_fragment_stage_mesh_input, construct_vertex_output, contruct_fragment_output,
    },
    shader_type::{MeshVertexOutput, PbrInput},
    shader_uniform::{
        MeshUniform, PointLightUniform, StandardMaterialFlags, StandardMaterialUniform, ViewUniform,
    },
};
use math::{Vec2, Vec3, Vec4};
use pipeline::{
    texture_sample, BindGroup, FragmentInput, FragmentOutput, Sampler, Texture, VertexInput,
    VertexOutput,
};

pub fn pbr_vertex_main(vertex_input: VertexInput, bind_groups: &mut Vec<BindGroup>) -> VertexOutput {
    let in_position: Vec3 = vertex_input.location[0].into();
    let in_normal: Vec3 = vertex_input.location[1].into();
    let in_texture_uv: Vec2 = vertex_input.location[2].into();
    // todo 切线接入
    let in_tangent: Vec4 = vertex_input.location[3].into();

    let view_uniform: ViewUniform = std::mem::take(&mut bind_groups[0][0]).into();
    let light_uniform: PointLightUniform = std::mem::take(&mut bind_groups[0][1]).into();
    let material_uniform: StandardMaterialUniform = std::mem::take(&mut bind_groups[1][0]).into();
    let texture: Texture = std::mem::take(&mut bind_groups[1][1]).into();
    let sampler: Sampler = std::mem::take(&mut bind_groups[1][2]).into();
    let mesh_uniform: MeshUniform = std::mem::take(&mut bind_groups[2][0]).into();

    // println!("P{:?}", light_uniform);
    let clip_position = view_uniform.view_proj * in_position.extend(1.);

    let world_position = mesh_uniform.model * in_position.extend(1.);
    // 法线世界坐标需要用模型变换的逆转置矩阵
    let world_normal = (mesh_uniform.inverse_transpose_model.to_mat3() * in_normal).normalize();
    let world_tangent = (mesh_uniform.model.to_mat3() * in_tangent.xyz()).extend(in_tangent.w);
    let out = MeshVertexOutput {
        position: clip_position,
        world_position,
        world_normal,
        uv: in_texture_uv,
        world_tangent,
    };

    bind_groups[0][0] = view_uniform.into();
    bind_groups[0][1] = light_uniform.into();
    bind_groups[1][0] = material_uniform.into();
    bind_groups[1][1] = texture.into();
    bind_groups[1][2] = sampler.into();
    bind_groups[2][0] = mesh_uniform.into();

    construct_vertex_output(&out)
}

pub fn pbr_fragment_main(input: FragmentInput, bind_groups: &mut Vec<BindGroup>) -> FragmentOutput {
    let fragment_in = construct_fragment_stage_mesh_input(&input);

    let view_uniform: ViewUniform = std::mem::take(&mut bind_groups[0][0]).into();
    let light_uniform: PointLightUniform = std::mem::take(&mut bind_groups[0][1]).into();
    let material_uniform: StandardMaterialUniform = std::mem::take(&mut bind_groups[1][0]).into();
    let base_color_texture: Texture = std::mem::take(&mut bind_groups[1][1]).into();
    let base_color_sampler: Sampler = std::mem::take(&mut bind_groups[1][2]).into();
    let emissive_texture: Texture = std::mem::take(&mut bind_groups[1][3]).into();
    let emissive_sampler: Sampler = std::mem::take(&mut bind_groups[1][4]).into();
    let metallic_roughness_texture: Texture = std::mem::take(&mut bind_groups[1][5]).into();
    let metallic_roughness_sampler: Sampler = std::mem::take(&mut bind_groups[1][6]).into();
    let normal_map_texture: Texture = std::mem::take(&mut bind_groups[1][7]).into();
    let normal_map_sampler: Sampler = std::mem::take(&mut bind_groups[1][8]).into();

    let mut output_color = material_uniform.base_color;
    output_color =
        texture_sample(&base_color_texture, &base_color_sampler, fragment_in.uv) * output_color;

    let mut pbr_input = PbrInput::default();

    pbr_input.material.base_color = output_color;
    pbr_input.material.reflectance = material_uniform.reflectance;
    pbr_input.material.flags = material_uniform.flags;

    let mut emissive = material_uniform.emissive;
    let mut metallic = material_uniform.metallic;
    let mut perceptual_roughness = material_uniform.perceptual_roughness;
    if material_uniform.flags & StandardMaterialFlags::EMISSIVE_TEXTURE.bits() != 0 {
        emissive = (emissive.xyz()
            * texture_sample(&emissive_texture, &emissive_sampler, fragment_in.uv).xyz())
        .extend(1.)
    }

    if material_uniform.flags & StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE.bits() != 0 {
        let metallic_roughness = texture_sample(
            &metallic_roughness_texture,
            &metallic_roughness_sampler,
            fragment_in.uv,
        );
        metallic = metallic * metallic_roughness.z;
        perceptual_roughness = perceptual_roughness * metallic_roughness.y;
    }
    pbr_input.material.emissive = emissive;
    pbr_input.material.metallic = metallic;
    pbr_input.material.perceptual_roughness = perceptual_roughness;
    // pbr_input.occlusion todo
    pbr_input.frag_coord = fragment_in.position;
    pbr_input.world_position = fragment_in.world_position;
    pbr_input.world_normal = fragment_in.world_normal;
    pbr_input.V = (view_uniform.world_position - fragment_in.world_position.xyz()).normalize();
    pbr_input.N = apply_normal_mapping(
        fragment_in.world_normal,
        fragment_in.world_tangent,
        fragment_in.uv,
        &normal_map_texture,
        &normal_map_sampler,
    );
    let output_color = pbr(pbr_input, &light_uniform);
    bind_groups[0][0] = view_uniform.into();
    bind_groups[0][1] = light_uniform.into();
    bind_groups[1][0] = material_uniform.into();
    bind_groups[1][1] = base_color_texture.into();
    bind_groups[1][2] = base_color_sampler.into();
    bind_groups[1][3] = emissive_texture.into();
    bind_groups[1][4] = emissive_sampler.into();
    bind_groups[1][5] = metallic_roughness_texture.into();
    bind_groups[1][6] = metallic_roughness_sampler.into();
    bind_groups[1][7] = normal_map_texture.into();
    bind_groups[1][8] = normal_map_sampler.into();
    contruct_fragment_output(output_color)
}
