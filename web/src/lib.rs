use cgmath::num_traits::{FloatConst, Signed};
use cgmath::{AbsDiffEq, Angle, InnerSpace, Transform, Zero};
use mid_brownie_testing::FractalNoise;
use plotters::style::RGBColor;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Type alias for the result of a drawing function.
pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Type used on the JS side to convert screen coordinates to chart
/// coordinates.
#[wasm_bindgen]
pub struct Chart {
    cache: FractalNoise<2>,
}

#[wasm_bindgen]
impl Chart {
    pub fn new(noise: u32, decay: f64, seed: i64) -> Self {
        let cache = FractalNoise::new(noise, decay, seed);
        Self { cache }
    }

    pub fn plot3d(
        &mut self,
        canvas: HtmlCanvasElement,
        show_upper_bound: bool,
        pitch: f64,
        yaw: f64,
        iterations: usize,
    ) -> Result<(), JsValue> {
        plot3d::draw(
            canvas,
            &mut self.cache,
            show_upper_bound,
            pitch,
            yaw,
            iterations,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

mod plot3d {
    use crate::DrawResult;
    use mid_brownie_testing::FractalNoise;
    use plotters::chart::ChartBuilder;
    use plotters::drawing::IntoDrawingArea;
    use plotters::prelude::SurfaceSeries;
    use plotters::style::{Color, RED};
    use plotters_canvas::CanvasBackend;
    use std::iter;
    use web_sys::HtmlCanvasElement;

    pub fn draw(
        canvas: HtmlCanvasElement,
        cache3d: &mut FractalNoise<2>,
        show_upper_bound: bool,
        pitch: f64,
        yaw: f64,
        iterations: usize,
    ) -> DrawResult<()> {
        let area = CanvasBackend::with_canvas_object(canvas)
            .unwrap()
            .into_drawing_area();
        let max = cache3d.upper_bound(0);

        let mut chart =
            ChartBuilder::on(&area).build_cartesian_3d(0..u32::MAX, 0f64..max, 0..u32::MAX)?;

        while cache3d.iterations() < iterations {
            if !cache3d.step_midpoints()? {
                break;
            };
        }

        let iterations = iterations.min(cache3d.iterations()) - 1;
        println!("showing {iterations} iterations");
        let midpoint = 1u32.reverse_bits() >> iterations;

        let graymap = |y: &f64| super::graymap(y, max).filled();
        let series = SurfaceSeries::xoz(
            iter::successors(Some(0), |s: &u32| s.checked_add(midpoint)),
            iter::successors(Some(0), |s: &u32| s.checked_add(midpoint)),
            |x, z| cache3d.values().get(&[x, z]).copied().unwrap(),
        )
        .style_func(&graymap);

        chart
            .with_projection(|mut pb| {
                pb.yaw = yaw;
                pb.pitch = pitch;
                pb.scale = 0.7;
                pb.into_matrix()
            })
            .draw_series(series)?;

        if show_upper_bound {
            let upper = cache3d.upper_bound(iterations);

            let series = SurfaceSeries::xoz(
                iter::successors(Some(midpoint / 2), |s: &u32| s.checked_add(midpoint)),
                iter::successors(Some(midpoint / 2), |s: &u32| s.checked_add(midpoint)),
                |x, z| {
                    let x = x.next_multiple_of(midpoint) - midpoint;
                    let z = z.next_multiple_of(midpoint) - midpoint;
                    [
                        [x, z],
                        [x.overflowing_add(midpoint).0, z],
                        [x, z.overflowing_add(midpoint).0],
                        [x.overflowing_add(midpoint).0, z.overflowing_add(midpoint).0],
                    ]
                    .into_iter()
                    .filter_map(|p| cache3d.values().get(&p))
                    .max_by(|f1, f2| f1.total_cmp(f2))
                    .unwrap()
                        + upper
                },
            )
            .style(&RED.mix(0.1));
            chart.draw_series(series)?;
        }

        Ok(())
    }
}

fn graymap(y: &f64, max: f64) -> RGBColor {
    let grayness = ((512.0) * ((1.0 + (*y - max) / max).powi(3))) as u8;
    RGBColor(grayness, grayness, grayness)
}
