use std::{f64::consts::PI, path::PathBuf};

use clap::Parser;
use colorgrad::CustomGradient;
use nalgebra::Vector2;
use plotters::prelude::*;

struct UniformLinearArray {
    n_sources: usize,
    source_spacing: f64,
    wavelength: f64,
    angle: f64,
}

impl UniformLinearArray {
    fn amplitude(&self, p: Vector2<f64>) -> f64 {
        (0..self.n_sources)
            .map(|i| {
                let i = i as f64;
                let s = Vector2::new(
                    (i - (self.n_sources - 1) as f64 * 0.5) * self.source_spacing,
                    0.0,
                );
                (((p - s).norm() * PI + self.angle * i) * self.wavelength).sin()
            })
            .sum::<f64>()
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, default_value = "1")]
    n: usize,
    #[clap(short, long, default_value = "0.5")]
    spacing: f64,
    #[clap(short, long, default_value = "1.0")]
    wavelength: f64,
    #[clap(short, long, default_value = "0.0")]
    angle: f64,
    #[clap(short, long)]
    out: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        n,
        spacing,
        wavelength,
        angle,
        out,
    } = Args::parse();

    let root = BitMapBackend::new(&out, (800, 800)).into_drawing_area();

    root.fill(&WHITE)?;

    let ula = UniformLinearArray {
        n_sources: n,
        source_spacing: spacing,
        wavelength,
        angle: angle.to_radians(),
    };

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
    let pixels = (0..(pw * ph))
        .map(|k| {
            let (x, y) = (
                xr.start + step.0 * (k % pw) as f64,
                yr.start + step.1 * (k / pw) as f64,
            );

            ((x, y), ula.amplitude(Vector2::new(x, y)))
        })
        .collect::<Vec<_>>();
    let max = pixels
        .iter()
        .map(|(_, a)| a.abs())
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();
    let grad = CustomGradient::new()
        .domain(&[-max, 0.0, max])
        .html_colors(&["blue", "white", "red"])
        .build()
        .unwrap();
    for ((x, y), v) in pixels {
        let color = grad.at(v);
        plotting_area.draw_pixel(
            (x, y),
            &RGBColor(
                (color.r * 255.) as _,
                (color.g * 255.) as _,
                (color.b * 255.) as _,
            ),
        )?;
    }

    root.present()?;

    Ok(())
}
