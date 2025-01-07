use gxhash::{HashMap, HashMapExt};
use mid_brownie_testing::{FractalNoise, find_point};
use plotters::backend::BitMapBackend;
use plotters::chart::{ChartBuilder, ChartContext};
use plotters::coord::Shift;
use plotters::coord::types::{RangedCoordf64, RangedCoordu64};
use plotters::drawing::{DrawingArea, IntoDrawingArea};
use plotters::prelude::{Cartesian2d, FontFamily, FontStyle};
use plotters::series::LineSeries;
use plotters::style::{BLACK, Color, FontDesc, WHITE};
use rand::seq::IteratorRandom;
use rand::thread_rng;
use std::error::Error;
use std::ops::ControlFlow;
use std::{env, iter};

fn show_line(
    area: &DrawingArea<BitMapBackend, Shift>,
    i: &usize,
    chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordu64, RangedCoordf64>>,
    values: &HashMap<[u64; 1], f64>,
) -> Result<(), Box<dyn Error>> {
    let series = LineSeries::new(
        values.iter().map(|(&[k], &v)| (k, v)),
        BLACK.stroke_width(5),
    );
    chart.draw_series(series)?;

    area.titled(
        &format!("n={i}"),
        FontDesc::new(FontFamily::SansSerif, 64.0, FontStyle::Normal).color(&BLACK),
    )?
    .present()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let seed = env::args().skip(1).next().map_or(0, |s| s.parse().unwrap());
    const NOISE: u64 = 10000u64;
    const ITERATIONS: usize = 16;
    let decay = 0.5f64;

    let max = NOISE as f64 / (1f64 - decay);

    let mut noise = FractalNoise::<1>::new(max / 2.0, NOISE, decay, seed);

    let area = BitMapBackend::gif("2d.gif", (1080, 1080), 1_000)?.into_drawing_area();
    let mut i = 1;
    let values = loop {
        area.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&area).build_cartesian_2d(0..u64::MAX, 0f64..max)?;

        noise = match noise.step_midpoints()? {
            ControlFlow::Continue(n) => {
                show_line(&area, &i, &mut chart, n.values())?;
                n
            }
            ControlFlow::Break(values) => {
                show_line(&area, &i, &mut chart, &values)?;
                break values;
            }
        };

        i += 1;
        if i > ITERATIONS {
            break noise.into_values();
        }
    };

    let mut rng = thread_rng();
    for (&picked, &value) in iter::from_fn(|| values.iter().choose(&mut rng)).take(100) {
        let guessed = find_point(picked, NOISE, decay, seed);
        assert_eq!(guessed, value)
    }

    Ok(())
}
