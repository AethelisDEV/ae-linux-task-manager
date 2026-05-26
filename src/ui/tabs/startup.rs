/// XDG Autostart Startup Applications UI Module
///
/// Implements the user interface for tracking and configuring applications
/// that launch automatically when the user starts their desktop session.
/// Renders entries in modern card layouts with interactive toggle controls.

use eframe::egui;
use crate::ui::theme::{ThemeColors, card_style};
use crate::system::startup::{StartupApp, list_startup_apps, set_startup_status};
use crate::ui::localization::{Language, tr};
use std::time::{Instant, Duration};

/// Manages UI state, autostart record caching, and user notification banners
/// for the Startup Applications manager.
pub struct StartupTab {
    /// Cached list of configured startup applications.
    pub cached_apps: Vec<StartupApp>,
    /// Instant when the startup apps list was last scanned from disk folders.
    pub last_load: Option<Instant>,
    /// Optional notification banner detailing successful configuration changes.
    pub success_message: Option<String>,
    /// Optional notification banner detailing failed file modifications.
    pub error_message: Option<String>,
}

impl StartupTab {
    /// Initializes a new StartupTab manager.
    pub fn new() -> Self {
        Self {
            cached_apps: Vec::new(),
            last_load: None,
            success_message: None,
            error_message: None,
        }
    }

    /// Renders the startup configuration dashboard, lists autostart items, and toggles states.
    pub fn render(&mut self, ui: &mut egui::Ui, colors: &ThemeColors, lang: Language) {
        // Auto-refresh cache once every 10 seconds or when invalidated
        let needs_reload = self.last_load.is_none() 
            || self.last_load.unwrap().elapsed() > Duration::from_secs(10);

        if needs_reload {
            self.cached_apps = list_startup_apps();
            self.last_load = Some(Instant::now());
        }

        ui.vertical(|ui| {
            // Title and manual refresh row
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tr("start_title", lang)).strong().size(14.0).color(colors.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(tr("btn_refresh", lang)).clicked() {
                        self.last_load = None;
                    }
                });
            });

            ui.add_space(10.0);

            // Banners for Autostart Modification Feedback
            if let Some(ref success) = self.success_message {
                ui.colored_label(colors.success, format!("✔ {}", success));
                ui.add_space(5.0);
            }
            if let Some(ref error) = self.error_message {
                ui.colored_label(colors.danger, format!("❌ {}", error));
                ui.add_space(5.0);
            }

            // Scrollable view containing autostart cards
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 10.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    if self.cached_apps.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(egui::RichText::new(tr("start_no_entries", lang)).color(colors.text_secondary));
                        });
                        return;
                    }

                    for idx in 0..self.cached_apps.len() {
                        let app = &self.cached_apps[idx];
                        
                        card_style(colors).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            
                            ui.horizontal(|ui| {
                                // App logo fallback icon
                                ui.label(egui::RichText::new("⚡").font(egui::FontId::proportional(20.0)).color(colors.accent_hover));
                                ui.add_space(10.0);

                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&app.name).strong().color(colors.text_primary));
                                    ui.label(egui::RichText::new(format!("{}: {}", if lang == Language::Turkish { "Komut" } else { "Command" }, app.exec)).size(11.0).color(colors.text_secondary));
                                });

                                // Dynamic toggle switch
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let mut check_val = app.enabled;
                                    let toggle_response = ui.checkbox(&mut check_val, "");
                                    
                                    if toggle_response.clicked() {
                                        let target_app = &self.cached_apps[idx];
                                        match set_startup_status(target_app, check_val) {
                                            Ok(_) => {
                                                self.success_message = Some(if lang == Language::Turkish {
                                                    format!(
                                                        "'{}' başlangıç durumu '{}' olarak güncellendi.", 
                                                        target_app.name, 
                                                        if check_val { "Etkin" } else { "Devre Dışı" }
                                                    )
                                                } else {
                                                    format!(
                                                        "Startup status of '{}' updated to '{}'.", 
                                                        target_app.name, 
                                                        if check_val { "Enabled" } else { "Disabled" }
                                                    )
                                                });
                                                self.error_message = None;
                                                self.last_load = None; // Force reload to apply modifications
                                            }
                                            Err(e) => {
                                                self.error_message = Some(if lang == Language::Turkish {
                                                    format!(
                                                        "'{}' güncellenirken hata oluştu: {}", 
                                                        target_app.name, 
                                                        e
                                                    )
                                                } else {
                                                    format!(
                                                        "Error updating '{}': {}", 
                                                        target_app.name, 
                                                        e
                                                    )
                                                });
                                                self.success_message = None;
                                            }
                                        }
                                    }
                                });
                            });
                        });
                    }
                });
        });
    }
}
