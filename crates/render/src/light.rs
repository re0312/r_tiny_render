use crate::{shader_uniform::PointLightUniform, Color, Transform};

pub struct PointLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub transform: Transform,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            color: Color::WHITE,
            intensity: 800.,
            range: 20.,
            radius: 0.,
            transform: Transform::from_xyz(0., 0., 0.),
        }
    }
}
impl PointLight {
    pub fn get_point_light_uniform(&self) -> PointLightUniform {
        PointLightUniform {
            color_inverse_square_range: (self.color.to_linear_rgba() * self.intensity)
                .xyz()
                .extend(1. / (self.range * self.range)),
            position_radius: self.transform.translation.extend(self.radius),
        }
    }
}
