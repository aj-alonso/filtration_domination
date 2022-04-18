use crate::distance_matrix::DistanceMatrix;
use num::Float;

#[derive(Clone, Copy)]
pub enum DensityEstimator<T: Copy> {
    Ball(T),
    Gaussian(T),
}

impl<T: Float> DensityEstimator<T> {
    pub fn estimate(&self, dists: &DistanceMatrix<T>) -> Vec<T> {
        match self {
            Self::Ball(radius) => ball_density(dists, *radius),
            Self::Gaussian(radius) => gaussian_density(dists, *radius),
        }
    }
}

pub fn ball_density<T: Float>(dists: &DistanceMatrix<T>, radius: T) -> Vec<T> {
    let n = dists.len();
    let mut densities: Vec<usize> = vec![0; n];
    let mut total: usize = 0;
    for u in 0..n {
        for v in (u + 1)..n {
            if *dists.get(u, v) <= radius {
                densities[u] += 1;
                densities[v] += 1;
                total += 2;
            }
        }
    }
    let total_f: T = T::from(total).unwrap();
    densities
        .into_iter()
        .map(|x| T::from(x).unwrap() / total_f)
        .collect()
}

pub fn gaussian_density<T: Float>(dists: &DistanceMatrix<T>, radius: T) -> Vec<T> {
    if dists.is_empty() {
        return vec![];
    }
    let n = dists.len();
    let mut densities: Vec<T> = vec![T::zero(); n];
    let mut total: T = T::zero();
    let h = radius * radius * T::from(2.).unwrap();
    for u in 0..n {
        for v in (u + 1)..n {
            let dist = *dists.get(u, v);
            let incr = (-dist * dist / h).exp();
            densities[u] = densities[u] + incr;
            densities[v] = densities[v] + incr;
            total = total + incr * T::from(2.).unwrap();
        }
    }
    densities.into_iter().map(|x| x / total).collect()
}

#[cfg(test)]
mod tests {
    use crate::distance_matrix::density_estimation::{ball_density, gaussian_density};
    use crate::distance_matrix::DistanceMatrix;

    #[test]
    fn ball_density_happy_case() {
        let mut dists = DistanceMatrix::new(3);
        dists.set(0, 1, 0.4);
        dists.set(0, 2, 0.2);
        dists.set(1, 2, 0.2);
        assert_eq!(ball_density(&dists, 0.2), [0.25, 0.25, 0.5]);
    }

    #[test]
    fn gaussian_density_happy_case() {
        let mut dists = DistanceMatrix::new(3);
        dists.set(0, 1, 0.4);
        dists.set(0, 2, 0.2);
        dists.set(1, 2, 0.2);
        assert_eq!(
            gaussian_density(&dists, 0.2),
            [0.2750918911708629, 0.2750918911708629, 0.4498162176582741]
        );
    }
}
