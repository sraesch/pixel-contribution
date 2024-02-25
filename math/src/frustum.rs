use arrayvec::ArrayVec;
use nalgebra_glm::{transpose, Mat4, Vec3};

use super::{Aabb, Plane};

/// A frustum defined by 6 planes which is usually used for modelling the camera view.
pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    /// Builds the internal planes from the given given model-view-projection
    /// matrix.
    ///
    /// # Arguments
    ///
    ///* `m` - The combined model view and projection matrix.
    pub fn from_projection(m: &Mat4) -> Self {
        let tm = transpose(m);

        // define equations
        let planes = ArrayVec::<Plane, 6>::from_iter((0..6).map(|i| -> Plane {
            let sign = if i % 2 == 0 { 1f32 } else { -1f32 };
            Plane::from_equation_with_normalization(&(tm.column(3) + sign * tm.column(i / 2)))
        }));

        Self {
            planes: unsafe { planes.into_inner_unchecked() },
        }
    }

    /// Checks if the given point is located inside the frustum.
    /// Returns true if the given point is located inside the frustum.
    ///
    /// # Arguments
    ///* `p` - The point to be checked
    pub fn is_point_inside(&self, p: &Vec3) -> bool {
        !self
            .planes
            .iter()
            .any(|plane| plane.signed_distance(p) < 0f32)
    }

    /// Checks if the given box is outside of the frustum. Returns true if the given aabb volume
    /// is outside of the frustum and false otherwise.
    ///
    /// # Arguments
    ///* `aabb` - The aabb volume to check.
    #[inline]
    pub fn is_aabb_outside(&self, aabb: &Aabb) -> bool {
        // The aabb volume is outside the frustum, if the volume is inside one of the negative
        // open half-spaces of one of the frustum plans.
        self.planes
            .iter()
            .any(|plane| plane.is_aabb_negative_half_space(aabb))
    }
}

#[cfg(test)]
mod test {
    use nalgebra_glm::perspective;

    use super::*;

    #[test]
    fn test_frustum_point() {
        // 90 degree in radians
        let angle = std::f32::consts::FRAC_PI_2;
        let proj = perspective(1f32, angle, 1f32, 10f32);

        let f = Frustum::from_projection(&proj);

        // test near-plane
        assert!(f.is_point_inside(&Vec3::new(0f32, 0f32, -4f32)));
        assert!(!f.is_point_inside(&Vec3::new(0f32, 0f32, -0.5f32)));
        assert!(f.is_point_inside(&Vec3::new(0f32, 0f32, -1.01f32)));

        // test far-plane
        assert!(!f.is_point_inside(&Vec3::new(0f32, 0f32, -10.5f32)));
        assert!(f.is_point_inside(&Vec3::new(0f32, 0f32, -9.5f32)));

        // test left-plane
        assert!(f.is_point_inside(&Vec3::new(
            (angle / 2f32).tan() * -2f32 * 0.9f32,
            0f32,
            -2f32
        )));
        assert!(!f.is_point_inside(&Vec3::new(
            (angle / 2f32).tan() * -2f32 * 1.01f32,
            0f32,
            -2f32,
        )));

        // test right-plane
        assert!(f.is_point_inside(&Vec3::new(
            (angle / 2f32).tan() * 2f32 * 0.9f32,
            0f32,
            -2f32
        )));
        assert!(!f.is_point_inside(&Vec3::new(
            (angle / 2f32).tan() * 2f32 * 1.01f32,
            0f32,
            -2f32,
        )));

        // test bottom-plane
        assert!(f.is_point_inside(&Vec3::new(
            0f32,
            (angle / 2f32).tan() * -2f32 * 0.9f32,
            -2f32
        )));
        assert!(!f.is_point_inside(&Vec3::new(
            0f32,
            (angle / 2f32).tan() * -2f32 * 1.01f32,
            -2f32
        )));

        // test top-plane
        assert!(f.is_point_inside(&Vec3::new(
            0f32,
            (angle / 2f32).tan() * 2f32 * 0.9f32,
            -2f32
        )));
        assert!(!f.is_point_inside(&Vec3::new(
            0f32,
            (angle / 2f32).tan() * 2f32 * 1.01f32,
            -2f32
        )));
    }

    #[test]
    fn test_aabb() {
        // 90 degree in radians
        let angle = std::f32::consts::FRAC_PI_2;
        let proj = perspective(1f32, angle, 1f32, 10f32);

        let f = Frustum::from_projection(&proj);

        assert!(f.is_aabb_outside(&Aabb::new_cube(&Vec3::new(0f32, 0f32, 1f32), 1f32)));

        assert!(f.is_aabb_outside(&Aabb::new_cube(&Vec3::new(0f32, 0f32, -12f32), 1f32)));
        assert!(!f.is_aabb_outside(&Aabb::new_cube(&Vec3::new(0f32, 0f32, -1f32), 1f32)));
        assert!(!f.is_aabb_outside(&Aabb::new_cube(&Vec3::new(0f32, 0f32, -4f32), 1f32)));
        assert!(f.is_aabb_outside(&Aabb::new_cube(
            &Vec3::new((angle / 2f32).tan() * -3f32 * 1.01f32, 0f32, -2f32),
            1f32,
        )));
    }
}
