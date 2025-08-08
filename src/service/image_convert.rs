use eframe::egui::{ColorImage, Color32};
use image::RgbImage;

pub fn rgb_image_to_color_image(image: &RgbImage) -> ColorImage {
    let size = [image.width() as usize, image.height() as usize];
    let mut pixels = Vec::with_capacity(size[0] * size[1]);

    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);
            pixels.push(Color32::from_rgb(pixel[0], pixel[1], pixel[2]));
        }
    }

    ColorImage { size, pixels }
}
