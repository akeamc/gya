use std::f64::consts::PI;

use nalgebra::Vector2;
use plotters::prelude::*;

const N_SOURCES: usize = 10;
const SOURCE_SPACING: f64 = 1.0;
const WAVELENGTH: f64 = 1.0;
const ANGLE: f64 = 0.0;

fn amplitude(p: Vector2<f64>) -> f64 {
    (0..N_SOURCES)
        .map(|i| {
            let i = i as f64;
            let s = Vector2::new((i - N_SOURCES as f64 * 0.5) * SOURCE_SPACING, 0.0);
            (((p - s).norm() * PI + ANGLE * i) * WAVELENGTH).sin()
        })
        .sum::<f64>() / N_SOURCES as f64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("interferens.png", (800, 800)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_cartesian_2d(-20.0..20.0, -20.0..20.0)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let plotting_area = chart.plotting_area();

    let range = plotting_area.get_pixel_range();

    let (pw, ph) = (range.0.end - range.0.start, range.1.end - range.1.start);
    let (xr, yr) = (chart.x_range(), chart.y_range());

    let step = (
        (xr.end - xr.start) / pw as f64,
        (yr.end - yr.start) / ph as f64,
    );
    for k in 0..(pw * ph) {
        let (x, y) = (
            xr.start + step.0 * (k % pw) as f64,
            yr.start + step.1 * (k / pw) as f64,
        );

        plotting_area.draw_pixel(
            (x, y),
            &BlackWhite::get_color(amplitude(Vector2::new(x, y))),
        )?;
    }

    root.present()?;

    Ok(())
}
