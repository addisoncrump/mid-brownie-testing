use gxhash::GxHasher;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div};

pub fn compute_midpoint<T>(i1: T, i2: T, v1: f64, v2: f64, noise: u64, seed: i64) -> f64
where
    T: Add + Div<u64> + Hash,
{
    let mut hasher = GxHasher::with_seed(seed);
    i1.hash(&mut hasher);
    i2.hash(&mut hasher);
    let sampled = (hasher.finish() % noise) as i64 - (noise >> 1) as i64;
    (v1 + v2) / 2f64 + sampled as f64
}
