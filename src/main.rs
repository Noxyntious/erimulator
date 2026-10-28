#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod debugs;
mod prefs;

use eframe::egui::{self};
use erimulator::nes::cartridge::Cartridge;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([530.0, 520.0])
        .with_maximize_button(false)
        .with_resizable(false),
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
    cartridge: Option<Cartridge>,
    error: Option<String>,
    ppu_view: bool,
    cpu_view: bool,
    ram_view: bool,
    controls: bool,
}

impl Default for Erimulator {
    fn default() -> Self {
        Self {
            framebuffer: vec![0; 256 * 240 * 3], // 256x240 framebuffer with 3 bytes per pixel (rgb)
            screen: None,
            cartridge: None,
            error: None,
            ppu_view: false,
            cpu_view: false,
            ram_view: false,
            controls: false,
        }
    }
}

impl eframe::App for Erimulator {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("NES", |ui| {
                    if ui.button("Load cartridge").clicked() {
                        if let Some(path) = rfd::FileDialog::new().add_filter("NES ROMs", &["nes"]).pick_file() {
                            match std::fs::read(&path) {
                                Ok(bytes) => match Cartridge::from_bytes(&bytes) {
                                    Ok(cart) => {
                                        self.cartridge = Some(cart);
                                        self.error = None;
                                    }
                                    Err(e) => {
                                        self.error = Some(format!("Failed to parse cartridge: {:?}", e));
                                        self.cartridge = None;
                                    }
                                },
                                Err(e) => {
                                    self.error = Some(format!("Failed to read file: {:?}", e));
                                    self.cartridge = None;
                                }
                            }
                        }
                    }
                });
                ui.menu_button("Preferences", |ui| {
                    if ui.button("Controls").clicked() {
                        self.controls = true;
                    }
                });
                ui.menu_button("Debugs", |ui| {
                    if ui.button("RAM viewer").clicked() {
                        self.ram_view = true;
                    }
                    if ui.button("PPU viewer").clicked() {
                        self.ppu_view = true;
                    }
                    if ui.button("CPU viewer").clicked() {
                        self.cpu_view = true;
                    }
                });
                ui.separator();
                if let Some(error) = &self.error {
                    ui.label(format!("Error: {}", error));
                } else if let Some(cart) = &self.cartridge {
                    ui.label(format!("Loaded cartridge: {:?}", cart.header));
                } else {
                    ui.label("No cartridge loaded");
                }
            });
        });
        let image = egui::ColorImage::from_rgb([256, 240], &self.framebuffer);
        egui::CentralPanel::default().show(ui, |ui| {
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
        if self.ppu_view {
            debugs::ppuview::show_ppu(&ui.ctx().clone(), &mut self.ppu_view);
        }
        if self.ram_view {
            debugs::ramview::show_ram(&ui.ctx().clone(), &mut self.ram_view);
        }
        if self.cpu_view {
            debugs::cpuview::show_cpu(&ui.ctx().clone(), &mut self.cpu_view);
        }
        if self.controls { // NO I DONT
            prefs::controls::show_controls(&ui.ctx().clone(), &mut self.controls);
        }
    }
}
