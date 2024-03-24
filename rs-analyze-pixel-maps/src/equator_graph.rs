use crate::PixelContributionMap;
use math::clamp;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Float32Array;

/// Create a series of values from the pixel contribution map by using the given angles.
///
/// # Arguments
/// * `pixel_contrib_map` - The pixel contribution map to use.
/// * `angle_list` - A list of angles to use.
#[wasm_bindgen]
pub fn create_equator_series(
    pixel_contrib_map: &PixelContributionMap,
    angle_list: Float32Array,
) -> Float32Array {
    let d = pixel_contrib_map.get_description();

    let result_values: Vec<f32> = angle_list
        .to_vec()
        .iter()
        .map(|angle| {
            let x = angle.cos();
            let y = angle.sin();

            let index = d.index_from_camera_dir(x, y, 0.0);

            pixel_contrib_map.get_value_at_index(index)
        })
        .collect();

    Float32Array::from(result_values.as_slice())
}

/// Create a series of values from the pixel contribution map by using the given angles.
///
/// # Arguments
/// * `pixel_contrib_map` - The pixel contribution map to use.
/// * `angle_list` - A list of angles to use.
#[wasm_bindgen]
pub fn create_equator_series_linear_interpolation(
    pixel_contrib_map: &PixelContributionMap,
    angle_list: Float32Array,
) -> Float32Array {
    const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
    let d = pixel_contrib_map.get_description();

    let result_values: Vec<f32> = angle_list
        .to_vec()
        .iter()
        .map(|angle| {
            // determine the previous and the next angle
            let prev_angle = (angle / FRAC_PI_2).floor() * FRAC_PI_2;
            let next_angle = clamp(prev_angle + FRAC_PI_2, 0f32, std::f32::consts::PI * 2f32);

            // evaluate the contribution at the previous and next angle
            let prev_index = d.index_from_camera_dir(prev_angle.cos(), prev_angle.sin(), 0f32);
            let next_index = d.index_from_camera_dir(next_angle.cos(), next_angle.sin(), 0f32);
            let prev_value = pixel_contrib_map.get_value_at_index(prev_index);
            let next_value = pixel_contrib_map.get_value_at_index(next_index);

            // determine the current value using linear interpolation
            let t = (angle - prev_angle) / (next_angle - prev_angle);
            prev_value * (1f32 - t) + next_value * t
        })
        .collect();

    Float32Array::from(result_values.as_slice())
}
