use gl::{
    self,
    types::{GLenum, GLsizei, GLuint},
};

use log::trace;
use nalgebra_glm::Vec4;

use crate::{
    error::Error,
    renderbuffer::{RenderBuffer, RenderBufferType},
    resource::{Bind, Resource},
    texture::{Texture, TextureData, TextureDescriptor},
    Result,
};

pub struct FrameBuffer {
    framebuffer_id: GLuint,
    depthbuffer: RenderBuffer,

    size: (u32, u32),

    texture_descriptors: Vec<TextureDescriptor>,
    render_textures: Vec<Texture>,
}

static mut CURRENT_FRAME_BUFFER: GLuint = 0;

impl Default for FrameBuffer {
    fn default() -> Self {
        Self {
            framebuffer_id: 0,
            depthbuffer: RenderBuffer::new(RenderBufferType::Depth),
            size: (0, 0),
            texture_descriptors: Vec::new(),
            render_textures: Vec::new(),
        }
    }
}

impl FrameBuffer {
    /// Clears the internal buffer
    pub fn clear(&mut self) {
        // unbind framebuffer
        if unsafe { CURRENT_FRAME_BUFFER } == self.framebuffer_id {
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
            unsafe { CURRENT_FRAME_BUFFER = 0 };
        }

        self.depthbuffer.release();
        self.render_textures.clear();
        self.release();
    }

    /// Generates a new framebuffer with texture attachments as specified by the given texture
    /// descriptors
    ///
    /// # Arguments
    /// * `descriptors` - The per texture descriptors.
    pub fn generate(&mut self, descriptors: Vec<TextureDescriptor>) -> Result<()> {
        // check if we have texture descriptors and get the framebuffer size
        match descriptors.first() {
            Some(d) => {
                self.size = (d.width, d.height);
            }
            None => {
                return Err(Error::FrameBuffer(
                    "Missing texture descriptors".to_string(),
                ));
            }
        }

        self.texture_descriptors = descriptors;

        // clear and recreate frame buffer
        self.clear();
        gl_call!(gl::GenFramebuffers(1, &mut self.framebuffer_id));
        self.bind();

        // reserve space for the render textures
        self.render_textures.reserve(self.texture_descriptors.len());

        // create render textures
        let mut attachments: Vec<GLenum> = Vec::with_capacity(self.texture_descriptors.len());
        for (index, descriptor) in self.texture_descriptors.iter().enumerate() {
            let texture_data = TextureData {
                data: None,
                descriptor: *descriptor,
            };

            let mut render_texture = Texture::default();
            render_texture.generate(&texture_data);

            let attachment: GLenum = gl::COLOR_ATTACHMENT0 + index as GLenum;
            attachments.push(attachment);
            gl_call!(gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                attachment,
                gl::TEXTURE_2D,
                render_texture.get_id(),
                0
            ));

            self.render_textures.push(render_texture);
        }

        // register render buffer's
        gl_call!(gl::DrawBuffers(
            attachments.len() as GLsizei,
            attachments.as_ptr()
        ));

        // Create depth render buffer
        self.depthbuffer.resize(self.size);
        self.depthbuffer.bind();

        gl_call!(gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT24,
            self.size.0 as GLsizei,
            self.size.1 as GLsizei
        ));

        gl_call!(gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            self.depthbuffer.get_id()
        ));

        match gl_call!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER)) {
            gl::FRAMEBUFFER_COMPLETE => Ok(()),
            gl::FRAMEBUFFER_UNDEFINED => {
                Err(Error::FrameBuffer("framebuffer is the default read or draw framebuffer, but the default framebuffer does not exist".to_string()))
            },
            gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => {
                Err(Error::FrameBuffer("Some of the framebuffer attachment points are framebuffer incomplete".to_string()))
            }
            gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
                Err(Error::FrameBuffer("Framebuffer does not have at least one image attached to it".to_string()))
            }
            gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => {
                Err(Error::FrameBuffer("Incomplete draw buffer".to_string()))
            }
            gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => {
                Err(Error::FrameBuffer("Incomplete read buffer".to_string()))
            }
            gl::FRAMEBUFFER_UNSUPPORTED => {
                Err(Error::FrameBuffer("The combination of internal formats of the attached images violates an implementation-dependent set of restrictions".to_string()))
            }
            gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => {
                Err(Error::FrameBuffer("Incomplete multisample".to_string()))
            }
            gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => {
                Err(Error::FrameBuffer("Some framebuffer attachment is layered, and any populated attachment is not layered, or if all populated color attachments are not from textures of the same target".to_string()))
            }
            _ => Err(Error::FrameBuffer("Unknown framebuffer issue".to_string())),
        }
    }

    /// Updates the size of the buffer. The function has no effect if the buffer is already created
    /// and has the same size as given.
    ///
    /// # Arguments
    /// * `size` - The new size of buffer
    pub fn resize(&mut self, size: (u32, u32)) -> Result<()> {
        // check if the size has changed
        if self.size == size {
            return Ok(());
        }

        // generate new texture descriptors
        let mut texture_descriptors = self.texture_descriptors.clone();
        for descriptor in texture_descriptors.iter_mut() {
            descriptor.width = size.0;
            descriptor.height = size.1;
        }

        // regenerate framebuffer
        self.generate(texture_descriptors)
    }

    /// Begin to draw into the framebuffer.
    ///
    /// # Arguments
    /// * `width`- The width of the framebuffer.
    /// * `height` - The height of the framebuffer.
    pub fn begin(&self, width: u32, height: u32) {
        self.bind();
        FrameBuffer::viewport(0, 0, width, height);
    }

    /// Returns a reference onto the render textures
    #[inline]
    pub fn get_textures(&self) -> &[Texture] {
        self.render_textures.as_ref()
    }

    /// Returns the internally stored depth buffer values
    ///
    /// # Returns
    /// * `width` - The width of the depth buffer.
    /// * `height` - The height of the depth buffer.
    /// * `values` - The depth buffer values.
    pub fn get_depth_buffer_values() -> (usize, usize, Vec<f32>) {
        // get the size of the viewport
        let mut view_port: [i32; 4] = [0; 4];
        gl_call!(gl::GetIntegerv(
            gl::VIEWPORT,
            view_port.as_mut_ptr() as *mut _
        ));

        let width = view_port[2] as usize;
        let height = view_port[3] as usize;

        let mut values: Vec<f32> = Vec::new();
        values.resize(width * height, 0.0);

        gl_call!(gl::ReadPixels(
            0,
            0,
            width as GLsizei,
            height as GLsizei,
            gl::DEPTH_COMPONENT,
            gl::FLOAT,
            values.as_mut_ptr() as *mut _
        ));

        (width, height, values)
    }
}

impl Bind for FrameBuffer {
    #[inline]
    fn bind(&self) {
        if unsafe { CURRENT_FRAME_BUFFER } != self.framebuffer_id {
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_id));
            unsafe {
                CURRENT_FRAME_BUFFER = self.framebuffer_id;
            }
        }
    }

    #[inline]
    fn unbind(&self) {
        gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
    }

    fn get_id(&self) -> u32 {
        self.framebuffer_id
    }
}

impl Resource for FrameBuffer {
    #[inline]
    fn is_valid(&self) -> bool {
        self.framebuffer_id > 0
    }

    #[inline]
    fn release(&mut self) {
        if self.is_valid() {
            gl_call!(gl::DeleteFramebuffers(1, &self.framebuffer_id));

            trace!("Released Frame buffer");

            self.framebuffer_id = 0;
        }
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        self.release();
    }
}

/// Frame buffer for rendering into it
impl FrameBuffer {
    /// Clears the current frame buffer with the specified color
    ///
    ///* `color` - The rgba color used for clearing the frame buffer
    pub fn clear_buffers(color: &Vec4) {
        gl_call!(gl::ClearColor(color[0], color[1], color[2], color[3]));
        gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
    }

    /// Enables and sets the blending function
    ///
    /// # Arguments
    /// * `src` - The source blending factor
    /// * `dst` - The destination blending factor
    pub fn set_blending(src: BlendFactor, dst: BlendFactor) {
        gl_call!(gl::Enable(gl::BLEND));
        gl_call!(gl::BlendFunc(src as GLenum, dst as GLenum));
    }

    /// Disables the blending.
    pub fn disable_blend() {
        gl_call!(gl::Disable(gl::BLEND));
    }

    /// Function for updating the viewport with the given rectangle
    ///
    ///* `x` - The x coordinate of the rectangle origin
    ///* `y` - The y coordinate of the rectangle origin
    ///* `width` - The width of the rectangle.
    ///* `height` - The height of the rectangle.
    pub fn viewport(x: u32, y: u32, width: u32, height: u32) {
        gl_call!(gl::Viewport(
            x as i32,
            y as i32,
            width as i32,
            height as i32
        ));
    }

    /// Enables/Disables the OpenGL depth test.
    ///
    ///* `enable` - The flag which determines if the depth testing is enabled or disabled
    #[inline]
    pub fn depthtest(enable: bool) {
        if enable {
            gl_call!(gl::Enable(gl::DEPTH_TEST));
        } else {
            gl_call!(gl::Disable(gl::DEPTH_TEST));
        }
    }

    /// Returns the main frame buffer.
    #[inline]
    pub fn main_frame_buffer() -> Self {
        Self::default()
    }
}

/// Blending factors
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendFactor {
    Zero = gl::ZERO,
    One = gl::ONE,
    SrcColor = gl::SRC_COLOR,
    OneMinusSrcColor = gl::ONE_MINUS_SRC_COLOR,
    DstColor = gl::DST_COLOR,
    OneMinusDstColor = gl::ONE_MINUS_DST_COLOR,
    SrcAlpha = gl::SRC_ALPHA,
    OneMinusSrcAlpha = gl::ONE_MINUS_SRC_ALPHA,
    DstAlpha = gl::DST_ALPHA,
    OneMinusDstAlpha = gl::ONE_MINUS_DST_ALPHA,
    ConstantColor = gl::CONSTANT_COLOR,
    OneMinusConstantColor = gl::ONE_MINUS_CONSTANT_COLOR,
    ConstantAlpha = gl::CONSTANT_ALPHA,
    OneMinusConstantAlpha = gl::ONE_MINUS_CONSTANT_ALPHA,
    SrcAlphaSaturate = gl::SRC_ALPHA_SATURATE,
}
