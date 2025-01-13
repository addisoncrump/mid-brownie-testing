use highway::HighwayHasher;
use mid_brownie_testing::{find_point, FractalNoise};
use plotters::backend::BitMapBackend;
use plotters::chart::{ChartBuilder, ChartContext};
use plotters::coord::cartesian::Cartesian3d;
use plotters::coord::types::{RangedCoordf64, RangedCoordu64};
use plotters::coord::Shift;
use plotters::drawing::{DrawingArea, IntoDrawingArea};
use plotters::prelude::{FontFamily, FontStyle};
use plotters::series::SurfaceSeries;
use plotters::style::{Color, FontDesc, ShapeStyle, BLACK, BLUE, WHITE};
use rand::seq::IteratorRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::error::Error;
use std::hash::BuildHasherDefault;
use std::{env, iter};

fn show_surface(
    area: &DrawingArea<BitMapBackend, Shift>,
    midpoint: u64,
    i: usize,
    chart: &mut ChartContext<
        BitMapBackend,
        Cartesian3d<RangedCoordu64, RangedCoordf64, RangedCoordu64>,
    >,
    values: &HashMap<[u64; 2], f64, BuildHasherDefault<HighwayHasher>>,
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
    let seed = env::args().nth(1).map_or(0, |s| s.parse().unwrap());
    const NOISE: u64 = 10000u64;
    const ITERATIONS: usize = 10;
    let noise = NOISE;
    let decay = 0.5f64;

    let max = noise as f64 / (1f64 - decay);

    let mut noise = FractalNoise::<2>::new(noise, decay, seed);

    let area = BitMapBackend::gif("3d.gif", (1080, 1080), 1_000)?.into_drawing_area();
    let values = loop {
        let midpoint = 1u64.reverse_bits() >> noise.iterations();

        if !noise.step_midpoints()? {
            break noise.into_values();
        };
        area.fill(&WHITE)?;

        let mut chart =
            ChartBuilder::on(&area).build_cartesian_3d(0..u64::MAX, 0f64..max, 0..u64::MAX)?;
        show_surface(
            &area,
            midpoint,
            noise.iterations(),
            &mut chart,
            noise.values(),
        )?;

        if noise.iterations() > ITERATIONS {
            break noise.into_values();
        }
    };

    Ok(())
}
