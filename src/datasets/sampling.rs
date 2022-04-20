use num::Float;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::Uniform;
use rand::Rng;
use std::f64::consts::PI;

use crate::points::{Point, PointCloud};

/// Sample n points from `\[0,1\]^DIM` uniformly.
pub fn sample_random_points<T: Float + SampleUniform, const DIM: usize>(
    n: usize,
) -> PointCloud<T, DIM> {
    let point_distribution = Uniform::new(T::zero(), T::one());
    let mut rng = rand::thread_rng();
    let mut point_cloud: PointCloud<T, DIM> = PointCloud::new();
    for _i in 0..n {
        let mut point_coordinates = [T::zero(); DIM];
        for coord in point_coordinates.iter_mut() {
            *coord = rng.sample(&point_distribution);
        }

        point_cloud.push_point(Point(point_coordinates));
    }

    point_cloud
}

/// Sample points from a torus in `R^3`.
pub fn sample_torus(n: usize) -> PointCloud<f64, 3> {
    let radius = 0.5;
    let center_distance = 2.;
    let mut rng = rand::thread_rng();
    let mut point_cloud = PointCloud::new();
    for _i in 0..n {
        let theta = rng.gen_range(0.0..1.0) * 2. * PI;
        let phi = rng.gen_range(0.0..1.0) * 2. * PI;
        let x = (center_distance + radius * theta.cos()) * phi.cos();
        let y = (center_distance + radius * theta.cos()) * phi.sin();
        let z = radius * theta.sin();
        point_cloud.push_point(Point([x, y, z]));
    }

    point_cloud
}

/// A plane rolled up into a spiral in R^3.
/// Equations are the same as in <https://jlmelville.github.io/smallvis/swisssne.html>.
pub fn sample_swiss_roll(n: usize) -> PointCloud<f64, 3> {
    let mut rng = rand::thread_rng();
    let mut point_cloud = PointCloud::new();
    for _i in 0..n {
        let phi = rng.gen_range(1.5..4.5) * PI;
        let psi = rng.gen_range(0.0..10.0);

        let x = phi * phi.cos();
        let y = phi * phi.sin();
        let z = psi;
        point_cloud.push_point(Point([x, y, z]))
    }

    point_cloud
}

/// Draws n points from the unit sphere in R^DIM, and adds outliers from [-2, 2]^DIM.
/// It can sample less points from a disc around the north pole.
///
/// The proportion of sampled points from the sphere is given in sample_weight.
/// Also, the proportion of sampled points from the disc of radius north_pole_radius is given in north_pole_weight.
pub fn sample_noisy_sphere<T: Float + SampleUniform, const DIM: usize>(
    n: usize,
    sample_weight: f32,
    north_pole_radius: T,
    north_pole_weight: f32,
) -> PointCloud<T, DIM> {
    let mut north_pole = Point([T::zero(); DIM]);
    north_pole.0[DIM - 1] = T::one();

    let mut rng = rand::thread_rng();
    let mut cloud = PointCloud(Vec::new());

    let mut samples: usize = 0;
    for _i in 0..n {
        let sample_coin: f32 = rng.gen_range(0.0..1.0);
        if sample_coin < sample_weight {
            samples += 1;
        }
    }
    add_outliers(n - samples, T::from(2).unwrap(), &mut cloud, &mut rng);

    let uni_dist = Uniform::new(-T::one(), T::one());
    while cloud.len() < n {
        let mut point = Point::random(&uni_dist, &mut rng);
        let norm = point.norm();

        if norm < T::one() && norm != T::zero() {
            point.normalize();

            if (point - north_pole).norm() < north_pole_radius {
                let coin: f32 = rng.gen_range(0.0..1.0);
                if coin < north_pole_weight {
                    cloud.0.push(point);
                }
            } else {
                cloud.0.push(point);
            }
        }
    }

    cloud
}

fn add_outliers<T: Float + SampleUniform, R: Rng, const DIM: usize>(
    n: usize,
    limit: T,
    cloud: &mut PointCloud<T, DIM>,
    rng: &mut R,
) {
    let uni_dist = Uniform::new(-limit * T::one(), limit * T::one());
    for _i in 0..n {
        let point = Point::random(&uni_dist, rng);
        cloud.0.push(point);
    }
}
