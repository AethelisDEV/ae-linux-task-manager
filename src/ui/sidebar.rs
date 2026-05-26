use eframe::egui;
use crate::ui::theme::ThemeColors;
use crate::ui::localization::{Language, tr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Processes,
    Performance,
    Services,
    Startup,
    Connections,
    FileLocks,
    About,
}

impl Tab {
    pub fn label(&self, lang: Language) -> &'static str {
        match self {
            Tab::Processes => tr("tab_processes", lang),
            Tab::Performance => tr("tab_performance", lang),
            Tab::Services => tr("tab_services", lang),
            Tab::Startup => tr("tab_startup", lang),
            Tab::Connections => tr("tab_connections", lang),
            Tab::FileLocks => tr("tab_file_locks", lang),
            Tab::About => tr("tab_about", lang),
        }
    }

    pub fn icon(&self, lang: Language) -> &'static str {
        match self {
            Tab::Processes => tr("tab_processes", lang),
            Tab::Performance => tr("tab_performance", lang),
            Tab::Services => tr("tab_services", lang),
            Tab::Startup => tr("tab_startup", lang),
            Tab::Connections => tr("tab_connections", lang),
            Tab::FileLocks => tr("tab_file_locks", lang),
            Tab::About => tr("tab_about", lang),
        }
    }
}

pub fn render_sidebar(ui: &mut egui::Ui, current_tab: &mut Tab, colors: &ThemeColors, lang: Language) {
    ui.vertical(|ui| {
        // App header/Logo at the top of the sidebar
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(
                egui::RichText::new("AE TaskManager")
                    .color(colors.text_primary)
                    .strong()
                    .size(18.0)
            );
            ui.label(
                egui::RichText::new("Linux Telemetry")
                    .color(colors.text_secondary)
                    .size(11.0)
            );
            ui.add_space(30.0);
        });

        // Add separators
        ui.colored_label(colors.border, "───────────────────");
        ui.add_space(10.0);

        // Render each Tab button
        for tab in &[
            Tab::Processes,
            Tab::Performance,
            Tab::Services,
            Tab::Startup,
            Tab::Connections,
            Tab::FileLocks,
            Tab::About,
        ] {
            let is_selected = *current_tab == *tab;
            
            let btn_color = if is_selected {
                colors.accent
            } else {
                colors.bg_sidebar
            };

            let text_color = if is_selected {
                colors.text_primary
            } else {
                colors.text_secondary
            };

            ui.add_space(4.0);
            
            // Custom button styling for active/inactive state
            let button = egui::Button::new(
                egui::RichText::new(tab.icon(lang))
                    .color(text_color)
                    .strong()
                    .size(13.0)
            )
            .fill(btn_color)
            .frame(true)
            .min_size(egui::vec2(160.0, 36.0));

            if ui.add(button).clicked() {
                *current_tab = *tab;
            }
        }

        ui.add_space(20.0);
        ui.colored_label(colors.border, "───────────────────");
        
        // Dynamic bottom info: display current session type (Wayland / X11)
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(20.0);
            
            let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "Linux".to_string());
            let display_server = if session_type.to_lowercase() == "wayland" {
                "Wayland 🌀"
            } else {
                "X11 🖥"
            };

            ui.label(
                egui::RichText::new(display_server)
                    .color(colors.text_secondary)
                    .size(11.0)
                    .strong()
            );
            
            ui.label(
                egui::RichText::new(tr("display_backend", lang))
                    .color(colors.border)
                    .size(10.0)
            );
            ui.add_space(10.0);
        });
    });
}
