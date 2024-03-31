use nalgebra_glm::Vec3;
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

/// Does a barycentric interpolation of the pixel contribution values based on the triangles of
/// the corresponding octahedron map. These are the 8 triangles of the octahedron
#[wasm_bindgen]
pub struct BarycentricInterpolator {
    /// The value at position (0, 0, 1) of the octahedron.
    top_value: f32,

    /// The value at position (0, 0, -1) of the octahedron.
    bottom_value: f32,

    /// The four values at the equator of the octahedron.
    /// The order is (1, 0, 0), (0, 1, 0), (-1, 0, 0), (0, -1, 0)
    equator_values: [f32; 4],

    desc: PixelContribColorMapDescriptor,
}

#[wasm_bindgen]
impl BarycentricInterpolator {
    /// Creates a new BarycentricInterpolator with the given values for the x, y and z axis.
    ///
    /// # Arguments
    /// `contrib_map` - The PixelContributionMap object to create the interpolator from.
    #[wasm_bindgen(constructor)]
    pub fn new(contrib_map: &PixelContributionMap) -> BarycentricInterpolator {
        let desc = contrib_map.get_description();

        let top_value =
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 0f32, 1f32));
        let bottom_value =
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 0f32, -1f32));

        let equator_values = [
            contrib_map.get_value_at_index(desc.index_from_camera_dir(1f32, 0f32, 0f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 1f32, 0f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(-1f32, 0f32, 0f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, -1f32, 0f32)),
        ];

        BarycentricInterpolator {
            top_value,
            bottom_value,
            equator_values,
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
        // re-project the direction vector to the octahedron
        let mut dir = Vec3::new(dir_x, dir_y, dir_z);
        dir /= (dir_x.abs() + dir_y.abs() + dir_z.abs()).max(1e-5);

        // Depending if the z component is positive or negative, we are in the top or bottom half
        // of the octahedron and take either the top or bottom value.
        let m = if dir.z >= 0f32 {
            self.top_value
        } else {
            self.bottom_value
        };

        // The absolute values of x and y are already defining the first two barycentric
        // coordinates. The third one is then the remaining value to reach 1.
        let x = dir.x.abs();
        let y = dir.y.abs();
        let z = 1f32 - x - y;

        // We have now to determine the corresponding equator values for x and y depending on the
        // sign of the x and y components.
        let x_value = if dir.x >= 0f32 {
            self.equator_values[0]
        } else {
            self.equator_values[2]
        };

        let y_value = if dir.y >= 0f32 {
            self.equator_values[1]
        } else {
            self.equator_values[3]
        };

        // The final value is then the weighted sum of the top/bottom value and the two equator
        // values.
        m * z + x_value * x + y_value * y
    }

    /// Returns the descriptor for the input map.
    pub fn get_descriptor(&self) -> PixelContribColorMapDescriptor {
        self.desc
    }
}
