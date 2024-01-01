use nalgebra_glm::Vec3;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Generate and returns the specified number of random colors.
/// Repeated calls always return the same colors
///
/// # Arguments
/// * `num_colors` - The number of colors to generate
pub fn create_color_map(num_colors: usize) -> Vec<Vec3> {
    let mut r = ChaCha8Rng::seed_from_u64(2);

    (0..num_colors)
        .map(move |_| {
            Vec3::new(
                (r.gen_range(0..0x100) as f32) / 255.0,
                (r.gen_range(0..0x100) as f32) / 255.0,
                (r.gen_range(0..0x100) as f32) / 255.0,
            )
        })
        .collect()
}
