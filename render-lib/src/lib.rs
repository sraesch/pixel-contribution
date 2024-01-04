#[macro_use]
pub mod gl_call;
pub mod camera;

mod datatype;
mod draw;
mod error;
mod framebuffer;
mod gpu_buffer;
mod gpu_mesh;
mod renderbuffer;
mod resource;
mod shader;
mod texture;
mod viewer;

pub use datatype::*;
pub use draw::*;
pub use error::*;
pub use framebuffer::*;
pub use gpu_buffer::*;
pub use gpu_mesh::*;
pub use renderbuffer::*;
pub use resource::*;
pub use shader::*;
pub use texture::*;
pub use viewer::*;

/// Enum for the different face culling modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceCulling {
    None,
    Front,
    Back,
    FrontAndBack,
}

/// Configures the face culling.
///
/// # Arguments
/// * `culling` - The face culling to use.
pub fn configure_culling(culling: FaceCulling) {
    match culling {
        FaceCulling::None => {
            gl_call!(gl::Disable(gl::CULL_FACE));
        }
        FaceCulling::Front => {
            gl_call!(gl::Enable(gl::CULL_FACE));
            gl_call!(gl::CullFace(gl::FRONT));
        }
        FaceCulling::Back => {
            gl_call!(gl::Enable(gl::CULL_FACE));
            gl_call!(gl::CullFace(gl::BACK));
        }
        FaceCulling::FrontAndBack => {
            gl_call!(gl::Enable(gl::CULL_FACE));
            gl_call!(gl::CullFace(gl::FRONT_AND_BACK));
        }
    }
}
