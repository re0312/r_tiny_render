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
    let mut output_color = input.materail.base_color;
    let emissive = input.materail.emissive;
    let metallic = input.materail.metallic;
    let perceptual_roughness = input.materail.perceptual_roughness;
    let roughness = perceptual_roughness.clamp(0.089, 1.) * perceptual_roughness.clamp(0.089, 1.);

    let NdotV = input.N.dot(input.V).max(0.0001);

    output_color
}
