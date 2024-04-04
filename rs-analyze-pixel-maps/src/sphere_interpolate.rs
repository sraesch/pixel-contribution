use math::clamp;
use nalgebra_glm::Vec3;
use wasm_bindgen::prelude::*;

use crate::{
    cartesian_to_polar, polar_to_cartesian, PixelContribColorMapDescriptor, PixelContributionMap,
};

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

/// Subdivides the unit sphere into a grid and interpolates the pixel contribution values based
/// on the grid cells using bi-linear interpolation.
#[wasm_bindgen]
pub struct GridInterpolator {
    /// The value at position (0, 0, 1) of the unit sphere.
    top_value: f32,

    /// The value at position (0, 0, -1) of the unit sphere.
    bottom_value: f32,

    /// The eight values at the equator of the unit sphere.
    equator_values: [f32; 8],

    /// The six values at the ring located at the top hemisphere of the unit sphere.
    top_hemisphere_values: [f32; 6],

    /// six values at the ring located at the bottom hemisphere of the unit sphere.
    bottom_hemisphere_values: [f32; 6],

    /// The descriptor of the input map.
    desc: PixelContribColorMapDescriptor,
}

#[wasm_bindgen]
impl GridInterpolator {
    /// Creates a new GridInterpolator with the given values for the x, y and z axis.
    ///
    /// # Arguments
    /// `contrib_map` - The PixelContributionMap object to create the interpolator from.
    #[wasm_bindgen(constructor)]
    pub fn new(contrib_map: &PixelContributionMap) -> Self {
        const FRAC_PI_4: f32 = std::f32::consts::FRAC_PI_4;

        let desc = contrib_map.get_description();

        // The two values at the poles of the octahedron.
        let top_value =
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 0f32, 1f32));
        let bottom_value =
            contrib_map.get_value_at_index(desc.index_from_camera_dir(0f32, 0f32, -1f32));

        // Sample the top hemisphere of the unit sphere.
        let mut top_hemisphere_values: [f32; 6] = Default::default();
        Self::sample_unit_sphere(FRAC_PI_4, contrib_map, &mut top_hemisphere_values);

        // The four values along the equator of the octahedron.
        let mut equator_values: [f32; 8] = Default::default();
        Self::sample_unit_sphere(0f32, contrib_map, &mut equator_values);

        // The four values at the bottom hemisphere of the octahedron.
        // Note that the directional vectors will be normalized.
        let mut bottom_hemisphere_values: [f32; 6] = Default::default();
        Self::sample_unit_sphere(-FRAC_PI_4, contrib_map, &mut bottom_hemisphere_values);

        Self {
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
        const FRAC_PI_4: f32 = std::f32::consts::FRAC_PI_4;

        // normalizes the given direction vector and converts it to polar coordinates
        let dir = Vec3::new(dir_x, dir_y, dir_z).normalize();
        let (alpha, beta) = cartesian_to_polar(&dir);

        if beta >= 0f32 {
            let i = (beta / FRAC_PI_4).floor();

            let (v0, v1) = match i as usize {
                0 => (
                    Self::linear_interpolate_on_unit_circle(&self.equator_values, alpha),
                    Self::linear_interpolate_on_unit_circle(&self.top_hemisphere_values, alpha),
                ),
                1 => (
                    Self::linear_interpolate_on_unit_circle(&self.top_hemisphere_values, alpha),
                    self.top_value,
                ),
                _ => (self.top_value, self.top_value),
            };

            let t = (beta - i * FRAC_PI_4) / FRAC_PI_4;

            v0 * (1f32 - t) + v1 * t
        } else {
            let i = (beta / FRAC_PI_4).ceil().abs();

            let (v0, v1) = match i as usize {
                0 => (
                    Self::linear_interpolate_on_unit_circle(&self.equator_values, alpha),
                    Self::linear_interpolate_on_unit_circle(&self.bottom_hemisphere_values, alpha),
                ),
                1 => (
                    Self::linear_interpolate_on_unit_circle(&self.bottom_hemisphere_values, alpha),
                    self.bottom_value,
                ),
                _ => (self.bottom_value, self.bottom_value),
            };

            let t = (beta + i * FRAC_PI_4).abs() / FRAC_PI_4;

            v0 * (1f32 - t) + v1 * t
        }
    }

    /// Assumes that the given values are distributed evenly on the unit circle and performs a
    /// linear interpolation between the values for the given angle x.
    ///
    /// # Arguments
    /// `values` - The values to interpolate between.
    /// `x` - The angle to interpolate the values for.
    fn linear_interpolate_on_unit_circle(values: &[f32], mut x: f32) -> f32 {
        const PI_2: f32 = std::f32::consts::PI * 2f32;

        // check for special case that values contains only one element
        if values.len() == 1 {
            return values[0];
        }

        // make sure x is in the range [0, 2*PI[
        if x < 0f32 {
            x = PI_2 + x % PI_2;
        } else {
            x %= PI_2;
        }

        // determine the size of a single segment
        let dx = PI_2 / values.len() as f32;

        // determine the index of the value that is left from x
        let i0 = clamp((x / dx).floor() as usize, 0, values.len() - 1);

        // determine the index of the value that is right from x
        let i1 = (i0 + 1) % values.len();

        // determine the linear interpolation factor of x between the two values
        let t = (x - i0 as f32 * dx) / dx;

        // linearly interpolate between the two values
        let a = values[i0];
        let b = values[i1];

        a * (1f32 - t) + b * t
    }

    /// Samples the circle on the unit sphere defined at the given beta angle.
    ///
    /// # Arguments
    /// `beta` - The angle beta to sample the unit sphere at.
    /// `contrib_map` - The PixelContributionMap object to sample the unit sphere from.
    /// `result` - The array to store the sampled values in.
    fn sample_unit_sphere(beta: f32, contrib_map: &PixelContributionMap, result: &mut [f32]) {
        let dx = std::f32::consts::PI * 2f32 / result.len() as f32;
        let desc = contrib_map.get_description();

        result.iter_mut().enumerate().for_each(|(i, value)| {
            let alpha = i as f32 * dx;

            let dir = polar_to_cartesian(alpha, beta);
            *value =
                contrib_map.get_value_at_index(desc.index_from_camera_dir(dir[0], dir[1], dir[2]));
        });
    }

    /// Returns the descriptor for the input map.
    pub fn get_descriptor(&self) -> PixelContribColorMapDescriptor {
        self.desc
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_linear_interpolate_on_unit_circle() {
        let values = [0f32, 2f32, 4f32, 6f32];

        for i in 0..4 {
            let x = i as f32 * std::f32::consts::PI * 2f32 / 4f32;
            let value = values[i];

            assert_eq!(
                GridInterpolator::linear_interpolate_on_unit_circle(&values, x),
                value
            );
        }
    }
}
