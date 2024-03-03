use std::{collections::HashMap, mem::size_of, path::Path};

use anyhow::{bail, Context, Result};
use cad_import::{
    loader::Manager,
    structure::{CADData, IndexData, Node, PrimitiveType, Shape},
    ID,
};
use math::{transform_vec3, Aabb};
use nalgebra_glm::{distance2, identity, Mat4, Vec3};
use rasterizer::BoundingSphere;
use render_lib::{Attribute, AttributeBlock, DataType, DrawCall, GPUBuffer, GPUBufferType, Shader};

/// A CAD model that can be rendered.
pub struct CADModel {
    gpu_meshes: Vec<GPUMesh>,
    instances: Vec<GPUMeshInstance>,

    shader: Shader,
    uniform_combined_mat: render_lib::Uniform,
    uniform_model_view_mat: render_lib::Uniform,

    aabb_volume: Aabb,
    bounding_sphere: BoundingSphere,
}

impl CADModel {
    /// Creates a new CAD model from the given file.
    ///
    /// # Arguments
    /// * `filename` - The path to the CAD file to load.
    pub fn new(filename: &Path) -> Result<Self> {
        let cad_data = Self::load_cad_data(filename)?;

        // Determine the AABB bounding volume of the CAD model. Will be used to compute the center
        // of the bounding sphere.
        let mut bounding_volume = Aabb::default();
        traverse(
            cad_data.get_root_node(),
            identity(),
            &mut update_node_transform,
            &mut bounding_volume,
            &mut |node, bounding_volume, transform| {
                node.get_shapes().iter().for_each(|shape| {
                    shape.get_parts().iter().for_each(|part| {
                        let mesh = part.get_mesh();
                        let mesh_vertices = mesh.get_vertices();

                        // extend the bounding volume by all vertices of the current shape
                        bounding_volume.extend_iter(
                            mesh_vertices
                                .get_positions()
                                .iter()
                                .map(|pos| transform_vec3(transform, &pos.0)),
                        );
                    });
                });
            },
        );

        // Determine the radius of the bounding sphere
        let mut bounding_sphere_radius = 0f32;
        let center = bounding_volume.get_center();
        traverse(
            cad_data.get_root_node(),
            identity(),
            &mut update_node_transform,
            &mut bounding_sphere_radius,
            &mut |node, radius, transform| {
                node.get_shapes().iter().for_each(|shape| {
                    shape.get_parts().iter().for_each(|part| {
                        let mesh = part.get_mesh();
                        let mesh_vertices = mesh.get_vertices();

                        // update the radius based on the vertices of the current shape
                        mesh_vertices.get_positions().iter().for_each(|pos| {
                            *radius =
                                radius.max(distance2(&transform_vec3(transform, &pos.0), &center))
                        });
                    });
                });
            },
        );

        bounding_sphere_radius = bounding_sphere_radius.sqrt();

        let bounding_sphere =
            BoundingSphere::from((bounding_volume.get_center(), bounding_sphere_radius));

        // create the shape map and the gpu meshes from the loaded cad data
        let mut global_context: (Vec<GPUMeshInstance>, Vec<GPUMesh>, HashMap<ID, usize>) =
            Default::default();
        traverse(
            cad_data.get_root_node(),
            identity(),
            &mut update_node_transform,
            &mut global_context,
            &mut |node, global_context, transform| {
                node.get_shapes().iter().for_each(|shape| {
                    let id = shape.get_id();

                    let instances = &mut global_context.0;
                    let gpu_meshes = &mut global_context.1;
                    let shape_map = &mut global_context.2;

                    // Get the corresponding gpu mesh index.
                    // Create the gpu mesh for the shape and register it in the shape map if it does not exist
                    // yet
                    let gpu_mesh_index = *shape_map.entry(id).or_insert_with(|| {
                        let new_index = gpu_meshes.len();
                        let (vertices, indices) = accumulate_mesh_data(shape);
                        let gpu_mesh = GPUMesh::new(&vertices, &indices);
                        gpu_meshes.push(gpu_mesh);

                        new_index
                    });

                    // create the instance
                    instances.push(GPUMeshInstance {
                        transform: *transform,
                        gpu_mesh: gpu_mesh_index,
                    });
                });
            },
        );

        let (instances, gpu_meshes, _) = global_context;

        // create the shader
        let mut shader = Shader::default();
        shader.load(
            include_str!("../shader/model.vert"),
            include_str!("../shader/model.frag"),
        )?;

        let uniform_combined_mat = shader.get_uniform("uniform_combined_mat")?;
        let uniform_model_view_mat = shader.get_uniform("uniform_model_view_mat")?;

        Ok(Self {
            gpu_meshes,
            instances,
            shader,
            uniform_combined_mat,
            uniform_model_view_mat,
            aabb_volume: bounding_volume,
            bounding_sphere,
        })
    }

    /// Renders the whole CAD model.
    ///
    /// # Arguments
    /// * `model_view_mat` - The model view matrix.
    /// * `proj_mat` - The projection matrix.
    pub fn render(&self, model_view_mat: &Mat4, proj_mat: &Mat4) {
        self.shader.bind();

        self.instances.iter().for_each(|instance| {
            let m = model_view_mat * instance.transform;
            let combined_mat = proj_mat * m;

            self.uniform_model_view_mat.set_matrix4(&m);
            self.uniform_combined_mat.set_matrix4(&combined_mat);

            self.gpu_meshes[instance.gpu_mesh].render();
        });
    }

    /// Returns the bounding sphere of the CAD model.
    #[inline]
    pub fn get_bounding_sphere(&self) -> &BoundingSphere {
        &self.bounding_sphere
    }

    /// Returns the AABB volume of the CAD model.
    #[inline]
    pub fn get_aabb_volume(&self) -> &Aabb {
        &self.aabb_volume
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
                let cad_data = loader
                    .read_file(file_path, mime_type)
                    .context(format!("Failed reading input file {:?}", file_path))?;

                return Ok(cad_data);
            }
        }

        bail!("Cannot find loader for the input file {:?}", file_path);
    }

    /// Tries to find the mime types for the given file based on the file extension.
    ///
    /// # Arguments
    /// * `manager` - The loader manager to use for the mime type lookup
    /// * `input_file` - The input file whose extension will be used
    fn determine_mime_types(manager: &Manager, input_file: &Path) -> Result<Vec<String>> {
        match input_file.extension() {
            Some(ext) => match ext.to_str() {
                Some(ext) => Ok(manager.get_mime_types_for_extension(ext)),
                None => {
                    bail!("Input file has invalid extension");
                }
            },
            None => {
                bail!("Input file has no extension");
            }
        }
    }
}

struct GPUMesh {
    positions: GPUBuffer,
    indices: GPUBuffer,
    num_indices: usize,

    draw_call: DrawCall,
}

/// A single instantiated GPU mesh.
struct GPUMeshInstance {
    pub transform: Mat4,
    pub gpu_mesh: usize,
}

impl GPUMesh {
    /// Creates a new GPU mesh from the given vertices and indices.
    ///
    /// # Arguments
    /// * `vertices` - The vertices of the mesh
    /// * `indices` - The indices of the mesh
    pub fn new(vertices: &[Vec3], indices: &[u32]) -> Self {
        let mut gpu_positions = GPUBuffer::new(GPUBufferType::Vertices);
        gpu_positions.set_data(vertices);

        let num_indices = indices.len();

        let mut gpu_indices = GPUBuffer::new(GPUBufferType::Indices);
        gpu_indices.set_data(indices);

        let mut gpu_mesh = Self {
            positions: gpu_positions,
            indices: gpu_indices,
            num_indices,

            draw_call: DrawCall::default(),
        };

        gpu_mesh.draw_call.set_data(&[AttributeBlock {
            vertex_data: &gpu_mesh.positions,
            attributes: vec![Attribute {
                offset: 0,
                stride: size_of::<f32>() * 3,
                num_components: 3,
                data_type: DataType::Float,
                is_integer: false,
                normalized: false,
            }],
        }]);

        gpu_mesh
    }

    /// Renders the mesh.
    pub fn render(&self) {
        self.draw_call.draw_with_indices(
            render_lib::PrimitiveType::Triangles,
            &self.indices,
            &render_lib::IndexData {
                datatype: DataType::UnsignedInt,
                offset: 0,
                num: self.num_indices,
            },
        );
    }
}

/// Traverses the given node and all its children.
/// There are two different context objects that can be used to store data during the traversal.
/// The first one is the `traversal_context`, which is only updated down the tree. This can be data
/// like the current transformation matrix.
/// The second one is the `global_context`, which is a global data like maps or other data
/// structures.
///
/// # Arguments
/// * `node` - The node to traverse.
/// * `shape_map` - The map that maps a shape id to the corresponding gpu mesh index.
/// * `gpu_meshes` - The gpu meshes that will be created.
/// * `instances` - The instances that will be created.
/// * `transform` - The transformation matrix of the parent node.
fn traverse<TraversalContext, UpdateTraversalContext, GlobalContext, UpdateGlobalContext>(
    node: &Node,
    traversal_context: TraversalContext,
    update_context: &mut UpdateTraversalContext,
    global_context: &mut GlobalContext,
    update_global_context: &mut UpdateGlobalContext,
) where
    UpdateTraversalContext: FnMut(&Node, TraversalContext) -> TraversalContext,
    TraversalContext: Clone,
    UpdateGlobalContext: FnMut(&Node, &mut GlobalContext, &TraversalContext),
{
    let traversal_context = update_context(node, traversal_context);

    update_global_context(node, global_context, &traversal_context);

    node.get_children().iter().for_each(|child| {
        traverse(
            child,
            traversal_context.clone(),
            update_context,
            global_context,
            update_global_context,
        );
    });
}

/// Updates the given transformation matrix with the transformation of the given node.
///
/// # Arguments
/// * `node` - The node to update the transformation matrix with.
/// * `transform` - The current transformation matrix.
fn update_node_transform(node: &Node, transform: Mat4) -> Mat4 {
    match node.get_transform() {
        Some(t) => transform * t,
        None => transform,
    }
}

/// Accumulates the parts of the given shape into a single vertex and index buffer.
///
/// # Arguments
/// * `shape` - The shape to accumulate.
fn accumulate_mesh_data(shape: &Shape) -> (Vec<Vec3>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    shape.get_parts().iter().for_each(|part| {
        let mesh = part.get_mesh();
        let primitives = mesh.get_primitives();
        let mesh_vertices = mesh.get_vertices();

        // we only support triangles
        if primitives.get_primitive_type() != PrimitiveType::Triangles {
            return;
        }

        // we only support 32-bit indices
        let mesh_indices = if let IndexData::Indices(mesh_indices) = primitives.get_raw_index_data()
        {
            mesh_indices
        } else {
            return;
        };

        // add indices to the index buffer
        let vertex_offset = vertices.len() as u32;
        indices.extend(mesh_indices.iter().map(|index| index + vertex_offset));

        // add vertices to the vertex buffer
        vertices.extend(mesh_vertices.get_positions().iter().map(|pos| pos.0));
    });

    (vertices, indices)
}
