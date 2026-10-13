#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)]

use eframe::egui::{self};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "erimulator v0.01",
        options,
        Box::new(|_cc| Ok(Box::<Erimulator>::default())),
    )
}

struct Erimulator {
    framebuffer: Vec<u8>,
    screen: Option<egui::TextureHandle>,
}

impl Default for Erimulator {
    fn default() -> Self {
        Self {
            framebuffer: vec![0; 256 * 240 * 3], // 256x240 framebuffer with 3 bytes per pixel (rgb)
            screen: None,
        }
    }
}

impl eframe::App for Erimulator {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let image = egui::ColorImage::from_rgb([256, 240], &self.framebuffer);
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("My egui Application");

            match &mut self.screen {
                Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
                None => {
                    self.screen = Some(ui.ctx().load_texture(
                        "screen",
                        image,
                        egui::TextureOptions::NEAREST,
                    ));
                }
            }

            if let Some(texture) = &self.screen {
                ui.add(
                    egui::Image::from_texture(texture).fit_to_exact_size(egui::vec2(512.0, 480.0)),
                );
            }
        });
    }
}
