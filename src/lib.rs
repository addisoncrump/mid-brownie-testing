use std::collections::HashMap;
use std::error::Error;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::iter;
use std::ops::{Add, Div, Mul};

use highway::HighwayHasher;

pub struct FractalNoise<const N: usize> {
    values: HashMap<[u64; N], f64, BuildHasherDefault<HighwayHasher>>,
    midpoint: u64,
    noise: u64,
    decay: f64,
    seed: i64,
    iterations: usize,
}

impl<const N: usize> FractalNoise<N> {
    pub fn new(initial: f64, noise: u64, decay: f64, seed: i64) -> Self {
        let mut values = HashMap::with_hasher(BuildHasherDefault::<HighwayHasher>::default());
        values.insert([0u64; N], initial);
        Self {
            values,
            midpoint: 1u64.reverse_bits(),
            noise,
            decay,
            seed,
            iterations: 0,
        }
    }

    fn next_points(&self, start: [u64; N]) -> impl Iterator<Item = ([u64; N], f64)> + use<'_, N> {
        let base = self.values[&start];
        iter::once((start, base)).chain((1..(1 << N)).map(move |combo| {
            let mut target = start;
            target
                .iter_mut()
                .zip(0..N)
                .for_each(|(n, o)| *n = n.add(self.midpoint * (combo >> o & 1)));
            let mut s = target;
            s.iter_mut()
                .zip(0..N)
                .for_each(|(n, o)| *n = n.overflowing_add(self.midpoint * (combo >> o & 1)).0);
            let f_val = self.values[&start];
            let s_val = self.values[&s];

            let computed = compute_midpoint(start, s, f_val, s_val, self.noise, self.seed);
            (target, computed)
        }))
    }

    pub fn step_midpoints(&mut self) -> Result<bool, Box<dyn Error>> {
        if self.noise == 0 {
            return Ok(false);
        }

        let next_values = self
            .values
            .keys()
            .copied()
            .flat_map(|start| self.next_points(start))
            .collect();
        self.values = next_values;

        self.midpoint >>= 1;
        self.noise = (self.noise as f64 * self.decay) as u64;
        self.iterations += 1;
        Ok(true)
    }

    pub fn values(&self) -> &HashMap<[u64; N], f64, BuildHasherDefault<HighwayHasher>> {
        &self.values
    }

    pub fn into_values(self) -> HashMap<[u64; N], f64, BuildHasherDefault<HighwayHasher>> {
        self.values
    }

    pub fn midpoint(&self) -> u64 {
        self.midpoint
    }

    pub fn iterations(&self) -> usize {
        self.iterations
    }
}

pub fn compute_midpoint<const N: usize>(
    i1: [u64; N],
    i2: [u64; N],
    v1: f64,
    v2: f64,
    noise: u64,
    seed: i64,
) -> f64 {
    let mut hasher = HighwayHasher::default();
    hasher.write_i64(seed);
    i1.hash(&mut hasher);
    i2.hash(&mut hasher);
    let sampled = (hasher.finish() % noise) as i64 - (noise >> 1) as i64;
    let computed = (v1 + v2) / 2f64 + sampled as f64;

    computed
}

pub fn find_point<const N: usize>(n: [u64; N], mut noise: u64, decay: f64, seed: i64) -> f64 {
    let max = noise as f64 / (1f64 - decay);

    let mut points = vec![([0u64; N], max / 2.0)];
    let lookup_or_compute = |points: &[([u64; N], f64)], midpoint: u64, target: [u64; N], noise| {
        points
            .iter()
            .copied()
            .find(|(p, _)| *p == target)
            .unwrap_or_else(|| {
                let nextpoint = midpoint << 1;
                let f = target.map(|v| v.div(&midpoint).div(&2).mul(nextpoint));
                let f_val = points.iter().copied().find(|(p, _)| *p == f).unwrap().1;
                let mut s = f;
                s.iter_mut()
                    .zip(f)
                    .zip(target)
                    .for_each(|((second, first), target)| {
                        *second = second
                            .overflowing_add((target - first).overflowing_shl(1).0)
                            .0;
                    });
                let s_val = points.iter().copied().find(|(p, _)| *p == s).unwrap().1;

                let computed = compute_midpoint(f, s, f_val, s_val, noise, seed);
                (target, computed)
            })
    };

    let mut midpoint = 1u64.reverse_bits();
    for _ in 1.. {
        if let Some((_, v)) = points.iter().find(|(p, _)| *p == n) {
            return *v;
        }

        // compute the next starting point
        let mut next = n;
        next.iter_mut()
            .zip(points[0].0)
            .for_each(|(n, p)| *n = (*n - p).div(&midpoint).mul(&midpoint).add(p));
        points = (0..(1 << N))
            .map(|combo| {
                let mut other = next;
                other
                    .iter_mut()
                    .zip(0..N)
                    .for_each(|(n, o)| *n = n.overflowing_add(midpoint * (combo >> o & 1)).0);
                lookup_or_compute(&points, midpoint, other, noise)
            })
            .collect::<Vec<_>>();

        midpoint >>= 1;
        let next_noise = (noise as f64 * decay) as u64;
        if next_noise == 0 {
            break;
        }
        noise = next_noise;
    }

    // last chance
    if let Some((_, v)) = points.iter().find(|(p, _)| *p == n) {
        return *v;
    }
    unreachable!("We must have computed n by now");
}
