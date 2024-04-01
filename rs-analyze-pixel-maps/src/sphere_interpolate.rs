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

/// Does a barycentric interpolation of the pixel contribution values based on the triangles of
/// the corresponding octahedron map. In contrast to the simple BarycentricInterpolator, this
/// has a higher resolution and therefore more triangles.
/// Each triangle is being split into 3 sub-triangles.
#[wasm_bindgen]
pub struct BarycentricInterpolatorFine {
    /// The value at position (0, 0, 1) of the octahedron.
    top_value: f32,

    /// The value at position (0, 0, -1) of the octahedron.
    bottom_value: f32,

    /// The four values at the top hemisphere of the octahedron located at each center of the
    /// four triangles.
    /// The order is (1, 1, 1), (-1, 1, 1), (-1, -1, 1), (1, -1, 1)
    top_hemisphere_values: [f32; 4],

    /// The four values at the bottom hemisphere of the octahedron located at each center of the
    /// four triangles.
    /// The order is (1, 1, -1), (-1, 1, -1), (-1, -1, -1), (1, -1, -1)
    bottom_hemisphere_values: [f32; 4],

    /// The four values at the equator of the octahedron.
    /// The order is (1, 0, 0), (0, 1, 0), (-1, 0, 0), (0, -1, 0)
    equator_values: [f32; 4],

    desc: PixelContribColorMapDescriptor,
}

#[wasm_bindgen]
impl BarycentricInterpolatorFine {
    /// Creates a new BarycentricInterpolator with the given values for the x, y and z axis.
    ///
    /// # Arguments
    /// `contrib_map` - The PixelContributionMap object to create the interpolator from.
    #[wasm_bindgen(constructor)]
    pub fn new(contrib_map: &PixelContributionMap) -> Self {
        let desc = contrib_map.get_description();

        // The two values at the poles of the octahedron.
        let top_value =
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 0f32, 1f32));
        let bottom_value =
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 0f32, -1f32));

        // The four values at the top hemisphere of the octahedron.
        // Note that the directional vectors will be normalized.
        let top_hemisphere_values = [
            contrib_map.get_value_at_index(desc.index_from_camera_dir(1f32, 1f32, 1f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(-1f32, 1f32, 1f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(-1f32, -1f32, 1f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(1f32, -1f32, 1f32)),
        ];

        // The four values along the equator of the octahedron.
        let equator_values = [
            contrib_map.get_value_at_index(desc.index_from_camera_dir(1f32, 0f32, 0f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 1f32, 0f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(-1f32, 0f32, 0f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, -1f32, 0f32)),
        ];

        // The four values at the bottom hemisphere of the octahedron.
        // Note that the directional vectors will be normalized.
        let bottom_hemisphere_values = [
            contrib_map.get_value_at_index(desc.index_from_camera_dir(1f32, 1f32, -1f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(-1f32, 1f32, -1f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(-1f32, -1f32, -1f32)),
            contrib_map.get_value_at_index(desc.index_from_camera_dir(1f32, -1f32, -1f32)),
        ];

        BarycentricInterpolatorFine {
            top_value,
            top_hemisphere_values,
            equator_values,
            bottom_hemisphere_values,
            bottom_value,
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

        // The absolute values of x and y are already defining the first two barycentric
        // coordinates. The third one is then the remaining value to reach 1.
        let x = dir.x.abs();
        let y = dir.y.abs();
        let z = 1f32 - x - y;

        // Determine the value at the center of the triangle by looking up the respective value
        // from the top or bottom hemisphere values.
        let middle_value = {
            let hemisphere_values = if dir.z >= 0f32 {
                &self.top_hemisphere_values
            } else {
                &self.bottom_hemisphere_values
            };

            if dir.x >= 0f32 {
                if dir.y >= 0f32 {
                    hemisphere_values[0]
                } else {
                    hemisphere_values[3]
                }
            } else if dir.y >= 0f32 {
                hemisphere_values[1]
            } else {
                hemisphere_values[2]
            }
        };

        // Depending if the z component is positive or negative, we are in the top or bottom half
        // of the octahedron and take either the top or bottom value.
        let m = if dir.z >= 0f32 {
            self.top_value
        } else {
            self.bottom_value
        };

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

        // We split the triangle ABC into 3 sub-triangles by adding the center M of the triangle.
        // M is defined as the average of the three vertices, i.e., M = (A + B + C) / 3.
        // We have to check in which of the 3 sub-triangles ABM, AMC, or MBC the position is.
        // This can easily decided by taking the smallest of the three barycentric coordinates.
        if x <= y && x <= z {
            // We are located in the sub-triangle MBC.
            let sigma = 3f32 * x;
            let y = y - x;
            let z = z - x;

            sigma * middle_value + y * y_value + z * m
        } else if y <= x && y <= z {
            // We are located in the sub-triangle AMC.
            let sigma = 3f32 * y;
            let x = x - y;
            let z = z - y;

            sigma * middle_value + x * x_value + z * m
        } else {
            // We are located in the sub-triangle ABM.
            let sigma = 3f32 * z;
            let x = x - z;
            let y = y - z;

            sigma * middle_value + x * x_value + y * y_value
        }
    }

    /// Returns the descriptor for the input map.
    pub fn get_descriptor(&self) -> PixelContribColorMapDescriptor {
        self.desc
    }
}
