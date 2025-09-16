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

        // make cursor invisible
        eprint!("\x1b[?25l");

        for i in 0..iterations {
            // TODO: implement static logger functionality for progress
            //       once implemented, replace other eprint(ln)! calls too
            eprintln!(
                "processing k-means iteration: [ {:>9} / {:>9} ]...",
                i + 1,
                iterations
            );

            // precompute cluster distances to skip some distance calculations later
            // only set for i < j -- note: dist[i][j] == dist[j][i]
            let mut cluster_distances = vec![vec![0.0; k]; k];
            for i in 0..k {
                for j in (i + 1)..k {
                    let dist = clusters[i].distance(&clusters[j]);
                    cluster_distances[i][j] = dist; // i < j case only
                }
            }

            // assign each point to the nearest cluster
            for (i, point) in data.iter().enumerate() {
                let mut closest_idx = 0;
                let mut closest_dist = clusters[0].distance(point);

                // print progress every 500 points
                if i % 500 == 0 {
                    if i > 0 {
                        // restore cursor position (write over previous status)
                        eprint!("\x1b[1F");
                    }
                    let label_len = "processing k-means iteration".len();
                    eprintln!(
                        "{:>label_len$}: [ {:>9} / {:>9} ]...",
                        "assigning point",
                        i,
                        data.len()
                    );
                }

                for j in 1..k {
                    // skip distance calculation if the cluster is too far away
                    let (a, b) = (closest_idx.min(j), closest_idx.max(j));
                    if cluster_distances[a][b] >= 2.0 * closest_dist {
                        // d(c_j, c_min) >= 2 * d(p, c_min)
                        // d(p,   c_j  ) >=     d(p, c_min)
                        continue;
                    }

                    let dist = clusters[j].distance(point);
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_idx = j;
                    }
                }

                assignments[i] = closest_idx;
            }

            // restore cursor position (write over previous status)
            eprint!("\x1b[2F");

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

        // make cursor visible again
        eprint!("\x1b[?25h");

        (clusters, assignments)
    }

    /// create a new context with a seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
        }
    }
}
