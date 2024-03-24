mod aabb;
mod bounding_sphere;
mod frustum;
mod plane;
mod ray;
mod utils;

pub use aabb::*;
pub use bounding_sphere::*;
pub use frustum::*;
pub use plane::*;
pub use ray::*;
pub use utils::*;

use nalgebra_glm::{vec4_to_vec3, Mat4, Vec3, Vec4};

/// Transforms the given vec3 with the given homogenous transformation matrix and returns the
/// transformed vec3.
///
/// # Arguments
/// * `t` - The 4x4 homogenous transformation matrix.
/// * `p` - The 3D vector to transform.
#[inline]
pub fn transform_vec3(t: &Mat4, p: &Vec3) -> Vec3 {
    let p = t * Vec4::new(p[0], p[1], p[2], 1f32);

    vec4_to_vec3(&p) / p[3]
}

/// Extracts the camera position from the given modelview matrix.
/// Returns None, if the modelview matrix is not invertible.
///
/// # Arguments
/// * `modelview` - The modelview matrix from which the camera position should be extracted.
#[inline]
pub fn extract_camera_position(modelview: &Mat4) -> Option<Vec3> {
    modelview
        .try_inverse()
        .map(|m| vec4_to_vec3(&m.column(3).into()))
}

/// The result of testing the intersection between two objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionTestResult {
    /// The right-hand side object is completely inside the left-hand side object.
    Inside,

    /// The right-hand side object is partially inside the left-hand side object.
    Intersecting,

    /// No Intersection at all
    Outside,
}
