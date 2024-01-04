use gl::types::*;

use std::ops::Drop;

use crate::{Bind, DataType, Error, GPUBuffer, Resource, Result};

/// The primitive type to be rendered
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum PrimitiveType {
    Points = 0,
    Lines = 1,
    LineLoop = 2,
    LineStrip = 3,
    Triangles = 4,
    TriangleStrip = 5,
    TriangleFan = 6,
}

/// An attribute of the vertex array on the GPU
pub struct Attribute {
    /// Specifies the number of components per generic vertex attribute. Must be 1, 2, 3, 4
    pub num_components: usize,

    /// The data type of the attribute, e.g., f32, i32, etc.
    pub data_type: DataType,

    /// Normalizes integer values, i.e., values are mapped to fixed point values in the range
    /// [-1, 1] or [0, 1] for signed or unsigned data types, respectively.
    pub normalized: bool,

    /// Specifies the byte offset between consecutive generic vertex attributes. If stride is 0,
    /// the generic vertex attributes are understood to be tightly packed in the array.
    pub stride: usize,

    /// The offset in bytes where the attribute starts
    pub offset: usize,

    /// This flag determines if the attribute is given over as integer attribute
    pub is_integer: bool,
}

impl Attribute {
    /// Binds the attribute to the given index
    ///
    ///* `index` - The index of the attribute.
    pub fn bind(&self, index: u32) {
        let offset = unsafe { std::ptr::null::<std::ffi::c_void>().add(self.offset) };

        if self.is_integer {
            gl_call!(gl::VertexAttribIPointer(
                index as GLuint,
                self.num_components as GLint,
                self.data_type as GLenum,
                self.stride as GLsizei,
                offset,
            ));
        } else {
            gl_call!(gl::VertexAttribPointer(
                index as GLuint,
                self.num_components as GLint,
                self.data_type as GLenum,
                self.normalized as u8,
                self.stride as GLsizei,
                offset,
            ));
        }
    }

    /// Checks if the attribute is valid and returns an error if not.
    pub fn is_valid(&self) -> Result<()> {
        let component_size = self.data_type.size() * self.num_components;
        if component_size > self.stride {
            return Err(Error::DrawCall(format!(
                "Stride must be at least {}, but is {}",
                component_size, self.stride
            )));
        }

        Ok(())
    }
}

/// The per page indices
#[derive(Clone, Debug)]
pub struct IndexData {
    /// The byte offset for the index data
    pub offset: usize,
    /// The number of indices
    pub num: usize,
    /// The data type of the indices
    pub datatype: DataType,
}

/// A single draw call
pub struct DrawCall {
    vertex_array: GLuint,
}

impl Default for DrawCall {
    fn default() -> Self {
        Self::new()
    }
}

/// Block of attribute data
pub struct AttributeBlock<'a> {
    /// the vertex data BLOB
    pub vertex_data: &'a GPUBuffer,
    /// the attributes of the attributes block
    pub attributes: Vec<Attribute>,
}

impl<'a> AttributeBlock<'a> {
    /// Checks if the attribute block is valid.
    pub fn is_valid(&self) -> Result<()> {
        for attrib in self.attributes.iter() {
            attrib.is_valid()?;
        }

        Ok(())
    }
}

impl DrawCall {
    pub fn new() -> DrawCall {
        DrawCall { vertex_array: 0 }
    }

    /// Sets the data of the draw call
    ///
    ///* `blocks` - The vertex attribute blocks
    pub fn set_data(&mut self, blocks: &[AttributeBlock]) {
        // make sure old data has been released
        self.release();

        // DEBUG: check if the attribute data is valid
        debug_assert!(
            blocks.iter().all(|block| block.is_valid().is_ok()),
            "Not all attributes are valid"
        );

        // generate and bind new vertex arrays buffer
        gl_call!(gl::GenVertexArrays(1, &mut self.vertex_array));
        gl_call!(gl::BindVertexArray(self.vertex_array));

        let mut idx = 0u32;
        for block in blocks.iter() {
            let vertex_data = block.vertex_data;
            let attributes = &block.attributes;

            // set vertex data buffer
            vertex_data.bind();

            // set vertex attributes
            for attribute in attributes.iter() {
                gl_call!(gl::EnableVertexAttribArray(idx as GLuint));

                attribute.bind(idx);

                idx += 1;
            }

            vertex_data.unbind();
        }

        gl_call!(gl::BindVertexArray(0));
    }

    /// Executes the draw call without using indices.
    ///
    ///* `primitive_type` - The primitives to be rendered
    ///* `start` - The index of the first primitive to be rendered
    ///* `num` - The number of indices to be rendered
    pub fn draw_no_indices(&self, primitive_type: PrimitiveType, start: u32, num: u32) {
        // make sure the draw call is valid/initialized
        if !self.is_valid() {
            return;
        }

        // determine the primitive type
        let gl_type = primitive_type as GLenum;

        // bind vertex data
        self.bind();

        gl_call!(gl::DrawArrays(
            gl_type,      // mode
            start as i32, // starting index in the enabled arrays
            num as i32,   // number of indices to be rendered
        ));

        DrawCall::unbind();
    }

    /// Executes the draw call
    ///
    ///* `primitive_type` - The primitives to be rendered
    ///* `indices` - Reference onto the GPU index data used for the rendering
    ///* `index_data` - The index data
    pub fn draw_with_indices(
        &self,
        primitive_type: PrimitiveType,
        indices: &GPUBuffer,
        index_data: &IndexData,
    ) {
        // make sure the draw call is valid/initialized
        if !self.is_valid() {
            return;
        }

        // determine the primitive type
        let gl_type = primitive_type as GLenum;

        // bind vertex data
        self.bind();

        indices.bind();

        let offset_bytes = index_data.offset as isize;
        let offset = unsafe { std::ptr::null::<std::ffi::c_void>().offset(offset_bytes) };

        gl_call!(gl::DrawElements(
            gl_type,                          // mode
            index_data.num as i32,            // number of indices to be rendered
            index_data.datatype.to_gl_type(), // the type of indices
            offset,                           // the offset
        ));

        DrawCall::unbind();
    }

    pub fn bind(&self) {
        gl_call!(gl::BindVertexArray(self.vertex_array));
    }

    pub fn unbind() {
        gl_call!(gl::BindVertexArray(0));
    }
}

impl Resource for DrawCall {
    #[inline]
    fn is_valid(&self) -> bool {
        self.vertex_array > 0
    }

    fn release(&mut self) {
        if self.is_valid() {
            // delete vertex array
            gl_call!(gl::DeleteVertexArrays(1, &self.vertex_array));

            self.vertex_array = 0;
        }
    }
}

impl Drop for DrawCall {
    fn drop(&mut self) {
        if self.is_valid() {
            gl_call!(gl::DeleteVertexArrays(1, &self.vertex_array));
        }
    }
}
