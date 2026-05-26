use eframe::egui;
use crate::system::SystemAggregator;
use crate::ui::TaskManagerUi;
use crate::ui::theme::apply_theme;

pub struct TaskManagerApp {
    aggregator: SystemAggregator,
    ui: TaskManagerUi,
}

impl TaskManagerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure theme, styling, fonts, and visuals
        apply_theme(&cc.egui_ctx);

        // Install egui image loaders natively to decode and render process PNG/SVG icons
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            aggregator: SystemAggregator::new(cc.egui_ctx.clone()),
            ui: TaskManagerUi::new(),
        }
    }
}

impl eframe::App for TaskManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Fetch real-time telemetry from background thread using zero-allocation read-lock borrow
        let state = self.aggregator.read_state();

        // Render the primary window interface
        self.ui.draw(ctx, &state);
    }
}
