use eframe::egui;

pub fn show_ram(ctx: &egui::Context, open: &mut bool) {
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("ram_viewer"),
        egui::ViewportBuilder::default()
            .with_title("RAM viewer")
            .with_inner_size([500.0, 500.0]),
        |ui, _class| {
            ui.label("ram whatevers");
            if ui.input(|i| i.viewport().close_requested()) {
                *open = false;
            }
        },
    );
}