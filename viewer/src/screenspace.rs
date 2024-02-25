use anyhow::Result;
use log::info;
use math::{transform_vec3, BoundingSphere};
use nalgebra_glm::{cross, dot, Mat4, Vec3};

/// Estimates the footprint in pixels on the screen for the given bounding sphere.
/// The result is the radius of the sphere projected onto the screen in pixels.
///
/// # Arguments
/// * `model_view` - The model view matrix.
/// * `perspective` - The perspective matrix.
/// * `sphere` - The bounding sphere.
/// * `height` - The height of the viewport in pixels.
pub fn estimate_screenspace_for_bounding_sphere(
    model_view: &Mat4,
    perspective: &Mat4,
    mut sphere: BoundingSphere,
    height: f32,
) -> Result<f32> {
    // extract f from the perspective matrix, which is
    // f = 1 / tan(fovy / 2)
    let f = perspective.m22;
    let fovy = (1f32 / f).atan() * 2.0;

    // transform the sphere into view space
    sphere.center = transform_vec3(model_view, &sphere.center);

    // estimate the radius of the bounding sphere on the screen
    let radius = estimate_bounding_sphere_radius_on_screen(fovy, &sphere) * height / 2.0;
    let a = radius;

    //// NEW CODE
    let cam_dir: Vec3 = Vec3::new(0.0, 0.0, -1.0);

    // project sphere position onto the camera direction vector
    let projected_pos = nalgebra_glm::dot(&sphere.center, &cam_dir) * cam_dir;

    // determine the first axis of the ellipse
    let mut axis1 = sphere.center - projected_pos;
    if axis1.norm() < 1e-6 {
        axis1 = Vec3::new(1.0, 0.0, 0.0);
    } else {
        axis1 = axis1.normalize();
    }

    // determine the second axis of the ellipse
    let axis2 = cross(&cam_dir, &axis1).normalize();

    info!("Axis1: {:?}", axis1);
    info!("Axis2: {:?}", axis2);

    // determine the angle in which the sphere is projected onto the screen
    let sphere_dir = sphere.center.normalize();
    let sphere_dir_angle = dot(&cam_dir, &sphere_dir).acos();
    let sphere_angle = (sphere.radius / sphere.center.norm()).asin() * 2f32;

    let min_sphere_angle = sphere_dir_angle - sphere_angle / 2.0;
    let max_sphere_angle = sphere_dir_angle + sphere_angle / 2.0;

    let x0 = min_sphere_angle.tan() / (fovy / 2.0).tan() / 2f32;
    let x1 = max_sphere_angle.tan() / (fovy / 2.0).tan() / 2f32;

    let len_pixel = (x1 - x0) * height;
    let b = len_pixel / 2.0;

    let ellipse_area = std::f32::consts::PI * a * b;

    /////////////

    Ok(ellipse_area)
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
    let d = sphere.center[2].abs();

    // project the ray that tangentially touches the sphere onto the plane that is 'd' units away
    // from the camera
    let phi = (sphere.radius / d).asin();
    let projected_radius = phi.tan() * d;

    // now compute half the length of the side of the frustum at the distance 'd'
    let r_capital = (fovy / 2.0).tan() * d;

    // use this radius to estimate how much the screen is being filled by the sphere
    projected_radius / r_capital
}
