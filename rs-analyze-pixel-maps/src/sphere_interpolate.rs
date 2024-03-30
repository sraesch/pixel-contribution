use wasm_bindgen::prelude::*;

use crate::{PixelContribColorMapDescriptor, PixelContributionMap};

/// A very simple interpolator, where only one value per axis is being stored.
#[wasm_bindgen]
pub struct ValuePerAxisInterpolator {
    x_axis: f32,
    y_axis: f32,
    z_axis: f32,

    desc: PixelContribColorMapDescriptor,
}

#[wasm_bindgen]
impl ValuePerAxisInterpolator {
    /// Creates a new ValuePerAxisInterpolator with the given values for the x, y and z axis.
    ///
    /// # Arguments
    /// `contrib_map` - The PixelContributionMap object to create the interpolator from.
    #[wasm_bindgen(constructor)]
    pub fn new(contrib_map: &PixelContributionMap) -> ValuePerAxisInterpolator {
        let desc = contrib_map.get_description();

        // determine the value for each axis by averaging the values of the two opposite directions
        let x_axis = (contrib_map
            .get_value_at_index(desc.index_from_camera_dir(-1f32, 0f32, 0f32))
            + contrib_map.get_value_at_index(desc.index_from_camera_dir(1f32, 0f32, 0f32)))
            / 2f32;

        let y_axis = (contrib_map
            .get_value_at_index(desc.index_from_camera_dir(0f32, -1f32, 0f32))
            + contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 1f32, 0f32)))
            / 2f32;

        let z_axis = (contrib_map
            .get_value_at_index(desc.index_from_camera_dir(0f32, 0f32, -1f32))
            + contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 0f32, 1f32)))
            / 2f32;

        ValuePerAxisInterpolator {
            x_axis,
            y_axis,
            z_axis,
            desc,
        }
    }

    /// Interpolates the pixel contribution value for the given camera direction vector.
    ///
    /// # Arguments
    /// `dir_x` - The x component of the camera direction vector.
    /// `dir_y` - The y component of the camera direction vector.
    /// `dir_z` - The z component of the camera direction vector.
    pub fn interpolate(&self, dir_x: f32, dir_y: f32, dir_z: f32) -> f32 {
        let dir_x = dir_x.abs();
        let dir_y = dir_y.abs();
        let dir_z = dir_z.abs();

        self.x_axis * dir_x + self.y_axis * dir_y + self.z_axis * dir_z
    }

    /// Returns the descriptor for the input map.
    pub fn get_descriptor(&self) -> PixelContribColorMapDescriptor {
        self.desc
    }
}
