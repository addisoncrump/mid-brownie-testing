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
    max: f64,
}

#[wasm_bindgen]
impl Chart {
    pub fn new(noise: u64, decay: f64, seed: i64) -> Self {
        let max = noise as f64 / (1f64 - decay);
        let initial = max / 2.0;
        Self {
            cache: FractalNoise::new(initial, noise, decay, seed),
            max,
        }
    }

    pub fn plot3d(
        &mut self,
        canvas: HtmlCanvasElement,
        pitch: f64,
        yaw: f64,
        iterations: usize,
    ) -> Result<(), JsValue> {
        plot3d::draw(canvas, &mut self.cache, pitch, yaw, self.max, iterations)
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

mod plot3d {
    use crate::DrawResult;
    use mid_brownie_testing::FractalNoise;
    use plotters::chart::ChartBuilder;
    use plotters::drawing::IntoDrawingArea;
    use plotters::prelude::{
        BLACK, BLUE, FontDesc, FontFamily, FontStyle, ShapeStyle, SurfaceSeries,
    };
    use plotters::style::{Color, WHITE};
    use plotters_canvas::CanvasBackend;
    use std::iter;
    use web_sys::HtmlCanvasElement;

    pub fn draw(
        canvas: HtmlCanvasElement,
        cache3d: &mut FractalNoise<2>,
        pitch: f64,
        yaw: f64,
        max: f64,
        iterations: usize,
    ) -> DrawResult<()> {
        let area = CanvasBackend::with_canvas_object(canvas)
            .unwrap()
            .into_drawing_area();
        area.fill(&WHITE)?;

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

        let series = SurfaceSeries::xoz(
            iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
            iter::successors(Some(0), |s: &u64| s.checked_add(midpoint)),
            |x, z| cache3d.values().get(&[x, z]).copied().unwrap(),
        )
        .style(ShapeStyle::from(BLUE.mix(0.5)).stroke_width(0));

        chart
            .with_projection(|mut pb| {
                pb.yaw = yaw;
                pb.pitch = pitch;
                pb.scale = 0.7;
                pb.into_matrix()
            })
            .draw_series(series)?;

        Ok(())
    }
}
