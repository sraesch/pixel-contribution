use std::{
    io::{BufWriter, Write},
    path::Path,
};

use image::RgbImage;
use log::debug;
use nalgebra_glm::Vec3;

use crate::{clamp, Error, Result};

#[derive(Clone)]
pub struct Frame {
    /// The width and height of the quadratic frame buffer
    size: usize,

    /// The id-buffer contains per pixel ids
    id_buffer: Vec<Option<u32>>,

    /// The depth buffer contains the per pixel depth.
    /// The depth buffer is optional.
    depth_buffer: Option<Vec<f32>>,
}

impl Frame {
    /// Creates a new empty frame
    ///
    /// # Arguments
    /// * `size` - The width and height of the quadratic frame buffer
    /// * `with_depths` - If true, the frame will contain a depth buffer.
    pub fn new_empty(size: usize, with_depths: bool) -> Self {
        let id_buffer: Vec<Option<u32>> = vec![None; size * size];

        let depth_buffer = if with_depths {
            Some(vec![0f32; size * size])
        } else {
            None
        };

        Self {
            size,
            id_buffer,
            depth_buffer,
        }
    }

    /// Returns the size of the quadratic frame buffer.
    pub fn get_frame_size(&self) -> usize {
        self.size
    }

    /// Returns a reference onto the id buffer.
    pub fn get_id_buffer(&self) -> &[Option<u32>] {
        &self.id_buffer
    }

    /// Returns a mutable reference onto the id buffer.
    pub fn get_id_buffer_mut(&mut self) -> &mut [Option<u32>] {
        &mut self.id_buffer
    }

    /// Returns a reference onto the depth buffer.
    pub fn get_depth_buffer(&self) -> Option<&[f32]> {
        self.depth_buffer.as_deref()
    }

    /// Returns a mutable reference onto the depth buffer.
    pub fn get_depth_buffer_mut(&mut self) -> Option<&mut [f32]> {
        self.depth_buffer.as_deref_mut()
    }

    /// Writes the depths of the given frame as PGM file with gray colors.
    ///
    /// # Arguments
    /// * `writer` - The writer to which the depth-buffer will be serialized as PGM.
    pub fn write_depth_buffer_as_pgm<W: Write>(&self, writer: W) -> Result<()> {
        let mut out = BufWriter::new(writer);

        let depths = self.get_depth_buffer().unwrap();
        let ids = self.get_id_buffer();

        // determine min/max
        let (min, max) = if depths.is_empty() {
            (0f32, 1f32)
        } else {
            let mut min = f32::MAX;
            let mut max = 0f32;

            for (depth, id) in depths.iter().zip(ids.iter()) {
                match id {
                    Some(_) => {
                        min = min.min(*depth);
                        max = max.max(*depth);
                    }
                    None => {}
                }
            }

            (min, max)
        };

        debug!("Writing depth buffer: Min/Max={}/{}", min, max);

        writeln!(out, "P2")?;
        writeln!(out, "{} {}", self.get_frame_size(), self.get_frame_size())?;
        writeln!(out, "255")?;

        ids.iter()
            .zip(depths.iter())
            .map(|(id, depth)| match id {
                Some(_) => {
                    if max > min {
                        ((1f32 - ((*depth - min) / (max - min))) * 255f32).round() as u32
                    } else {
                        128u32
                    }
                }
                None => 0,
            })
            .enumerate()
            .try_for_each(|(index, depth)| -> std::io::Result<()> {
                write!(out, "{} ", depth)?;

                if index > 0 && index % self.get_frame_size() == 0 {
                    writeln!(out)?;
                }

                Ok(())
            })?;

        Ok(())
    }

    /// Writes the id buffer of the given frame as colored image.
    ///
    /// # Arguments
    /// * `filename` - The filename to which the id-buffer will be serialized to.
    /// * `create_palette` - Callback for creating color palette for the given number of ids.
    pub fn write_id_buffer<P, F>(&self, filename: P, mut create_palette: F) -> Result<()>
    where
        P: AsRef<Path>,
        F: FnMut(usize) -> Vec<Vec3>,
    {
        let frame_size = self.get_frame_size() as u32;
        let mut img = RgbImage::new(frame_size, frame_size);

        let ids = self.get_id_buffer();

        // determine the maximal id
        let num_ids: usize = if ids.is_empty() {
            0
        } else {
            let n: u32 = ids.iter().map(|id| id.unwrap_or(0)).max().unwrap();
            (n as usize) + 1
        };

        let colors = create_palette(num_ids);
        assert_eq!(colors.len(), num_ids);

        img.pixels_mut().zip(ids.iter()).for_each(|(pixel, id)| {
            // determine the color for the pixel based on the id
            let color = match id {
                Some(id) => colors[*id as usize],
                None => Vec3::new(0f32, 0f32, 0f32),
            };

            pixel[0] = clamp((color[0] * 255f32) as u32, 0, 255) as u8;
            pixel[1] = clamp((color[1] * 255f32) as u32, 0, 255) as u8;
            pixel[2] = clamp((color[2] * 255f32) as u32, 0, 255) as u8;
        });

        img.save(filename).map_err(|e| Error::IO(format!("{}", e)))
    }
}
