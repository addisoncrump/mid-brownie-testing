use mid_brownie_testing::FractalNoise;
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
    initial_noise: u64,
    initial_decay: f64,
    max: f64,
}

#[wasm_bindgen]
impl Chart {
    pub fn new(noise: u64, decay: f64, seed: i64) -> Self {
        let max = noise as f64 / (1f64 - decay);
        let initial = max / 2.0;
        Self {
            cache: FractalNoise::new(initial, noise, decay, seed),
            initial_noise: noise,
            initial_decay: decay,
            max,
        }
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
            self.initial_noise,
            self.initial_decay,
            pitch,
            yaw,
            self.max,
            iterations,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

mod plot3d {
    use crate::DrawResult;
    use mid_brownie_testing::{upper_bound, FractalNoise};
    use plotters::chart::ChartBuilder;
    use plotters::drawing::IntoDrawingArea;
    use plotters::prelude::SurfaceSeries;
    use plotters::style::{Color, RGBColor, RED};
    use plotters_canvas::CanvasBackend;
    use std::iter;
    use web_sys::HtmlCanvasElement;

    pub fn draw(
        canvas: HtmlCanvasElement,
        cache3d: &mut FractalNoise<2>,
        show_upper_bound: bool,
        mut noise: u64,
        decay: f64,
        pitch: f64,
        yaw: f64,
        max: f64,
        iterations: usize,
    ) -> DrawResult<()> {
        let area = CanvasBackend::with_canvas_object(canvas)
            .unwrap()
            .into_drawing_area();

        let mut chart =
            ChartBuilder::on(&area).build_cartesian_3d(0..u64::MAX, 0f64..max, 0..u64::MAX)?;

        while cache3d.iterations() < iterations {
            if !cache3d.step_midpoints()? {
                break;
            };
        }

        let iterations = iterations.min(cache3d.iterations()) - 1;
        println!("showing {iterations} iterations");
        let midpoint = 1u64.reverse_bits() >> iterations;

        let graymap = |y: &f64| {
            let grayness = ((512.0) * ((1.0 + (*y - max) / max).powi(3))) as u8;
            RGBColor(grayness, grayness, grayness).filled()
        };
        let series = SurfaceSeries::xoz(
            iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
            iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
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
            for _ in 0..iterations {
                noise = (noise as f64 * decay) as u64;
            }

            let series = SurfaceSeries::xoz(
                iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
                iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
                |x, z| {
                    [
                        [x, z],
                        [x, z.overflowing_add(midpoint).0],
                        [x.overflowing_add(midpoint).0, z],
                        [x.overflowing_add(midpoint).0, z.overflowing_add(midpoint).0],
                    ]
                    .into_iter()
                    .filter_map(|p| cache3d.values().get(&p))
                    .max_by(|f1, f2| f1.total_cmp(f2))
                    .unwrap()
                        + upper_bound::<2>(noise, decay)
                },
            )
            .style(&RED.mix(0.1));
            chart.draw_series(series)?;
        }

        Ok(())
    }
}
