use crate::{DrawCall, GPUBuffer, IndexData, PrimitiveType};

pub struct GPUMesh {
    primitive_type: PrimitiveType,
    draw_call: DrawCall,
    indices: GPUBuffer,
    index_data: IndexData,
}

impl GPUMesh {
    pub fn new(
        primitive_type: PrimitiveType,
        draw_call: DrawCall,
        indices: GPUBuffer,
        index_data: IndexData,
    ) -> Self {
        Self {
            primitive_type,
            draw_call,
            indices,
            index_data,
        }
    }

    /// Draws the gpu mesh
    pub fn draw(&self) {
        self.draw_call
            .draw_with_indices(self.primitive_type, &self.indices, &self.index_data);
    }
}
