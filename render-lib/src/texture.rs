use gl::types::*;
use std::io::{BufWriter, Write};

use crate::{Bind, DataType, Error, Resource, Result};

/// The pixel format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PixelFormat {
    Rgb = gl::RGB,
    Rgba = gl::RGBA,
}

/// The texture filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Filtering {
    Linear = gl::LINEAR,
    Nearest = gl::NEAREST,
}

/// The descriptor for a texture
#[derive(Clone, Copy)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,

    pub format: PixelFormat,
    pub filtering: Filtering,

    pub datatype: DataType,
}

/// The input texture data
pub struct TextureData<'a> {
    pub descriptor: TextureDescriptor,
    pub data: Option<&'a [u8]>,
}

impl<'a> TextureData<'a> {
    /// Writes the texture data as ppm
    ///
    ///* `dst` - The writer which consumes the written image data
    pub fn debug_write_ppm<W>(&self, dst: W) -> Result<()>
    where
        W: Write,
    {
        // check if texture data is supported
        if self.descriptor.datatype != DataType::UnsignedByte
            || self.descriptor.format != PixelFormat::Rgb
        {
            return Err(Error::IO("Unsupported texture data format".to_string()));
        }

        // create writer
        let mut writer = BufWriter::new(dst);

        // write header
        writeln!(writer, "P3")?;
        write!(
            writer,
            "{} {}\n255",
            self.descriptor.width, self.descriptor.height
        )?;

        // write actual image data
        for (i, c) in self.data.unwrap().iter().enumerate() {
            if i % (3 * self.descriptor.width as usize) == 0 {
                writeln!(writer)?;
            }

            write!(writer, "{} ", c)?;
        }

        writeln!(writer)?;

        Ok(())
    }
}

/// Texture object on the GPU
#[derive(Debug, Default)]
pub struct Texture {
    id: GLuint,
}

impl Texture {
    /// Generates the texture data
    ///
    ///* `data` - The new texture data
    pub fn generate(&mut self, data: &TextureData) {
        self.release();

        gl_call!(gl::GenTextures(1, &mut self.id));

        let f = data.descriptor.filtering as GLint;
        let datatype = data.descriptor.datatype as GLenum;
        let texfmt = data.descriptor.format as GLenum;
        let width = data.descriptor.width as GLint;
        let height = data.descriptor.height as GLint;
        let ptr = match data.data {
            Some(data) => data.as_ptr() as *const gl::types::GLvoid,
            None => std::ptr::null(),
        };

        self.bind();
        gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, f));
        gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, f));
        gl_call!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as GLint
        ));
        gl_call!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as GLint
        ));

        gl_call!(gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            texfmt as i32,
            width,
            height,
            0,
            texfmt,
            datatype,
            ptr
        ));

        self.unbind();
    }

    /// Updates the texture data on the GPU
    ///
    ///* `data` - The new texture data
    pub fn update(&mut self, data: &TextureData) -> Result<()> {
        if !self.is_valid() {
            return Err(Error::Texture(
                "Texture is invalid -> Updating failed".to_string(),
            ));
        }

        let datatype = data.descriptor.datatype as GLenum;
        let texfmt = data.descriptor.format as GLenum;
        let width = data.descriptor.width as GLint;
        let height = data.descriptor.height as GLint;
        let ptr = match data.data {
            Some(data) => data.as_ptr() as *const gl::types::GLvoid,
            None => std::ptr::null(),
        };

        self.bind();

        gl_call!(gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            width,
            height,
            texfmt,
            datatype,
            ptr
        ));

        self.unbind();

        Ok(())
    }

    pub fn bind_with_unit(&self, texture_unit: u32) {
        gl_call!(gl::ActiveTexture(gl::TEXTURE0 + texture_unit as GLenum));
        gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.id));
    }
}

impl Bind for Texture {
    /// Binds the current texture
    #[inline]
    fn bind(&self) {
        self.bind_with_unit(0);
    }

    /// Unbinds the currently bound texture
    #[inline]
    fn unbind(&self) {
        gl_call!(gl::BindTexture(gl::TEXTURE_2D, 0));
    }

    #[inline]
    fn get_id(&self) -> u32 {
        self.id
    }
}

impl Resource for Texture {
    /// Releases the texture resource
    #[inline]
    fn release(&mut self) {
        if self.is_valid() {
            gl_call!(gl::DeleteTextures(1, &self.id));
            self.id = 0;
        }
    }

    /// Returns true if the texture object is valid and false otherwise
    #[inline]
    fn is_valid(&self) -> bool {
        self.id > 0
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.release();
    }
}
