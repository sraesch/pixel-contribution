use nalgebra_glm::{distance2, Vec3};

use crate::Aabb;

/// A conservative bounding sphere that encloses a set of objects.
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundingSphere {
    /// The center of the bounding sphere.
    pub center: Vec3,

    /// The radius of the bounding sphere.
    pub radius: f32,
}

impl BoundingSphere {
    /// Creates a new bounding sphere from the given points
    ///
    /// # Arguments
    /// * `iter` - The points to create the bounding sphere from.
    pub fn new_from_iter<I>(iter: I) -> Self
    where
        I: Iterator<Item = Vec3> + Clone,
    {
        let iter = iter.into_iter();

        // Compute center of the bounding sphere by computing the AABB
        // Note: This is not the optimal way to compute the bounding sphere center,
        //       but it is fast and simple.
        let aabb = Aabb::from_iter(iter.clone());
        let center = aabb.get_center();

        // determine the radius of the bounding sphere by computing the maximum distance
        // from the center to any point in the set
        let radius = iter.fold(0f32, |max_dist, p| max_dist.max(distance2(&center, &p)));
        let radius = radius.sqrt();

        Self { center, radius }
    }

    /// Creates a new sphere that tightly bounds the given AABB.
    ///
    /// # Arguments
    /// * `aabb` - The AABB to create the sphere from.
    pub fn from_aabb(aabb: &Aabb) -> Self {
        let center = aabb.get_center();
        let radius = aabb.get_size().norm() / 2.0;

        Self { center, radius }
    }
}

impl From<(Vec3, f32)> for BoundingSphere {
    fn from(sphere: (Vec3, f32)) -> Self {
        Self {
            center: sphere.0,
            radius: sphere.1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sphere_from_aabb() {
        let cube: Aabb = Aabb::new_cube(&Vec3::new(0.5, 0.5, 0.5), 1.0);

        assert_eq!(cube.get_center(), Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(cube.get_size(), Vec3::new(1.0, 1.0, 1.0));

        let sphere = BoundingSphere::from_aabb(&cube);

        assert_eq!(sphere.center, Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(sphere.radius, 0.8660254);
    }
} // Add this closing curly brace

#[test]
fn test_sphere_from_scene() {
    // create many points on the surface of a sphere
    let points: Vec<Vec3> = (0..1000)
        .zip(0..1000)
        .map(|(i, j)| {
            let i = i as f32 / 100f32;
            let j = j as f32 / 100f32;
            let alpha = i * 2f32 * std::f32::consts::PI;
            let beta = (j - 0.5f32) * std::f32::consts::PI;

            let x = beta.sin() * alpha.cos();
            let y = beta.sin() * alpha.sin();
            let z = beta.cos();

            Vec3::new(x, y, z)
        })
        .collect();

    let sphere = BoundingSphere::new_from_iter(points.iter().cloned());

    let l = sphere.center.norm();
    assert!(l < 1e-6);
    assert!((sphere.radius - 1.0).abs() < 1e-6);
}
