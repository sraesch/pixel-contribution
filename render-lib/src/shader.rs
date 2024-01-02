use gl::types::*;
use nalgebra_glm::{Mat3, Mat4, Vec3, Vec4};
use static_assertions::assert_eq_size;

use std::ffi::CString;

use crate::{Error, Result};

/// Uniform shader variable
pub struct Uniform {
    location: GLint,
}

impl Default for Uniform {
    fn default() -> Self {
        Self::new()
    }
}

impl Uniform {
    pub fn new() -> Uniform {
        Uniform { location: -1 }
    }

    #[inline]
    pub fn set_vector3(&self, vec: &Vec3) {
        assert_eq_size!(Vec3, [f32; 3]);

        let vec_ptr = vec as *const _;
        let vec_raw_ptr = vec_ptr as *const f32;

        gl_call!(gl::Uniform3fv(self.location, 1, vec_raw_ptr));
    }

    #[inline]
    pub fn set_vector4(&self, vec: &Vec4) {
        assert_eq_size!(Vec4, [f32; 4]);

        let vec_ptr = vec as *const _;
        let vec_raw_ptr = vec_ptr as *const f32;

        gl_call!(gl::Uniform4fv(self.location, 1, vec_raw_ptr));
    }

    #[inline]
    pub fn set_matrix3(&self, mat: &Mat3) {
        assert_eq_size!(Mat3, [f32; 9]);

        let mat_ptr = mat as *const _;
        let mat_raw_ptr = mat_ptr as *const f32;

        gl_call!(gl::UniformMatrix3fv(self.location, 1, 0, mat_raw_ptr));
    }

    #[inline]
    pub fn set_matrix4(&self, mat: &Mat4) {
        assert_eq_size!(Mat4, [f32; 16]);

        let mat_ptr = mat as *const _;
        let mat_raw_ptr = mat_ptr as *const f32;

        gl_call!(gl::UniformMatrix4fv(self.location, 1, 0, mat_raw_ptr));
    }

    #[inline]
    pub fn set_int(&self, x: i32) {
        gl_call!(gl::Uniform1i(self.location, x));
    }

    #[inline]
    pub fn set_float(&self, x: f32) {
        gl_call!(gl::Uniform1f(self.location, x));
    }

    /// Returns true if the uniform variable is valid and false otherwise
    #[inline]
    pub fn valid(&self) -> bool {
        self.location >= 0
    }
}

/// GPU shader program
pub struct Shader {
    program: GLuint,
    vertex_shader: GLuint,
    fragment_shader: GLuint,
}

impl Default for Shader {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader {
    /// Returns a new shader object
    pub fn new() -> Shader {
        Self {
            program: 0,
            vertex_shader: 0,
            fragment_shader: 0,
        }
    }

    /// Releases the internal OpenGL shader program object data.
    pub fn release(&mut self) {
        // first delete program
        if self.program > 0 {
            gl_call!(gl::DeleteProgram(self.program));
            self.program = 0;
        }

        // delete vertex shader
        if self.vertex_shader > 0 {
            gl_call!(gl::DeleteShader(self.vertex_shader));
            self.vertex_shader = 0;
        }

        // delete vertex shader
        if self.fragment_shader > 0 {
            gl_call!(gl::DeleteShader(self.fragment_shader));
            self.fragment_shader = 0;
        }
    }

    /// Loads, compiles and links the provided shader source codes.
    ///
    ///* `vertex_code` - The vertex shader source code
    ///* `frag_code` - The fragment shader source code
    pub fn load(&mut self, vertex_code: &str, frag_code: &str) -> Result<()> {
        // release old shader data
        self.release();

        // create shader objects
        self.vertex_shader = Shader::load_shader(vertex_code, gl::VERTEX_SHADER)?;
        self.fragment_shader = Shader::load_shader(frag_code, gl::FRAGMENT_SHADER)?;

        // create shader program object
        self.program = gl_call!(gl::CreateProgram());

        // attach and link shaders
        gl_call!(gl::AttachShader(self.program, self.vertex_shader));
        gl_call!(gl::AttachShader(self.program, self.fragment_shader));
        gl_call!(gl::LinkProgram(self.program));

        // check link status of the program
        self.is_valid()
    }

    /// Binds the shader
    pub fn bind(&self) {
        gl_call!(gl::UseProgram(self.program));
    }

    /// Unbinds the currently resource.rs shader
    pub fn unbind() {
        gl_call!(gl::UseProgram(0));
    }

    /// Returns the uniform variable corresponding to the given name.
    /// If the uniform variable could not be found, the result is invalid
    ///
    ///* `name` - The name of the uniform variable
    pub fn get_uniform(&self, name: &str) -> Result<Uniform> {
        let cstr_name = CString::new(name).unwrap();

        let location = gl_call!(gl::GetUniformLocation(self.program, cstr_name.as_ptr()));
        if location < 0 {
            return Err(Error::Shader(format!(
                "Could not find shader variable '{}'",
                name
            )));
        }

        Ok(Uniform { location })
    }

    /// Creates the shader object for the given source and kind.
    ///
    ///* `source` - The source of the shader
    ///* `kind` - The OpenGL constant for the shader type, e.g. vertex, fragment, etc.
    fn load_shader(source: &str, kind: GLenum) -> Result<GLuint> {
        let cstr_source = CString::new(source).unwrap();

        let id = gl_call!(gl::CreateShader(kind));

        gl_call!(gl::ShaderSource(
            id,
            1,
            &cstr_source.as_ptr(),
            std::ptr::null()
        ));
        gl_call!(gl::CompileShader(id));

        let mut success: GLint = 1;

        gl_call!(gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success));

        if success == 0 {
            let mut len: GLint = 0;
            gl_call!(gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len));

            let error = Shader::create_whitespace_cstring_with_len(len as usize);

            gl_call!(gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut GLchar
            ));

            return Err(Error::Shader(error.to_string_lossy().into_owned()));
        }

        Ok(id)
    }

    /// Checks if the shader program is valid
    pub fn is_valid(&self) -> Result<()> {
        let mut success: GLint = 1;
        gl_call!(gl::GetProgramiv(
            self.program,
            gl::LINK_STATUS,
            &mut success
        ));

        if success == 0 {
            // determine length of the error message
            let mut len: GLint = 0;
            gl_call!(gl::GetProgramiv(
                self.program,
                gl::INFO_LOG_LENGTH,
                &mut len
            ));

            // allocate space for the error message
            let error = Shader::create_whitespace_cstring_with_len(len as usize);

            // get error message
            gl_call!(gl::GetProgramInfoLog(
                self.program,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut GLchar,
            ));

            return Err(Error::Shader(error.to_string_lossy().into_owned()));
        }

        Ok(())
    }

    fn create_whitespace_cstring_with_len(len: usize) -> CString {
        // allocate buffer of correct size
        let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
        // fill it with len spaces
        buffer.extend([b' '].iter().cycle().take(len));
        // convert buffer to CString
        unsafe { CString::from_vec_unchecked(buffer) }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.release();
    }
}
