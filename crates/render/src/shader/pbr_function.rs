use math::{Vec2, Vec3, Vec4};
use pipeline::{texture_sample, Sampler, Texture};

use crate::shader_type::PbrInput;

pub fn apply_normal_mapping(
    world_normal: Vec3,
    world_tangent: Vec4,
    uv: Vec2,
    texture: &Texture,
    _sampler: &Sampler,
) -> Vec3 {
    // 切线空间计算，按照 mikktspace 标准
    let mut N = world_normal;
    let T = world_tangent.xyz();
    let B = world_tangent.w * N.cross(T);
    let mut Nt = texture_sample(texture, _sampler, uv).xyz();
    Nt = Nt * 2. - 1.;
    Nt.x * T + Nt.y * B + Nt.z * N
}

pub fn pbr(input: PbrInput) -> Vec4 {
    let mut output_color = input.material.base_color;
    let emissive = input.material.emissive;
    let metallic = input.material.metallic;
    let perceptual_roughness = input.material.perceptual_roughness;
    let roughness = perceptual_roughness.clamp(0.089, 1.) * perceptual_roughness.clamp(0.089, 1.);

    let NdotV = input.N.dot(input.V).max(0.0001);

    let reflectance = input.material.reflectance;
    let F0 = output_color.xyz() * metallic + 0.16 * reflectance * reflectance * (1.0 - metallic);

    // 散射强度和金属性负相关
    let diffuse_color = output_color.xyz() * (1.0 - metallic);

    // 计算反射方向
    let R = -input.V - 2. * input.N.dot(input.V) * input.N;

    output_color
}
