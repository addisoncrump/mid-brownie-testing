use gxhash::{HashMap, HashMapExt};
use mid_brownie_testing::{FractalNoise, find_point};
use plotters::backend::BitMapBackend;
use plotters::chart::{ChartBuilder, ChartContext};
use plotters::coord::Shift;
use plotters::coord::cartesian::Cartesian3d;
use plotters::coord::types::{RangedCoordf64, RangedCoordu64};
use plotters::drawing::{DrawingArea, IntoDrawingArea};
use plotters::prelude::{Cartesian2d, FontFamily, FontStyle, LineSeries};
use plotters::series::SurfaceSeries;
use plotters::style::{BLACK, BLUE, Color, FontDesc, ShapeStyle, WHITE};
use rand::seq::IteratorRandom;
use rand::thread_rng;
use std::error::Error;
use std::ops::ControlFlow;
use std::{env, iter};

fn show_surface(
    area: &DrawingArea<BitMapBackend, Shift>,
    midpoint: u64,
    i: &usize,
    chart: &mut ChartContext<
        BitMapBackend,
        Cartesian3d<RangedCoordu64, RangedCoordf64, RangedCoordu64>,
    >,
    values: &HashMap<[u64; 2], f64>,
) -> Result<(), Box<dyn Error>> {
    let series = SurfaceSeries::xoz(
        iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
        iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
        |x, z| values.get(&[x, z]).copied().unwrap(),
    )
    .style(ShapeStyle::from(BLUE.mix(0.5)).stroke_width(0));

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
    const ITERATIONS: usize = 10;
    let mut noise = NOISE;
    let decay = 0.5f64;

    let max = noise as f64 / (1f64 - decay);

    let mut noise = FractalNoise::<2>::new(max / 2.0, noise, decay, seed);

    let area = BitMapBackend::gif("3d.gif", (1080, 1080), 1_000)?.into_drawing_area();
    let mut i = 1;
    let values = loop {
        area.fill(&WHITE)?;
        let mut chart =
            ChartBuilder::on(&area).build_cartesian_3d(0..u64::MAX, 0f64..max, 0..u64::MAX)?;
        let midpoint = noise.midpoint();

        noise = match noise.step_midpoints()? {
            ControlFlow::Continue(n) => {
                show_surface(&area, midpoint, &i, &mut chart, n.values())?;
                n
            }
            ControlFlow::Break(values) => {
                show_surface(&area, midpoint, &i, &mut chart, &values)?;
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
