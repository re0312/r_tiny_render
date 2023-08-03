use crate::{
    math::{Mat4, Vec2},
    transform::Transform,
};

pub trait CameraProjection {
    fn get_projection_matrix(&self) -> Mat4;
    fn update(&mut self, width: f32, height: f32);
    fn far(&self) -> f32;
}
pub struct PerspectiveProjection {
    /// The vertical field of view (FOV) in radians.
    ///
    /// Defaults to a value of π/4 radians or 45 degrees.
    pub fov: f32,

    /// The aspect ratio (width divided by height) of the viewing frustum.
    ///
    /// Bevy's [`camera_system`](crate::camera::camera_system) automatically
    /// updates this value when the aspect ratio of the associated window changes.
    ///
    /// Defaults to a value of `1.0`.
    pub aspect_ratio: f32,

    /// The distance from the camera in world units of the viewing frustum's near plane.
    ///
    /// Objects closer to the camera than this value will not be visible.
    ///
    /// Defaults to a value of `0.1`.
    pub near: f32,

    /// The distance from the camera in world units of the viewing frustum's far plane.
    ///
    /// Objects farther from the camera than this value will not be visible.
    ///
    /// Defaults to a value of `1000.0`.
    pub far: f32,
}

impl CameraProjection for PerspectiveProjection {
    #[rustfmt::skip]
    fn get_projection_matrix(&self) -> Mat4 {
        let near_z = -self.near;
        let far_z = -self.far;
        let height_near = 2.0 * (self.fov / 2.0).tan() * self.near;
        let width_near=self.aspect_ratio *height_near;
        let persp_to_ortho = Mat4::from_rows_slice(&[
            near_z, 0., 0., 0.,
            0., near_z, 0., 0.,
            0., 0., near_z + far_z, -near_z * far_z,
            0., 0., 1., 0.,
        ]);
        let ortho_translation = Mat4::from_rows_slice(&[
            1., 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., -(near_z + far_z) / 2.,
            0., 0., 0., 1.,
        ]);
        let ortho_scale = Mat4::from_rows_slice(&[
            2. / width_near, 0., 0., 0.,
            0., 2. / height_near, 0., 0.,
            0., 0., 2. / (near_z - far_z), 0.,
            0., 0., 0., 1.,
        ]);
        ortho_scale * ortho_translation * persp_to_ortho
    }

    fn update(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        PerspectiveProjection {
            fov: std::f32::consts::PI / 4.0,
            near: 1.,
            far: 1000.0,
            aspect_ratio: 1.0,
        }
    }
}

#[derive(Default)]
pub struct Viewport {
    /// The physical position to render this viewport to within the [`RenderTarget`] of this [`Camera`].
    /// (0,0) corresponds to the top-left corner
    pub physical_position: Vec2,
    /// The physical size of the viewport rectangle to render to within the [`RenderTarget`] of this [`Camera`].
    /// The origin of the rectangle is in the top-left corner.
    pub physical_size: Vec2,
}

impl Viewport {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self {
            physical_position: position,
            physical_size: size,
        }
    }
    pub fn size(&self) -> f32 {
        self.physical_size.length_squared() / 2.
    }
    #[rustfmt::skip]
    pub fn get_viewport_matrix(&self) -> Mat4 {
        Mat4::from_rows_slice(&[
            self.physical_size.x/2. , 0. , 0. , self.physical_size.x/2.,
            0. , self.physical_size.y/2. , 0. , self.physical_size.y/2.,
            0. , 0. , 1. , 0.,
            0. , 0. , 0. , 1.
        ])
    }
}
#[derive(Default)]
pub struct Camera {
    pub transform: Transform,
    pub projectiton: PerspectiveProjection,
    pub viewport: Viewport,
}

impl Camera {
    #[rustfmt::skip]
    pub fn get_view_matrix(&self) -> Mat4 {
        let translation = Mat4::from_rows_slice(&[
            1. , 0. , 0. , -self.transform.translation.x,
            0. , 1. , 0. , -self.transform.translation.y,
            0. , 0. , 1. , -self.transform.translation.z,
            0. , 0. , 0. , 1.,
        ]);
        let rotation =self.transform.rotation.inverse().to_mat4();
        rotation*translation
    }
}
