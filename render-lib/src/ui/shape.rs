use std::mem::size_of;

use nalgebra_glm::Vec2;

use crate::{
    Attribute, AttributeBlock, DataType, DrawCall, GPUBuffer, GPUBufferType, GPUMesh, IndexData,
    PrimitiveType,
};

/// A shape describes the geometry of a widget.
pub struct Shape {
    /// The mesh that represents the shape.
    mesh: GPUMesh,
}

impl Shape {
    /// Creates a new shape with the given vertices and indices.
    /// The vertices and indices define a triangle mesh.
    ///
    /// # Arguments
    /// * `vertices` - The vertices of the shape.
    /// * `indices` - The indices of the shape.
    pub fn new(vertices: &[Vec2], indices: &[u32]) -> Self {
        let positions = GPUBuffer::new_with_data(GPUBufferType::Vertices, vertices);

        let mut draw_call = DrawCall::new();
        draw_call.set_data(&[AttributeBlock {
            vertex_data: &positions,
            attributes: vec![Attribute {
                offset: 0,
                stride: size_of::<f32>() * 2,
                num_components: 2,
                data_type: DataType::Float,
                is_integer: false,
                normalized: false,
            }],
        }]);

        let num_indices = indices.len();
        let indices = GPUBuffer::new_with_data(GPUBufferType::Indices, indices);

        let index_data = IndexData {
            offset: 0,
            num: num_indices,
            datatype: DataType::UnsignedInt,
        };

        let mesh = GPUMesh::new(PrimitiveType::Triangles, draw_call, indices, index_data);

        Self { mesh }
    }

    /// Returns rectangle shape.
    pub fn rectangle() -> Self {
        let vertices: [f32; 8] = [1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32];
        let indices: [u32; 6] = [0, 1, 2, 2, 1, 3];

        Self::new(
            &[
                Vec2::new(vertices[0], vertices[1]),
                Vec2::new(vertices[2], vertices[3]),
                Vec2::new(vertices[4], vertices[5]),
                Vec2::new(vertices[6], vertices[7]),
            ],
            &indices,
        )
    }

    /// Renders the shape.
    pub fn render(&self) {
        self.mesh.draw();
    }
}
