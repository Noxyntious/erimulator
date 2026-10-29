use eframe::egui;

use erimulator::nes::cpu::Cpu;

pub fn show_cpu(ctx: &egui::Context, open: &mut bool, mut cpu: Option<&mut Cpu>) {
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("cpu_viewer"),
        egui::ViewportBuilder::default()
            .with_title("CPU viewer")
            .with_inner_size([500.0, 500.0]),
        |ui, _class| {
            if let Some(cpu) = cpu.as_deref_mut() {
                if ui.button("Step").clicked() {
                    cpu.step();
                }
                if ui.button("Step frame").clicked() {
                    cpu.step();
                }
                ui.label(format!("PC: {:04X}", cpu.pc));
                ui.label(format!("A: {:02X}", cpu.a));
                ui.label(format!("X: {:02X}", cpu.x));
                ui.label(format!("Y: {:02X}", cpu.y));
                ui.label(format!("SP: {:02X}", cpu.sp));
                ui.label(format!("Status: {:02X}", cpu.status));
                ui.label(format!("Cycles: {}", cpu.cycles));
            } else {
                ui.label("No CPU");
            }
            if ui.input(|i| i.viewport().close_requested()) {
                *open = false;
            }
        },
    );
}