use anyhow::{bail, Result};
use log::{debug, error};
use math::{extract_camera_position, BoundingSphere};
use nalgebra_glm::{Mat4, Vec3};

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
    sphere: &BoundingSphere,
    height: f32,
) -> Result<f32> {
    // extract f from the perspective matrix, which is
    // f = 1 / tan(fovy / 2)
    let f = perspective.m22;
    let aspect = f / perspective.m11;
    let width = height * aspect;
    let fovy = (1f32 / f).atan() * 2.0;

    // extract the camera position from the modelview matrix
    let cam_pos = match extract_camera_position(model_view) {
        Some(cam_pos) => {
            debug!("Camera position: {:?}", cam_pos);
            cam_pos
        }
        None => {
            error!("Failed to extract camera position from model view matrix");
            bail!("Failed to extract camera position from model view matrix")
        }
    };

    // estimate the radius of the bounding sphere on the screen
    let radius = estimate_bounding_sphere_radius_on_screen(&cam_pos, fovy, sphere) * height / 2.0;

    // the area of the 2D projected sphere
    let area = radius * radius * std::f32::consts::PI;

    Ok(area)
}

/// Estimates the radius of the bounding sphere on the screen in the range [0, 1].
/// A value of 1 means that the sphere fills the screen completely.
/// Note: This does not take the aspect ratio or the frustum into account.
///
/// # Arguments
/// * `cam_pos` - The position of the camera.
/// * `fovy` - The field of view in y-direction in radians.
/// * `sphere` - The bounding sphere.
fn estimate_bounding_sphere_radius_on_screen(
    cam_pos: &Vec3,
    fovy: f32,
    sphere: &BoundingSphere,
) -> f32 {
    let d = nalgebra_glm::distance(cam_pos, &sphere.center);

    // project the ray that tangentially touches the sphere onto the plane that is 'd' units away
    // from the camera
    let phi = (sphere.radius / d).asin();
    let projected_radius = phi.tan() * d;

    // now compute half the length of the side of the frustum at the distance 'd'
    let r_capital = (fovy / 2.0).tan() * d;

    // use this radius to estimate how much the screen is being filled by the sphere
    projected_radius / r_capital
}
