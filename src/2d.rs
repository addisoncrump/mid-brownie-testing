#![feature(array_windows)]
#![feature(iter_intersperse)]

use gxhash::gxhash64;
use mid_brownie_testing::compute_midpoint;
use plotters::backend::{BitMapBackend, SVGBackend};
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::{FontDesc, FontFamily, FontStyle};
use plotters::series::LineSeries;
use plotters::style::{BLACK, Color, WHITE};
use std::env;
use std::error::Error;

fn find_point(n: u64, mut noise: u64, decay: f64, seed: i64, iterations: usize) -> f64 {
    let max = noise as f64 / (1f64 - decay);

    let mut min = max / 2.0;
    let mut max = max / 2.0;

    let mut i1 = 0u64;
    let mut i2 = u64::MAX;

    if n == i1 {
        return min;
    } else if n == i2 {
        return max;
    }

    let mut midpoint = 1u64.reverse_bits();
    for _ in 1..=iterations {
        let next = compute_midpoint(i1, i2, min, max, noise, seed);
        let next_idx = i1 + midpoint;
        if n == next_idx {
            return next; // we're done
        } else if n & midpoint == 0 {
            max = next;
            i2 = next_idx;
        } else {
            min = next;
            i1 = next_idx;
        }

        midpoint >>= 1;
        let next_noise = (noise as f64 * decay) as u64;
        if next_noise == 0 {
            break;
        }
        noise = next_noise;
    }

    unreachable!("We must have computed n by now");
}

fn main() -> Result<(), Box<dyn Error>> {
    let seed = env::args().skip(1).next().map_or(0, |s| s.parse().unwrap());
    const NOISE: u64 = 10000u64;
    const ITERATIONS: usize = 16;
    let mut noise = NOISE;
    let decay = 0.5f64;

    let max = noise as f64 / (1f64 - decay);

    let mut sequence = Vec::new();

    sequence.push((0u64, max / 2.0));
    sequence.push((u64::MAX, max / 2.0));

    let mut midpoint = 1u64.reverse_bits();

    let area = BitMapBackend::gif("2d.gif", (1080, 1080), 1_000)?.into_drawing_area();
    for i in 1usize..=ITERATIONS {
        area.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&area).build_cartesian_2d(0..u64::MAX, 0f64..max)?;

        let mut midpoints = sequence.array_windows().map(|&[(i1, v1), (i2, v2)]| {
            (i1 + midpoint, compute_midpoint(i1, i2, v1, v2, noise, seed))
        });

        sequence = sequence
            .iter()
            .copied()
            .intersperse_with(|| midpoints.next().unwrap())
            .collect();

        let series = LineSeries::new(sequence.iter().copied(), BLACK.stroke_width(5));

        chart.draw_series(series)?;

        area.titled(
            &format!("n={i}"),
            FontDesc::new(FontFamily::SansSerif, 64.0, FontStyle::Normal).color(&BLACK),
        )?
        .present()?;

        midpoint >>= 1;
        let next_noise = (noise as f64 * decay) as u64;
        if next_noise == 0 {
            break;
        }
        noise = next_noise;
    }

    for (picked, value) in sequence {
        let guessed = find_point(picked, NOISE, decay, seed, ITERATIONS);
        assert_eq!(guessed, value)
    }

    Ok(())
}
