use std::{
    io::{BufReader, Read},
    path::Path,
};

use math::BoundingSphere;
use nalgebra_glm::{Mat4, Vec3};
use pixel_contrib_types::PixelContributionMaps;

use crate::{
    screen_space::{ScreenSpaceEstimator, ScreenSpaceResult},
    Error, Result,
};

/// The pixel contribution estimator is used to estimate the contribution of a pixel to the final
/// image. It uses a combination of screen space estimation of the bounding volume and precomputed
/// pixel contribution maps to estimate the pixel contribution.
pub struct PixelContribution {
    /// The precomputed pixel contribution maps.
    maps: PixelContributionMaps,

    /// The position of the camera.
    cam_pos: Vec3,

    /// The screen space estimator used to estimate the screen space of the bounding volume.
    sphere_estimator: ScreenSpaceEstimator,
}

impl PixelContribution {
    /// Creates a new pixel contribution estimator with the given maps.
    ///
    /// # Arguments
    /// `maps` - The maps to use for encoding the pixel contribution.
    pub fn new(maps: PixelContributionMaps) -> Self {
        Self {
            maps,
            cam_pos: Vec3::zeros(),
            sphere_estimator: Default::default(),
        }
    }

    /// Creates a new pixel contribution estimator from the given reader.
    ///
    /// # Arguments
    /// `reader` - The reader to read the pixel contribution maps from.
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let maps = PixelContributionMaps::from_reader(reader)?;

        Ok(Self::new(maps))
    }

    /// Creates a new pixel contribution estimator from the given path.
    ///
    /// # Arguments
    /// * `path` - The path from which the pixel contribution should be read.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        Self::from_reader(&mut BufReader::new(file))
    }

    /// Updates the internal camera configuration.
    ///
    /// # Arguments
    /// `model_view` - The model-view matrix of the camera.
    /// `perspective` - The perspective matrix of the camera.
    /// `height` - The height of the frame buffer in pixels.
    pub fn update_camera(
        &mut self,
        model_view: Mat4,
        perspective: Mat4,
        height: f32,
    ) -> Result<()> {
        self.sphere_estimator
            .update_camera(model_view, perspective, height);

        // extract the camera position from the model-view matrix
        self.cam_pos = math::extract_camera_position(&model_view)
            .ok_or_else(|| Error::InvalidArgument("Model-View matrix is invalid".to_owned()))?;

        Ok(())
    }

    /// Estimates the pixel contribution of the given bounding sphere.
    ///
    /// # Arguments
    /// `sphere` - The bounding sphere to estimate the pixel contribution for.
    pub fn estimate_pixel_contribution(&self, sphere: &BoundingSphere) -> f32 {
        // First make a prediction of the pixels that the bounding sphere will cover.
        let (predicted_sphere_pixels, classification) = self
            .sphere_estimator
            .estimate_screen_space_for_bounding_sphere(*sphere);

        // If the sphere is either completely outside or completely inside the frustum, we can just
        // return the predicted pixel contribution.
        if classification != ScreenSpaceResult::PartiallyVisible {
            return predicted_sphere_pixels;
        }

        // If the sphere is visible, make the estimated pixel contribution more precise by using
        // the pixel contribution maps.
        let cam_dir = nalgebra_glm::normalize(&(sphere.center - self.cam_pos));
        let sphere_angle = Self::estimate_camera_angle(&self.cam_pos, sphere);
        let pixel_contrib_value = self
            .maps
            .get_pixel_contrib_for_camera_dir(cam_dir, sphere_angle);

        predicted_sphere_pixels * pixel_contrib_value
    }

    /// Returns the pixel contribution maps used by this estimator.
    #[inline]
    pub fn get_maps(&self) -> &PixelContributionMaps {
        &self.maps
    }

    /// Estimates the angle of the camera based on the bounding sphere. The further away the sphere
    /// is, the smaller the angle will be. That is, the angle is the angle of the camera frustum
    /// that is covered by the bounding sphere.
    ///
    /// # Arguments
    /// * `cam_pos` - The position of the camera.
    /// * `sphere` - The bounding sphere.
    #[inline]
    fn estimate_camera_angle(cam_pos: &Vec3, sphere: &BoundingSphere) -> f32 {
        let d = nalgebra_glm::distance(cam_pos, &sphere.center);
        (sphere.radius / d).asin() * 2f32
    }
}
