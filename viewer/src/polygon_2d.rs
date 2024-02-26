use nalgebra_glm::Vec2;

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
}
