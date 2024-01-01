use nalgebra_glm::Vec3;
use rasterizer::{Aabb, Scene};

/// A sphere defined by its center and radius.
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    /// Creates a new sphere from the given scene by computing the bounding sphere of the scene.
    ///
    /// # Arguments
    /// * `scene` - The scene to create the sphere from.
    pub fn from_scene(scene: &Scene) -> Self {
        let bounding_sphere = scene.compute_bounding_sphere();

        Self::from(bounding_sphere)
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

impl From<(Vec3, f32)> for Sphere {
    fn from(sphere: (Vec3, f32)) -> Self {
        Self {
            center: sphere.0,
            radius: sphere.1,
        }
    }
}

#[cfg(test)]
mod test {
    use cad_import::loader::{loader_off::LoaderOff, Loader, MemoryResource};

    use super::*;

    #[test]
    fn test_sphere_from_aabb() {
        let cube: Aabb = Aabb::new_cube(&Vec3::new(0.5, 0.5, 0.5), 1.0);

        assert_eq!(cube.get_center(), Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(cube.get_size(), Vec3::new(1.0, 1.0, 1.0));

        let sphere = Sphere::from_aabb(&cube);

        assert_eq!(sphere.center, Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(sphere.radius, 0.8660254);
    }

    #[test]
    fn test_sphere_from_scene() {
        let scene_data = include_bytes!("../../test_data/models/sphere.off");
        let memory_resource = MemoryResource::new(scene_data, "model/off".to_owned());
        let off_loader = LoaderOff::new();
        let cad_data = off_loader.read(&memory_resource).unwrap();
        let scene = Scene::new_from_cad(&cad_data).unwrap();

        let sphere = Sphere::from_scene(&scene);

        assert_eq!(sphere.center, Vec3::new(0.0, 0.0, 0.0));
        assert!((sphere.radius - 1.0).abs() < 1e-6);
    }
}
