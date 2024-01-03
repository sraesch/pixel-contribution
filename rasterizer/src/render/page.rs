use math::Aabb;

use crate::scene::{CompressedPositions, Triangle};

/// The object-id map maps local 8-bit object ids to global 32-bit object ids.
pub type ObjectIdMap = [u32; 256];

/// A single assembled page.
pub struct Page {
    pub num_vertices: usize,

    /// The object-id map that maps from the local object ids to the global object ids.
    /// The index of the array is the local object id and the value is the global object id.
    pub object_id_map: ObjectIdMap,

    /// The list of local object ids for each triangle that are mapped via the object id map to
    /// the global object ids
    pub local_object_ids: Vec<u8>,

    /// The compressed positions
    pub position: CompressedPositions,

    /// The list of triangles
    pub triangles: Vec<Triangle>,

    /// The bounding volume of the page, is used for culling and screen space error computation
    pub aabb: Aabb,
}
