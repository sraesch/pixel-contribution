use std::{
    io::{BufReader, BufWriter},
    path::Path,
};

use image::RgbImage;
use pixel_contrib_types::{PixelContributionMap, PixelContributionMaps};

use crate::{ColorMap, Result};

/// Additional functions for loading and saving pixel contribution maps from and to files.
pub trait PixelContributionMapFile: Sized {
    /// Writes the pixel contribution map to the given path as binary file.
    ///
    /// # Arguments
    /// * `path` - The path to which the pixel contribution should be written.
    fn write_file<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Reads the pixel contribution map from the given path.
    ///
    /// # Arguments
    /// * `path` - The path from which the pixel contribution should be read.
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
}

impl PixelContributionMapFile for PixelContributionMaps {
    fn write_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = std::fs::File::create(path)?;

        self.write_writer(&mut BufWriter::new(file))?;

        Ok(())
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let result = Self::from_reader(&mut BufReader::new(file))?;

        Ok(result)
    }
}

/// Additional functions for exporting the pixel contribution map as image.
pub trait PixelContributionMapImageExport {
    /// Writes the pixel contribution map to the given path as image.
    ///
    /// # Arguments
    /// * `path` - The path to which the image should be written.
    /// * `color_map` - The color map to use for encoding the pixel contribution.
    fn write_image<P: AsRef<Path>, C: ColorMap>(&self, path: P, color_map: C) -> Result<()>;
}

impl PixelContributionMapImageExport for PixelContributionMap {
    fn write_image<P: AsRef<Path>, C: ColorMap>(&self, path: P, color_map: C) -> Result<()> {
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
}
