extern crate cfg_if;
extern crate wasm_bindgen;

use std::{cell::RefCell, rc::Rc};

pub use equator_graph::*;
pub use angle_interpolate::*;
pub use math;
use nalgebra_glm::Vec3;
pub use utils::*;

mod equator_graph;
mod angle_interpolate;
mod utils;

use math::clamp;
use wasm_bindgen::{prelude::*, Clamped};
use wasm_bindgen_futures::{
    js_sys::{ArrayBuffer, Float32Array, Uint8Array},
    JsFuture,
};
use web_sys::{ImageData, Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
#[derive(Clone)]
pub struct PixelContributionMap {
    inner: Rc<pixel_contrib_types::PixelContributionMap>,
}

#[wasm_bindgen]
impl PixelContributionMap {
    /// Creates a new PixelContributionMap object.
    ///
    /// # Arguments
    /// * `descriptor` - The descriptor for the pixel contribution map.
    /// * `values` - The pixel contribution values.
    #[wasm_bindgen(constructor)]
    pub fn new(descriptor: PixelContribColorMapDescriptor, values: Float32Array) -> Self {
        let descriptor = pixel_contrib_types::PixelContribColorMapDescriptor::new(
            descriptor.map_size,
            descriptor.camera_angle,
        );

        let values = values.to_vec();

        Self {
            inner: Rc::new(pixel_contrib_types::PixelContributionMap {
                descriptor,
                pixel_contrib: values,
            }),
        }
    }

    /// Returns the descriptor for the pixel contribution map.
    pub fn get_description(&self) -> PixelContribColorMapDescriptor {
        self.inner.descriptor.into()
    }

    /// Returns the pixel contribution value at the given index.
    ///
    /// # Arguments
    /// * `index` - The index of the pixel contribution value.
    pub fn get_value_at_index(&self, index: usize) -> f32 {
        self.inner.pixel_contrib[index]
    }

    /// Returns the pixel contribution values as image data with the given scale.
    ///
    /// # Arguments
    /// * `scale` - The scale to apply to the pixel contribution values.
    pub fn draw_image(&self, scale: f32) -> ImageData {
        let values = self.inner.pixel_contrib.as_slice();

        let g = colorgrad::turbo();
        let (min_val, max_val) = g.domain();

        let size = self.inner.descriptor.size() as u32;

        // ImageData::new_with_u8_clamped_array_and_sh(data, size, size);
        let mut data = vec![0u8; 4 * size as usize * size as usize];

        data.chunks_exact_mut(4)
            .zip(values.iter())
            .for_each(|(out, value)| {
                let value = clamp(value * scale, 0f32, 1f32) as f64;
                let value: f64 = (1f64 - value) * min_val + value * max_val;
                let color = g.at(value).to_rgba8();

                out[0] = color[0];
                out[1] = color[1];
                out[2] = color[2];
                out[3] = 255;
            });

        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&data), size, size).unwrap()
    }
}

#[wasm_bindgen]
#[derive(Default, Clone)]
pub struct PixelContributionMaps {
    inner: RefCell<Vec<PixelContributionMap>>,
}

#[wasm_bindgen]
impl PixelContributionMaps {
    /// Returns an empty PixelContributionMaps object.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(Vec::new()),
        }
    }

    /// Creates a shallow copy of this PixelContributionMaps object.
    pub fn create_shallow_copy(&self) -> PixelContributionMaps {
        let inner = self.inner.borrow().clone();
        Self {
            inner: RefCell::new(inner),
        }
    }

    /// Creates a new PixelContributionMaps object by loading the data from the given URL.
    ///
    /// # Arguments
    /// * `url` - The URL to load the data from.
    pub async fn from_reader(url: &str) -> Result<PixelContributionMaps, JsValue> {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(url, &opts)?;

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        if !resp_value.is_instance_of::<Response>() {
            return Err(JsValue::from_str("fetch did not return a Response"));
        }

        let resp: Response = resp_value.dyn_into().unwrap();

        let blob = JsFuture::from(resp.array_buffer()?).await?;
        if !blob.is_instance_of::<ArrayBuffer>() {
            return Err(JsValue::from_str(
                "Response.arrayBuffer did not return an ArrayBuffer",
            ));
        }
        let array = blob.dyn_into::<ArrayBuffer>().unwrap();
        let array = &Uint8Array::new(&array).to_vec();

        let mut reader = std::io::Cursor::new(array);
        let maps = match pixel_contrib_types::PixelContributionMaps::from_reader(&mut reader) {
            Ok(maps) => maps,
            Err(e) => return Err(JsValue::from_str(&format!("{}", e))),
        };

        let inner: Vec<PixelContributionMap> = maps
            .maps
            .into_iter()
            .map(|map| PixelContributionMap {
                inner: Rc::new(map),
            })
            .collect();

        Ok(PixelContributionMaps {
            inner: RefCell::new(inner),
        })
    }

    /// Returns the number of PixelContributionMap objects in this PixelContributionMaps object.
    pub fn size(&self) -> usize {
        self.inner.borrow().len()
    }

    /// Returns the PixelContributionMap object at the given index.
    pub fn get_map(&self, index: usize) -> PixelContributionMap {
        self.inner.borrow()[index].clone()
    }

    /// Returns the descriptor for the pixel contribution map at the given index.
    pub fn get_map_descriptor(&self, index: usize) -> PixelContribColorMapDescriptor {
        self.inner.borrow()[index].inner.descriptor.into()
    }

    /// Returns the pixel contribution value at the given index and position.
    ///
    /// # Arguments
    /// * `index` - The index of the pixel contribution map.
    /// * `pos_x` - The x position of the pixel contribution value.
    /// * `pos_y` - The y position of the pixel contribution value.
    pub fn get_value(&self, index: usize, pos_x: usize, pos_y: usize) -> f32 {
        let size = self.get_map_descriptor(index).map_size;
        self.inner.borrow()[index].inner.pixel_contrib[pos_y * size + pos_x]
    }

    /// Returns the pixel contribution values at the given position for all pixel contribution
    /// maps.
    ///
    /// # Arguments
    /// * `pos_x` - The x position of the pixel contribution values.
    /// * `pos_y` - The y position of the pixel contribution values.
    pub fn get_values_at_pos(&self, pos_x: usize, pos_y: usize) -> Float32Array {
        let inner = self.inner.borrow();
        let result = Float32Array::new_with_length(inner.len() as u32);

        inner.iter().enumerate().for_each(|(i, map)| {
            let size = map.inner.descriptor.size();
            let pos_index = pos_y * size + pos_x;

            let value = map.inner.pixel_contrib[pos_index];

            result.set_index(i as u32, value);
        });

        result
    }

    /// Adds a PixelContributionMap object to this PixelContributionMaps object.
    ///
    /// # Arguments
    /// * `map` - The PixelContributionMap object to add.
    pub fn add_map(&mut self, map: PixelContributionMap) {
        self.inner.borrow_mut().push(map);
    }
}

/// The descriptor for the pixel contribution map.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PixelContribColorMapDescriptor {
    /// The size of the quadratic pixel contribution map.
    pub map_size: usize,

    /// The camera angle for the pixel contribution map. The angle is in radians.
    /// A value of 0 means that the camera is orthographic.
    pub camera_angle: f32,
}

#[wasm_bindgen]
impl PixelContribColorMapDescriptor {
    pub fn index_from_camera_dir(&self, dir_x: f32, dir_y: f32, dir_z: f32) -> usize {
        pixel_contrib_types::PixelContribColorMapDescriptor::new(self.map_size, self.camera_angle)
            .index_from_camera_dir(Vec3::new(dir_x, dir_y, dir_z))
    }
}

impl From<pixel_contrib_types::PixelContribColorMapDescriptor> for PixelContribColorMapDescriptor {
    fn from(descriptor: pixel_contrib_types::PixelContribColorMapDescriptor) -> Self {
        Self {
            map_size: descriptor.size(),
            camera_angle: descriptor.camera_angle(),
        }
    }
}
