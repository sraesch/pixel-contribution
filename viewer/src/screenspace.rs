use anyhow::Result;
use log::info;
use math::{transform_vec3, BoundingSphere, Frustum, IntersectionTestResult};
use nalgebra_glm::{cross, dot, Mat4, Vec2, Vec3};

/// An estimator for the footprint in pixels in the screenspace.
pub struct ScreenspaceEstimator {
    /// The model view matrix.
    model_view: Mat4,

    /// The perspective matrix.
    perspective: Mat4,

    /// The frustum of the camera.
    frustum: Frustum,

    /// The field of view in y-direction in radians.
    fovy: f32,

    /// The width of the viewport in pixels.
    pub width: f32,

    /// The height of the viewport in pixels.
    pub height: f32,
}

impl Default for ScreenspaceEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenspaceEstimator {
    /// Creates a new screenspace estimator.
    pub fn new() -> Self {
        Self {
            model_view: Mat4::identity(),
            perspective: Mat4::identity(),
            frustum: Frustum::default(),
            fovy: std::f32::consts::FRAC_PI_2,
            width: 512.0,
            height: 512.0,
        }
    }

    /// Updates the camera parameters for the screenspace estimator.
    ///
    /// # Arguments
    /// * `model_view` - The model view matrix.
    /// * `perspective` - The perspective matrix.
    /// * `height` - The height of the viewport in pixels.
    pub fn update_camera(&mut self, model_view: Mat4, perspective: Mat4, height: f32) {
        self.model_view = model_view;
        self.perspective = perspective;

        self.fovy = (1f32 / perspective.m22).atan() * 2.0;

        self.height = height;

        let aspect = perspective.m22 / perspective.m11;
        self.width = height * aspect;

        self.frustum = Frustum::from_projection(&perspective);
    }

    /// Estimates the footprint in pixels on the screen for the given bounding sphere.
    /// The result is the overall estimated number of pixels of sphere, projected onto the screen in pixels.
    ///
    /// # Arguments
    /// * `sphere` - The bounding sphere.
    pub fn estimate_screenspace_for_bounding_sphere(
        &self,
        mut sphere: BoundingSphere,
    ) -> Result<f32> {
        // transform the sphere into view space
        sphere.center = transform_vec3(&self.model_view, &sphere.center);

        // Check special case where the camera is inside the sphere.
        // In this case, the footprint is the entire screen.
        if sphere.center.norm_squared() <= sphere.radius * sphere.radius {
            return Ok(self.width * self.height);
        }

        // Test the bounding sphere with the frustum, i.e., check if the sphere is visible at all.
        let result = self.frustum.test_sphere(&sphere);
        if result == IntersectionTestResult::Outside {
            return Ok(0.0);
        }

        let ellipse_2d = self.project_sphere_onto_screen(&sphere);
        info!("Ellipse: {:?}", ellipse_2d);

        let ellipse_area = ellipse_2d.area();

        Ok(ellipse_area)
    }

    /// Projects the given view space position onto the screen.
    ///
    /// # Arguments
    /// * `view_space_pos` - The position in view space.
    fn project_onto_screen(&self, view_space_pos: &Vec3) -> Vec2 {
        let clip_space_pos = transform_vec3(&self.perspective, view_space_pos);
        Vec2::new(
            (clip_space_pos[0] + 1.0) * 0.5 * self.width,
            (clip_space_pos[1] + 1.0) * 0.5 * self.height,
        )
    }

    fn project_sphere_onto_screen(&self, sphere: &BoundingSphere) -> Ellipse2D {
        let screen_center = Vec2::new(self.width, self.height) * 0.5;
        let center = self.project_onto_screen(&sphere.center);

        // determine the first axis of the 2D ellipse
        let mut axis1 = center - screen_center;
        if axis1.norm() < 1e-6 {
            axis1 = Vec2::new(1.0, 0.0);
        } else {
            axis1 = axis1.normalize();
        }

        // determine the second axis of the 2D ellipse
        let axis2 = Vec2::new(-axis1[1], axis1[0]);

        // estimate the smaller radius of the 2D ellipse, i.e., axis 2
        let radius =
            estimate_bounding_sphere_radius_on_screen(self.fovy, sphere) * self.height / 2.0;

        // determine the larger radius
        let sphere_dir = sphere.center.normalize();
        let cam_dir: Vec3 = Vec3::new(0.0, 0.0, -1.0);
        let sphere_dir_angle = dot(&cam_dir, &sphere_dir).acos();
        let sphere_angle = (sphere.radius / sphere.center.norm()).asin() * 2f32;

        let min_sphere_angle = sphere_dir_angle - sphere_angle / 2.0;
        let max_sphere_angle = sphere_dir_angle + sphere_angle / 2.0;

        let x0 = min_sphere_angle.tan() / (self.fovy / 2.0).tan() / 2f32;
        let x1 = max_sphere_angle.tan() / (self.fovy / 2.0).tan() / 2f32;

        let len_pixel = (x1 - x0) * self.height;
        let larger_radius = len_pixel / 2.0;

        Ellipse2D {
            center,
            axis1: axis1 * larger_radius,
            axis2: axis2 * radius,
        }
    }
}

/// Estimates the radius of the bounding sphere on the screen in the range [0, 1].
/// A value of 1 means that the sphere fills the screen completely.
/// The position of the sphere is assumed to be in view space.
/// Note: This does not take the aspect ratio or the frustum into account.
///
/// # Arguments
/// * `fovy` - The field of view in y-direction in radians.
/// * `sphere` - The bounding sphere.
fn estimate_bounding_sphere_radius_on_screen(fovy: f32, sphere: &BoundingSphere) -> f32 {
    // the distance of the sphere to the camera projection plane
    let d = -sphere.center[2];

    // project the ray that tangentially touches the sphere onto the plane that is 'd' units away
    // from the camera
    let phi = (sphere.radius / d).asin();
    let projected_radius = phi.tan();

    // now compute half the length of the side of the frustum at the distance 'd'
    let r_capital = (fovy / 2.0).tan();

    // use this radius to estimate how much the screen is being filled by the sphere
    projected_radius / r_capital
}

/// A 2D ellipse in screen space.
#[derive(Debug, Clone, Copy)]
struct Ellipse2D {
    pub center: Vec2,
    pub axis1: Vec2,
    pub axis2: Vec2,
}

impl Ellipse2D {
    /// Returns the area of the ellipse.
    pub fn area(&self) -> f32 {
        let a = self.axis1.norm();
        let b = self.axis2.norm();
        std::f32::consts::PI * a * b
    }

    /// Tests if the 2D ellipse intersects with the given rectangle.
    pub fn test_intersection(&self, width: f32, height: f32) -> IntersectionTestResult {
        let min_x = self.center[0] - self.axis1[0];
        let max_x = self.center[0] + self.axis1[0];
        let min_y = self.center[1] - self.axis2[1];
        let max_y = self.center[1] + self.axis2[1];

        min_x < width && max_x > 0.0 && min_y < height && max_y > 0.0
    }
}
