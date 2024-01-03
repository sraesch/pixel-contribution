use core::num;

use nalgebra_glm::Vec3;

/// Creates a sphere with the given radius and number of segments.
///
/// # Arguments
/// * `radius` - The radius of the sphere
/// * `num_rings` - The number of segments in the sphere
/// * `num_sectors` - The number of segments in the sphere
pub fn create_sphere(radius: f32, num_rings: usize, num_sectors: usize) -> (Vec<Vec3>, Vec<u32>) {
    // create the vertices
    let mut vertices = Vec::with_capacity((num_rings - 2) * num_sectors + 2);
    vertices.push(Vec3::new(0.0, -radius, 0.0));

    // we skip the first and the list ring as these are the poles
    for r in 1..(num_rings - 1) {
        // map the ring index to the value from ]-0.5, 0.5[
        let r = r as f32 / (num_rings - 1) as f32 - 0.5;
        // compute the angle of the ring from -pi/2 to pi/2
        let beta = std::f32::consts::PI * r;

        // for the current ring, we create the vertices by going through the sectors of the ring
        for s in 0..num_sectors {
            // map the sector index to the value from [0, 1[
            let s = s as f32 / num_sectors as f32;

            // compute the angle of the sector from 0 to 2pi
            let alpha = 2f32 * std::f32::consts::PI * s;

            let y = beta.sin();
            let x = alpha.cos() * beta.cos();
            let z = alpha.sin() * beta.cos();

            vertices.push(Vec3::new(x * radius, y * radius, z * radius));
        }
    }

    vertices.push(Vec3::new(0.0, radius, 0.0));

    let mut indices = Vec::with_capacity(num_rings * num_sectors * 4);

    let num_sectors = num_sectors as u32;
    let num_rings = num_rings as u32;

    // create the indices for the bottom pole
    let bottom_pole_index = 0;
    let vertex_offset = 1;
    for s in 0..num_sectors {
        indices.push(bottom_pole_index);
        indices.push(s + vertex_offset);
        indices.push((s + 1) % num_sectors + vertex_offset);
    }

    // create the indices for the rings
    for r in 1..(num_rings - 2) {
        let vertex_offset = 1 + (r - 1) * num_sectors;
        for s in 0..num_sectors {
            indices.push(s + vertex_offset);
            indices.push(s + vertex_offset + num_sectors);
            indices.push((s + 1) % num_sectors + vertex_offset + num_sectors);

            indices.push(s + vertex_offset);
            indices.push((s + 1) % num_sectors + vertex_offset + num_sectors);
            indices.push((s + 1) % num_sectors + vertex_offset);
        }
    }

    // create the indices for the top pole
    let vertex_offset = vertices.len() as u32 - 1 - num_sectors;
    let top_pole_index = vertices.len() as u32 - 1;
    for s in 0..num_sectors {
        indices.push(top_pole_index);
        indices.push(s + vertex_offset);
        indices.push((s + 1) % num_sectors + vertex_offset);
    }

    (vertices, indices)
}
