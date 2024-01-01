use nalgebra_glm::{Vec2, Vec3};

#[inline]
fn wrap_octahedron_normal_value(v1: f32, v2: f32) -> f32 {
    (1.0 - v2.abs()) * (if v1 >= 0.0 { 1.0 } else { -1.0 })
}

/// Consumes a normal and returns the encoded octahedron normal as a 2D vector in the range [0, 1].
///
/// # Arguments
/// * `normal` - The normal to encode
pub fn encode_octahedron_normal(normal: Vec3) -> Vec2 {
    let normal = normal.normalize();
    let abs_sum = normal[0].abs() + normal[1].abs() + normal[2].abs();

    let mut normal = normal;

    normal[0] /= abs_sum;
    normal[1] /= abs_sum;

    if normal[2] < 0.0 {
        let tmp = normal[0];
        normal[0] = wrap_octahedron_normal_value(normal[0], normal[1]);
        normal[1] = wrap_octahedron_normal_value(normal[1], tmp);
    }

    Vec2::new(normal[0] * 0.5 + 0.5, normal[1] * 0.5 + 0.5)
}

/// Consumes a normal encoded as octahedron in the range [0,1] and returns the decoded normal.
///
/// # Arguments
/// * `octahedron` - The normal encoded as octahedron.
pub fn decode_octahedron_normal(octahedron: Vec2) -> Vec3 {
    let octahedron = octahedron * 2.0 - Vec2::new(1.0, 1.0);
    let z = 1.0 - octahedron[0].abs() - octahedron[1].abs();

    let x = if z >= 0.0 {
        octahedron[0]
    } else {
        wrap_octahedron_normal_value(octahedron[0], octahedron[1])
    };

    let y = if z >= 0.0 {
        octahedron[1]
    } else {
        wrap_octahedron_normal_value(octahedron[1], octahedron[0])
    };

    Vec3::new(x, y, z).normalize()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_octahedron_encoding() {
        let num = 20;
        let pi = std::f32::consts::PI;
        let pi_2 = std::f32::consts::FRAC_PI_2;

        for i in 0..(num + 1) {
            // beta is angle between -PI/2 and +PI/2
            let beta: f32 = (i as f32) / (num as f32) * pi - pi_2;

            // determine the radius on the 2D XY-plane
            let r2 = beta.cos();

            // determine value for Z
            let z = beta.sin();

            for j in 0..num {
                // alpha is angle between 0 and 2 * PI
                let alpha: f32 = (j as f32) / (num as f32) * 2.0 * pi;

                // determine value for X and Y
                let x = alpha.cos() * r2;
                let y = alpha.sin() * r2;

                let nrm = Vec3::new(x, y, z);

                // octahedron encoding
                let octahedron = encode_octahedron_normal(nrm);
                assert!(0.0 <= octahedron[0] && octahedron[0] <= 1.0);
                assert!(0.0 <= octahedron[1] && octahedron[1] <= 1.0);

                // octahedron decoding
                let nrm2 = decode_octahedron_normal(octahedron);

                // compute error
                let angle_error = (1.0 - nrm.dot(&nrm2)).abs();

                assert!(angle_error <= 1e-6, "Decoding error is too high");
            }
        }
    }
}
