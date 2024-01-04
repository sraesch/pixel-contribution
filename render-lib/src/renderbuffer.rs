use gl::types::{GLenum, GLsizei, GLuint};

use crate::{Bind, Resource};

use log::trace;

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub enum RenderBufferType {
    Depth = 0x81A6,
}

pub struct RenderBuffer {
    renderbuffer_id: GLuint,
    buffer_type: RenderBufferType,
    buffer_size: (u32, u32),
}

impl RenderBuffer {
    pub fn new(buffer_type: RenderBufferType) -> Self {
        Self {
            renderbuffer_id: 0,
            buffer_type,
            buffer_size: (0, 0),
        }
    }

    /// Updates the internal size of the buffer. Function has no effect if the buffer is allocated
    /// with the same size as given.
    ///
    /// # Arguments
    /// * `buffer_size` - The new size of the buffer
    pub fn resize(&mut self, buffer_size: (u32, u32)) {
        if self.buffer_size != buffer_size {
            self.generate_id();
            self.bind();

            gl_call!(gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                self.buffer_type as GLenum,
                buffer_size.0 as GLsizei,
                buffer_size.1 as GLsizei,
            ));

            self.buffer_size = buffer_size;
        }
    }

    /// Generates an OpenGL render buffer id, if this did not already happen
    fn generate_id(&mut self) {
        if self.is_valid() {
            return;
        }

        // generate buffer id
        let mut renderbuffer_id: GLuint = 0;

        gl_call!(gl::GenRenderbuffers(1, &mut renderbuffer_id));

        self.renderbuffer_id = renderbuffer_id;
    }
}

impl Bind for RenderBuffer {
    #[inline]
    fn bind(&self) {
        gl_call!(gl::BindRenderbuffer(gl::RENDERBUFFER, self.renderbuffer_id));
    }

    #[inline]
    fn get_id(&self) -> u32 {
        self.renderbuffer_id
    }

    #[inline]
    fn unbind(&self) {
        gl_call!(gl::BindRenderbuffer(gl::RENDERBUFFER, 0));
    }
}

impl Resource for RenderBuffer {
    #[inline]
    fn is_valid(&self) -> bool {
        self.renderbuffer_id > 0
    }

    #[inline]
    fn release(&mut self) {
        if self.is_valid() {
            gl_call!(gl::DeleteRenderbuffers(1, &self.renderbuffer_id));

            trace!("Released Render buffer");

            self.renderbuffer_id = 0;
        }
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        self.release();
    }
}
