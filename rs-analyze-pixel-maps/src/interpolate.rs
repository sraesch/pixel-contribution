use nalgebra_glm::{Mat2, Vec2};

use crate::{PixelContributionMap, PixelContributionMaps};

use wasm_bindgen::prelude::*;

/// A simple linear interpolator for pixel contributions that interpolates between the first and
/// last pixel contribution map using the angle.
#[wasm_bindgen]
pub struct LinearAngle {
    first_map: PixelContributionMap,
    last_map: PixelContributionMap,
}

#[wasm_bindgen]
impl LinearAngle {
    /// Creates a new linear pixel contribution interpolator from the given pixel contribution maps.
    #[wasm_bindgen(constructor)]
    pub fn new(contrib_maps: &PixelContributionMaps) -> LinearAngle {
        let n = contrib_maps.size();

        assert!(
            n >= 2,
            "At least 2 pixel contribution maps are required for the linear interpolator."
        );

        let first_map = contrib_maps.get_map(0);
        let last_map = contrib_maps.get_map(n - 1);

        LinearAngle {
            first_map,
            last_map,
        }
    }

    pub fn interpolate(&self, angle: f32, pos_x: usize, pos_y: usize) -> f32 {
        let first_map = &self.first_map;
        let last_map = &self.last_map;

        let first_angle = first_map.get_description().camera_angle;
        let last_angle = last_map.get_description().camera_angle;

        let index = pos_y * first_map.get_description().map_size + pos_x;

        let first_value = first_map.get_value_at_index(index);
        let last_value = last_map.get_value_at_index(index);

        let f = (angle - first_angle) / (last_angle - first_angle);
        first_value * (1f32 - f) + last_value * f
    }
}

/// A angle interpolator for pixel contributions that interpolates between the first and
/// last pixel contribution map using the angle.
/// In contrast to the linear interpolator, this interpolator uses the tangent of the angle
/// to interpolate the pixel contributions.
#[wasm_bindgen]
pub struct TangentAngle {
    first_map: PixelContributionMap,
    last_map: PixelContributionMap,
}

#[wasm_bindgen]
impl TangentAngle {
    /// Creates a new linear pixel contribution interpolator from the given pixel contribution maps.
    #[wasm_bindgen(constructor)]
    pub fn new(contrib_maps: &PixelContributionMaps) -> TangentAngle {
        let n = contrib_maps.size();

        let first_map = contrib_maps.get_map(0);
        let last_map = contrib_maps.get_map(n - 1);

        TangentAngle {
            first_map,
            last_map,
        }
    }

    pub fn get_name(&self) -> String {
        "Angle".to_string()
    }

    pub fn interpolate(&self, angle: f32, pos_x: usize, pos_y: usize) -> f32 {
        let first_map = &self.first_map;
        let last_map = &self.last_map;

        let first_angle = first_map.get_description().camera_angle;
        let last_angle = last_map.get_description().camera_angle;

        let index = pos_y * first_map.get_description().map_size + pos_x;

        let first_value = first_map.get_value_at_index(index);
        let last_value = last_map.get_value_at_index(index);

        let a_start = (first_angle / 2.0).tan();
        let a_last = (last_angle / 2.0).tan();

        let a = (angle / 2.0).tan();
        let t = (a - a_start) / (a_last - a_start);

        first_value * (1f32 - t) + last_value * t
    }
}

/// A quadratic interpolator for pixel contributions that interpolates using a quadratic polynomial
/// using the first, middle and last pixel contribution map based on the angle as input.
#[wasm_bindgen]
pub struct QuadraticAngle {
    first_map: PixelContributionMap,
    middle_map: PixelContributionMap,
    last_map: PixelContributionMap,

    mat: Mat2,
}

#[wasm_bindgen]
impl QuadraticAngle {
    /// Creates a new quadratic pixel contribution interpolator from the given pixel contribution maps.
    #[wasm_bindgen(constructor)]
    pub fn new(contrib_maps: &PixelContributionMaps) -> QuadraticAngle {
        let n = contrib_maps.size();

        assert!(
            n >= 3,
            "At least 3 pixel contribution maps are required for the quadratic interpolator."
        );

        let first_map = contrib_maps.get_map(0);
        let middle_map = contrib_maps.get_map(n / 2);
        let last_map = contrib_maps.get_map(n - 1);

        let x0 = first_map.get_description().camera_angle;
        let x1 = middle_map.get_description().camera_angle;
        let x2 = last_map.get_description().camera_angle;

        assert_eq!(x0, 0f32, "The first angle must be 0");
        assert!(x0 < x1 && x1 < x2, "The angles are not in ascending order");

        let mat = Mat2::new(x1 * x1, x1, x2 * x2, x2).try_inverse().unwrap();

        QuadraticAngle {
            first_map,
            middle_map,
            last_map,
            mat,
        }
    }

    pub fn get_name(&self) -> String {
        "Quadratic".to_string()
    }

    pub fn interpolate(&self, angle: f32, pos_x: usize, pos_y: usize) -> f32 {
        let first_map = &self.first_map;
        let middle_map = &self.middle_map;
        let last_map = &self.last_map;

        let index = pos_y * first_map.get_description().map_size + pos_x;

        let y0 = first_map.get_value_at_index(index);
        let y1 = middle_map.get_value_at_index(index);
        let y2 = last_map.get_value_at_index(index);

        // determine the polynomial coefficients a,b,c
        let c = y0; // as x0 = 0
        let rhs = Vec2::new(y1 - c, y2 - c);
        let coefficients = self.mat * rhs;
        let a = coefficients[0];
        let b = coefficients[1];

        a * angle * angle + b * angle + c
    }
}
