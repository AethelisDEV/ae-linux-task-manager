pub mod app;
pub mod backend;
pub mod system;
pub mod ui;

use app::TaskManagerApp;
use eframe::egui;

fn main() -> eframe::Result {
    let mut options = eframe::NativeOptions::default();
    
    // Set premium window viewport properties (size, title, limits)
    options.viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(900.0, 600.0))
        .with_min_inner_size(egui::vec2(800.0, 500.0))
        .with_title("AE TaskManager - Linux System Telemetry");

    eframe::run_native(
        "ae_taskmanager",
        options,
        Box::new(|cc| Ok(Box::new(TaskManagerApp::new(cc)))),
    )
}
