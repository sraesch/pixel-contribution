use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use image::RgbImage;
use nalgebra_glm::{Vec2, Vec3};
use rasterizer::clamp;
use serde::{Deserialize, Serialize};

use crate::{
    octahedron::{decode_octahedron_normal, encode_octahedron_normal},
    ColorMap, Error, Result,
};

/// The resulting pixel contribution for all possible views.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct PixelContributionMap {
    pub descriptor: PixelContribColorMapDescriptor,

    /// The 2D map for the pixel contribution of each view. Each position on the map
    /// represents a view. The normalized position (u,v) is mapped to a normal using octahedral
    /// projection. The normal then defines the camera view direction onto the object.
    /// The pixel contribution values are in the range [0, 1].
    pub pixel_contrib: Vec<f32>,
}

impl PixelContributionMap {
    /// Creates a new pixel contribution map for the given size with 0 contribution.
    ///
    /// # Arguments
    /// * `descriptor` - The descriptor for the pixel contribution map.
    pub fn new(descriptor: PixelContribColorMapDescriptor) -> Self {
        Self {
            descriptor,
            pixel_contrib: vec![0.0; descriptor.num_values()],
        }
    }

    /// Writes the pixel contribution map to the given path as image.
    ///
    /// # Arguments
    /// * `path` - The path to which the image should be written.
    /// * `color_map` - The color map to use for encoding the pixel contribution.
    pub fn write_image<P: AsRef<Path>, C: ColorMap>(&self, path: P, color_map: C) -> Result<()> {
        let size = self.descriptor.size() as u32;
        let mut img = RgbImage::new(size, size);

        self.pixel_contrib
            .iter()
            .zip(img.pixels_mut())
            .for_each(|(p, pixel)| {
                let c = color_map.map(*p as f64);

                pixel[0] = c.0;
                pixel[1] = c.1;
                pixel[2] = c.2;
            });

        img.save(path)?;

        Ok(())
    }

    /// Writes the pixel contribution map to the given path as binary file.
    ///
    /// # Arguments
    /// * `path` - The path to which the pixel contribution should be written.
    pub fn write_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = std::fs::File::create(path)?;

        self.write_writer(&mut BufWriter::new(file))
    }

    /// Writes the pixel contribution map to the given writer as binary file.
    ///
    /// # Arguments
    /// * `writer` - The writer to which the pixel contribution should be written.
    pub fn write_writer<W: Write>(&self, writer: &mut W) -> Result<()> {
        bincode::serialize_into(writer, self)
            .map_err(|e| Error::Internal(format!("Failed to encode: {}", e)))?;

        Ok(())
    }

    /// Reads the pixel contribution map from the given path.
    ///
    /// # Arguments
    /// * `path` - The path from which the pixel contribution should be read.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;

        Self::from_reader(&mut BufReader::new(file))
    }

    /// Reads the pixel contribution map from the given reader.
    ///
    /// # Arguments
    /// * `reader` - The reader from which the pixel contribution should be read.
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let pixel_contrib = bincode::deserialize_from(reader)
            .map_err(|e| Error::IO(format!("Failed to decode: {}", e)))?;

        Ok(pixel_contrib)
    }
}

/// The descriptor for the pixel contribution map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PixelContribColorMapDescriptor {
    /// The size of the quadratic pixel contribution map.
    map_size: usize,
}

impl PixelContribColorMapDescriptor {
    /// Creates a new descriptor for the pixel contribution map.
    ///
    /// # Arguments
    /// * `size` - The size of the quadratic pixel contribution map.
    pub fn new(size: usize) -> Self {
        Self { map_size: size }
    }

    /// Returns the size of the quadratic pixel contribution map.
    #[inline]
    pub fn size(&self) -> usize {
        self.map_size
    }

    /// Returns total number of values for the pixel contribution map.
    #[inline]
    pub fn num_values(&self) -> usize {
        self.map_size * self.map_size
    }

    /// Returns the camera direction vector for the given index.
    ///
    /// # Arguments
    /// * `index` - The index of the pixel contribution value.
    pub fn camera_dir_from_index(&self, index: usize) -> Vec3 {
        let u = (index % self.map_size) as f32 + 0.5;
        let v = (index / self.map_size) as f32 + 0.5;

        let uv = Vec2::new(u, v) / self.map_size as f32;

        decode_octahedron_normal(uv)
    }

    /// Returns the index for the given camera direction vector.
    ///
    /// # Arguments
    /// * `dir` - The camera direction vector to the object.
    pub fn index_from_camera_dir(&self, dir: Vec3) -> usize {
        let uv = encode_octahedron_normal(dir) * self.map_size as f32 - Vec2::new(0.5, 0.5);

        let u = clamp(uv.x.round() as usize, 0, self.map_size - 1);
        let v = clamp(uv.y.round() as usize, 0, self.map_size - 1);

        v * self.map_size + u
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialization() {
        let descriptor = PixelContribColorMapDescriptor::new(16);

        let mut pixel_contrib = PixelContributionMap::new(descriptor);
        pixel_contrib
            .pixel_contrib
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| {
                *p = i as f32 / 255.0;
            });

        let mut buf = Vec::new();
        pixel_contrib.write_writer(&mut buf).unwrap();

        let pixel_contrib2 = PixelContributionMap::from_reader(&mut buf.as_slice()).unwrap();

        assert_eq!(pixel_contrib.descriptor, pixel_contrib2.descriptor);
        assert_eq!(pixel_contrib.pixel_contrib, pixel_contrib2.pixel_contrib);
    }

    #[test]
    fn test_camera_dir_index_mapping() {
        let map_sizes = [16, 32, 64, 128, 256, 512, 1024];

        for map_size in map_sizes {
            let descriptor = PixelContribColorMapDescriptor::new(map_size);

            for i in 0..descriptor.num_values() {
                let dir = descriptor.camera_dir_from_index(i);
                let index = descriptor.index_from_camera_dir(dir);

                assert_eq!(i, index);
            }
        }
    }
}
