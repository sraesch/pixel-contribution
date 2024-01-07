use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use image::RgbImage;
use serde::{Deserialize, Serialize};

use crate::{ColorMap, Error, Result};

/// The resulting pixel contribution for all possible views.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct PixelContributionMap {
    /// The size of the quadratic pixel contribution map.
    pub size: usize,

    /// The 2D map for the pixel contribution of each view. Each pixel position represents a view.
    /// The normalized pixel position (u,v) is mapped to a normal using octahedral projection.
    /// The normal then defines the camera view direction.
    /// The values are in the range [0, 1].
    pub pixel_contrib: Vec<f32>,
}

impl PixelContributionMap {
    /// Creates a new pixel contribution map with the given size.
    ///
    /// # Arguments
    /// * `size` - The size of the quadratic pixel contribution map.
    pub fn new(size: usize) -> Self {
        Self {
            size,
            pixel_contrib: vec![0.0; size * size],
        }
    }

    /// Writes the pixel contribution map to the given path as image.
    ///
    /// # Arguments
    /// * `path` - The path to which the image should be written.
    /// * `color_map` - The color map to use for encoding the pixel contribution.
    pub fn write_image<P: AsRef<Path>, C: ColorMap>(&self, path: P, color_map: C) -> Result<()> {
        let mut img = RgbImage::new(self.size as u32, self.size as u32);

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialization() {
        let mut pixel_contrib = PixelContributionMap::new(16);
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

        assert_eq!(pixel_contrib.size, pixel_contrib2.size);
        assert_eq!(pixel_contrib.pixel_contrib, pixel_contrib2.pixel_contrib);
    }
}
