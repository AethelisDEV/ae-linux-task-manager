/// Systemd Services Management UI Module
///
/// Provides a graphical, high-fidelity table listing all systemd services,
/// supporting real-time filter searching, service control actions (start, stop, etc.),
/// and safe privilege escalation workers that prevent GUI freezing.

use eframe::egui;
use crate::ui::theme::{ThemeColors, card_style};
use crate::system::systemd::{ServiceInfo, list_systemd_services, manage_service};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Instant, Duration};

/// Manages structural state, cached services, text search filters,
/// and background execution channels for the systemd services dashboard.
pub struct ServicesTab {
    /// The current text search query to filter service names or descriptions.
    pub search_query: String,
    /// Cached vector of all queried Systemd service units.
    pub cached_services: Vec<ServiceInfo>,
    /// Instant when the systemd cache was last refreshed from systemctl.
    pub last_load: Option<Instant>,
    /// Receiver half of the channel capturing results from administrative threads.
    pub rx: Receiver<Result<String, String>>,
    /// Sender half of the channel used by background worker threads to notify completions.
    pub tx: Sender<Result<String, String>>,
    /// Optional notification banner detailing successful executions.
    pub status_success: Option<String>,
    /// Optional notification banner detailing failed operations or cancellations.
    pub status_error: Option<String>,
}

impl ServicesTab {
    /// Initializes a new ServicesTab and spawns the safe execution channel.
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            search_query: String::new(),
            cached_services: Vec::new(),
            last_load: None,
            rx,
            tx,
            status_success: None,
            status_error: None,
        }
    }

    /// Renders the services manager view, lists units in cards, and dispatches actions.
    pub fn render(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        // Drain any incoming background execution statuses
        while let Ok(res) = self.rx.try_recv() {
            match res {
                Ok(msg) => {
                    self.status_success = Some(msg);
                    self.status_error = None;
                    self.last_load = None; // Invalidate cache to force immediate reload
                }
                Err(err) => {
                    self.status_error = Some(err);
                    self.status_success = None;
                }
            }
        }

        // Periodically refresh the cached list (every 8 seconds) or when invalidated
        let needs_reload = self.last_load.is_none() 
            || self.last_load.unwrap().elapsed() > Duration::from_secs(8);

        if needs_reload {
            if let Ok(list) = list_systemd_services() {
                self.cached_services = list;
                self.last_load = Some(Instant::now());
            }
        }

        ui.vertical(|ui| {
            // Header: Search query and Manual Refresh
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔍 Ara:").color(colors.text_secondary));
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Servis adı veya açıklama ara...")
                        .desired_width(260.0)
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄 Listeyi Yenile").clicked() {
                        self.last_load = None; // Forces reload on next check
                    }
                });
            });

            ui.add_space(10.0);

            // Banners for Administrative operations feedback
            if let Some(ref success) = self.status_success {
                ui.colored_label(colors.success, format!("✔ {}", success));
                ui.add_space(5.0);
            }
            if let Some(ref error) = self.status_error {
                ui.colored_label(colors.danger, format!("❌ {}", error));
                ui.add_space(5.0);
            }

            // Render Services inside card styles
            card_style(colors).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height() - 10.0);

                // Table Header
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    
                    let col_name_w = ui.available_width() * 0.25;
                    let col_load_w = ui.available_width() * 0.12;
                    let col_active_w = ui.available_width() * 0.12;
                    let col_sub_w = ui.available_width() * 0.12;
                    let col_desc_w = ui.available_width() * 0.39;

                    ui.add(egui::Button::new(egui::RichText::new("Servis Adı").strong()).fill(colors.bg_card).min_size(egui::vec2(col_name_w, 24.0)));
                    ui.add(egui::Button::new(egui::RichText::new("Yükleme").strong()).fill(colors.bg_card).min_size(egui::vec2(col_load_w, 24.0)));
                    ui.add(egui::Button::new(egui::RichText::new("Etkinlik").strong()).fill(colors.bg_card).min_size(egui::vec2(col_active_w, 24.0)));
                    ui.add(egui::Button::new(egui::RichText::new("Alt Durum").strong()).fill(colors.bg_card).min_size(egui::vec2(col_sub_w, 24.0)));
                    ui.add(egui::Button::new(egui::RichText::new("Açıklama").strong()).fill(colors.bg_card).min_size(egui::vec2(col_desc_w, 24.0)));
                });

                ui.add_space(5.0);
                ui.colored_label(colors.border, "────────────────────────────────────────────────────────────────────────────────────────");
                ui.add_space(5.0);

                // Filter list
                let query = self.search_query.to_lowercase();
                let filtered: Vec<&ServiceInfo> = self.cached_services.iter()
                    .filter(|s| {
                        s.name.to_lowercase().contains(&query) || s.description.to_lowercase().contains(&query)
                    })
                    .collect();

                let row_height = 28.0;
                let total_rows = filtered.len();

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .auto_shrink([false; 2])
                    .show_rows(ui, row_height, total_rows, |ui, row_range| {
                        ui.spacing_mut().item_spacing.y = 2.0;

                        if total_rows == 0 {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(egui::RichText::new("Eşleşen servis bulunamadı").color(colors.text_secondary));
                            });
                            return;
                        }

                        for idx in row_range {
                            let s = filtered[idx];
                            let response = ui.add(
                                egui::Button::new("")
                                    .min_size(egui::vec2(ui.available_width(), 26.0))
                                    .fill(egui::Color32::TRANSPARENT)
                                    .frame(true)
                            );

                            let s_name = s.name.clone();
                            let tx = self.tx.clone();

                            // Inject Services Context Menu with pkexec background threads
                            response.context_menu(|ui| {
                                ui.set_min_width(220.0);
                                ui.style_mut().visuals.widgets.hovered.bg_fill = colors.accent.linear_multiply(0.3);
                                ui.style_mut().visuals.widgets.active.bg_fill = colors.accent;

                                ui.label(egui::RichText::new(format!("⚙ {}", s_name)).strong().color(colors.text_secondary));
                                ui.separator();

                                let trigger_action = |ui: &mut egui::Ui, label: &str, action: &'static str, color: egui::Color32, s_name: &str, tx: &Sender<Result<String, String>>| {
                                    if ui.button(egui::RichText::new(label).color(color)).clicked() {
                                        let s_name_inner = s_name.to_string();
                                        let tx_inner = tx.clone();
                                        let ctx = ui.ctx().clone();
                                        std::thread::spawn(move || {
                                            let res = manage_service(&s_name_inner, action)
                                                .map(|_| format!("Servis '{}' üzerinde '{}' işlemi başarıyla tamamlandı.", s_name_inner, action))
                                                .map_err(|e| format!("Servis işlemi başarısız: {}", e));
                                            let _ = tx_inner.send(res);
                                            ctx.request_repaint();
                                        });
                                        ui.close_menu();
                                    }
                                };

                                trigger_action(ui, "▶  Servisi Başlat", "start", colors.success, &s_name, &tx);
                                trigger_action(ui, "⏹  Servisi Durdur", "stop", colors.danger, &s_name, &tx);
                                trigger_action(ui, "🔄  Yeniden Başlat", "restart", colors.warning, &s_name, &tx);
                                ui.separator();
                                trigger_action(ui, "⚡  Başlangıçta Etkinleştir", "enable", colors.text_primary, &s_name, &tx);
                                trigger_action(ui, "❌  Açılışta Devre Dışı Bırak", "disable", colors.text_secondary, &s_name, &tx);
                            });

                            let rect = response.rect;
                            let y_pos = rect.min.y + 5.0;

                            let col_name_w = rect.width() * 0.25;
                            let col_load_w = rect.width() * 0.12;
                            let col_active_w = rect.width() * 0.12;
                            let col_sub_w = rect.width() * 0.12;

                            // Column 1: Icon and Name
                            ui.painter().text(
                                egui::pos2(rect.min.x + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                format!("⚙  {}", s.name),
                                egui::FontId::proportional(12.0),
                                colors.text_primary,
                            );

                            // Column 2: Load State
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_name_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                &s.load_state,
                                egui::FontId::proportional(12.0),
                                colors.text_secondary,
                            );

                            // Column 3: Active State
                            let active_color = if s.active_state == "active" {
                                colors.success
                            } else if s.active_state == "failed" {
                                colors.danger
                            } else {
                                colors.text_secondary
                            };

                            ui.painter().text(
                                egui::pos2(rect.min.x + col_name_w + col_load_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                &s.active_state,
                                egui::FontId::proportional(12.0),
                                active_color,
                            );

                            // Column 4: Sub State
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_name_w + col_load_w + col_active_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                &s.sub_state,
                                egui::FontId::proportional(12.0),
                                colors.text_secondary,
                            );

                            // Column 5: Description
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_name_w + col_load_w + col_active_w + col_sub_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                &s.description,
                                egui::FontId::proportional(12.0),
                                colors.text_primary,
                            );
                        }
                    });
            });
        });
    }
}
