use nalgebra_glm::Vec3;

/// Convert polar coordinates on the unit sphere defined by the angle $\alpha$ and $\beta to
/// 3D cartesian coordinates.
/// The polar coordinates are defined as follows:
/// * $\alpha$ is the angle in the x-y plane from the x-axis.
/// * $\beta$ is the angle from the z-axis where $\beta = 0$ is along the equator.
///
/// # Arguments
/// * `alpha` - The angle $\alpha$ in radians.
/// * `beta` - The angle $\beta$ in radians.
pub fn polar_to_cartesian(alpha: f32, beta: f32) -> Vec3 {
    let x = beta.cos() * alpha.cos();
    let y = beta.cos() * alpha.sin();
    let z = beta.sin();

    Vec3::new(x, y, z)
}

/// Convert 3D cartesian coordinates to polar coordinates on the unit sphere.
///
/// # Arguments
/// * `p` - the 3D cartesian coordinates.
pub fn cartesian_to_polar(p: &Vec3) -> (f32, f32) {
    // compute the length of the projected vector on the x-y plane
    let r = (p.x.powi(2) + p.y.powi(2)).sqrt();

    let alpha = if r < 5e-8 {
        0.0
    } else {
        let mut alpha = p.y.atan2(p.x);
        if alpha < 0.0 {
            alpha += 2.0 * std::f32::consts::PI;
        }

        alpha
    };

    let beta = p.z.atan2(r);

    (alpha, beta)
}

#[cfg(test)]
mod test {
    use math::clamp;
    use nalgebra_glm::distance;

    use super::*;

    #[test]
    fn test_polar_to_cartesian() {
        let p = polar_to_cartesian(0.0, 0.0);
        assert!(distance(&p, &Vec3::new(1.0, 0.0, 0.0)) < 1e-6);

        let p = polar_to_cartesian(std::f32::consts::FRAC_PI_2, 0.0);
        assert!(distance(&p, &Vec3::new(0.0, 1.0, 0.0)) < 1e-6);

        let p = polar_to_cartesian(0.0, std::f32::consts::PI);
        assert!(distance(&p, &Vec3::new(-1.0, 0.0, 0.0)) < 1e-6);

        let p = polar_to_cartesian(std::f32::consts::PI * 3.0 / 2.0, 0.0);
        assert!(distance(&p, &Vec3::new(0.0, -1.0, 0.0)) < 1e-6);

        let p = polar_to_cartesian(0.0, std::f32::consts::FRAC_PI_2);
        assert!(distance(&p, &Vec3::new(0.0, 0.0, 1.0)) < 1e-6);

        let p = polar_to_cartesian(0.0, -std::f32::consts::FRAC_PI_2);
        assert!(distance(&p, &Vec3::new(0.0, 0.0, -1.0)) < 1e-6);

        let p = polar_to_cartesian(
            std::f32::consts::FRAC_PI_2 / 2f32,
            std::f32::consts::FRAC_PI_2 / 2f32,
        );
        assert!(distance(&p, &Vec3::new(0.5, 0.5, 0.70710677)) < 1e-6);
    }

    #[test]
    fn test_cartesian_to_polar() {
        let (alpha, beta) = cartesian_to_polar(&Vec3::new(1.0, 0.0, 0.0));
        assert!(
            (alpha - 0.0).abs() < 1e-6,
            "alpha should be 0, but is {}",
            alpha
        );
        assert!(
            (beta - 0.0).abs() < 1e-6,
            "beta should be 0, but is {}",
            beta
        );

        let (alpha, beta) = cartesian_to_polar(&Vec3::new(0.0, 1.0, 0.0));
        assert!(
            (alpha - std::f32::consts::FRAC_PI_2).abs() < 1e-6,
            "alpha should be pi/2, but is {}",
            alpha
        );
        assert!(
            (beta - 0.0).abs() < 1e-6,
            "beta should be 0, but is {}",
            beta
        );

        let (alpha, beta) = cartesian_to_polar(&Vec3::new(-1.0, 0.0, 0.0));
        assert!(
            (alpha - std::f32::consts::PI).abs() < 1e-6,
            "alpha should be pi, but is {}",
            alpha
        );
        assert!(
            (beta - 0.0).abs() < 1e-6,
            "beta should be 0, but is {}",
            beta
        );

        let (alpha, beta) = cartesian_to_polar(&Vec3::new(0.0, -1.0, 0.0));
        assert!(
            (alpha - std::f32::consts::PI * 3.0 / 2.0).abs() < 1e-6,
            "alpha should be 3pi/2, but is {}",
            alpha
        );
        assert!(
            (beta - 0.0).abs() < 1e-6,
            "beta should be 0, but is {}",
            beta
        );

        let (alpha, beta) = cartesian_to_polar(&Vec3::new(0.0, 0.0, 1.0));
        assert!(
            (alpha - 0.0).abs() < 1e-6,
            "alpha should be 0, but is {}",
            alpha
        );
        assert!(
            (beta - std::f32::consts::FRAC_PI_2).abs() < 1e-6,
            "beta should be pi/2, but is {}",
            beta
        );

        let (alpha, beta) = cartesian_to_polar(&Vec3::new(0.0, 0.0, -1.0));
        assert!(
            (alpha - 0.0).abs() < 1e-6,
            "alpha should be 0, but is {}",
            alpha
        );
        assert!(
            (beta + std::f32::consts::FRAC_PI_2).abs() < 1e-6,
            "beta should be -pi/2, but is {}",
            beta
        );

        let (alpha, beta) = cartesian_to_polar(&Vec3::new(0.5, 0.5, 0.70710677));
        assert!(
            (alpha - std::f32::consts::FRAC_PI_2 / 2f32).abs() < 1e-6,
            "alpha should be pi/4, but is {}",
            alpha
        );
        assert!(
            (beta - std::f32::consts::FRAC_PI_2 / 2f32).abs() < 1e-6,
            "beta should be pi/4, but is {}",
            beta
        );
    }

    #[test]
    fn test_cartesian_polar_transformation() {
        for alpha in 0..360 {
            let alpha = clamp(
                alpha as f32 * std::f32::consts::PI / 180.0,
                0f32,
                2f32 * std::f32::consts::PI,
            );

            for beta in -90..90 {
                let beta = clamp(
                    beta as f32 * std::f32::consts::PI / 180.0,
                    -std::f32::consts::FRAC_PI_2,
                    std::f32::consts::FRAC_PI_2,
                );

                let d = polar_to_cartesian(alpha, beta);
                let (a, b) = cartesian_to_polar(&d);
                assert!((0f32..=2f32 * std::f32::consts::PI).contains(&a));
                assert!((-std::f32::consts::FRAC_PI_2..=std::f32::consts::FRAC_PI_2).contains(&b));

                let d2 = polar_to_cartesian(a, b);

                assert!(
                    distance(&d, &d2) < 1e-6,
                    "alpha: {}, beta: {}, d: {:?}, d2: {:?}",
                    alpha,
                    beta,
                    d,
                    d2
                );
            }
        }
    }
}
