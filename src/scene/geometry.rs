use anyhow::{bail, Result};
use nalgebra_glm::Vec3;

pub type Triangle = [u32; 3];

/// A single piece of triangulated geometry
#[derive(Clone)]
pub struct Geometry {
    pub positions: Vec<Vec3>,
    pub triangles: Vec<Triangle>,
}

impl Geometry {
    /// Returns new geometry object based on the provided triangle geometry
    ///
    /// # Arguments
    /// * `positions` - A list of vertex positions that span the geometry
    /// * `triangles` - A list of triangles defined upon the vertex positions
    pub fn new(positions: Vec<Vec3>, triangles: Vec<Triangle>) -> Result<Self> {
        // check if the maximal index is out of range
        if let Some(max_index) = triangles.iter().map(|t| t.iter().max().unwrap()).max() {
            if *max_index as usize >= positions.len() {
                bail!(
                    "The maximal index is {}, but there are only {} positions",
                    *max_index,
                    positions.len()
                );
            }
        }

        Ok(Self {
            positions,
            triangles,
        })
    }
}
