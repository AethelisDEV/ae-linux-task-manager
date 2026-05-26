use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, PlotBounds};

pub struct PerformanceChart {
    pub name: String,
    pub unit: String,
    pub max_value: f32,
    pub color: egui::Color32,
}

impl PerformanceChart {
    pub fn new(name: &str, unit: &str, max_value: f32, color: egui::Color32) -> Self {
        Self {
            name: name.to_string(),
            unit: unit.to_string(),
            max_value,
            color,
        }
    }

    pub fn draw(&self, ui: &mut egui::Ui, data: &[f32]) {
        // Prepare data points
        let points: PlotPoints = data
            .iter()
            .enumerate()
            .map(|(i, &val)| [i as f64, val as f64])
            .collect();

        // Line representation
        let line = Line::new(points)
            .color(self.color)
            .width(2.0)
            .fill(0.0); // No fill or transparent fill to match Windows 11 sleek graph

        // Height of the chart
        let plot_height = 140.0;

        let current_val = data.last().cloned().unwrap_or(0.0);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&self.name)
                        .color(egui::Color32::from_rgb(240, 240, 240))
                        .strong()
                        .size(14.0)
                );
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.1} {}", current_val, self.unit))
                            .color(self.color)
                            .strong()
                            .size(14.0)
                    );
                });
            });

            ui.add_space(6.0);

            // Renders the plot area
            let plot = Plot::new(&self.name)
                .height(plot_height)
                .show_axes(false)
                .show_grid(true)
                .allow_drag(false)
                .allow_scroll(false)
                .allow_zoom(false)
                .include_y(0.0)
                .include_y(self.max_value as f64)
                .label_formatter(|_, _| String::new()) // Disable hover labels to keep clean
                .center_y_axis(true)
                .x_axis_formatter(|_, _| String::new())
                .y_axis_formatter(|_, _| String::new());

            plot.show(ui, |plot_ui| {
                // Pin scale to 0 .. max_value
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    [0.0, 0.0],
                    [data.len() as f64 - 1.0, self.max_value as f64],
                ));
                plot_ui.line(line);
            });
        });
    }
}
