#![cfg_attr(nightly, feature(generic_const_exprs))]

use cgmath::{AbsDiffEq, InnerSpace, Point3, Vector3};
use std::collections::HashMap as StdHashMap;
use std::error::Error;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::iter;
use std::ops::{Add, Div, IndexMut, Mul, RangeInclusive};

type HashMap<T, U> = StdHashMap<T, U, BuildHasherDefault<HighwayHasher>>;

use highway::HighwayHasher;

const EPSILON: f64 = 0.00001;

#[derive(Debug, Clone)]
pub struct FractalNoise<const N: usize> {
    values: HashMap<[u32; N], f64>,
    noise: [f64; 32],
    bounds: [f64; 32],
    decay: f64,
    seed: i64,
    iterations: usize,
}

impl<const N: usize> FractalNoise<N> {
    pub fn new(noise: f64, decay: f64, seed: i64) -> Self {
        let values = HashMap::with_hasher(BuildHasherDefault::<HighwayHasher>::default());
        let noise: [f64; 32] = iter::successors(Some(noise), |&prev| Some(prev * decay))
            .take(32)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut bounds = noise;
        for i in (0..(bounds.len() - 1)).rev() {
            bounds[i] += bounds[i + 1];
        }
        let mut result = Self {
            values,
            noise,
            bounds,
            decay,
            seed,
            iterations: 0,
        };
        let initial = result.upper_bound(0) / 2.0;
        result.values.insert([0u32; N], initial);
        result
    }

    fn next_points(
        &self,
        start: [u32; N],
        noise: f64,
    ) -> impl Iterator<Item = ([u32; N], f64)> + use<'_, N> {
        let midpoint = 1u32.reverse_bits() >> self.iterations;
        let base = self.values[&start];
        iter::once((start, base)).chain((1..(1 << N)).map(move |combo| {
            let mut target = start;
            target
                .iter_mut()
                .zip(0..N)
                .for_each(|(n, o)| *n = n.add(midpoint * (combo >> o & 1)));
            let mut s = target;
            s.iter_mut()
                .zip(0..N)
                .for_each(|(n, o)| *n = n.overflowing_add(midpoint * (combo >> o & 1)).0);
            let f_val = self.values[&start];
            let s_val = self.values[&s];

            let computed = compute_midpoint(start, s, f_val, s_val, noise, self.seed);
            (target, computed)
        }))
    }

    pub fn step_midpoints(&mut self) -> Result<bool, Box<dyn Error>> {
        let noise = self.noise(self.iterations);
        if noise.abs_diff_eq(&0.0, EPSILON) {
            return Ok(false);
        }

        let next_values = self
            .values
            .keys()
            .copied()
            .flat_map(|start| self.next_points(start, noise))
            .collect();
        self.values = next_values;

        self.iterations += 1;
        Ok(true)
    }

    // pub fn noise(&mut self, iterations: usize) -> f64 {
    //     loop {
    //         match self.noise.get(iterations) {
    //             Some(noise) => return *noise,
    //             None => {
    //                 let prev = self.noise.last().copied().unwrap();
    //                 if prev.abs_diff_eq(&0.0, EPSILON) {
    //                     return 0.0;
    //                 }
    //                 self.noise.push(prev * self.decay);
    //             }
    //         }
    //     }
    // }

    pub fn noise(&mut self, iterations: usize) -> f64 {
        self.noise[iterations]
    }

    pub fn decay(&self) -> f64 {
        self.decay
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }

    pub fn values(&self) -> &HashMap<[u32; N], f64> {
        &self.values
    }

    pub fn into_values(self) -> HashMap<[u32; N], f64> {
        self.values
    }

    pub fn iterations(&self) -> usize {
        self.iterations
    }

    // pub fn upper_bound(&mut self, iterations: usize) -> f64 {
    //     // the bound tightness here could theoretically be improved with knowing the dimensions
    //     // however, I am not smart enough to work out a proper bound on dimensions > 2
    //     // for this reason, I just fallback to the upper bound
    //     let noise = self.noise(iterations) as f64;
    //     if N == 1 {
    //         (noise / (1.0 - self.decay) + noise) / 3.0
    //     } else {
    //         noise / (1.0 - self.decay)
    //     }
    // }

    pub fn upper_bound(&mut self, iterations: usize) -> f64 {
        self.bounds[iterations]
    }

    #[cfg(not(nightly))]
    pub fn cached_bounds_for(
        &mut self,
        point: [u32; N],
        height: f64,
    ) -> (bool, RangeInclusive<f64>, [u32; N], u32) {
        self.cached_bounds_for_inner::<Vec<([u32; N], f64)>>(point, height)
    }

    #[cfg(nightly)]
    pub fn cached_bounds_for(
        &mut self,
        point: [u32; N],
        height: f64,
    ) -> (bool, RangeInclusive<f64>, [u32; N], u32)
    where
        [(); 1 << N]:,
    {
        self.cached_bounds_for_inner::<[([u32; N], f64); 1 << N]>(point, height)
    }

    fn lookup_or_compute(&mut self, midpoint: u32, target: [u32; N], noise: f64) -> f64 {
        if let Some(&existing) = self.values.get(&target) {
            existing
        } else {
            let nextpoint = midpoint << 1;
            let f = target.map(|v| v.div(&midpoint).div(&2).mul(nextpoint));
            let f_val = self.values[&f];
            let mut s = f;
            s.iter_mut()
                .zip(f)
                .zip(target)
                .for_each(|((second, first), target)| {
                    *second = second
                        .overflowing_add((target - first).overflowing_shl(1).0)
                        .0;
                });
            let s_val = self.values[&s];

            let computed = compute_midpoint(f, s, f_val, s_val, noise, self.seed);
            self.values.insert(target, computed);
            computed
        }
    }

    fn cached_bounds_for_inner<PA: ValidPointsArray<([u32; N], f64), N>>(
        &mut self,
        point: [u32; N],
        height: f64,
    ) -> (bool, RangeInclusive<f64>, [u32; N], u32) {
        let mut last_bound = f64::NEG_INFINITY..=f64::INFINITY;

        let mut midpoint = 1u32.reverse_bits();
        let mut iterations = 0;
        let mut points = PA::init(iter::once(([0u32; N], self.values[&[0u32; N]])));

        while 0 < midpoint {
            let noise = self.noise(iterations);
            if noise.abs_diff_eq(&0.0, EPSILON) {
                return (true, last_bound, points[0].0, midpoint << 1);
            }

            let (minpoint, maxpoint) = points
                .as_ref()
                .iter()
                .map(|(_, v)| *v)
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(fmin, fmax), f| {
                    (fmin.min(f), fmax.max(f))
                });

            let bound = self.upper_bound(iterations);
            last_bound = (minpoint - bound)..=(maxpoint + bound);

            if !last_bound.contains(&height) {
                return (false, last_bound, points[0].0, midpoint << 1);
            }

            // compute the next starting point
            let mut next = point;
            next.iter_mut()
                .zip(points[0].0)
                .for_each(|(n, p)| *n = (*n - p).div(&midpoint).mul(&midpoint).add(p));
            points = PA::init((0..(1 << N)).map(|combo| {
                let mut other = next;
                other
                    .iter_mut()
                    .zip(0..N)
                    .for_each(|(n, o)| *n = n.overflowing_add(midpoint * (combo >> o & 1)).0);
                (other, self.lookup_or_compute(midpoint, other, noise))
            }));
            self.values.extend(points.as_ref().iter().copied());

            midpoint >>= 1;
            iterations += 1;
        }
        (true, last_bound, points[0].0, 1)
    }

    #[cfg(not(nightly))]
    pub fn find_point(&mut self, n: [u32; N]) -> f64 {
        self.find_point_inner::<Vec<([u32; N], f64)>>(n)
    }

    #[cfg(nightly)]
    pub fn find_point(&mut self, n: [u32; N]) -> f64
    where
        [(); 1 << N]:,
    {
        find_point_inner::<[([u32; N], f64); 1 << N]>(n)
    }

    fn find_point_inner<PA: ValidPointsArray<([u32; N], f64), N>>(&mut self, n: [u32; N]) -> f64 {
        // fast-track: maybe we have this computed
        if let Some(&v) = self.values.get(&n) {
            return v;
        }

        let mut midpoint = 1u32.reverse_bits();
        let mut base = ([0u32; N], *self.values.get(&[0u32; N]).unwrap());
        for iterations in 0.. {
            if base.0 == n {
                return base.1;
            }

            // compute the next starting point
            let mut next = n;
            next.iter_mut()
                .zip(base.0)
                .for_each(|(n, p)| *n = (*n - p).div(&midpoint).mul(&midpoint).add(p));
            let noise = self.noise(iterations);
            let points = PA::init((0..(1 << N)).map(|combo| {
                let mut other = next;
                other
                    .iter_mut()
                    .zip(0..N)
                    .for_each(|(n, o)| *n = n.overflowing_add(midpoint * (combo >> o & 1)).0);
                (other, self.lookup_or_compute(midpoint, other, noise))
            }));
            base = points[0];

            midpoint >>= 1;
        }

        // last chance
        if let Some(&v) = self.values.get(&n) {
            return v;
        }
        unreachable!("We must have computed n by now");
    }
}

pub fn compute_midpoint<const N: usize>(
    i1: [u32; N],
    i2: [u32; N],
    v1: f64,
    v2: f64,
    noise: f64,
    seed: i64,
) -> f64 {
    let mut hasher = HighwayHasher::default();
    hasher.write_i64(seed);
    i1.hash(&mut hasher);
    i2.hash(&mut hasher);
    let sampled = (hasher.finish() as u32 as f64 % noise) - noise * 0.5;

    (v1 + v2) * 0.5 + sampled
}

trait ValidPointsArray<T, const N: usize>: IndexMut<usize, Output = T> + AsRef<[T]> {
    fn init(source: impl Iterator<Item = T>) -> Self;
}

impl<T, const N: usize> ValidPointsArray<T, N> for Vec<T> {
    fn init(source: impl Iterator<Item = T>) -> Self {
        source.collect()
    }
}

#[cfg(nightly)]
impl<T, const N: usize> ValidPointsArray<T, N> for [T; 1 << N]
where
    [T; 1 << N]:,
    T: Copy,
{
    fn init(mut source: impl Iterator<Item = T>) -> Self {
        let next = source.next().unwrap();
        let mut result = [next; 1 << N];
        result[1..].iter_mut().zip(source).for_each(|(r, s)| *r = s);
        result
    }
}

#[derive(Clone, Copy)]
struct RectangularPrism {
    lower: Point3<f64>,
    upper: Point3<f64>,
}

impl RectangularPrism {
    fn around(base: [u32; 2], cache: &mut FractalNoise<2>, nextpoint: u32) -> Self {
        Self::new(
            [
                [0, 0],
                [0, nextpoint],
                [nextpoint, 0],
                [nextpoint, nextpoint],
            ]
            .into_iter()
            .map(|[offset_x, offset_z]| {
                let [x, z] = [
                    base[0].overflowing_add(offset_x).0,
                    base[1].overflowing_add(offset_z).0,
                ];
                let y = cache.find_point([x, z]);
                Point3::new(x as f64, y, z as f64)
            }),
        )
    }

    fn new(mut source: impl Iterator<Item = Point3<f64>>) -> Self {
        let mut lower = source.next().unwrap();
        let mut upper = lower;
        for next in source {
            lower = lower.zip(next, |p1, p2| p1.min(p2));
            upper = upper.zip(next, |p1, p2| p1.max(p2));
        }
        Self { lower, upper }
    }

    fn intersect(&self, ray: &Ray) -> Option<(f64, f64)> {
        let t1 = (self.lower.x - ray.origin.x) * ray.inv_dir.x;
        let t2 = (self.upper.x - ray.origin.x) * ray.inv_dir.x;
        let t3 = (self.lower.y - ray.origin.y) * ray.inv_dir.y;
        let t4 = (self.upper.y - ray.origin.y) * ray.inv_dir.y;
        let t5 = (self.lower.z - ray.origin.z) * ray.inv_dir.z;
        let t6 = (self.upper.z - ray.origin.z) * ray.inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax >= 0.0 {
            if tmin <= tmax {
                return Some((tmin, tmax));
            }
        }
        None
    }

    fn contains(&self, point: Point3<f64>) -> bool {
        self.lower.x < point.x
            && self.lower.y < point.y
            && self.lower.z < point.z
            && point.x < self.upper.x
            && point.z < self.upper.z
    }
}

#[derive(Copy, Clone)]
pub struct Ray {
    direction: Vector3<f64>,
    inv_dir: Vector3<f64>,
    origin: Point3<f64>,
}

impl Ray {
    pub fn new(direction: Vector3<f64>, origin: Point3<f64>) -> Self {
        let normalised = direction.normalize();
        Self {
            direction: normalised,
            inv_dir: normalised.map(|f| 1.0 / f),
            origin,
        }
    }

    fn intersection_candidates<'a, T: Iterator<Item = (usize, Option<[u32; 2]>)>>(
        &self,
        noise: &'a mut FractalNoise<2>,
        nextpoint: u32,
        options: T,
    ) -> impl Iterator<Item = (usize, RectangularPrism, f64)> + use<'_, 'a, T> {
        options
            .filter_map(move |(i, c)| c.map(|c| (i, RectangularPrism::around(c, noise, nextpoint))))
            .filter_map(move |(i, prism)| {
                let mut unbounded = prism;
                unbounded.lower.y = f64::MIN;
                unbounded.upper.y = f64::MAX;
                unbounded.intersect(&self).map(|(t, _)| (i, prism, t))
            })
    }

    pub fn intersect(&self, noise: &mut FractalNoise<2>, max: f64) -> Option<Point3<f64>> {
        let global_bounds = RectangularPrism::new(
            [
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(
                    u32::MAX as f64 + 1.0,
                    noise.upper_bound(0),
                    u32::MAX as f64 + 1.0,
                ),
            ]
            .into_iter(),
        );
        let Some((entry, _)) = global_bounds.intersect(&self) else {
            return None;
        };

        let mut direction = [None, None];
        let mut intersection = if entry < 0.0 { 0.0 } else { entry };
        while intersection < max {
            let marched = self.origin + self.direction * intersection;
            let query = [marched.x as u32, marched.z as u32];
            let (terminated, range, base, nextpoint) = noise.cached_bounds_for(query, marched.y);
            if terminated {
                if let Some((intersection, _)) =
                    RectangularPrism::around(base, noise, nextpoint).intersect(self)
                {
                    // println!("found intersection at: {actual}!");
                    return Some(self.origin + self.direction * intersection);
                }
            }

            let mut prism = RectangularPrism::around(base, noise, nextpoint);
            prism.lower.y = *range.start() + EPSILON;
            prism.upper.y = *range.end() - EPSILON;
            let base_intersection = prism
                .intersect(&self)
                .map(|(t, _)| t)
                .filter(|t| *t > intersection && t.is_normal());
            let Some(actual) = ({
                let options = [
                    base[0].checked_sub(nextpoint).map(|x| [x, base[1]]),
                    base[0].checked_add(nextpoint).map(|x| [x, base[1]]),
                    base[1].checked_sub(nextpoint).map(|z| [base[0], z]),
                    base[1].checked_add(nextpoint).map(|z| [base[0], z]),
                ];

                match direction {
                    [None, None] => self
                        .intersection_candidates(noise, nextpoint, options.into_iter().enumerate())
                        .filter(|(_, _, t)| *t > intersection && t.is_normal())
                        .inspect(|(i, _, _)| {
                            if let Some(direction) = direction.iter_mut().find(|o| o.is_none()) {
                                *direction = Some(*i)
                            }
                        })
                        .map(|(_, _, t)| t)
                        .chain(base_intersection)
                        .min_by(|t1, t2| t1.total_cmp(t2)),
                    [Some(first), None] => self
                        .intersection_candidates(
                            noise,
                            nextpoint,
                            [
                                first.overflowing_sub(1).0 % 4,
                                first,
                                first.overflowing_add(1).0 % 4,
                            ]
                            .into_iter()
                            .map(|i| (i, options[i])),
                        )
                        .filter(|(_, _, t)| *t > intersection && t.is_normal())
                        .inspect(|(i, _, _)| {
                            if *i != first {
                                if let Some(direction) = direction.iter_mut().find(|o| o.is_none())
                                {
                                    *direction = Some(*i)
                                }
                            }
                        })
                        .map(|(_, _, t)| t)
                        .chain(base_intersection)
                        .min_by(|t1, t2| t1.total_cmp(t2)),
                    [Some(first), Some(second)] => self
                        .intersection_candidates(
                            noise,
                            nextpoint,
                            [first, second].into_iter().map(|i| (i, options[i])),
                        )
                        .map(|(_, _, t)| t)
                        .filter(|t| *t > intersection && t.is_normal())
                        .chain(base_intersection)
                        .min_by(|t1, t2| t1.total_cmp(t2)),
                    _ => unreachable!("This is not possible by construction."),
                }
            }) else {
                return None;
            };

            intersection = actual;
        }
        None
    }
}
