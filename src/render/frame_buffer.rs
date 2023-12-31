use nalgebra_glm::Vec3;

use crate::utils::clamp;

use super::{Frame, Histogram};

/// The data type used for the depth buffer.
pub type DepthBufferPrecision = u32;

/// Transforms the given depth value, between 0 and 1 to the integer-based depth value.
///
/// # Arguments
/// * `depth` - The depth-value in floating-point encoding.
#[inline]
pub fn to_depth_buffer_precision(depth: f32) -> DepthBufferPrecision {
    debug_assert!(depth >= 0f32 && depth <= 1f32);

    const F_MAX: f32 = DepthBufferPrecision::MAX as f32;
    let depth = (depth * F_MAX) as DepthBufferPrecision;

    depth
}

/// The trait for the depth buffer
pub trait DepthBuffer {
    /// Creates and returns a new empty depth-buffer.
    ///
    /// # Arguments
    /// * `size` - The size of the quadratic depth-buffer.
    fn new(size: usize) -> Self;

    /// Returns the size of the depth-buffer.
    fn get_size(&self) -> usize;

    /// Clears the depth buffer.
    fn clear(&mut self);

    /// Tries to write the given depth-value into the specified position. Returns true if the depth
    /// value has been written and false otherwise, i.e., if the depth-test failed.
    ///
    /// # Arguments
    /// * `x` - The x-coordinate of the position.
    /// * `y` - The y-coordinate of the position.
    /// * `depth_value` - The depth-value to write.
    fn write(&mut self, x: usize, y: usize, depth_value: DepthBufferPrecision) -> bool;

    /// Returns a reference onto the internally stored depth values. The depth values are stored
    /// row-wise.
    fn get_depth_values(&self) -> &[DepthBufferPrecision];

    /// Merges an other depth-buffer into this depth-buffer.
    ///
    /// # Arguments
    /// * `rhs` - The depth-buffer to merge into this depth-buffer.
    fn merge_depth_buffer<D2: DepthBuffer>(&mut self, rhs: &D2);

    /// Returns true if at least one pixel of the rectangle with the given depth is visible.
    ///
    /// # Arguments
    /// * `x0` - The left x-coordinate.
    /// * `y0` - The bottom y-coordinate.
    /// * `x1` - The right x-coordinate.
    /// * `y1` - The top y-coordinate.
    /// * `depth` - The depth value of the rectangle to check.
    fn is_rectangle_visible(
        &self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        depth: DepthBufferPrecision,
    ) -> bool;
}

/// A single quadratic frame buffer for rasterization operations.
pub struct FrameBuffer<D: DepthBuffer> {
    /// The size of the quadratic frame buffer.
    pub size: usize,

    /// The depth buffer that stores the depth values.
    pub depth_buffer: D,

    /// The id buffer that stores the per pixel object ids.
    pub id_buffer: Vec<Option<u32>>,
}

impl<D: DepthBuffer> FrameBuffer<D> {
    /// Creates and returns a new empty frame buffer
    ///
    /// # Arguments
    /// * `size` - The size of the quadratic frame buffer.
    pub fn new(size: usize) -> Self {
        let depth_buffer = D::new(size);
        let id_buffer = vec![None; size * size];

        Self {
            size,
            depth_buffer,
            id_buffer,
        }
    }

    /// Clears the frame buffer, i.e., resets all depth values and all object ids.
    pub fn clear(&mut self) {
        self.depth_buffer.clear();
        self.id_buffer.fill(None);
    }

    /// Rasterizes the triangle given in its window coordinates.
    ///
    /// # Arguments
    /// * `id` - The object id to which the triangle belongs to.
    /// * `p0` - The first vertex of the triangle in window coordinates.
    /// * `p1` - The second vertex of the triangle in window coordinates.
    /// * `p2` - The third vertex of the triangle in window coordinates.
    pub fn rasterize(&mut self, id: u32, p0: &Vec3, p1: &Vec3, p2: &Vec3) {
        // sort the vertices in ascending order with respect to their y coordinate

        if p0.y <= p1.y && p0.y <= p2.y {
            // case 1: p0 has smallest y-coordinate
            if p1.y <= p2.y {
                self.fill_triangle(id, p0, p1, p2);
            } else {
                self.fill_triangle(id, p0, p2, p1);
            }
        } else if p1.y <= p0.y && p1.y <= p2.y {
            // case 2: p1 has smallest y-coordinate
            if p0.y <= p2.y {
                self.fill_triangle(id, p1, p0, p2);
            } else {
                self.fill_triangle(id, p1, p2, p0);
            }
        } else {
            // case 3: p2 has smallest y-coordinate
            if p0.y <= p1.y {
                self.fill_triangle(id, p2, p0, p1);
            } else {
                self.fill_triangle(id, p2, p1, p0);
            }
        }
    }

    /// Computes the histogram for the ids inside the frame buffer.
    ///
    /// # Arguments
    /// * `histogram` - The histogram to update.
    pub fn compute_histogram(&self, histogram: &mut Histogram) {
        // determine the maximal id
        let max_id = self
            .id_buffer
            .iter()
            .map(|id| id.unwrap_or(0))
            .max()
            .unwrap_or(0);

        histogram.resize(max_id as usize + 1, 0);
        histogram.fill(0u32);

        for id in self.id_buffer.iter() {
            match id {
                Some(id) => {
                    histogram[*id as usize] += 1;
                }
                None => {}
            }
        }
    }

    /// Returns the data stored in the framebuffer.
    ///
    /// # Arguments
    /// * `f` - Reference for storing the resulting frame.
    pub fn get_frame(&self, f: &mut Frame) {
        assert_eq!(f.get_frame_size(), self.size);

        // set ids
        let id_buffer = f.get_id_buffer_mut();
        id_buffer.copy_from_slice(&self.id_buffer);

        match f.get_depth_buffer_mut() {
            Some(depth_buffer) => {
                depth_buffer
                    .iter_mut()
                    .zip(
                        self.depth_buffer
                            .get_depth_values()
                            .iter()
                            .map(|x| (*x as f32) / (DepthBufferPrecision::MAX as f32)),
                    )
                    .for_each(|(dst, src)| *dst = src);
            }
            None => {}
        }
    }

    /// Merges the right-hand side frame buffer into this one using the depth-buffer.
    ///
    /// # Arguments
    /// * `rhs` - The right-hand side frame buffer to merge.
    pub fn merge_frame_buffer<D2: DepthBuffer>(&mut self, rhs: &FrameBuffer<D2>) {
        // update the internal id-buffer
        self.id_buffer
            .iter_mut()
            .zip(self.depth_buffer.get_depth_values().iter())
            .zip(
                rhs.id_buffer
                    .iter()
                    .zip(rhs.depth_buffer.get_depth_values().iter()),
            )
            .for_each(|((dst_id, dst_depth), (src_id, src_depth))| {
                if *src_depth < *dst_depth {
                    *dst_id = *src_id;
                }
            });

        // update the internal depth-buffer
        self.depth_buffer.merge_depth_buffer(&rhs.depth_buffer);
    }

    /// Returns the size of the quadratic frame buffer.
    #[inline]
    pub fn get_frame_size(&self) -> usize {
        self.size
    }

    /// Rasterizes the given triangle with the assumption that the points are sorted in ascending
    /// order with respect to their y-coordinates.
    ///
    /// # Arguments
    /// * `id` - The object id to which the triangle belongs to.
    /// * `p0` - The first vertex of the triangle in window coordinates.
    /// * `p1` - The second vertex of the triangle in window coordinates.
    /// * `p2` - The third vertex of the triangle in window coordinates.
    fn fill_triangle(&mut self, id: u32, p0: &Vec3, p1: &Vec3, p2: &Vec3) {
        let (y0, y1, y2) = (p0[1], p1[1], p2[1]);

        debug_assert!(y0 <= y1 && y1 <= y2);

        if y0.round() == y2.round() {
            // check special case, where the triangle is a line
            let y = y0.round();

            // make sure that the line is inside the frame
            if y >= 0f32 && y < self.size as f32 {
                let y = y as usize;

                let (x0, x1, depth0, depth1) = if p0.x <= p2.x {
                    (p0.x, p2.x, p0.z, p2.z)
                } else {
                    (p2.x, p0.x, p2.z, p0.z)
                };

                self.draw_scanline(id, y, x0, x1, depth0, depth1);
            }
        } else if y0.round() == y1.round() {
            // check for top-flat case
            self.fill_top_flat_triangle(id, p0, p1, p2);
        } else if y1.round() == y2.round() {
            // check for bottom-flat case
            self.fill_bottom_flat_triangle(id, p0, p1, p2);
        } else {
            // ok we have that the y-coordinates define a strict ascending order
            // thus we split the triangle in a bottom and top flat triangle, but need to define
            // a new point p3

            let lambda = (y1 - y0) / (y2 - y0);
            assert!(
                lambda >= 0f32 && lambda <= 1f32,
                "Lambda must be between 0 and 1, but is {}. y0={}, y1={}, y2={}",
                lambda,
                y0,
                y1,
                y2
            );

            let x3 = p0[0] + lambda * (p2[0] - p0[0]);
            let z3 = p0[2] + lambda * (p2[2] - p0[2]);

            let p3 = Vec3::new(x3, y1, z3);

            self.fill_bottom_flat_triangle(id, p0, p1, &p3);
            self.fill_top_flat_triangle(id, p1, &p3, p2);
        }
    }

    /// Draws a triangle with a horizontal bottom, i.e. p1[1] == p2[1]
    ///
    /// # Arguments
    /// * `id` - The object id to which the triangle belongs to.
    /// * `p0` - The first vertex of the triangle in window coordinates.
    /// * `p1` - The second vertex of the triangle in window coordinates.
    /// * `p2` - The third vertex of the triangle in window coordinates.
    fn fill_bottom_flat_triangle(&mut self, id: u32, p0: &Vec3, p1: &Vec3, p2: &Vec3) {
        let max_y = self.size as f32 - 1f32;

        // p1 and p2 are both on the same height and p0 is at least lower or equal
        debug_assert!(p1[1].round() == p2[1].round());
        debug_assert!(p0[1] <= p1[1]);
        let y1 = p2[1];

        // if p0 is not strictly lower, then the triangle is degenerated and we won't draw it
        if p0[1] == p1[1] {
            return;
        }

        let y0 = p0[1];

        debug_assert!(y0 < y1);

        // sort out extreme cases
        if y1 < 0f32 || y0 > max_y {
            return;
        }

        // clamp y0 and y1 s.t. they fit into the current frame
        let y0m = clamp(y0.round(), 0f32, max_y) as usize;
        let y1m = clamp(y1.round(), 0f32, max_y) as usize;

        // compute the start and end of the bottom
        let (left_x, right_x, left_depth, right_depth) = if p1[0] < p2[0] {
            (p1[0], p2[0], p1[2], p2[2])
        } else {
            (p2[0], p1[0], p2[2], p1[2])
        };

        for y in y0m..(y1m + 1) {
            let yf = clamp((y as f32 - y0) / (y1 - y0), 0f32, 1f32);
            debug_assert!(0f32 <= yf && yf <= 1f32);

            let x0 = (p0[0] + yf * (left_x - p0[0])).round();
            let x1 = (p0[0] + yf * (right_x - p0[0])).round();
            debug_assert!(x0 <= x1);

            let depth0 = p0[2] + yf * (left_depth - p0[2]);
            let depth1 = p0[2] + yf * (right_depth - p0[2]);

            self.draw_scanline(id, y, x0, x1, depth0, depth1);
        }
    }

    /// Draws a triangle with a horizontal top, i.e. p0[1] == p1[1]
    ///
    /// # Arguments
    /// * `id` - The id of the object to which the triangle belongs to.
    /// * `p0` - The first vertex of the triangle in window coordinates.
    /// * `p1` - The second vertex of the triangle in window coordinates.
    /// * `p2` - The third vertex of the triangle in window coordinates.
    fn fill_top_flat_triangle(&mut self, id: u32, p0: &Vec3, p1: &Vec3, p2: &Vec3) {
        let max_y = self.size as f32 - 1f32;

        // p0 and p1 are both on the same height and p2 is at least higher or equal
        debug_assert!(p0[1].round() == p1[1].round());
        debug_assert!(p1[1] <= p2[1]);
        let y1 = p2[1];

        // if p2 is not strictly higher, then the triangle is degenerated and we won't draw it
        if p2[1] == p0[1] {
            return;
        }

        let y0 = p0[1];

        debug_assert!(y0 < y1);

        // sort out extreme cases
        if y1 < 0f32 || y0 > max_y {
            return;
        }

        // clamp y0 and y1 s.t. they fit into the current frame
        let y0m = clamp(y0.round(), 0f32, max_y) as usize;
        let y1m = clamp(y1.round(), 0f32, max_y) as usize;

        // compute the start and end of the top
        let (left_x, right_x, left_depth, right_depth) = if p0[0] < p1[0] {
            (p0[0], p1[0], p0[2], p1[2])
        } else {
            (p1[0], p0[0], p1[2], p0[2])
        };

        // draw the scan lines
        for y in y0m..(y1m + 1) {
            let yf = clamp((y1 - y as f32) / (y1 - y0), 0f32, 1f32);
            debug_assert!(0f32 <= yf && yf <= 1f32);

            let x0 = (p2[0] + yf * (left_x - p2[0])).round();
            let x1 = (p2[0] + yf * (right_x - p2[0])).round();
            debug_assert!(x0 <= x1);

            let depth0 = p2[2] + yf * (left_depth - p2[2]);
            let depth1 = p2[2] + yf * (right_depth - p2[2]);

            self.draw_scanline(id, y, x0, x1, depth0, depth1);
        }
    }

    /// Draws a single horizontal line at y-th position from x0 to x1 with given respective depth
    /// values depth0 and depth1.
    ///
    /// # Arguments
    /// * `id` - The id of the object to which the scan-line belongs to.
    /// * `y` - The y-value of the horizontal line.
    /// * `x0` - The left x-value of the line
    /// * `x1` - The right x-value of the line
    /// * `depth0` - The depth-value of the left side of the line.
    /// * `depth1` - The depth-value of the right side of the line.
    fn draw_scanline(&mut self, id: u32, y: usize, x0: f32, x1: f32, depth0: f32, depth1: f32) {
        debug_assert!(y < self.size);
        debug_assert!(x0 <= x1);

        let x0 = x0.round();
        let x1 = x1.round();

        let max_x = self.size as f32 - 1f32;

        // check special case where the line is completely out of the frame
        if x1 < 0f32 || x0 > max_x {
            return;
        }

        // clamp line to the window coordinates
        let x0m = clamp(x0, 0f32, max_x) as usize;
        let x1m = clamp(x1, 0f32, max_x) as usize;
        let dd: f32 = if x1 > x0 {
            (depth1 - depth0) / (x1 - x0)
        } else {
            0f32
        };

        for x in x0m..(x1m + 1) {
            let depth = depth0 + ((x as f32) - x0) * dd;
            self.draw_pixel(id, x, y, depth);
        }
    }

    /// Draws a single pixel with the given id and depth and checks if the pixel is within bounds.
    ///
    /// # Arguments
    /// * `id` - The id of the pixel.
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    /// * `depth` - The depth value of the pixel.
    #[inline]
    fn draw_pixel(&mut self, id: u32, x: usize, y: usize, depth: f32) {
        debug_assert!(x < self.size || y < self.size);

        // make sure depth is within bounds and valid
        if depth < 0f32 || depth > 1f32 || depth.is_infinite() || depth.is_nan() {
            return;
        }

        let depth = to_depth_buffer_precision(depth);

        if !self.depth_buffer.write(x, y, depth) {
            return;
        }

        // compute pixel index
        let index = y * self.size + x;

        // update id-buffer
        self.id_buffer[index] = Some(id);
    }
}

impl<D: DepthBuffer> Default for FrameBuffer<D> {
    fn default() -> Self {
        Self::new(0)
    }
}

/// A simple depth buffer implementation that stores the depth values in a vector.
pub struct SimpleDepthBuffer {
    /// The depth values of the buffer.
    depth_values: Vec<DepthBufferPrecision>,

    /// The size of the quadratic buffer.
    size: usize,
}

impl DepthBuffer for SimpleDepthBuffer {
    fn new(size: usize) -> Self {
        Self {
            size,
            depth_values: vec![DepthBufferPrecision::MAX; size * size],
        }
    }

    #[inline]
    fn get_size(&self) -> usize {
        self.size
    }

    fn clear(&mut self) {
        self.depth_values.fill(DepthBufferPrecision::MAX);
    }

    #[inline]
    fn write(&mut self, x: usize, y: usize, depth_value: DepthBufferPrecision) -> bool {
        debug_assert!(x < self.size && y < self.size);

        let dst = &mut self.depth_values[y * self.size + x];
        if *dst > depth_value {
            *dst = depth_value;
            true
        } else {
            false
        }
    }

    #[inline]
    fn get_depth_values(&self) -> &[DepthBufferPrecision] {
        &self.depth_values
    }

    fn merge_depth_buffer<D2: DepthBuffer>(&mut self, rhs: &D2) {
        let rhs_values = rhs.get_depth_values();
        self.depth_values
            .iter_mut()
            .zip(rhs_values.iter())
            .for_each(|(dst, src)| {
                if *dst > *src {
                    *dst = *src;
                }
            });
    }

    fn is_rectangle_visible(
        &self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        depth: DepthBufferPrecision,
    ) -> bool {
        assert!(x0 <= x1 && y0 <= y1);

        let size = self.size;
        let depth_values = &self.depth_values;

        let x0 = clamp(x0, 0, size - 1);
        let y0 = clamp(y0, 0, size - 1);
        let x1 = clamp(x1, 0, size - 1);
        let y1 = clamp(y1, 0, size - 1);

        for y in y0..(y1 + 1) {
            let scanline = &depth_values[(y * size + x0)..(y * size + x1 + 1)];

            let highest_depth = scanline.iter().max().cloned().unwrap();
            if highest_depth > depth {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Small helper function to compute the area for the given triangle.
    fn compute_triangle_area(p0: &Vec3, p1: &Vec3, p2: &Vec3) -> f32 {
        let a: Vec3 = p1 - p0;
        let b: Vec3 = p2 - p0;

        a.cross(&b).norm() / 2f32
    }

    #[test]
    fn test_fill_bottom_flat_triangle() {
        let size = 128;

        let mut f = FrameBuffer::<SimpleDepthBuffer>::new(size);

        let id = 42;

        let p0 = Vec3::new(20f32, 10f32, 0.5f32);
        let p1 = Vec3::new(40f32, 40f32, 0.5f32);
        let p2 = Vec3::new(10f32, 40f32, 0.5f32);

        f.fill_bottom_flat_triangle(id, &p0, &p1, &p2);

        let area = compute_triangle_area(&p0, &p1, &p2);

        let num_ids = f.id_buffer.iter().filter(|i| **i == Some(id)).count();
        println!("Num Ids: {}", num_ids);
        println!("Triangle Area: {}", area);

        let height = 30f32;
        let error_eps = height * 2f32;

        assert!(((num_ids as f32) - area).abs() <= error_eps);

        let mut last_line_length: usize = 0;

        for y in 0..size {
            let mut x_start = size;
            let mut x_end: usize = 0;

            for x in 0..size {
                let id = f.id_buffer[y * size + x];
                if id == Some(42) {
                    x_start = x_start.min(x);
                    x_end = x_end.max(x);
                }
            }

            let line_length = if x_start > x_end {
                0
            } else {
                x_end - x_start + 1
            };

            assert!(y < 10 || y > 40 || line_length > 0);
            assert!(y < 10 || y > 40 || last_line_length <= line_length);

            last_line_length = line_length;
        }
    }

    #[test]
    fn test_fill_top_flat_triangle() {
        let size = 128;

        let mut f = FrameBuffer::<SimpleDepthBuffer>::new(size);

        let id = 42;

        let p0 = Vec3::new(40f32, 10f32, 0.5f32);
        let p1 = Vec3::new(10f32, 10f32, 0.5f32);
        let p2 = Vec3::new(20f32, 40f32, 0.5f32);

        f.fill_top_flat_triangle(id, &p0, &p1, &p2);

        let area = compute_triangle_area(&p0, &p1, &p2);

        let num_ids = f.id_buffer.iter().filter(|i| **i == Some(id)).count();
        println!("Num Ids: {}", num_ids);
        println!("Triangle Area: {}", area);

        let height = 30f32;
        let error_eps = height * 2f32;

        assert!(((num_ids as f32) - area).abs() <= error_eps);

        let mut last_line_length: usize = size;

        for y in 0..size {
            let mut x_start = size;
            let mut x_end: usize = 0;

            for x in 0..size {
                let id = f.id_buffer[y * size + x];
                if id == Some(42) {
                    x_start = x_start.min(x);
                    x_end = x_end.max(x);
                }
            }

            let line_length = if x_start > x_end {
                0
            } else {
                x_end - x_start + 1
            };

            assert!(y < 10 || y > 40 || line_length > 0);
            assert!(y <= 10 || y > 40 || last_line_length >= line_length);

            last_line_length = line_length;
        }
    }
}
