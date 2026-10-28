use eframe::egui;

pub fn show_ppu(ctx: &egui::Context, open: &mut bool) {
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("ppu_viewer"),
        egui::ViewportBuilder::default()
            .with_title("PPU viewer")
            .with_inner_size([500.0, 500.0]),
        |ui, _class| {
            ui.label("ppu whatevers");
            if ui.input(|i| i.viewport().close_requested()) {
                *open = false;
            }
        },
    );
}