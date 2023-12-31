use nalgebra_glm::{Mat4, Vec3};

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

impl View {
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
