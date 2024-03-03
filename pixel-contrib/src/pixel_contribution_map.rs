use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use byteorder::{ReadBytesExt, WriteBytesExt};
use image::RgbImage;
use nalgebra_glm::{Vec2, Vec3};
use rasterizer::clamp;

use crate::{
    octahedron::{decode_octahedron_normal, encode_octahedron_normal},
    ColorMap, Error, Result,
};

const PIXEL_CONTRIBUTION_MAP_VERSION: u32 = 1;
const PIXEL_CONTRIBUTION_MAP_IDENTIFIER: [u8; 4] = *b"PCMP";

/// The pixel contribution maps for different configurations
#[derive(Clone, PartialEq)]
pub struct PixelContributionMaps {
    maps: Vec<PixelContributionMap>,
}

impl Default for PixelContributionMaps {
    fn default() -> Self {
        Self::new()
    }
}

impl PixelContributionMaps {
    pub fn new() -> Self {
        Self { maps: Vec::new() }
    }

    /// Creates a new pixel contribution maps object from the given maps.
    ///
    /// # Arguments
    /// * `maps` - The pixel contribution maps to use.
    pub fn from_maps(mut maps: Vec<PixelContributionMap>) -> Self {
        Self::sort_maps(&mut maps);

        Self { maps }
    }

    /// Adds a new pixel contribution map.
    ///
    /// # Arguments
    /// * `map` - The pixel contribution map to add.
    pub fn add_map(&mut self, map: PixelContributionMap) {
        self.maps.push(map);
        Self::sort_maps(&mut self.maps);
    }

    /// Returns a reference to the pixel contribution maps.
    pub fn get_maps(&self) -> &[PixelContributionMap] {
        &self.maps
    }

    /// Returns the pixel contribution for the given camera direction vector.
    ///
    /// # Arguments
    /// * `dir` - The camera direction vector to the object.
    /// * `angle` - The angle of the camera to return the contribution values for.
    pub fn get_pixel_contrib_for_camera_dir(&self, dir: Vec3, angle: f32) -> f32 {
        let (i0, i1) = self.search_starting_map_for_angle(angle);

        let map0 = &self.maps[i0];
        let p0 = map0.get_pixel_contrib_for_camera_dir(dir);
        if let Some(i1) = i1 {
            let map1 = &self.maps[i1];
            let p1 = map1.get_pixel_contrib_for_camera_dir(dir);

            let a0 = map0.descriptor.camera_angle();
            let a1 = map1.descriptor.camera_angle();

            let t = (a1 - angle) / (a1 - a0);

            p0 * t + p1 * (1.0 - t)
        } else {
            p0
        }
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
        let header: PixelContributionMapHeader = Default::default();

        // Write the header
        writer.write_all(header.identifier.as_ref())?;
        writer.write_u32::<byteorder::LittleEndian>(header.version)?;

        // Write the number of pixel contribution maps
        writer.write_u32::<byteorder::LittleEndian>(self.maps.len() as u32)?;

        // Write the pixel contribution maps
        for map in &self.maps {
            // Write the descriptor
            let map_size = map.descriptor.map_size as u32;
            let angle = map.descriptor.camera_angle;

            writer.write_u32::<byteorder::LittleEndian>(map_size)?;
            writer.write_f32::<byteorder::LittleEndian>(angle)?;

            // Write the pixel contribution
            for x in &map.pixel_contrib {
                writer.write_f32::<byteorder::LittleEndian>(*x)?;
            }
        }

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
        // Read the header and check if it is valid
        let mut header = PixelContributionMapHeader::default();
        reader.read_exact(header.identifier.as_mut())?;
        header.version = reader.read_u32::<byteorder::LittleEndian>()?;
        header.check()?;

        // Read the number of pixel contribution maps
        let num_maps = reader.read_u32::<byteorder::LittleEndian>()? as usize;

        // Read the pixel contribution maps
        let mut maps = Vec::with_capacity(num_maps);
        for _ in 0..num_maps {
            // Read the descriptor
            let map_size = reader.read_u32::<byteorder::LittleEndian>()? as usize;
            let angle = reader.read_f32::<byteorder::LittleEndian>()?;

            let descriptor = PixelContribColorMapDescriptor::new(map_size, angle);

            // Read the pixel contribution
            let mut pixel_contrib = Vec::with_capacity(map_size * map_size);
            for _ in 0..map_size * map_size {
                pixel_contrib.push(reader.read_f32::<byteorder::LittleEndian>()?);
            }

            maps.push(PixelContributionMap {
                descriptor,
                pixel_contrib,
            });
        }

        Self::sort_maps(&mut maps);

        Ok(Self { maps })
    }

    /// Helper function used to ensure that the maps are always sorted in ascending order w.r.t
    /// their camera angles.
    ///
    /// # Arguments
    /// * `maps` - The maps to sort.
    fn sort_maps(maps: &mut [PixelContributionMap]) {
        maps.sort_by(|m1, m2| {
            m1.descriptor
                .camera_angle()
                .partial_cmp(&m2.descriptor.camera_angle())
                .unwrap()
        });
    }

    /// Searches for the pair of maps where the given angle is between their camera angles.
    /// If the angle os out of range, only one angle is being returned.
    ///
    /// # Arguments
    /// * `angle` - The given angle to search the pair of maps for.
    fn search_starting_map_for_angle(&self, angle: f32) -> (usize, Option<usize>) {
        for (i, m) in self.maps.iter().enumerate() {
            let cur_angle = m.descriptor.camera_angle();

            // If the current angle is already too large, then it must be the first element and we
            // stop.
            // If we've reached the end, we also stop here
            if cur_angle > angle || i + 1 >= self.maps.len() {
                return (i, None);
            }

            // get the next angle and check if it is larger or equal
            let next_angle = self.maps[i + 1].descriptor.camera_angle();
            if next_angle >= angle {
                return (i, Some(i + 1));
            }
        }

        (self.maps.len() - 1, None)
    }
}

/// The resulting pixel contribution for all possible views.
#[derive(Clone, PartialEq)]
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

    /// Returns the pixel contribution for the given camera direction vector.
    ///
    /// # Arguments
    /// * `dir` - The camera direction vector to the object.
    pub fn get_pixel_contrib_for_camera_dir(&self, dir: Vec3) -> f32 {
        let index = self.descriptor.index_from_camera_dir(dir);
        self.pixel_contrib[index]
    }
}

/// The descriptor for the pixel contribution map.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PixelContribColorMapDescriptor {
    /// The size of the quadratic pixel contribution map.
    map_size: usize,

    /// The camera angle for the pixel contribution map. The angle is in radians.
    /// A value of 0 means that the camera is orthographic.
    camera_angle: f32,
}

impl PixelContribColorMapDescriptor {
    /// Creates a new descriptor for the pixel contribution map.
    ///
    /// # Arguments
    /// * `size` - The size of the quadratic pixel contribution map.
    /// * `camera_angle` - The camera angle for the pixel contribution map.
    ///                    The angle is in radians. A value of 0 means that the camera is
    ///                    orthographic.
    pub fn new(size: usize, camera_angle: f32) -> Self {
        Self {
            map_size: size,
            camera_angle,
        }
    }

    /// Returns the size of the quadratic pixel contribution map.
    #[inline]
    pub fn size(&self) -> usize {
        self.map_size
    }

    /// Returns the camera angle for the pixel contribution map.
    /// The angle is in radians. A value of 0 means that the camera is orthographic.
    #[inline]
    pub fn camera_angle(&self) -> f32 {
        self.camera_angle
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

/// The header for the pixel contribution map used for serialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PixelContributionMapHeader {
    /// The identifier for the pixel contribution map file.
    identifier: [u8; 4],

    /// The version of the pixel contribution map file.
    version: u32,
}

impl Default for PixelContributionMapHeader {
    fn default() -> Self {
        Self {
            identifier: PIXEL_CONTRIBUTION_MAP_IDENTIFIER,
            version: PIXEL_CONTRIBUTION_MAP_VERSION,
        }
    }
}

impl PixelContributionMapHeader {
    /// Checks if the given header is valid.
    ///
    /// # Arguments
    /// * `header` - The header to check.
    pub fn check(&self) -> Result<()> {
        if self.identifier != PIXEL_CONTRIBUTION_MAP_IDENTIFIER {
            return Err(Error::IO("Invalid identifier".to_string()));
        }

        if self.version != PIXEL_CONTRIBUTION_MAP_VERSION {
            return Err(Error::IO("Invalid version".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialization() {
        let angle = 90.0f32.to_radians();
        let descriptor = PixelContribColorMapDescriptor::new(16, angle);

        let mut pixel_contrib = PixelContributionMap::new(descriptor);
        pixel_contrib
            .pixel_contrib
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| {
                *p = i as f32 / 255.0;
            });

        let pixel_contribs = PixelContributionMaps {
            maps: vec![pixel_contrib.clone()],
        };

        let mut buf = Vec::new();
        pixel_contribs.write_writer(&mut buf).unwrap();

        let pixel_contribs2 = PixelContributionMaps::from_reader(&mut buf.as_slice()).unwrap();
        assert_eq!(pixel_contribs2.get_maps().len(), 1);

        let pixel_contrib2 = &pixel_contribs2.get_maps()[0];
        assert_eq!(pixel_contrib.descriptor, pixel_contrib2.descriptor);
        assert_eq!(pixel_contrib.pixel_contrib, pixel_contrib2.pixel_contrib);
    }

    #[test]
    fn test_serialization2() {
        let random_bytes = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0xFF];
        assert!(PixelContributionMaps::from_reader(&mut &random_bytes[..]).is_err());
    }

    #[test]
    fn test_camera_dir_index_mapping() {
        let map_sizes = [16, 32, 64, 128, 256, 512, 1024];

        for map_size in map_sizes {
            let descriptor = PixelContribColorMapDescriptor::new(map_size, 0f32);

            for i in 0..descriptor.num_values() {
                let dir = descriptor.camera_dir_from_index(i);
                let index = descriptor.index_from_camera_dir(dir);

                assert_eq!(i, index);
            }
        }
    }
}
