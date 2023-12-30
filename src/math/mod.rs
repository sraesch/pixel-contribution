mod aabb;

pub use aabb::*;
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
    let p = vec4_to_vec3(&p) / p[3];

    p
}
