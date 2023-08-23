use std::f32::consts::PI;

use math::{Vec2, Vec3, Vec4};
use pipeline::{texture_sample, Sampler, Texture};

use crate::{shader_type::PbrInput, shader_uniform::PointLightUniform};

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

pub fn F_AB(perceptual_roughness: f32, NoV: f32) -> Vec2 {
    let c0 = Vec4::new(-1.0, -0.0275, -0.572, 0.022);
    let c1 = Vec4::new(1.0, 0.0425, 1.04, -0.04);
    let r = c0 * perceptual_roughness + c1;
    let a004 = (r.x * r.x).min(f32::powf(2., -9.28 * NoV)) * r.x + r.y;
    return Vec2::new(-1.04, 1.04) * a004 + Vec2::new(r.z, r.w);
    // let a004 = min(r.x * r.x, (-9.28 * NoV)) * r.x + r.y;
}

// Disney approximation
// See https://google.github.io/filament/Filament.html#citation-burley12
pub fn Fd_Burley(roughness: f32, NoV: f32, NoL: f32, LoH: f32) -> f32 {
    let f90 = 0.5 + 2.0 * roughness * LoH * LoH;
    let lightScatter = F_Schlick(1.0, f90, NoL);
    let viewScatter = F_Schlick(1.0, f90, NoV);
    return lightScatter * viewScatter * (1.0 / PI);
}
// 微表面法线分布函数 GGX分布模型 高光D项
pub fn D_GGX(roughness: f32, NoH: f32, h: Vec3) -> f32 {
    let oneMinusNoHSquared = 1.0 - NoH * NoH;
    let a = NoH * roughness;
    let k = roughness / (oneMinusNoHSquared + a * a);
    let d = k * k * (1.0 / PI);
    return d;
}
// 微表面遮蔽和阴影 高光G项
fn V_SmithGGXCorrelated(roughness: f32, NoV: f32, NoL: f32) -> f32 {
    let a2 = roughness * roughness;
    let lambdaV = NoL * ((NoV - a2 * NoV) * NoV + a2).sqrt();
    let lambdaL = NoV * ((NoL - a2 * NoL) * NoL + a2).sqrt();
    let v = 0.5 / (lambdaV + lambdaL);
    return v;
}
// 菲涅尔现象
fn fresnel(f0: Vec3, LoH: f32) -> Vec3 {
    // f_90 suitable for ambient occlusion
    // see https://google.github.io/filament/Filament.html#lighting/occlusion
    let f90 = f0.dot(Vec3::splat(50.0 * 0.33)).clamp(0., 1.);
    return F_Schlick_vec(f0, f90, LoH);
}
fn F_Schlick(f0: f32, f90: f32, VoH: f32) -> f32 {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * f32::powf(1.0 - VoH, 5.0);
}
// Fresnel function
// see https://google.github.io/filament/Filament.html#citation-schlick94
// F_Schlick(v,h,f_0,f_90) = f_0 + (f_90 − f_0) (1 − v⋅h)^5
fn F_Schlick_vec(f0: Vec3, f90: f32, VoH: f32) -> Vec3 {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * f32::powf(1.0 - VoH, 5.0);
}

// 光线衰减系数，除了I/d^2,光线应该更加平滑的降到0并且距离不能发生除0错误
// https://google.github.io/filament/Filament.html#mjx-eqn-pointLightLuminousPower
pub fn distance_attenuation(distance_square: f32, inverse_range_squared: f32) -> f32 {
    let factor = distance_square * inverse_range_squared;
    let smoothFactor = (1.0 - factor * factor).clamp(0., 1.);
    let attenuation = smoothFactor * smoothFactor;
    return attenuation * 1.0 / distance_square.max(0.0001);
}
pub fn pbr(input: PbrInput, light_uniform: &PointLightUniform) -> Vec4 {
    let mut output_color = input.material.base_color;
    let emissive = input.material.emissive;
    let metallic = input.material.metallic;
    let perceptual_roughness = input.material.perceptual_roughness;
    let roughness = perceptual_roughness.clamp(0.089, 1.) * perceptual_roughness.clamp(0.089, 1.);

    let NdotV = input.N.dot(input.V).max(0.0001);

    let reflectance = input.material.reflectance;

    // 对于导体和电介质 映射反射率到F0
    let F0 = output_color.xyz() * metallic + 0.16 * reflectance * reflectance * (1.0 - metallic);

    // 散射强度和金属性负相关
    let diffuse_color = output_color.xyz() * (1.0 - metallic);

    // 计算反射方向
    let R = -input.V - 2. * input.N.dot(input.V) * input.N;

    // 计算环境反射参数，
    let f_ab = F_AB(perceptual_roughness, NdotV);

    let light_to_frag = light_uniform.position_radius.xyz() - input.world_position.xyz();
    let distance_square = light_to_frag.dot(light_to_frag);
    // 光源衰减
    let attenuation =
        distance_attenuation(distance_square, light_uniform.color_inverse_square_range.w);

    // 计算高光部分
    // 这个主要是用来计算球形光源的近似,实际上暂时没用到,其实就是把球形光源近似为点光源
    // 参考 http://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf p14-16
    let a = roughness;
    let center_to_ray = light_to_frag.dot(R) * R - light_to_frag;
    let closest_point = light_to_frag
        + center_to_ray
            * (light_uniform.position_radius.w * (1. / center_to_ray.length())).clamp(0., 1.);
    let spec_length_inverse = 1. / closest_point.length();
    let normaliztion_factor =
        a / (a + light_uniform.position_radius.w * 0.5 * spec_length_inverse).clamp(0., 1.);
    let specular_intersity = normaliztion_factor * normaliztion_factor;

    let L = closest_point * spec_length_inverse;
    let H = (L + input.V).normalize();
    let NoL = input.N.dot(L).clamp(0., 1.);
    let NoH = input.N.dot(H).clamp(0., 1.);
    let LoH = L.dot(H).clamp(0., 1.);

    // BRDF s
    let D = D_GGX(roughness, NoH, H);
    let V = V_SmithGGXCorrelated(roughness, NdotV, NoL);
    let F = fresnel(F0, LoH);

    // 高光项
    let mut specular_light = (specular_intersity * D * V) * F;
    // Multiscattering approximation有可能在微表面上经过多次反射又到正确的朝向
    // 参考 https://google.github.io/filament/Filament.html#listing_energycompensationimpl
    // Fr = Fr * (F0 * (1. / f_ab.x - 1.) + 1.);

    // Diffuse.散射项
    // Comes after specular since its NoL is used in the lighting equation.
    let L = light_to_frag.normalize();
    let H = (L + input.V).normalize();
    let NoL = input.N.dot(L).clamp(0., 1.);
    let NoH = input.N.dot(H).clamp(0., 1.);
    let LoH = L.dot(H).clamp(0., 1.);

    let diffuse = diffuse_color * Fd_Burley(roughness, NdotV, NoL, LoH);

    let direct_light = (diffuse + specular_light)
        * light_uniform.color_inverse_square_range.xyz()
        * attenuation
        * NoL;

    let emissive_light = emissive.xyz() * output_color.w;

    (direct_light + emissive_light).extend(output_color.w)
}
