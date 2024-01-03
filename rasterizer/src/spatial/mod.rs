use std::{collections::HashMap, ops::Range};

use math::{transform_vec3, Aabb};
use nalgebra_glm::{Mat4, Vec3};

use crate::{
    render::{ObjectIdMap, Page},
    scene::{CompressedPositions, Geometry, NumBits, Triangle},
};

pub mod simple;

/// A slice of geometry, i.e., a only a part of the geometry is being referenced.
pub struct GeometrySlice<'a> {
    /// Reference to the geometry where the chunk comes from
    pub geometry: &'a Geometry,

    /// The range of vertices
    pub vertex_range: Range<usize>,

    /// The triangle range for this chunk
    pub triangle_range: Range<usize>,
}

impl<'a> GeometrySlice<'a> {
    /// Returns the slice of vertices associated to the geometry slice.
    #[inline]
    pub fn get_vertices(&self) -> &[Vec3] {
        let positions = &self.geometry.positions;
        &positions[self.vertex_range.start..self.vertex_range.end]
    }

    /// Returns the slice of triangles associated to the geometry slice.
    #[inline]
    pub fn get_triangles(&self) -> &[Triangle] {
        let triangles = &self.geometry.triangles;
        &triangles[self.triangle_range.start..self.triangle_range.end]
    }
}

/// An instantiated chunk of geometry belonging to an object.
pub struct Chunk<'a> {
    /// The slice of geometry that belongs to the chunk
    pub geo_slice: GeometrySlice<'a>,

    /// The transformation of the chunk into global coordinate system
    pub transform: Mat4,

    /// The respective object it.
    pub id: u32,
}

impl<'a> Chunk<'a> {
    /// Computes and returns the AABB volume of the chunk.
    pub fn compute_aabb(&self) -> Aabb {
        let mut aabb = Aabb::new();

        let chunk_vertices = self.geo_slice.get_vertices();
        for vertex in chunk_vertices.iter() {
            let p = transform_vec3(&self.transform, vertex);
            aabb.extend_pos(&p);
        }

        aabb
    }
}

/// A reference page is a spatially sorted composition of chunks of geometry.
/// A page reference is only defined through reference and does not own the geometry data.
pub struct PageReference {
    /// The sorted list of chunks that define the page
    pub chunks: Range<usize>,
}

impl PageReference {
    /// Computes and returns the AABB volume for the page.
    ///
    /// # Arguments
    /// * `page_references` - The page references object to which this page reference belongs to.
    pub fn compute_aabb(&self, page_references: &PageReferences) -> Aabb {
        let mut aabb = Aabb::new();

        let chunks = &page_references.chunks;
        for chunk in &chunks[self.chunks.start..self.chunks.end] {
            let vol = chunk.compute_aabb();
            aabb.extend_bbox(&vol);
        }

        aabb
    }

    /// Returns true if the page reference is empty and false otherwise.
    ///
    /// # Arguments
    /// * `page_references` - The page references object to which this page reference belongs to.
    pub fn is_empty(&self, page_references: &PageReferences) -> bool {
        let chunks = &page_references.chunks;
        for chunk in &chunks[self.chunks.start..self.chunks.end] {
            if !chunk.geo_slice.triangle_range.is_empty() {
                return false;
            }
        }

        true
    }
}

/// A list of spatially sorted pages.
pub struct PageReferences<'a> {
    /// All the chunks of the spatial sorting, sorted in a certain order
    chunks: Vec<Chunk<'a>>,

    /// The list of pages.
    pages: Vec<PageReference>,
}

impl<'a> PageReferences<'a> {
    /// Creates a new page references object based on the given data.
    pub fn new(chunks: Vec<Chunk<'a>>, pages: Vec<PageReference>) -> Self {
        Self { chunks, pages }
    }

    /// Creates and returns the pages based on the page references.
    pub fn create_pages(&self) -> Vec<Page> {
        let pages = self
            .pages
            .iter()
            .filter(|page| !page.is_empty(self))
            .map(|page_ref| {
                let mut positions_raw = Vec::new();
                let mut triangles = Vec::new();
                let mut local_object_ids = Vec::new();
                let mut global_to_local_ids: HashMap<u32, u8> = HashMap::new();

                // iterate over the chunks of the current page reference
                for chunk in &self.chunks[page_ref.chunks.start..page_ref.chunks.end] {
                    let vertex_offset = positions_raw.len() as u32;
                    let local_id = global_to_local_ids.get_local_id(chunk.id);

                    // append transformed positions to the resulting positions vector
                    let in_positions = chunk.geo_slice.get_vertices();
                    positions_raw.extend(
                        in_positions
                            .iter()
                            .map(|p| transform_vec3(&chunk.transform, p)),
                    );

                    // append the local object id for each triangle
                    local_object_ids.resize(chunk.geo_slice.triangle_range.len(), local_id);

                    // append triangles to the page
                    let chunk_vertex_offset = chunk.geo_slice.vertex_range.start as u32;
                    let in_triangles = chunk.geo_slice.get_triangles();
                    triangles.extend(in_triangles.iter().map(|t| {
                        debug_assert!(
                            t[0] >= chunk_vertex_offset
                                && t[1] >= chunk_vertex_offset
                                && t[2] >= chunk_vertex_offset
                        );

                        [
                            t[0] - chunk_vertex_offset + vertex_offset,
                            t[1] - chunk_vertex_offset + vertex_offset,
                            t[2] - chunk_vertex_offset + vertex_offset,
                        ]
                    }));
                }

                let num_vertices = positions_raw.len();
                let num_bits = NumBits::Bit16;
                let position = CompressedPositions::new(&positions_raw, num_bits);

                let aabb = positions_raw.iter().fold(Aabb::new(), |mut aabb, p| {
                    aabb.extend_pos(p);
                    aabb
                });

                // create the reverse map, i.e., local id -> global id
                let object_id_map = global_to_local_ids.create_object_id_map();

                Page {
                    num_vertices,
                    object_id_map,
                    local_object_ids,
                    position,
                    triangles,
                    aabb,
                }
            })
            .collect();

        pages
    }
}

/// Trait used internally for creating the local object ids and the corresponding id map
trait GlobalObjectIDMapBuilder {
    /// Returns a local id for the given global id.
    ///
    /// # Arguments
    /// * `global_id` - The global id that gets mapped to the local one.
    fn get_local_id(&mut self, global_id: u32) -> u8;

    /// Returns the resulting object id map
    fn create_object_id_map(&self) -> ObjectIdMap;
}

impl GlobalObjectIDMapBuilder for HashMap<u32, u8> {
    fn get_local_id(&mut self, global_id: u32) -> u8 {
        let potential_new_local_id = self.len();
        assert!(
            potential_new_local_id < 256,
            "Local id must be less than 256"
        );
        let local_id = self
            .entry(global_id)
            .or_insert(potential_new_local_id as u8);

        *local_id
    }

    fn create_object_id_map(&self) -> ObjectIdMap {
        let mut object_id_map: ObjectIdMap = [0u32; 256];
        self.iter().for_each(|(global_id, local_id)| {
            object_id_map[*local_id as usize] = *global_id;
        });

        object_id_map
    }
}
