pub mod service;

use eframe::egui;
use eframe::egui::{TextureHandle, TextureOptions};
use image::RgbImage;
use std::time::Instant;

use crate::service::mandelbrot_renderer::render_mandelbrot;
use crate::service::image_convert::rgb_image_to_color_image;

struct MandelbrotApp {
    width: u32,
    height: u32,
    max_iter: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    texture: Option<TextureHandle>,
    last_mouse_pos: Option<egui::Pos2>,
    needs_repaint: bool,
    rendering: bool,
    should_ignore_pending_inputs: bool,
    last_render_time: Option<std::time::Duration>,
    target_render_time: f64,
    iter_cap_user: usize
}

impl Default for MandelbrotApp {
    fn default() -> Self {
        let width = 800;
        let height = 800;
        let max_iter = 25;
        let x_min = -2.5;
        let x_max = 1.0;
        let y_min = -1.5;
        let y_max = 1.5;
        Self {
            width,
            height,
            max_iter,
            x_min,
            x_max,
            y_min,
            y_max,
            texture: None,
            last_mouse_pos: None,
            needs_repaint: true,
            rendering: true,
            should_ignore_pending_inputs: false,
            last_render_time: None,
            target_render_time: 0.5,
            iter_cap_user: 3,
        }
    }
}

impl MandelbrotApp {
    fn get_center_coordinates(&self) -> (f64, f64) {
        let center_x = (self.x_min + self.x_max) / 2.0;
        let center_y = (self.y_min + self.y_max) / 2.0;
        (center_x, center_y)
    }

    fn adjust_iterations(&mut self) {
        if let Some(render_time) = self.last_render_time {
            println!(
                "Current: render={:.2}s target={:.2}s iters={}",
                render_time.as_secs_f64(),
                self.target_render_time,
                self.max_iter
            );
            let new_iter = (self.max_iter as f64 * self.target_render_time / render_time.as_secs_f64()) as usize;
            self.max_iter = new_iter.clamp(8, (self.iter_cap_user as f64).exp2().ceil() as usize);
        }
    }
}

impl eframe::App for MandelbrotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.needs_repaint = false;

        // Process button clicks before rendering logic
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Interactive Mandelbrot Set Viewer");
            ui.label("Use mouse wheel to zoom, drag left mouse button to pan.");

            if let Some(texture) = &self.texture {
                ui.image(texture);
            }

            let (center_x, center_y) = self.get_center_coordinates();
            ui.label(format!("Center: ({:.30}, {:.30})", center_x, center_y));
            if let Some(render_time) = self.last_render_time {
                ui.label(format!("Render time: {:.3}s, Iterations: {}", 
                    render_time.as_secs_f64(), self.max_iter));
            }

            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    self.target_render_time = (self.target_render_time - 1.0).max(0.5);
                    self.needs_repaint = true;
                }
                ui.label(format!("Target render time: {:.1}s", self.target_render_time));
                if ui.button("+").clicked() {
                    self.target_render_time += 1.0;
                    self.needs_repaint = true;
                }
            });
             ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    self.iter_cap_user = (self.iter_cap_user - 1).max(3);
                    self.needs_repaint = true;
                }
                ui.label(format!("Max. {} iterations", (self.iter_cap_user as f64).exp2()));
                if ui.button("+").clicked() {
                    self.iter_cap_user += 1;
                    self.needs_repaint = true;
                }
            });
            if ui.button("Re-render").clicked() {
                println!("Manual re-render triggered.");
                self.needs_repaint = true;
            }
        });
        if self.rendering && !self.should_ignore_pending_inputs {
            self.should_ignore_pending_inputs = true;
        }
        if !self.rendering {
            if self.should_ignore_pending_inputs {
                self.should_ignore_pending_inputs = false;
                self.last_mouse_pos = None;
            } else {
                for event in ctx.input(|i| i.events.clone()) {
                    if let egui::Event::Scroll(scroll_delta) = event {
                        let zoom_factor = (scroll_delta.y.signum() * 8.0 * 0.1).exp() as f64;

                        if let Some(mouse_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                            let avail = ctx.available_rect();
                            let mouse_x = mouse_pos.x.clamp(0.0, avail.width());
                            let mouse_y = mouse_pos.y.clamp(0.0, avail.height());

                            let mouse_norm_x = (mouse_x / avail.width()) as f64;
                            let mouse_norm_y = (mouse_y / avail.height()) as f64;

                            let center_x = self.x_min + mouse_norm_x * (self.x_max - self.x_min);
                            let center_y = self.y_min + mouse_norm_y * (self.y_max - self.y_min);

                            let width = self.x_max - self.x_min;
                            let height = self.y_max - self.y_min;

                            let new_width = width / zoom_factor;
                            let new_height = height / zoom_factor;

                            self.x_min = center_x - mouse_norm_x * new_width;
                            self.x_max = self.x_min + new_width;
                            self.y_min = center_y - mouse_norm_y * new_height;
                            self.y_max = self.y_min + new_height;

                            self.needs_repaint = true;
                        }
                    }
                }
                if ctx.input(|i| i.pointer.primary_pressed()) {
                    self.last_mouse_pos = ctx.input(|i| i.pointer.hover_pos());
                } 
                else if ctx.input(|i| i.pointer.primary_released()) {
                    if let (Some(current_pos), Some(last_pos)) = (
                        ctx.input(|i| i.pointer.hover_pos()),
                        self.last_mouse_pos,
                    ) {
                        let delta = current_pos - last_pos;

                        let width = self.x_max - self.x_min;
                        let height = self.y_max - self.y_min;

                        let dx = -(delta.x as f64) * width / (self.width as f64);
                        let dy = -(delta.y as f64) * height / (self.height as f64);

                        self.x_min += dx;
                        self.x_max += dx;
                        self.y_min += dy;
                        self.y_max += dy;

                        self.needs_repaint = true;
                        self.last_mouse_pos = None;
                    }
                }
            }
        } else {
            self.last_mouse_pos = None;
        }

        if self.needs_repaint {
            self.rendering = true;
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Interactive Mandelbrot Set Viewer");
                ui.label("Use mouse wheel to zoom, drag left mouse button to pan.");

                if let Some(texture) = &self.texture {
                    ui.image(texture);
                }
                ui.label("Rendering...");
            });

            ctx.request_repaint();

            return;
        }

        if self.rendering {
            let render_start = Instant::now();
            
            let rgb_image: RgbImage = render_mandelbrot(
                self.width,
                self.height,
                self.x_min,
                self.x_max,
                self.y_min,
                self.y_max,
                self.max_iter,
            );

            self.last_render_time = Some(render_start.elapsed());
            self.adjust_iterations();

            let color_image = rgb_image_to_color_image(&rgb_image);

            if let Some(texture) = &mut self.texture {
                texture.set(color_image, TextureOptions::default());
            } else {
                self.texture = Some(ctx.load_texture(
                    "mandelbrot_texture",
                    color_image,
                    TextureOptions::default(),
                ));
            }

            self.rendering = false;
            self.should_ignore_pending_inputs = true;
        }

        if self.needs_repaint || self.rendering {
            ctx.request_repaint();
        }
    }
}

fn main() {
    let app = MandelbrotApp::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(820.0, 950.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Mandelbrot Viewer",
        native_options,
        Box::new(|_cc| Box::new(app)),
    );
}
