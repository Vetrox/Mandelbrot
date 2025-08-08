use image::{RgbImage, Rgb};
use num_complex::{Complex, ComplexFloat};

use crate::service::mandelbrot_calc::mandelbrot_iterations;

pub fn render_mandelbrot(
    width: u32,
    height: u32,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    max_iter: usize,
) -> RgbImage {
    let mut img = RgbImage::new(width, height);

    for px in 0..width {
        for py in 0..height {
            // Map pixel coordinate to complex plane
            let cx = x_min + (px as f64 / width as f64) * (x_max - x_min);
            let cy = y_min + (py as f64 / height as f64) * (y_max - y_min);
            let c = Complex::new(cx, cy);

            let iter = mandelbrot_iterations(c, max_iter);

            if iter == max_iter {
                // for points inside the set
                img.put_pixel(px, py, Rgb([0, 20, 20]));
            } else {
                let scale = ((iter + 1 )as f64).log(100f64);
                let max_scale = ((max_iter + 1) as f64).log(100f64);
                let scale2 = 10f64.expf(((iter + 1 )as f64));
                let max_scale2 = 10f64.expf((max_iter + 1) as f64);
                let ratio = scale / max_scale;
                let ratio2 = scale2 / max_scale2;

                let color_value = (255.0 * ratio) as u8;
                let color_value2 = (255.0 * ratio2) as u8;
                img.put_pixel(px, py, Rgb([100, color_value2, color_value]));
            };
        }
    }

    img
}