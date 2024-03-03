use crate::polygon_2d::{ArrayConstructor, ArrayConstructorTrait, Polygon2D};
use math::{BoundingSphere, Frustum, IntersectionTestResult};
use nalgebra_glm::{mat4_to_mat3, zero, Mat3, Mat4, Vec2, Vec3};

/// An estimator for the footprint in pixels in the screen space.
pub struct ScreenSpaceEstimator {
    /// The model view transformation.
    model_view: Mat3,
    model_view_translation: Vec3,

    /// The compressed perspective matrix.
    perspective: Vec2,

    /// The frustum of the camera.
    frustum: Frustum,

    /// height * cotan(fovy / 2), where fovy the field of view in y-direction in radians is.
    height_fovy_cotan_2: f32,

    /// The width of the viewport in pixels.
    pub width: f32,

    /// The height of the viewport in pixels.
    pub height: f32,
}

impl Default for ScreenSpaceEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenSpaceEstimator {
    /// Creates a new screen space estimator.
    pub fn new() -> Self {
        Self {
            model_view: zero(),
            model_view_translation: zero(),
            perspective: Vec2::new(1f32, 1f32),
            frustum: Frustum::default(),
            height_fovy_cotan_2: 512.0,
            width: 512.0,
            height: 512.0,
        }
    }

    /// Updates the camera parameters for the screen space estimator.
    ///
    /// # Arguments
    /// * `model_view` - The model view matrix.
    /// * `perspective` - The perspective matrix.
    /// * `height` - The height of the viewport in pixels.
    pub fn update_camera(&mut self, model_view: Mat4, perspective: Mat4, height: f32) {
        self.model_view = mat4_to_mat3(&model_view);
        self.model_view_translation = model_view.column(3).xyz();

        self.perspective = Vec2::new(perspective.m11, perspective.m22);

        self.height_fovy_cotan_2 = perspective.m22 * height;

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
    /// * `out_polygon` - The polygon that approximates the projected 2D ellipse of the sphere.
    pub fn estimate_screen_space_for_bounding_sphere(&self, mut sphere: BoundingSphere) -> f32 {
        // transform the sphere into view space
        sphere.center = self.model_view * sphere.center + self.model_view_translation;

        // Check special case where the camera is inside the sphere.
        // In this case, the footprint is the entire screen.
        if sphere.center.norm_squared() <= sphere.radius * sphere.radius {
            return self.width * self.height;
        }

        // Test the bounding sphere with the frustum, i.e., check if the sphere is visible at all.
        let intersection_test = self.frustum.test_sphere(&sphere);
        if intersection_test == IntersectionTestResult::Outside {
            return 0.0;
        }

        // 1. --- Compute the smaller radius of the projected 2D ellipse ---
        let radius = estimate_bounding_sphere_radius_on_screen(self.height_fovy_cotan_2, &sphere);

        // 2. --- Determine the larger radius ---
        // Determine the directional vector to the sphere center
        let sphere_distance = sphere.center.norm();
        let sphere_dir = sphere.center / sphere_distance;

        // Compute the angle between the camera direction (0,0,-1) and the sphere direction
        let sphere_dir_angle = (-sphere_dir.z).acos();

        // Compute half the angle of the sphere, i.e., the angle of the cone that tightly encloses
        // the sphere.
        let sphere_angle = (sphere.radius / sphere_distance).asin();

        // We use the plane spanned by the camera direction vector and the vector that points to
        // the center of the sphere. The plane is orthogonal to the camera direction vector and
        // thus appears as 1D line on the screen.
        // Compute the minimum and maximum angle of the cone, that encapsulates the sphere,
        // along this line.
        // We can then use these angles to determine the range of the sphere along the projected
        // 1D line onto the screen.
        let min_sphere_angle = sphere_dir_angle - sphere_angle;
        let max_sphere_angle = sphere_dir_angle + sphere_angle;

        let x0 = min_sphere_angle.tan() * self.height_fovy_cotan_2 * 0.25;
        let x1 = max_sphere_angle.tan() * self.height_fovy_cotan_2 * 0.25;

        let larger_radius = x1 - x0;

        // 3. --- Compute the area of the ellipse ---
        if intersection_test == IntersectionTestResult::Inside {
            std::f32::consts::PI * radius * larger_radius
        } else {
            // Determine the two axis of the 2D ellipse
            let screen_center = Vec2::new(self.width, self.height) * 0.5;
            let mut axis1 = self.project_onto_screen(&sphere.center) - screen_center;
            if axis1.norm() < 1e-6 {
                axis1 = Vec2::new(1.0, 0.0);
            } else {
                axis1 = axis1.normalize();
            }

            let axis2 = Vec2::new(-axis1[1], axis1[0]);

            // Determine the center of the projected 2D ellipse on the screen
            let ellipse_center: Vec2 = screen_center + axis1 * (x0 + x1);

            // create a polygonal approximation of the projected 2D ellipse
            let ellipse_polygon: Polygon2D<8> = Self::create_polygon_from_ellipse(
                &ellipse_center,
                &axis1,
                &axis2,
                larger_radius,
                radius,
            );

            let full_area = ellipse_polygon.compute_area();
            let partial_area =
                ellipse_polygon.compute_area_with_overlapping_rectangle(self.width, self.height);

            let ratio = partial_area / full_area;

            std::f32::consts::PI * radius * larger_radius * ratio
        }
    }

    /// Projects the given view space position onto the screen.
    ///
    /// # Arguments
    /// * `view_space_pos` - The position in view space.
    fn project_onto_screen(&self, view_space_pos: &Vec3) -> Vec2 {
        let clip_space_pos = Vec2::new(
            view_space_pos[0] * self.perspective[0],
            view_space_pos[1] * self.perspective[1],
        ) / -view_space_pos[2];

        Vec2::new(
            (clip_space_pos[0] + 1.0) * 0.5 * self.width,
            (clip_space_pos[1] + 1.0) * 0.5 * self.height,
        )
    }

    /// Creates a 2D polygon that approximates the projected 2D ellipse of the given bounding sphere.
    ///
    /// # Arguments
    /// * `sphere_center` - The center of the ellipse on the screen.
    /// * `axis1` - The first normalized axis of the ellipse.
    /// * `axis2` - The second normalized axis of the ellipse.
    /// * `radius1` - The radius of the ellipse along the first axis.
    /// * `radius2` - The radius of the ellipse along the second axis.
    fn create_polygon_from_ellipse<const N: usize>(
        center: &Vec2,
        axis1: &Vec2,
        axis2: &Vec2,
        radius1: f32,
        radius2: f32,
    ) -> Polygon2D<N>
    where
        ArrayConstructor<N>: ArrayConstructorTrait,
    {
        // create the vertices of the 2D ellipse
        let coefficients = Self::create_ellipse_polygon_coefficients::<N>();
        let mut vertices = [Vec2::zeros(); N];
        coefficients
            .iter()
            .zip(vertices.iter_mut())
            .for_each(|(c, v)| {
                *v = center + axis1 * c[0] * radius1 + axis2 * c[1] * radius2;
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

/// Estimates the radius of the bounding sphere on the screen in pixels.
/// The position of the sphere is assumed to be in view space.
/// Note: This does not take the aspect ratio or the frustum into account.
///
/// # Arguments
/// * `height_fovy_cotan_2` - The height * cotan(fovy / 2), where fovy the field of view in y-direction in radians is.
/// * `sphere` - The bounding sphere.
fn estimate_bounding_sphere_radius_on_screen(
    height_fovy_cotan_2: f32,
    sphere: &BoundingSphere,
) -> f32 {
    // The distance of the sphere projected onto the camera projection plane.
    let d = -sphere.center[2];

    // project the ray that tangentially touches the sphere onto the plane that is 'd' units away
    // from the camera
    let x = sphere.radius / d;
    let projected_radius = x / (1f32 - x * x).sqrt();

    // use this radius to estimate how much the screen is being filled by the sphere
    projected_radius * height_fovy_cotan_2 * 0.5
}

#[cfg(test)]
mod test {
    use super::*;

    /// Tests the screen space estimator with a sphere that is completely visible and directly in
    /// front of the camera in its center.
    #[test]
    fn test_screen_space_estimator_sphere_center() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 646f32;
        let model_view = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -2.1213202, 1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.97583085, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.3999999, -1.0, 0.0, 0.0,
            -1.697056, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!((result - 262207f32).abs() / 262207f32 < 1e-5);
    }

    /// Tests the screen space estimator with a sphere that is partially visible. The sphere is
    /// located to the left of the screen and is partially out of the window.
    #[test]
    fn test_screen_space_estimator_sphere_left_side() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -3.6970365, 0.17255715,
            -3.4511428, 1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -2.152261, -1.0, 0.0, 0.0,
            -6.4209323, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!((result - 47856f32).abs() / 47856f32 < 5e-3);
    }

    /// Tests the screen space estimator with a sphere that is partially visible. The sphere is
    /// located to the right of the screen and is partially out of the window.
    #[test]
    fn test_screen_space_estimator_sphere_right_side() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 3.5916266, 0.2105576,
            -3.2393477, 1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -2.0324519, -1.0, 0.0, 0.0,
            -5.5346313, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!((result - 49536f32).abs() / 49536f32 < 5e-3);
    }

    /// Tests the screen space estimator with a sphere that is partially visible. The sphere is
    /// located to the top of the screen and is partially out of the window.
    #[test]
    fn test_screen_space_estimator_sphere_top_side() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            0.99309623,
            0.07378686,
            0.091189094,
            0.0,
            -0.09551708,
            0.9599192,
            0.2634989,
            0.0,
            -0.0680914,
            -0.27038985,
            0.96034,
            0.0,
            0.11187549,
            2.077688,
            -3.196443,
            1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -2.008181, -1.0, 0.0, 0.0,
            -5.3612695, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!(
            (result - 59398f32).abs() / 59398f32 < 2e-2,
            "Result: {}, Should: {}",
            result,
            59398f32
        );
    }

    /// Tests the screen space estimator with a sphere that is partially visible. The sphere is
    /// located to the bottom of the screen and is partially out of the window.
    #[test]
    fn test_screen_space_estimator_sphere_bottom_side() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            0.6043273,
            0.0,
            0.7967362,
            0.0,
            0.0033197245,
            0.99999136,
            -0.0025180236,
            0.0,
            -0.7967292,
            0.0041666552,
            0.60432214,
            0.0,
            0.006021142,
            -1.1806747,
            -2.1771443,
            1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.4315789, -1.0, 0.0, 0.0,
            -1.8551263, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!(
            (result - 135952f32).abs() / 135952f32 < 1e-2,
            "Result: {}, Should: {}",
            result,
            135952f32
        );
    }

    /// Tests the screen space estimator with a sphere that is very big and does not fit on the
    /// screen and is covering most of the screen.
    #[test]
    fn test_screen_space_estimator_sphere_big() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.7484075, 1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            -1.1890486,
            -1.0,
            0.0,
            0.0,
            -0.73156685,
            0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!(
            (result - 443467f32).abs() / 443467f32 < 6e-2,
            "Result: {}, Should: {}",
            result,
            443467f32
        );
    }

    /// Tests the screen space estimator with a sphere that is at the bottom right corner and only
    /// partially visible.
    #[test]
    fn test_screen_space_estimator_sphere_bottom_right() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.8732692, -1.3417664,
            -2.2300825, 1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.4615251, -1.0, 0.0, 0.0,
            -2.008282, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!(
            (result - 96079f32).abs() / 96079f32 < 5e-3,
            "Result: {}, Should: {}",
            result,
            96079f32
        );
    }

    /// Tests the screen space estimator with a sphere that is at the top right corner and only
    /// partially visible.
    #[test]
    fn test_screen_space_estimator_sphere_top_right() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            0.83405703,
            0.0,
            -0.55167824,
            0.0,
            -0.4021004,
            0.68465465,
            -0.6079172,
            0.0,
            0.37770903,
            0.7288677,
            0.571041,
            0.0,
            1.7835734,
            1.1846286,
            -2.6521535,
            1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.7002845, -1.0, 0.0, 0.0,
            -3.34279, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!(
            (result - 103496f32).abs() / 103496f32 < 4e-2,
            "Result: {}, Should: {}",
            result,
            103496f32
        );
    }

    /// Tests the screen space estimator with a sphere that is at the top left corner and only
    /// partially visible.
    #[test]
    fn test_screen_space_estimator_sphere_top_left() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            0.7748143,
            1.1545633e-8,
            0.6321889,
            0.0,
            -0.21924871,
            0.9379358,
            0.26871246,
            0.0,
            -0.5929526,
            -0.34680885,
            0.7267261,
            0.0,
            -2.459809,
            1.9747595,
            -2.8255568,
            1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.7983762, -1.0, 0.0, 0.0,
            -3.9494693, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!(
            (result - 65179f32).abs() / 65179f32 < 1e-2,
            "Result: {}, Should: {}",
            result,
            65179f32
        );
    }

    /// Tests the screen space estimator with a sphere that is at the bottom left corner and only
    /// partially visible.
    #[test]
    fn test_screen_space_estimator_sphere_bottom_left() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            0.99929696,
            -1.8613356e-9,
            0.03749121,
            0.0,
            0.030035013,
            0.5985018,
            -0.80055827,
            0.0,
            -0.022438556,
            0.8011215,
            0.59808105,
            0.0,
            -1.2603166,
            -0.7094321,
            -2.149794,
            1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.4161072, -1.0, 0.0, 0.0,
            -1.7772415, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!(
            (result - 155577f32).abs() / 155577f32 < 3e-2,
            "Result: {}, Should: {}",
            result,
            155577f32
        );
    }

    /// Tests the screen space estimator with a sphere where the camera is fully inside the sphere.
    #[test]
    fn test_screen_space_estimator_sphere_camera_inside() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.2909417, 1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            -1.0000019,
            -1.0,
            0.0,
            0.0,
            -6.824531e-6,
            0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert!(
            (result - 480000f32).abs() / 480000f32 < 1e-3,
            "Result: {}, Should: {}",
            result,
            480000f32
        );
    }

    /// Tests the screen space estimator with a sphere outside the camera frustum.
    #[test]
    fn test_screen_space_estimator_sphere_outside() {
        let mut estimator = ScreenSpaceEstimator::new();

        let height = 600f32;
        let model_view = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -10.286586, 1.3572578,
            -4.2860775, 1.0,
        )
        .transpose();
        let projection = Mat4::new(
            0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -2.6245716, -1.0, 0.0, 0.0,
            -10.409276, 0.0,
        )
        .transpose();

        estimator.update_camera(model_view, projection, height);

        let sphere = BoundingSphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: std::f32::consts::SQRT_2,
        };

        let result = estimator.estimate_screen_space_for_bounding_sphere(sphere);
        assert_eq!(result, 0f32);
    }
}
