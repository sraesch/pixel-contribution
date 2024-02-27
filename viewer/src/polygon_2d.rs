use nalgebra_glm::{zero, Vec2};

/// A 2D polygon.
pub struct Polygon2D<const N: usize> {
    /// The vertices of the polygon in counterclockwise order.
    pub vertices: [Vec2; N],
}

impl<const N: usize> Polygon2D<N> {
    /// Creates a new 2D polygon.
    ///
    /// # Arguments
    /// * `vertices` - The vertices of the polygon in counterclockwise order.
    pub fn new(vertices: [Vec2; N]) -> Self {
        Self { vertices }
    }

    /// Computes the area of the polygon.
    pub fn compute_area(&self) -> f32 {
        // Using the Shoelace formula to compute the area of the polygon.
        // https://en.wikipedia.org/wiki/Shoelace_formula
        self.vertices
            .iter()
            .zip(self.vertices.iter().cycle().skip(1))
            .fold(0.0, |acc, (v1, v2)| acc + v1.x * v2.y - v1.y * v2.x)
            / 2.0
    }

    pub fn compute_area_with_overlapping_rectangle(&self, width: f32, height: f32) -> f32 {
        let mut area = 0f32;

        // Initialize index by finding the vertex that is right from the X-axis.
        // If there is no such vertex, the polygon is completely on the left side of the X-axis.
        // In this case, the area is 0.
        let start_index = match self.vertices.iter().position(|v| v.x >= 0f32) {
            Some(index) => index,
            None => return 0f32,
        };

        // The last vertex that is right from the X-axis
        let mut last_vertex = self.vertices[start_index];
        let mut cur_vertex = 1;
        while cur_vertex < N + 1 {
            let v2 = &self.vertices[(cur_vertex + start_index) % N];

            // If the next vertex is also on the right side of the x-axis
            if v2.x >= 0f32 {
                area += last_vertex.x * v2.y - last_vertex.y * v2.x;
                last_vertex = *v2;
                cur_vertex += 1;
                continue;
            }

            // Compute first intersection point of the line with the x-axis
            let vi1 = Self::intersection_with_axis::<0>(&last_vertex, v2, 0f32);

            // Add the contribution of the line-segment defined by last_vertex and vi1
            area += last_vertex.x * vi1.y - last_vertex.y * vi1.x;

            // Find the next vertex that is right from the x-axis.
            // Note that there must be such a vertex as the starting vertex is on the right side
            // of the x-axis.
            let next_vertex = (cur_vertex + 1..N + 1)
                .find(|i| self.vertices[(i + start_index) % N].x >= 0f32)
                .unwrap();

            let vi2 = {
                let v1 = &self.vertices[(next_vertex + start_index - 1) % N];
                let v2 = &self.vertices[(next_vertex + start_index) % N];

                Self::intersection_with_axis::<0>(v1, v2, 0f32)
            };

            area += vi1.x * vi2.y - vi1.y * vi2.x;

            // update the current vertex and the last vertex that is right from the x-axis
            last_vertex = vi2;
            cur_vertex = next_vertex;
        }

        area / 2f32
    }

    /// Computes the intersection with the line defined by the two given vertices and the
    /// specified axis at the given position.
    ///
    /// # Arguments
    /// * `v1` - The first vertex of the line.
    /// * `v2` - The second vertex of the line.
    /// * `axis_offset` - The offset of the axis along the other axis.
    #[inline]
    fn intersection_with_axis<const A: usize>(v1: &Vec2, v2: &Vec2, axis_offset: f32) -> Vec2 {
        let t = (axis_offset - v2[A]) / (v1[A] - v2[A]);
        t * v1 + (1f32 - t) * v2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area_cube() {
        let polygon = Polygon2D::new([
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ]);
        assert_eq!(polygon.compute_area(), 1.0);
    }

    #[test]
    fn test_area_triangle() {
        let polygon = Polygon2D::new([
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
        ]);
        assert_eq!(polygon.compute_area(), 0.5);
    }

    #[test]
    fn test_intersection_with_axis() {
        let v1 = Vec2::new(0.0, 0.0);
        let v2 = Vec2::new(1.0, 1.0);
        let axis_offset = 0.5;
        let intersection = Polygon2D::<2>::intersection_with_axis::<0>(&v1, &v2, axis_offset);
        assert_eq!(intersection, Vec2::new(0.5, 0.5));

        let axis_offset = 0.0;
        let intersection = Polygon2D::<2>::intersection_with_axis::<0>(&v1, &v2, axis_offset);
        assert_eq!(intersection, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_compute_area_with_overlapping_rectangle_simple() {
        let polygon = Polygon2D::new([
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ]);
        assert_eq!(
            polygon.compute_area_with_overlapping_rectangle(1.0, 1.0),
            1f32
        );
    }

    #[test]
    fn test_compute_area_with_overlapping_rectangle_x_axis() {
        let polygon = Polygon2D::new([
            Vec2::new(-1.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-2.0, 1.0),
        ]);
        assert_eq!(
            polygon.compute_area_with_overlapping_rectangle(0.0, 1.0),
            1f32
        );
    }

    #[test]
    fn test_compute_area_with_overlapping_rectangle_circle() {
        const N: usize = 4;
        let mut vertices = [zero(); N];

        for i in 0..N {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / N as f32;
            vertices[i] = Vec2::new(angle.cos(), angle.sin() + 1f32);
        }

        let polygon = Polygon2D::new(vertices);
        let full_area = polygon.compute_area();
        let half_area = polygon.compute_area_with_overlapping_rectangle(1.0, 2.0);

        assert!((full_area - half_area * 2f32).abs() < 1e-6);
    }
}
