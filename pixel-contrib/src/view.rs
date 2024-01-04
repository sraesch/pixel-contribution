use nalgebra_glm::{ortho, Mat4, Vec3};
use rasterizer::BoundingSphere;

use crate::CameraConfig;

/// A camera view defined by its view and projection matrix.
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

impl View {
    /// Creates a new view to fit the given sphere and fovy.
    ///
    /// # Arguments
    /// * `sphere` - The sphere that the view should fit.
    /// * `camera_config` - The camera configuration for which the view should be created.
    /// * `dir` - The direction of the camera, i.e, the directional vector which points toward the
    ///           object.
    pub fn new_from_sphere(
        sphere: &BoundingSphere,
        camera_config: CameraConfig,
        dir: Vec3,
    ) -> Self {
        let (projection_matrix, distance) = match camera_config {
            CameraConfig::Orthographic => {
                Self::create_projection_matrix_for_orthographic_camera(sphere)
            }
            CameraConfig::Perspective { fovy } => {
                Self::create_projection_matrix_for_perspective_camera(sphere, fovy)
            }
        };

        // normalize the direction vector
        assert!(
            dir.norm() > 0f32,
            "dir must be a non-zero vector, but is {}",
            dir
        );
        let dir = dir.normalize();

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

    /// Creates a projection matrix for a perspective camera defined by the given fovy.
    /// Returns the projection matrix and the distance between the camera and the center of the
    /// bounding sphere.
    ///
    /// # Arguments
    /// * `sphere` - The sphere that the view should fit.
    /// * `fovy` - The field of view in y-direction in radians.
    fn create_projection_matrix_for_perspective_camera(
        sphere: &BoundingSphere,
        fovy: f32,
    ) -> (Mat4, f32) {
        assert!(
            fovy > 0f32 && fovy < std::f32::consts::PI,
            "fovy must be in (0, PI)"
        );

        // determine the distance between the camera and the center of the sphere s.t. it fits
        // perfectly into it
        let distance = sphere.radius / (fovy / 2f32).sin();

        // far plane is at the back of the sphere
        let far = distance + sphere.radius;

        // near plane is at the front of the sphere
        // NOTE: near is positive as long as fovy < 180°
        let near = distance - sphere.radius;

        (nalgebra_glm::perspective(1f32, fovy, near, far), distance)
    }

    /// Creates a projection matrix for an orthographic camera that fits the given sphere.
    ///
    /// # Arguments
    /// * `sphere` - The sphere that the view should fit.
    fn create_projection_matrix_for_orthographic_camera(sphere: &BoundingSphere) -> (Mat4, f32) {
        let radius = sphere.radius;

        // The distance between the camera and the center of the sphere is the radius of the
        // sphere plus the radius of the sphere as tolerance.
        let distance = radius * 2f32;

        (
            ortho(-radius, radius, -radius, radius, radius, radius * 4f32),
            distance,
        )
    }
}
