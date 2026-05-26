/// File Lock & Descriptor Tracking UI Module
///
/// Implements the user interface for identifying and resolving file locks.
/// Users enter a target path, and the application scans the active descriptors
/// in the background, showing locking processes and offering inline termination.

use eframe::egui;
use crate::ui::theme::{ThemeColors, card_style};
use crate::system::file_lock::{FileLockInfo, find_locking_processes};
use crate::system::process::ProcessManager;
use crate::ui::localization::{Language, tr};

/// Manages structural state, target paths, locking records,
/// and operation feedback alerts for the file locks dashboard.
pub struct FileLocksTab {
    /// The target filesystem path (file or folder) to inspect.
    pub target_path: String,
    /// Collection of parsed process locks returned by the `/proc` sweeping query.
    pub cached_locks: Vec<FileLockInfo>,
    /// Optional notification banner detailing successful process shutdowns.
    pub status_success: Option<String>,
    /// Optional notification banner detailing search or termination failures.
    pub status_error: Option<String>,
}

impl FileLocksTab {
    /// Initializes a new FileLocksTab.
    pub fn new() -> Self {
        Self {
            target_path: String::new(),
            cached_locks: Vec::new(),
            status_success: None,
            status_error: None,
        }
    }

    /// Triggers a dynamic `/proc` file locks lookup sweep for the currently entered target path.
    pub fn trigger_search(&mut self, lang: Language) {
        if self.target_path.trim().is_empty() {
            self.status_error = Some(if lang == Language::Turkish {
                "Lütfen geçerli bir dosya veya klasör yolu girin.".to_string()
            } else {
                "Please enter a valid file or folder path.".to_string()
            });
            self.status_success = None;
            return;
        }

        self.cached_locks = find_locking_processes(&self.target_path);
        
        if self.cached_locks.is_empty() {
            self.status_success = Some(if lang == Language::Turkish {
                "Bu dosya veya klasörü kullanan herhangi bir süreç bulunamadı.".to_string()
            } else {
                "No processes found holding handles on this file or folder.".to_string()
            });
            self.status_error = None;
        } else {
            self.status_success = Some(if lang == Language::Turkish {
                format!("Toplam {} süreç kilit bulundu.", self.cached_locks.len())
            } else {
                format!("Found total of {} process locks.", self.cached_locks.len())
            });
            self.status_error = None;
        }
    }

    /// Renders the file locks search dashboard and lists locking processes.
    pub fn render(&mut self, ui: &mut egui::Ui, colors: &ThemeColors, lang: Language) {
        ui.vertical(|ui| {
            // Path lookup entry bar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(if lang == Language::Turkish { "📂 Yol (Path):" } else { "📂 Path:" }).color(colors.text_secondary));
                ui.add(
                    egui::TextEdit::singleline(&mut self.target_path)
                        .hint_text(tr("lock_search_hint", lang))
                        .desired_width(340.0)
                );

                if ui.button(tr("lock_btn_scan", lang)).clicked() {
                    self.trigger_search(lang);
                }
            });

            ui.add_space(10.0);

            // Banners for diagnostic feedback
            if let Some(ref success) = self.status_success {
                ui.colored_label(colors.success, format!("✔ {}", success));
                ui.add_space(5.0);
            }
            if let Some(ref error) = self.status_error {
                ui.colored_label(colors.danger, format!("❌ {}", error));
                ui.add_space(5.0);
            }

            // Results tabular listings card container
            card_style(colors).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height() - 10.0);

                // Table Header
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    
                    let col_pid_w = ui.available_width() * 0.12;
                    let col_name_w = ui.available_width() * 0.25;
                    let col_user_w = ui.available_width() * 0.18;
                    let col_path_w = ui.available_width() * 0.33;
                    let col_act_w = ui.available_width() * 0.12;

                    ui.add(egui::Button::new(egui::RichText::new(tr("lock_hdr_pid", lang)).strong()).fill(colors.bg_card).min_size(egui::vec2(col_pid_w, 24.0)));
                    ui.add(egui::Button::new(egui::RichText::new(tr("lock_hdr_name", lang)).strong()).fill(colors.bg_card).min_size(egui::vec2(col_name_w, 24.0)));
                    ui.add(egui::Button::new(egui::RichText::new(tr("proc_hdr_user", lang)).strong()).fill(colors.bg_card).min_size(egui::vec2(col_user_w, 24.0)));
                    ui.add(egui::Button::new(egui::RichText::new(tr("lock_hdr_path", lang)).strong()).fill(colors.bg_card).min_size(egui::vec2(col_path_w, 24.0)));
                    ui.add(egui::Button::new(egui::RichText::new(tr("lock_hdr_action", lang)).strong()).fill(colors.bg_card).min_size(egui::vec2(col_act_w, 24.0)));
                });

                ui.add_space(5.0);
                ui.colored_label(colors.border, "────────────────────────────────────────────────────────────────────────────────────────");
                ui.add_space(5.0);

                let row_height = 28.0;
                let total_rows = self.cached_locks.len();

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .auto_shrink([false; 2])
                    .show_rows(ui, row_height, total_rows, |ui, row_range| {
                        ui.spacing_mut().item_spacing.y = 2.0;

                        if total_rows == 0 {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(egui::RichText::new(if lang == Language::Turkish { "Henüz bir arama yapılmadı veya kilitli dosya bulunamadı" } else { "No search performed yet or no locked files found" }).color(colors.text_secondary));
                            });
                            return;
                        }

                        for idx in row_range {
                            let l = &self.cached_locks[idx];
                            
                            let response = ui.add(
                                egui::Button::new("")
                                    .min_size(egui::vec2(ui.available_width(), 26.0))
                                    .fill(egui::Color32::TRANSPARENT)
                                    .frame(true)
                            );

                            let rect = response.rect;
                            let y_pos = rect.min.y + 5.0;

                            let col_pid_w = rect.width() * 0.12;
                            let col_name_w = rect.width() * 0.25;
                            let col_user_w = rect.width() * 0.18;
                            let col_path_w = rect.width() * 0.33;

                            // Column 1: PID
                            ui.painter().text(
                                egui::pos2(rect.min.x + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                l.pid.to_string(),
                                egui::FontId::proportional(12.0),
                                colors.text_secondary,
                            );

                            // Column 2: Process Name
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_pid_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                &l.process_name,
                                egui::FontId::proportional(12.0),
                                colors.text_primary,
                            );

                            // Column 3: User
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_pid_w + col_name_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                &l.username,
                                egui::FontId::proportional(12.0),
                                colors.text_secondary,
                            );

                            // Column 4: File Path
                            let path_label = ui.put(
                                egui::Rect::from_min_size(
                                    egui::pos2(rect.min.x + col_pid_w + col_name_w + col_user_w + 8.0, y_pos - 3.0),
                                    egui::vec2(col_path_w - 16.0, 20.0)
                                ),
                                egui::Label::new(egui::RichText::new(&l.open_path).color(colors.text_primary).size(12.0))
                                    .truncate()
                            );
                            path_label.on_hover_text(&l.open_path);

                            // Column 5: End Task Action Button
                            let action_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.min.x + col_pid_w + col_name_w + col_user_w + col_path_w + 8.0, y_pos - 4.0),
                                egui::vec2(60.0, 20.0)
                            );

                            let end_btn = egui::Button::new(egui::RichText::new(tr("btn_terminate", lang)).size(10.0).strong().color(colors.text_primary))
                                .fill(colors.danger);

                            if ui.put(action_rect, end_btn).clicked() {
                                let pid = l.pid;
                                match ProcessManager::kill_process(pid, false) {
                                    Ok(_) => {
                                        self.status_success = Some(if lang == Language::Turkish {
                                            format!("PID {} başarıyla kapatıldı.", pid)
                                        } else {
                                            format!("PID {} terminated successfully.", pid)
                                        });
                                        self.status_error = None;
                                        // Refresh the search automatically to verify lock is cleared!
                                        self.cached_locks = find_locking_processes(&self.target_path);
                                    }
                                    Err(e) => {
                                        self.status_error = Some(if lang == Language::Turkish {
                                            format!("Kapatma başarısız: {}", e)
                                        } else {
                                            format!("Termination failed: {}", e)
                                        });
                                        self.status_success = None;
                                    }
                                }
                            }
                        }
                    });
            });
        });
    }
}
