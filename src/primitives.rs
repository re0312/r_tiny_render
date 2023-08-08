use crate::math::{Mat4, Vec3, Vec4};

#[derive(Clone, Copy, Debug, Default)]
pub struct HalfSpace {
    normal_d: Vec4,
}

impl HalfSpace {
    /// Constructs a `HalfSpace` from a 4D vector whose first 3 components
    /// represent the bisecting plane's unit normal, and the last component signifies
    /// the distance from the origin to the plane along the normal.
    /// The constructor ensures the normal vector is normalized and the distance is appropriately scaled.
    #[inline]
    pub fn new(normal_d: Vec4) -> Self {
        Self {
            normal_d: normal_d * normal_d.xyz().length().recip(),
        }
    }

    /// Returns the unit normal vector of the bisecting plane that characterizes the `HalfSpace`.
    #[inline]
    pub fn normal(&self) -> Vec3 {
        self.normal_d.xyz()
    }

    /// Returns the distance from the origin to the bisecting plane along the plane's unit normal vector.
    /// This distance helps determine the position of a point `p` on the bisecting plane, as per the equation `n.p + d = 0`.
    #[inline]
    pub fn d(&self) -> f32 {
        self.normal_d.w
    }

    /// Returns the bisecting plane's unit normal vector and the distance from the origin to the plane.
    #[inline]
    pub fn normal_d(&self) -> Vec4 {
        self.normal_d
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Frustum {
    pub half_spaces: [HalfSpace; 6],
}

impl Frustum {
    /// Returns a frustum derived from `view_projection`.
    #[inline]
    pub fn from_view_projection(view_projection: &Mat4) -> Self {
        let mut frustum = Frustum::from_view_projection_no_far(view_projection);
        frustum.half_spaces[5] = HalfSpace::new(view_projection.row(2));
        frustum
    }

    /// Returns a frustum derived from `view_projection`,
    /// but with a custom far plane.
    #[inline]
    pub fn from_view_projection_custom_far(
        view_projection: &Mat4,
        view_translation: &Vec3,
        view_backward: &Vec3,
        far: f32,
    ) -> Self {
        let mut frustum = Frustum::from_view_projection_no_far(view_projection);
        let far_center = *view_translation - far * *view_backward;
        frustum.half_spaces[5] =
            HalfSpace::new(view_backward.extend(-view_backward.dot(far_center)));
        frustum
    }

    // NOTE: This approach of extracting the frustum half-space from the view
    // projection matrix is from Foundations of Game Engine Development 2
    // Rendering by Lengyel.
    /// Returns a frustum derived from `view_projection`,
    /// without a far plane.
    fn from_view_projection_no_far(view_projection: &Mat4) -> Self {
        let row3 = view_projection.row(3);
        let mut half_spaces = [HalfSpace::default(); 6];
        for (i, half_space) in half_spaces.iter_mut().enumerate().take(5) {
            let row = view_projection.row(i / 2);
            *half_space = HalfSpace::new(if (i & 1) == 0 && i != 4 {
                row3 + row
            } else {
                row3 - row
            });
        }
        Self { half_spaces }
    }

}
