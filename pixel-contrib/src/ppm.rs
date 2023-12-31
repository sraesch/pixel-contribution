use std::{fs::File, path::Path};

use anyhow::Result;
use nalgebra_glm::Vec3;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rasterizer::Frame;

/// Writes the ids of the given frame as PPM file with random colors for the ids.
///
/// # Arguments
/// * `file_name` - The filename for the file to which the ids are written
/// * `frame` - The frame to write.
pub fn write_id_buffer(file_name: &Path, frame: &Frame) -> Result<()> {
    let out = File::create(file_name)?;

    frame.write_id_buffer_as_ppm(out, gen_random_colors)?;

    Ok(())
}

/// Writes the depths of the given frame as PPM file with gray colors.
///
/// # Arguments
/// * `file_name` - The filename for the file to which the depths are written
/// * `frame` - The frame to write.
pub fn write_depth_buffer(file_name: &Path, frame: &Frame) -> Result<()> {
    let out = File::create(file_name)?;

    frame.write_depth_buffer_as_pgm(out)?;

    Ok(())
}

/// Generate and returns the specified number of random colors.
/// Repeated calls always return the same colors
///
/// # Arguments
/// * `num_colors` - The number of colors to generate
fn gen_random_colors(num_colors: usize) -> Vec<Vec3> {
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
