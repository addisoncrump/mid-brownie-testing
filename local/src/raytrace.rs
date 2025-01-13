use cgmath::{Angle, Deg, InnerSpace, PerspectiveFov, Point3, Vector3};
use mid_brownie_testing::{FractalNoise, Ray};
use plotters::backend::BitMapBackend;
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::RGBColor;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::error::Error;
use std::iter;

fn main() -> Result<(), Box<dyn Error>> {
    const DIM: u32 = 720;
    let area = BitMapBackend::new("3d.png", (DIM, DIM)).into_drawing_area();

    let mut cache3d = FractalNoise::<2>::new(1000.0, 0.5, 1);

    let resolution = area.dim_in_pixel();
    let max = cache3d.upper_bound(0);

    // let origin = Point3::new(-(1i64 << 18) as f64, average, -(1i64 << 18) as f64);

    let pixels = (0..resolution.0)
        .flat_map(|x| {
            iter::repeat((x, (1u64 << 31) * x as u64 / DIM as u64))
                .zip((0..resolution.1).map(|y| (y, (1u64 << 31) * y as u64 / DIM as u64)))
        })
        .par_bridge()
        .map_with(cache3d, |cache3d, ((x_pixel, x), (y_pixel, y))| {
            // .map(|((x_pixel, x), (y_pixel, y))| {
            let direction = Vector3::new(0f64, -1f64, 0f64).normalize();
            let ray = Ray::new(direction, Point3::new(x as f64 + 0.5, max, y as f64 + 0.5));
            ray.intersect(cache3d, max)
                .map(|point| ((x_pixel as i32, y_pixel as i32), point))
        })
        .flatten()
        .collect::<Vec<_>>();

    for (pixel, point) in pixels {
        let color = graymap(&point.y, max);
        area.draw_pixel(pixel, &color)?;
    }

    area.present()?;

    Ok(())
}

fn graymap(y: &f64, max: f64) -> RGBColor {
    let grayness = ((512.0) * ((1.0 + (*y - max) / max).powi(3))) as u8;
    RGBColor(grayness, grayness, grayness)
}

#[cfg(test)]
mod test {
    use cgmath::{Point3, Vector3};
    use mid_brownie_testing::{FractalNoise, Ray};
    use std::error::Error;

    #[test]
    fn centerpoint_visible() -> Result<(), Box<dyn Error>> {
        let mut cache3d = FractalNoise::<2>::new(1.0, 0.9, 5);
        let max = cache3d.upper_bound(0);

        let ray = Ray::new(
            Vector3::new(0f64, -1f64, 0f64),
            Point3::new(
                1u32.reverse_bits() as f64 + 0.5,
                max,
                1u32.reverse_bits() as f64 + 0.5,
            ),
        );

        let intersection = ray.intersect(&mut cache3d, max);

        println!("{intersection:?}");

        Ok(())
    }
}
