mod compressed;
mod geometry;

pub use compressed::*;
pub use geometry::*;
use math::{transform_vec3, Aabb, BoundingSphere};

use std::{collections::HashMap, path::Path};

use cad_import::{
    loader::Manager,
    structure::{CADData, Mesh, Node, PrimitiveType, Shape},
    ID,
};

use log::{info, warn};
use nalgebra_glm::{Mat4, Vec3};

use crate::{Error, Result};

/// A single instantiated geometry
#[derive(Clone)]
pub struct Instance {
    pub geometry_index: usize,
    pub transform: Mat4,
}

/// A scene is the overall loaded geometry data
pub struct Scene {
    geometries: Vec<Geometry>,
    instances: Vec<Instance>,
}

impl Scene {
    /// Creates a new scene by loading the CAD data from the given path.
    ///
    /// # Arguments
    /// * `file_path` - The path to load the CAD data from.
    pub fn new(file_path: &Path) -> Result<Self> {
        let cad_data = Self::load_cad_data(file_path)?;
        Self::new_from_cad(&cad_data)
    }

    /// Creates a new scene by loading the CAD data from the given CAD data object.
    ///
    /// # Arguments
    /// * `cad_data` - The CAD data object to create the scene from.
    pub fn new_from_cad(cad_data: &CADData) -> Result<Self> {
        let root_node = cad_data.get_root_node();
        let mut global_state = GlobalTraversalState::new();

        Self::create_data(root_node, &mut global_state, Mat4::identity())?;

        let (geometries, instances) = global_state.into_geometries_and_instances();

        Ok(Self {
            geometries,
            instances,
        })
    }

    /// Returns an empty scene. Should only be used for debugging.
    pub fn empty() -> Self {
        Self {
            geometries: Vec::new(),
            instances: Vec::new(),
        }
    }

    /// Returns the geometries of the scene.
    pub fn get_geometries(&self) -> &[Geometry] {
        &self.geometries
    }

    /// Returns the instances of the scene, i.e., instantiated geometries.
    pub fn get_instances(&self) -> &[Instance] {
        &self.instances
    }

    /// Computes and returns the AABB bounding volume for the given scene.
    pub fn compute_aabb(&self) -> Aabb {
        let geometries = &self.geometries;

        self.instances.iter().fold(Aabb::new(), |aabb, instance| {
            let geo = &geometries[instance.geometry_index];

            geo.positions
                .iter()
                .map(|p| transform_vec3(&instance.transform, p))
                .fold(aabb, |aabb, p| {
                    let mut aabb = aabb;
                    aabb.extend_pos(&p);

                    aabb
                })
        })
    }

    /// Computes and returns an approximated bounding sphere for the given scene.
    /// It is approximated by computing the AABB and then determining the furthest point
    /// from the center of the AABB. It returns the center and the radius of the sphere.
    pub fn compute_bounding_sphere(&self) -> BoundingSphere {
        let aabb = self.compute_aabb();

        let center = aabb.get_center();

        let geometries = &self.geometries;

        // determine the furthest point from the center of the aabb using quadratic distance
        let radius = self.instances.iter().fold(0f32, |value, instance| {
            let geo = &geometries[instance.geometry_index];

            let d = geo.positions.iter().fold(0f32, |value, p| {
                let p = transform_vec3(&instance.transform, p);
                let d = nalgebra_glm::distance2(&center, &p);

                value.max(d)
            });

            value.max(d)
        });

        BoundingSphere::from((center, radius.sqrt()))
    }

    /// Prints statistics about the loaded scene.
    pub fn print_scene_stats(&self) {
        let mut num_vertices = 0usize;
        let mut num_triangles = 0usize;

        let mut num_unique_vertices = 0usize;
        let mut num_unique_triangles = 0usize;

        info!("Num geometries: {}", self.get_geometries().len());
        info!("Num instances: {}", self.get_instances().len());

        for geo in self.get_geometries().iter() {
            num_unique_vertices += geo.positions.len();
            num_unique_triangles += geo.triangles.len();
        }

        for instance in self.get_instances().iter() {
            let geo = &self.get_geometries()[instance.geometry_index];
            num_vertices += geo.positions.len();
            num_triangles += geo.triangles.len();
        }

        info!("Num Vertices (Unique): {}", num_unique_vertices);
        info!("Num Triangles (Unique): {}", num_unique_triangles);

        info!("Num Vertices (Instantiated): {}", num_vertices);
        info!("Num Triangles (Instantiated): {}", num_triangles);
    }

    /// Tries to find the mime types for the given file based on the file extension.
    ///
    /// # Arguments
    /// * `input_file` - The input file whose extension will be used
    fn determine_mime_types(manager: &Manager, input_file: &Path) -> Result<Vec<String>> {
        match input_file.extension() {
            Some(ext) => match ext.to_str() {
                Some(ext) => Ok(manager.get_mime_types_for_extension(ext)),
                None => Err(Error::InvalidArgument(format!(
                    "Input file {:?} has invalid extension",
                    input_file
                ))),
            },
            None => Err(Error::InvalidArgument(format!(
                "Input file {:?} has no extension",
                input_file
            ))),
        }
    }

    /// Tries to load the cad data from the given path
    ///
    /// # Arguments
    /// * `file_path` - The path to load the CAD data from.
    fn load_cad_data(file_path: &Path) -> Result<CADData> {
        let manager = Manager::new();

        let mime_types = Self::determine_mime_types(&manager, file_path)?;

        for mime_type in mime_types.iter() {
            if let Some(loader) = manager.get_loader_by_mime_type(mime_type.as_str()) {
                let cad_data = loader.read_file(file_path, mime_type)?;

                return Ok(cad_data);
            }
        }

        Err(Error::IO(format!(
            "Cannot find loader for the input file {:?}",
            file_path
        )))
    }

    /// Creates all geometries and instances by traversing over the structure.
    ///
    /// # Arguments
    /// * `node` - The currently visited node.
    /// * `global_state` - Reference onto the global traversal state.
    /// * `transform` - The accumulated transformation matrix.
    fn create_data(
        node: &Node,
        global_state: &mut GlobalTraversalState,
        transform: Mat4,
    ) -> Result<()> {
        // update the transformation matrix
        let transform = match node.get_transform() {
            Some(t) => transform * t,
            None => transform,
        };

        // register all shape instances
        for shape in node.get_shapes() {
            let geometry_index = Self::get_geo_index(global_state, shape.as_ref())?;

            global_state.instances.push(Instance {
                geometry_index,
                transform,
            });
        }

        // visit children and register their geometry
        for child in node.get_children() {
            Self::create_data(child, global_state, transform)?;
        }

        Ok(())
    }

    /// Returns the geometry index for the given shape. If the shape has not been yet processed,
    /// a corresponding geometry object will be created and registered in the shape/geo map.
    ///
    /// # Arguments
    /// * `global_state` - The global state that contains the geometries and the shape/geo map.
    /// * `shape` - The shape to process.
    fn get_geo_index(global_state: &mut GlobalTraversalState, shape: &Shape) -> Result<usize> {
        let shape_geo_map = &mut global_state.shape_geo_map;
        let geometries = &mut global_state.geometries;

        let shape_id = shape.get_id();

        // check if the shape has already been processed
        if let Some(geo_index) = shape_geo_map.get(&shape_id) {
            return Ok(*geo_index);
        }

        // create geometry from the shape and retrieve geometry index
        let geo = Self::create_geometry_from_shape(shape)?;
        let geometry_index = geometries.len();
        geometries.push(geo);

        // add entry to the shape/geo map
        shape_geo_map.insert(shape_id, geometry_index);

        Ok(geometry_index)
    }

    /// Creates a geometry object from the given shape.
    ///
    /// # Arguments
    /// * `shape` - The shape whose data are transformed into a geometry object.
    fn create_geometry_from_shape(shape: &Shape) -> Result<Geometry> {
        let mut positions = Vec::new();
        let mut triangles = Vec::new();

        for part in shape.get_parts() {
            Self::extend_mesh(&mut positions, &mut triangles, part.get_mesh().as_ref())?;
        }

        let geo = Geometry::new(positions, triangles)?;

        Ok(geo)
    }

    /// Extends the given mutable references onto the positions and triangles
    /// by adding the given CAD mesh.
    ///
    /// # Arguments
    /// * `dst_positions` - Mutable reference for storing the parsed positions
    /// * `dst_triangles` - Mutable reference for storing the parsed triangles
    /// * `mesh` - The CAD-mesh that is transformed into a geometry.
    fn extend_mesh(
        dst_positions: &mut Vec<Vec3>,
        dst_triangles: &mut Vec<Triangle>,
        mesh: &Mesh,
    ) -> Result<()> {
        let primitives = mesh.get_primitives();
        let positions = mesh.get_vertices().get_positions();

        let index_offset = dst_positions.len() as u32;

        match primitives.get_primitive_type() {
            PrimitiveType::Triangles => {
                match primitives.get_raw_index_data().get_indices_ref() {
                    Some(indices) => {
                        dst_triangles.extend(indices.iter().as_slice().windows(3).step_by(3).map(
                            |t| {
                                [
                                    t[0] + index_offset,
                                    t[1] + index_offset,
                                    t[2] + index_offset,
                                ]
                            },
                        ));
                    }
                    None => {
                        warn!(
                            "Geometry without indices is not being supported and will be skipped",
                        );
                        return Ok(());
                    }
                }

                dst_positions.extend(positions.iter().map(|p| Vec3::new(p.0[0], p.0[1], p.0[2])));

                Ok(())
            }
            _ => {
                warn!(
                    "Primitive '{:?}' is not supported and is being skipped!",
                    primitives.get_primitive_type()
                );
                Ok(())
            }
        }
    }
}

struct GlobalTraversalState {
    pub geometries: Vec<Geometry>,
    pub instances: Vec<Instance>,
    pub shape_geo_map: HashMap<ID, usize>,
}

impl GlobalTraversalState {
    /// Returns a new empty global traversal context.
    pub fn new() -> Self {
        Self {
            geometries: Vec::new(),
            instances: Vec::new(),
            shape_geo_map: HashMap::new(),
        }
    }

    /// Transforms the scene into an array of geometries and instances.
    pub fn into_geometries_and_instances(self) -> (Vec<Geometry>, Vec<Instance>) {
        let geometries = self.geometries;
        let instances = self.instances;

        (geometries, instances)
    }
}
