pub struct PointLight {
  pub color: Color,
  pub intensity: f32,
  pub range: f32,
  pub radius: f32,
  pub shadows_enabled: bool,
  pub shadow_depth_bias: f32,
  /// A bias applied along the direction of the fragment's surface normal. It is scaled to the
  /// shadow map's texel size so that it can be small close to the camera and gets larger further
  /// away.
  pub shadow_normal_bias: f32,
}
