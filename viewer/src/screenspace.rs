use anyhow::Result;
use log::info;
use math::{transform_vec3, BoundingSphere, Frustum, IntersectionTestResult};
use nalgebra_glm::{dot, Mat4, Vec2, Vec3};

use crate::polygon_2d::Polygon2D;

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
        let intersection_test = self.frustum.test_sphere(&sphere);
        if intersection_test == IntersectionTestResult::Outside {
            return Ok(0.0);
        }

        // 1. --- Compute the smaller radius of the projected 2D ellipse ---
        let radius =
            estimate_bounding_sphere_radius_on_screen(self.fovy, &sphere) * self.height / 2.0;

        // 2. --- Determine the larger radius ---
        // Determine the directional vector to the sphere center
        let sphere_dir = sphere.center.normalize();

        // Compute the angle between the camera direction and the sphere direction
        let cam_dir: Vec3 = Vec3::new(0.0, 0.0, -1.0);
        let sphere_dir_angle = dot(&cam_dir, &sphere_dir).acos();

        // Compute half the angle of the sphere, i.e., the angle of the cone that tightly encloses
        // the sphere.
        let sphere_angle = (sphere.radius / sphere.center.norm()).asin();

        // Compute the minimum and maximum angle of the cone and translate it into
        // coordinates onto the plane spanned by the camera direction and the sphere
        // direction.
        let min_sphere_angle = sphere_dir_angle - sphere_angle;
        let max_sphere_angle = sphere_dir_angle + sphere_angle;

        let x0 = min_sphere_angle.tan() / (self.fovy / 2f32).tan();
        let x1 = max_sphere_angle.tan() / (self.fovy / 2f32).tan();

        let len_pixel = (x1 - x0) * self.height;
        let larger_radius = len_pixel / 4f32;

        // 3. --- Compute the area of the ellipse ---
        if intersection_test == IntersectionTestResult::Inside {
            Ok(std::f32::consts::PI * radius * larger_radius)
        } else {
            info!("Sphere is partially visible, but not completely.");

            // create a polygonal approximation of the projected 2D ellipse
            let ellipse_polygon: Polygon2D<16> =
                self.create_polygon_from_ellipse(&sphere.center, larger_radius, radius);

            Ok(0f32)
        }
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

    /// Creates a 2D polygon that approximates the projected 2D ellipse of the given bounding sphere.
    ///
    /// # Arguments
    /// * `sphere_center` - The center of the bounding sphere in view space.
    /// * `big_radius` - The larger radius of the ellipse.
    /// * `small_radius` - The smaller radius of the ellipse.
    fn create_polygon_from_ellipse<const N: usize>(
        &self,
        sphere_center: &Vec3,
        big_radius: f32,
        small_radius: f32,
    ) -> Polygon2D<N> {
        let screen_center = Vec2::new(self.width, self.height) * 0.5;
        let center = self.project_onto_screen(sphere_center);

        // determine the first axis of the 2D ellipse
        let mut axis1 = center - screen_center;
        if axis1.norm() < 1e-6 {
            axis1 = Vec2::new(1.0, 0.0);
        } else {
            axis1 = axis1.normalize();
        }

        // determine the second axis of the 2D ellipse
        let axis2 = Vec2::new(-axis1[1], axis1[0]);

        // create the vertices of the 2D ellipse
        let coefficients = Self::create_ellipse_polygon_coefficients::<N>();
        let mut vertices = [Vec2::zeros(); N];
        coefficients
            .iter()
            .zip(vertices.iter_mut())
            .for_each(|(c, v)| {
                *v = center + axis1 * c[0] * big_radius + axis2 * c[1] * small_radius;
            });

        Polygon2D::new(vertices)
    }

    /// Creates the coefficients for the 2D ellipse, i.e., equally spaced points on the unit
    /// circle in counterclockwise order.
    fn create_ellipse_polygon_coefficients<const N: usize>() -> [Vec2; N] {
        let mut coefficients = [Vec2::zeros(); N];

        coefficients.iter_mut().enumerate().for_each(|(i, c)| {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (N as f32);
            *c = Vec2::new(angle.cos(), angle.sin());
        });

        coefficients
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
