use std::mem::size_of;

use anyhow::Result;
use log::info;
use nalgebra_glm::Mat4;
use pixel_contrib::PixelContributionMap;
use render_lib::{
    Attribute, AttributeBlock, Bind, DataType, DrawCall, Filtering, GPUBuffer, GPUBufferType,
    IndexData, PrimitiveType, Shader, Texture, TextureData, TextureDescriptor, Uniform,
};

use crate::geometry::create_sphere;

pub struct Sphere {
    positions: GPUBuffer,
    indices: GPUBuffer,
    num_indices: usize,

    texture: Texture,

    shader: Shader,

    uniform_texture: Uniform,
    uniform_combined_mat: Uniform,
    uniform_transparency: Uniform,

    draw_call: DrawCall,
}

impl Default for Sphere {
    fn default() -> Self {
        Self::new()
    }
}

impl Sphere {
    /// Creates a new sphere
    pub fn new() -> Self {
        Self {
            positions: GPUBuffer::new(GPUBufferType::Vertices),
            indices: GPUBuffer::new(GPUBufferType::Indices),
            num_indices: 0,

            texture: Texture::default(),

            shader: Shader::default(),

            uniform_texture: Uniform::default(),
            uniform_combined_mat: Uniform::default(),
            uniform_transparency: Uniform::default(),

            draw_call: DrawCall::default(),
        }
    }

    /// Setups the sphere and loads the corresponding image file.
    ///
    /// # Arguments
    /// * `pixel_contribution` - The pixel contribution data.
    pub fn setup(&mut self, pixel_contribution: &PixelContributionMap) -> Result<()> {
        info!("Setup sphere...");

        let vert_shader = include_str!("../shader/sphere.vert");
        let frag_shader = include_str!("../shader/sphere.frag");

        info!("compile shader...");
        self.shader.load(vert_shader, frag_shader)?;
        self.uniform_texture = self.shader.get_uniform("uniform_texture").unwrap();
        self.uniform_combined_mat = self.shader.get_uniform("uniform_combined_mat").unwrap();
        self.uniform_transparency = self.shader.get_uniform("uniform_transparency").unwrap();
        info!("compile shader...DONE");

        // initialize texture
        let pixel_contrib_data: &[u8] = unsafe {
            std::slice::from_raw_parts(
                pixel_contribution.pixel_contrib.as_ptr() as *const u8,
                pixel_contribution.pixel_contrib.len() * size_of::<f32>(),
            )
        };
        self.texture.generate(&TextureData {
            descriptor: TextureDescriptor {
                width: pixel_contribution.descriptor.size() as u32,
                height: pixel_contribution.descriptor.size() as u32,
                format: render_lib::PixelFormat::Gray,
                filtering: Filtering::Linear,
                datatype: DataType::Float,
            },
            data: Some(pixel_contrib_data),
        });

        // initializes sphere geometry
        let sphere_geo = create_sphere(1.0, 100, 100);
        self.positions.set_data(&sphere_geo.0);
        self.indices.set_data(&sphere_geo.1);
        self.num_indices = sphere_geo.1.len();

        // create vertex array
        self.draw_call.set_data(&[AttributeBlock {
            vertex_data: &self.positions,
            attributes: vec![Attribute {
                offset: 0,
                stride: size_of::<f32>() * 3,
                num_components: 3,
                data_type: DataType::Float,
                is_integer: false,
                normalized: false,
            }],
        }]);

        info!("Setup sphere...DONE");
        Ok(())
    }

    /// Renders the sphere
    ///
    /// # Arguments
    /// * `combined_mat` - The combined matrix of the camera, i.e., the projection matrix
    ///                    multiplied by the view matrix.
    /// * `transparency` - The transparency of the sphere. The value must be in the range [0, 1].
    ///                    A value of 0 means that the sphere is completely transparent, while a
    ///                    value of 1 means that the sphere is completely opaque.
    pub fn render(&self, combined_mat: &Mat4, transparency: f32) {
        self.shader.bind();
        self.texture.bind();
        self.uniform_texture.set_int(0);
        self.uniform_combined_mat.set_matrix4(combined_mat);
        self.uniform_transparency.set_float(transparency);
        self.draw_call.draw_with_indices(
            PrimitiveType::Triangles,
            &self.indices,
            &IndexData {
                datatype: DataType::UnsignedInt,
                offset: 0,
                num: self.num_indices,
            },
        );
    }
}
