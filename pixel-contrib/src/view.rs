use nalgebra_glm::{Mat4, Vec3};
use rasterizer::Aabb;

pub struct View {
    /// The view matrix
    pub view_matrix: Mat4,

    /// The projection matrix
    pub projection_matrix: Mat4,
}

impl Default for View {
    fn default() -> Self {
        Self {
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
        }
    }
}

/// A sphere defined by its center and radius.
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    /// Creates a new sphere from the given AABB.
    ///
    /// # Arguments
    /// * `aabb` - The AABB that the sphere should fit.
    pub fn from_aabb(aabb: &Aabb) -> Self {
        let center = aabb.get_center();
        let radius = aabb.get_size().norm() / 2f32;

        Self { center, radius }
    }
}

impl View {
    /// Creates a new view for the given AABB and fovy.
    ///
    /// # Arguments
    /// * `aabb` - The AABB that the view should fit.
    /// * `fovy` - The field of view in y-direction in radians.
    /// * `dir` - The direction of the camera, i.e, the directional vector which points toward the
    ///           object.
    pub fn new_from_aabb(aabb: &Aabb, fovy: f32, dir: Vec3) -> Self {
        let sphere = Sphere::from_aabb(aabb);
        Self::new_from_sphere(&sphere, fovy, dir)
    }

    /// Creates a new view to fit the given sphere and fovy.
    ///
    /// # Arguments
    /// * `sphere` - The sphere that the view should fit.
    /// * `fovy` - The field of view in y-direction in radians.
    /// * `dir` - The direction of the camera, i.e, the directional vector which points toward the
    ///           object.
    pub fn new_from_sphere(sphere: &Sphere, fovy: f32, dir: Vec3) -> Self {
        assert!(
            fovy > 0f32 && fovy < std::f32::consts::PI,
            "fovy must be in (0, PI)"
        );

        assert!(
            dir.norm() > 0f32,
            "dir must be a non-zero vector, but is {}",
            dir
        );

        // normalize the direction vector
        let dir = dir.normalize();

        // determine the distance between the camera and the center of the sphere s.t. it fits
        // perfectly into it
        let distance = sphere.radius / (fovy / 2f32).sin();

        // far plane is at the back of the sphere
        let far = distance + sphere.radius;

        // near plane is at the front of the sphere
        // NOTE: near is positive as long as fovy < 180°
        let near = distance - sphere.radius;

        // compute perspective matrix
        let projection_matrix = nalgebra_glm::perspective(
            1f32, // aspect
            fovy, near, far,
        );

        // determine the center of the camera
        let camera_center = sphere.center - dir * distance;
        let up = if dir.z.abs() < 0.95f32 {
            Vec3::new(0f32, 0f32, 1f32)
        } else {
            Vec3::new(0f32, 1f32, 0f32)
        };

        let view_matrix = nalgebra_glm::look_at(&camera_center, &sphere.center, &up);

        Self {
            view_matrix,
            projection_matrix,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_aabb() {
        let cube = Aabb::new_cube(&Vec3::new(0f32, 0f32, 0f32), 1f32);
        let sphere = Sphere::from_aabb(&cube);
        assert_eq!(sphere.center, Vec3::new(0.0f32, 0.0f32, 0.0f32));
        assert_eq!(sphere.radius, 0.75f32.sqrt());
    }
}
