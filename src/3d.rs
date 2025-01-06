#![feature(array_windows)]
#![feature(iter_intersperse)]

use cgmath::Vector2;
use gxhash::{HashMap, HashMapExt};
use mid_brownie_testing::compute_midpoint;
use plotters::backend::{BitMapBackend, SVGBackend};
use plotters::chart::{ChartBuilder, MeshStyle};
use plotters::drawing::IntoDrawingArea;
use plotters::series::SurfaceSeries;
use plotters::style::{BLUE, Color, ShapeStyle, WHITE};
use std::error::Error;
use std::num::NonZero;
use std::{env, iter};

fn produce_midpoints(
    values: &mut HashMap<Vector2<u64>, f64>,
    midpoint: u64,
    noise: u64,
    seed: i64,
    xzs: impl Iterator<Item = (u64, u64)>,
) {
    let next_values = xzs
        .flat_map(|(x, z)| {
            let values = &values; // allows the later move ||
            let point = Vector2::new(x, z);
            let base = values[&point];
            iter::once((point, base)).chain(
                [(midpoint, 0), (0, midpoint), (midpoint, midpoint)]
                    .into_iter()
                    .map(move |(midpoint_x, midpoint_z)| {
                        let center = Vector2::new(x + midpoint_x, z + midpoint_z);
                        let dest = Vector2::new(
                            x.overflowing_add(midpoint_x.overflowing_shl(1).0).0,
                            z.overflowing_add(midpoint_z.overflowing_shl(1).0).0,
                        );
                        let target = values[&dest];
                        (
                            center,
                            compute_midpoint(point, dest, base, target, noise, seed),
                        )
                    }),
            )
        })
        .collect();
    *values = next_values;
}

fn main() -> Result<(), Box<dyn Error>> {
    let seed = env::args().skip(1).next().map_or(0, |s| s.parse().unwrap());
    const NOISE: u64 = 10000u64;
    const ITERATIONS: usize = 10;
    let mut noise = NOISE;
    let decay = 0.5f64;

    let max = noise as f64 / (1f64 - decay);

    let mut values = HashMap::new();
    values.insert(Vector2::new(0u64, 0u64), max / 2.0);

    let mut midpoint = 1u64.reverse_bits();

    for i in 1usize..=ITERATIONS {
        let filename = format!("out-{i}.bmp");
        let area = BitMapBackend::new(&filename, (1920, 1080)).into_drawing_area();
        area.fill(&WHITE)?;

        let mut chart =
            ChartBuilder::on(&area).build_cartesian_3d(0..u64::MAX, 0f64..max, 0..u64::MAX)?;

        let (nextpoint, _) = midpoint.overflowing_shl(1);
        if nextpoint != 0 {
            produce_midpoints(
                &mut values,
                midpoint,
                noise,
                seed,
                iter::successors(Some(0), |s: &u64| s.checked_add(nextpoint)).flat_map(|x| {
                    iter::repeat(x).zip(iter::successors(Some(0), |s: &u64| {
                        s.checked_add(nextpoint)
                    }))
                }),
            )
        } else {
            produce_midpoints(&mut values, midpoint, noise, seed, iter::once((0, 0)));
        };

        let series = SurfaceSeries::xoz(
            iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
            iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
            |x, z| values.get(&Vector2::new(x, z)).copied().unwrap(),
        )
        .style(ShapeStyle::from(BLUE.mix(0.5)).stroke_width(0));

        chart.draw_series(series)?;

        area.present()?;

        midpoint >>= 1;
        let next_noise = (noise as f64 * decay) as u64;
        if next_noise == 0 {
            break;
        }
        noise = next_noise;
    }

    Ok(())
}
