use eframe::egui;
use crate::system::SystemState;
use crate::backend::PlatformInfo;
use crate::ui::theme::{ThemeColors, card_style};

pub struct AboutTab {
    platform_info: PlatformInfo,
}

impl AboutTab {
    pub fn new() -> Self {
        Self {
            platform_info: PlatformInfo::collect(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &SystemState, colors: &ThemeColors) {
        // Re-poll dynamic platform details when tab is loaded
        if ui.visuals().dark_mode {
            self.platform_info = PlatformInfo::collect();
        }

        ui.vertical(|ui| {
            ui.heading(egui::RichText::new("Linux Host Diagnostics").color(colors.text_primary));
            ui.add_space(5.0);
            ui.label(egui::RichText::new("A holistic overview of your Linux operating environment and graphical server topology.").color(colors.text_secondary));
            ui.add_space(15.0);

            ui.horizontal(|ui| {
                ui.set_width(ui.available_width());

                // Left Column: OS & Kernel info
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() * 0.48);
                    
                    card_style(colors).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.heading(egui::RichText::new("System Context").color(colors.accent).size(15.0));
                        ui.add_space(10.0);

                        about_row(ui, "Hostname", &state.hostname, colors);
                        about_row(ui, "OS Distribution", &state.os_version, colors);
                        about_row(ui, "Kernel Release", &state.kernel_version, colors);
                        about_row(ui, "Architecture", std::env::consts::ARCH, colors);
                        about_row(ui, "Telemetry Refresh Rate", "1.0s (Background Poller Thread)", colors);
                    });
                });

                ui.add_space(10.0);

                // Right Column: Display Server Details
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());

                    card_style(colors).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.heading(egui::RichText::new("Graphical Server Profile").color(colors.accent).size(15.0));
                        ui.add_space(10.0);

                        let server_desc = self.platform_info.display_server.to_string();
                        about_row(ui, "Active Server Type", &server_desc, colors);

                        if self.platform_info.wayland.is_active {
                            about_row(ui, "Compositor Name", &self.platform_info.wayland.compositor, colors);
                            about_row(ui, "Wayland Socket", &self.platform_info.wayland.socket_name, colors);
                            if let Some(ref path) = self.platform_info.wayland.socket_path {
                                about_row(ui, "Socket Connection Path", path, colors);
                            }
                        } else if self.platform_info.x11.is_active {
                            about_row(ui, "Window Manager", &self.platform_info.x11.window_manager, colors);
                            about_row(ui, "X11 Display Name", &self.platform_info.x11.display_name, colors);
                            if let Some(ref auth) = self.platform_info.x11.xauthority {
                                about_row(ui, "XAuthority Profile", auth, colors);
                            }
                        } else {
                            about_row(ui, "Session Protocol", "Fallback Terminal / Headless", colors);
                        }
                    });
                });
            });

            ui.add_space(20.0);

            // Tech Stack Credit Banner at bottom
            card_style(colors).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("AE TaskManager (Rust Telemetry Interface)")
                                .color(colors.text_primary)
                                .strong()
                        );
                        ui.label(
                            egui::RichText::new("Built as a premium system metrics interface utilizing Rust, eframe, and sysinfo.")
                                .color(colors.text_secondary)
                                .size(11.0)
                        );
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(colors.success, "COMPLIANT NATIVE");
                    });
                });
            });
        });
    }
}

fn about_row(ui: &mut egui::Ui, key: &str, value: &str, colors: &ThemeColors) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(key).color(colors.text_secondary).size(10.0).strong());
        ui.add_space(2.0);
        ui.label(egui::RichText::new(value).color(colors.text_primary).size(12.0));
        ui.add_space(4.0);
        ui.colored_label(colors.border, "───────────────────────────────────────────────────────");
        ui.add_space(4.0);
    });
}
