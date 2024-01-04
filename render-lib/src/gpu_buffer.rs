use crate::resource::*;

use gl::types::*;

use log::trace;
use std::ops::Drop;

/// The type of data inside a GPU buffer
#[derive(Debug, Clone, PartialEq)]
#[repr(u32)]
pub enum GPUBufferType {
    Vertices = gl::ARRAY_BUFFER,
    Indices = gl::ELEMENT_ARRAY_BUFFER,
}

impl GPUBufferType {
    /// Returns the corresponding OpenGL type
    #[inline]
    fn to_gl_type(&self) -> GLenum {
        self.clone() as GLenum
    }
}

/// A gpu buffer containing GPU memory data
pub struct GPUBuffer {
    buffer_id: GLuint,
    buffer_type: GPUBufferType,
}

impl GPUBuffer {
    /// Creates a new empty GPU buffer
    ///
    ///* `buffer_type` - The type of the GPU buffer
    #[inline]
    pub fn new(buffer_type: GPUBufferType) -> Self {
        GPUBuffer {
            buffer_id: 0,
            buffer_type,
        }
    }

    /// Creates a new GPU buffer with the given data
    ///
    ///* `buffer_type` - The type of the GPU buffer
    ///* `data` - The raw data of the GPU buffer
    pub fn new_with_data<T>(buffer_type: GPUBufferType, data: &[T]) -> Self {
        let mut result = Self::new(buffer_type);

        result.set_data(data);

        result
    }

    /// Creates an empty new GPU buffer with the given size.
    ///
    ///* `buffer_type` - The type of the GPU buffer
    ///* `num_bytes` - The size in bytes of the GPU buffer.
    pub fn new_with_size(buffer_type: GPUBufferType, num_bytes: usize) -> Self {
        let mut result = Self::new(buffer_type);

        result.resize(num_bytes);

        result
    }

    /// Sets the data of the GPU buffer
    ///
    ///* `data` - The CPU data to be copied into the GPU buffer
    #[inline]
    pub fn set_data<T>(&mut self, data: &[T]) {
        let num_bytes = std::mem::size_of_val(data) as gl::types::GLsizeiptr;
        let buffer_type = self.buffer_type.to_gl_type();

        self.generate_id();
        self.bind();

        gl_call!(gl::BufferData(
            buffer_type,
            num_bytes,
            data.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        ));
    }

    /// Sets a part of the data of the GPU buffer
    ///
    /// # Arguments
    /// * `offset` - The position where to copy the data
    /// * `data` - The CPU data to be copied into the GPU buffer
    #[inline]
    pub fn set_sub_data<T>(&mut self, offset: usize, data: &[T]) {
        let num_bytes = std::mem::size_of_val(data) as gl::types::GLsizeiptr;
        let buffer_type = self.buffer_type.to_gl_type();

        self.generate_id();
        self.bind();

        gl_call!(gl::BufferSubData(
            buffer_type,
            offset as GLintptr,
            num_bytes,
            data.as_ptr() as *const gl::types::GLvoid
        ));
    }

    /// Resizes the buffer to the given size
    ///
    ///* `num_bytes` - The new size in bytes.
    #[inline]
    pub fn resize(&mut self, num_bytes: usize) {
        let buffer_type = self.buffer_type.to_gl_type();

        self.generate_id();
        self.bind();

        gl_call!(gl::BufferData(
            buffer_type,
            num_bytes as GLsizeiptr,
            std::ptr::null(),
            gl::STATIC_DRAW,
        ));
    }

    /// Returns the type of the buffer
    #[inline]
    pub fn get_type(&self) -> GPUBufferType {
        self.buffer_type.clone()
    }

    /// Generates an OpenGL buffer id, if this did not already happen
    fn generate_id(&mut self) {
        if self.buffer_id > 0 {
            return;
        }

        // generate buffer id
        let mut buffer_id: GLuint = 0;

        gl_call!(gl::GenBuffers(1, &mut buffer_id));

        self.buffer_id = buffer_id;
    }
}

impl Drop for GPUBuffer {
    fn drop(&mut self) {
        self.release();
    }
}

impl Bind for GPUBuffer {
    fn get_id(&self) -> u32 {
        self.buffer_id
    }

    /// Binds the vertex buffer object
    #[inline]
    fn bind(&self) {
        let gl_type = self.buffer_type.to_gl_type();

        gl_call!(gl::BindBuffer(gl_type, self.buffer_id));
    }

    /// Unbinds the vertex buffer
    #[inline]
    fn unbind(&self) {
        let gl_type = self.buffer_type.to_gl_type();

        gl_call!(gl::BindBuffer(gl_type, 0));
    }
}

impl Resource for GPUBuffer {
    #[inline]
    fn is_valid(&self) -> bool {
        self.buffer_id > 0
    }

    #[inline]
    fn release(&mut self) {
        // release vertex buffer object
        if self.is_valid() {
            gl_call!(gl::DeleteBuffers(1, &self.buffer_id));

            trace!("Released GPU buffer");

            self.buffer_id = 0;
        }
    }
}
