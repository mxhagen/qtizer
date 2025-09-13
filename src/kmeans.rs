use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};

/// trait for types that can be clustered using k-means
pub trait Kmeansable {
    /// output type for summation during mean calculation
    type Sum: Clone + std::fmt::Debug;

    /// initial value for sum
    fn zero() -> Self::Sum;

    /// distance function, according to which clustering is performed
    fn distance(&self, other: &Self) -> f64;

    /// summation for mean calculation
    fn add(sum: &Self::Sum, other: &Self) -> Self::Sum;

    /// division for mean calculation
    fn div(sum: &Self::Sum, count: usize) -> Self;
}

// TODO: look for speedups before parallelizing
//       - k-d tree for nearest neighbor search?
//       - triangle inequality to skip distance calculations?

/// context for k-means clustering, containing an rng to initialize clusters
pub struct Context<R = SmallRng>
where
    R: rand::Rng,
{
    rng: R,
}

impl Context<SmallRng> {
    /// k-means clustering for pixel data
    ///
    /// returns (clusters, assignments), such that for any given `x = assignments[i]`, `data[i]` belongs to `clusters[x]`
    pub fn k_means<T>(&mut self, data: &[T], k: usize, iterations: usize) -> (Vec<T>, Vec<usize>)
    where
        T: Kmeansable + Clone,
    {
        let mut assignments: Vec<usize> = vec![0; data.len()];
        let mut clusters = data
            .choose_multiple(&mut self.rng, k)
            .cloned()
            .collect::<Vec<_>>();

        for i in 0..iterations {
            // TODO: implement static logger functionality for progress
            eprintln!("processing k-means iteration {}/{}...", i + 1, iterations);

            // assign each point to the nearest cluster
            for (i, point) in data.iter().enumerate() {
                let min_idx = clusters
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| f64::total_cmp(&a.distance(point), &b.distance(point)))
                    .unwrap()
                    .0;

                assignments[i] = min_idx;
            }

            // move cluster to mean of its assigned points
            let mut counts: Vec<usize> = vec![0; k];
            let mut sums = vec![T::zero(); k];

            for (i, point) in data.iter().enumerate() {
                let cluster_idx = assignments[i];
                counts[cluster_idx] += 1;
                sums[cluster_idx] = T::add(&sums[cluster_idx], point);
            }

            for i in 0..k {
                if counts[i] != 0 {
                    clusters[i] = T::div(&sums[i].clone(), counts[i]);
                }
            }
        }

        (clusters, assignments)
    }

    /// create a new context with a seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
        }
    }
}
