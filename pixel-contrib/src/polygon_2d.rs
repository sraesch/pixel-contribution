use nalgebra_glm::{zero, Vec2};

/// A 2D polygon.
#[derive(Debug, Clone, Copy)]
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
    #[inline]
    pub fn compute_area(&self) -> f32 {
        compute_area_for_given_vertices(self.vertices.as_ref())
    }

    /// Computes the intersection of the given vertices with the given axis.
    ///
    /// # Arguments
    /// * `width` - The width of the rectangle.
    /// * `height` - The height of the rectangle.
    pub fn compute_area_with_overlapping_rectangle(&self, width: f32, height: f32) -> f32 {
        const CAP: usize = 36;
        let mut buf1: [Vec2; CAP] = [zero(); CAP];
        let mut buf2: [Vec2; CAP] = [zero(); CAP];

        let n = Self::cut_with_axis::<0, CAP>(0f32, self.vertices.as_ref(), 1f32, &mut buf1);
        let n = Self::cut_with_axis::<0, CAP>(width, &buf1[..n], -1f32, &mut buf2);
        let n = Self::cut_with_axis::<1, CAP>(0f32, &buf2[..n], 1f32, &mut buf1);
        let n = Self::cut_with_axis::<1, CAP>(height, &buf1[..n], -1f32, &mut buf2);

        compute_area_for_given_vertices(&buf2[..n])
    }

    /// Cuts the given vertices with the given axis and removes either the negative or positive
    /// part, depending on the factor.
    /// The result is stored in the out_vertices array and the number of vertices in the result is
    /// returned.
    ///
    /// # Arguments
    /// * `axis_offset` - The offset of the axis along the other axis.
    /// * `in_vertices` - The vertices of the polygon to cut.
    /// * `factor` - The factor for the defined axis to check if the vertex is on the correct side.
    /// * `out_vertices` - The vertices of the polygon after the cut.
    #[inline]
    fn cut_with_axis<const A: usize, const CAP: usize>(
        axis_offset: f32,
        in_vertices: &[Vec2],
        factor: f32,
        out_vertices: &mut [Vec2],
    ) -> usize {
        let n = in_vertices.len();
        if n == 0 {
            return 0;
        }

        let mut num_result_vertices = 0;

        let mut v1 = *in_vertices.first().unwrap();
        for i in 0..in_vertices.len() {
            let v2 = in_vertices[(i + 1) % n];

            let x1 = v1[A] - axis_offset;
            let x2 = v2[A] - axis_offset;

            // add the vertex if it is on the correct side of the axis
            if x1 * factor >= 0f32 {
                out_vertices[num_result_vertices] = v1;
                num_result_vertices += 1;
            }

            // check if the axis is intersecting
            if x1 * x2 < 0f32 {
                let t = x2 / (x2 - x1);
                out_vertices[num_result_vertices] = t * v1 + (1f32 - t) * v2;
                num_result_vertices += 1;
            }

            v1 = v2;
        }

        num_result_vertices
    }
}

/// Computes the area of the polygon defined by the given vertices.
///
/// # Arguments
/// * `vertices` - The vertices of the polygon in counterclockwise order.
fn compute_area_for_given_vertices(vertices: &[Vec2]) -> f32 {
    // Using the Shoelace formula to compute the area of the polygon.
    // https://en.wikipedia.org/wiki/Shoelace_formula
    vertices
        .iter()
        .zip(vertices.iter().cycle().skip(1))
        .fold(0.0, |acc, (v1, v2)| acc + v1.x * v2.y - v1.y * v2.x)
        / 2.0
}

#[cfg(test)]
mod tests {
    use nalgebra_glm::zero;

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
            polygon.compute_area_with_overlapping_rectangle(1.0, 1.0),
            1f32
        );
    }

    #[test]
    fn test_compute_area_with_overlapping_rectangle_circle() {
        const N: usize = 4;
        let mut vertices = [zero(); N];

        vertices.iter_mut().enumerate().for_each(|(i, v)| {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / N as f32;
            *v = Vec2::new(angle.cos(), angle.sin() + 1f32);
        });

        let polygon = Polygon2D::new(vertices);
        let full_area = polygon.compute_area();
        let half_area = polygon.compute_area_with_overlapping_rectangle(1.0, 2.0);

        assert!((full_area - half_area * 2f32).abs() < 1e-6);
    }

    #[test]
    fn test_compute_area_with_overlapping_rectangle_circle_corners() {
        const N: usize = 4;

        let width = 2f32;
        let height = 2f32;

        let corners = [
            Vec2::new(0f32, 0f32),
            Vec2::new(width, 0f32),
            Vec2::new(width, height),
            Vec2::new(0f32, height),
        ];

        for (i, corner) in corners.iter().enumerate() {
            let mut vertices = [zero(); N];

            vertices.iter_mut().enumerate().for_each(|(i, v)| {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / N as f32;
                *v = Vec2::new(angle.cos(), angle.sin()) + corner;
            });

            let polygon = Polygon2D::new(vertices);
            let full_area = polygon.compute_area();
            let quarter_area = polygon.compute_area_with_overlapping_rectangle(width, height);

            assert!(
                (full_area - quarter_area * 4f32).abs() < 1e-6,
                "Corner({})={}: Is={}, Should={}",
                i,
                corner,
                quarter_area,
                full_area / 4f32
            );
        }
    }

    #[test]
    fn test_compute_area_with_overlapping_rectangle_circle_huge() {
        const N: usize = 32;
        let mut vertices = [zero(); N];

        vertices.iter_mut().enumerate().for_each(|(i, v)| {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / N as f32;
            *v = Vec2::new(angle.cos(), angle.sin()) * 10f32 + Vec2::new(0.5f32, 0.5f32);
        });

        let polygon = Polygon2D::new(vertices);
        let area = polygon.compute_area_with_overlapping_rectangle(1.0, 1.0);

        assert!((area - 1f32).abs() < 1e-6);
    }

    #[test]
    fn test_compute_area_with_overlapping_rectangle_huge() {
        let rectangle = [
            Vec2::new(0.5, 2.0),
            Vec2::new(0.5, -2.0),
            Vec2::new(2.0, -2.0),
            Vec2::new(2.0, 2.0),
        ];

        let polygon = Polygon2D::new(rectangle);
        let area = polygon.compute_area_with_overlapping_rectangle(1.0, 1.0);

        assert!(
            (area - 0.5f32).abs() < 1e-6,
            "Is={}, Should={}",
            area,
            0.5f32
        );
    }
}
