use num_complex::Complex;

pub fn mandelbrot_iterations(c: Complex<f64>, max_iter: usize) -> usize {
    let mut z = Complex::new(0.0, 0.0);
    let mut i = 0;

    while i < max_iter && z.norm_sqr() <= 4.0 {
        z = z * z + c;
        i += 1;
    }

    i
}