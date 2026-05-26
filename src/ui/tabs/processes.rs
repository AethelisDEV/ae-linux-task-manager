use eframe::egui;
use crate::system::process::{ProcessInfo, ProcessManager};
use crate::ui::theme::{ThemeColors, card_style};
use std::sync::mpsc::{Receiver, Sender};
use std::process::Command;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Pid,
    Cpu,
    Memory,
    GpuMemory,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct ProcessesTab {
    pub search_query: String,
    pub selected_pid: Option<u32>,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub kill_error: Option<String>,
    pub kill_success: Option<String>,

    // Cache to prevent sorting/filtering every frame
    pub cached_list: Vec<ProcessInfo>,
    pub last_state_update: Option<std::time::Instant>,
    pub last_query: String,
    pub last_sort_col: SortColumn,
    pub last_sort_dir: SortDirection,

    // Channels for background process termination/management
    pub rx: Receiver<Result<String, String>>,
    pub tx: Sender<Result<String, String>>,
    pub show_properties: Option<ProcessInfo>,

    // Tree View State
    pub tree_view: bool,
    pub expanded_pids: std::collections::HashSet<u32>,
    pub cached_tree_list: Vec<(ProcessInfo, usize, bool, bool)>, // (ProcessInfo, Indentation, Expanded, HasChildren)
    pub last_tree_view: bool,
}

impl ProcessesTab {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut expanded_pids = std::collections::HashSet::new();
        // Expand some standard root processes by default for a nice warm startup feel
        expanded_pids.insert(1);
        Self {
            search_query: String::new(),
            selected_pid: None,
            sort_column: SortColumn::Cpu,
            sort_direction: SortDirection::Descending,
            kill_error: None,
            kill_success: None,
            cached_list: Vec::new(),
            last_state_update: None,
            last_query: String::new(),
            last_sort_col: SortColumn::Cpu,
            last_sort_dir: SortDirection::Descending,
            rx,
            tx,
            show_properties: None,
            tree_view: false,
            expanded_pids,
            cached_tree_list: Vec::new(),
            last_tree_view: false,
        }
    }


    pub fn render(&mut self, ui: &mut egui::Ui, raw_processes: &[ProcessInfo], last_update: std::time::Instant, colors: &ThemeColors) {
        // Poll background task channel for completed actions (admin kills, opened locations)
        while let Ok(result) = self.rx.try_recv() {
            match result {
                Ok(success_msg) => {
                    self.kill_success = Some(success_msg);
                    self.kill_error = None;
                    self.selected_pid = None; // Reset selection on success
                }
                Err(error_msg) => {
                    self.kill_error = Some(error_msg);
                    self.kill_success = None;
                }
            }
        }

        ui.vertical(|ui| {
            // Header actions row (Search and End Task)

            ui.horizontal(|ui| {
                // Search bar
                ui.label(egui::RichText::new("🔍 Search:").color(colors.text_secondary));
                let search_box = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search process name or PID...")
                        .desired_width(260.0)
                );
                
                if search_box.changed() {
                    // Reset selection when search changes
                    self.selected_pid = None;
                }

                ui.add_space(20.0);
                if ui.checkbox(&mut self.tree_view, egui::RichText::new("🌳 Ağaç Görünümü (Tree)").color(colors.text_primary)).changed() {
                    self.last_state_update = None; // Invalidate cache to force reload
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let has_selection = self.selected_pid.is_some();
                    
                    let kill_btn = egui::Button::new(
                        egui::RichText::new("⏹  End Task")
                            .color(if has_selection { colors.text_primary } else { colors.text_secondary })
                            .strong()
                    )
                    .fill(if has_selection { colors.danger } else { colors.border })
                    .min_size(egui::vec2(100.0, 30.0));

                    if ui.add_enabled(has_selection, kill_btn).clicked() {
                        if let Some(pid) = self.selected_pid {
                            match ProcessManager::kill_process(pid, false) {
                                Ok(_) => {
                                    self.kill_success = Some(format!("Sent termination signal to PID {}", pid));
                                    self.kill_error = None;
                                    self.selected_pid = None;
                                }
                                Err(e) => {
                                    self.kill_error = Some(format!("Failed to kill PID {}: {}", pid, e));
                                    self.kill_success = None;
                                }
                            }
                        }
                    }
                });
            });

            ui.add_space(10.0);

            // Banners for Process Killing Feedback
            if let Some(ref success) = self.kill_success {
                ui.colored_label(colors.success, success);
                ui.add_space(5.0);
            }
            if let Some(ref error) = self.kill_error {
                ui.colored_label(colors.danger, error);
                ui.add_space(5.0);
            }

            // Determine if we need to refresh our cached sorted and filtered process list
            let cache_needs_update = self.last_state_update != Some(last_update)
                || self.last_query != self.search_query
                || self.last_sort_col != self.sort_column
                || self.last_sort_dir != self.sort_direction
                || self.last_tree_view != self.tree_view;

            if cache_needs_update {
                let mut filtered: Vec<ProcessInfo> = raw_processes
                    .iter()
                    .filter(|p| {
                        if self.search_query.is_empty() {
                            true
                        } else {
                            let query = self.search_query.to_lowercase();
                            p.name.to_lowercase().contains(&query) || p.pid.to_string().contains(&query)
                        }
                    })
                    .cloned()
                    .collect();

                // Sort processes based on active column and direction
                filtered.sort_by(|a, b| {
                    let ordering = match self.sort_column {
                        SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                        SortColumn::Pid => a.pid.cmp(&b.pid),
                        SortColumn::Cpu => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
                        SortColumn::Memory => a.memory_bytes.cmp(&b.memory_bytes),
                        SortColumn::GpuMemory => a.gpu_memory_bytes.cmp(&b.gpu_memory_bytes),
                    };

                    if self.sort_direction == SortDirection::Descending {
                        ordering.reverse()
                    } else {
                        ordering
                    }
                });

                self.cached_list = filtered;

                // Build tree cache if tree view is enabled
                if self.tree_view {
                    self.cached_tree_list = build_flat_tree(
                        raw_processes,
                        &self.expanded_pids,
                        &self.search_query,
                        self.sort_column,
                        self.sort_direction,
                    );
                }

                self.last_state_update = Some(last_update);
                self.last_query = self.search_query.clone();
                self.last_sort_col = self.sort_column;
                self.last_sort_dir = self.sort_direction;
                self.last_tree_view = self.tree_view;
            }

            let sort_column = self.sort_column;
            let sort_direction = self.sort_direction;
            let selected_pid = self.selected_pid;

            let mut toggle_pid = None;
            // Renders the processes inside a card container
            let (clicked_col, next_selected_pid) = card_style(colors).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height() - 10.0);

                let mut clicked_col = None;
                let mut next_selected_pid = selected_pid;

                // Table Headers Row
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    
                    let header_style = |name: &str, active: bool, dir: SortDirection| {
                        let suffix = if active {
                            if dir == SortDirection::Descending { "  ▼" } else { "  ▲" }
                        } else {
                            ""
                        };
                        egui::RichText::new(format!("{}{}", name, suffix)).strong()
                    };

                    // Define sizes for each column
                    let col_name_w = ui.available_width() * 0.35;
                    let col_pid_w = ui.available_width() * 0.10;
                    let col_cpu_w = ui.available_width() * 0.12;
                    let col_mem_w = ui.available_width() * 0.15;
                    let col_gpu_w = ui.available_width() * 0.15;
                    let col_user_w = ui.available_width() * 0.13;

                    // Column Name
                    let btn_name = ui.add(
                        egui::Button::new(header_style("Process Name", sort_column == SortColumn::Name, sort_direction))
                            .min_size(egui::vec2(col_name_w, 24.0))
                            .fill(colors.bg_card)
                    );
                    if btn_name.clicked() { clicked_col = Some(SortColumn::Name); }

                    // Column PID
                    let btn_pid = ui.add(
                        egui::Button::new(header_style("PID", sort_column == SortColumn::Pid, sort_direction))
                            .min_size(egui::vec2(col_pid_w, 24.0))
                            .fill(colors.bg_card)
                    );
                    if btn_pid.clicked() { clicked_col = Some(SortColumn::Pid); }

                    // Column CPU %
                    let btn_cpu = ui.add(
                        egui::Button::new(header_style("CPU %", sort_column == SortColumn::Cpu, sort_direction))
                            .min_size(egui::vec2(col_cpu_w, 24.0))
                            .fill(colors.bg_card)
                    );
                    if btn_cpu.clicked() { clicked_col = Some(SortColumn::Cpu); }

                    // Column RAM
                    let btn_mem = ui.add(
                        egui::Button::new(header_style("Memory", sort_column == SortColumn::Memory, sort_direction))
                            .min_size(egui::vec2(col_mem_w, 24.0))
                            .fill(colors.bg_card)
                    );
                    if btn_mem.clicked() { clicked_col = Some(SortColumn::Memory); }

                    // Column GPU Memory
                    let btn_gpu = ui.add(
                        egui::Button::new(header_style("GPU Memory", sort_column == SortColumn::GpuMemory, sort_direction))
                            .min_size(egui::vec2(col_gpu_w, 24.0))
                            .fill(colors.bg_card)
                    );
                    if btn_gpu.clicked() { clicked_col = Some(SortColumn::GpuMemory); }

                    // Column User
                    ui.add(
                        egui::Button::new(egui::RichText::new("User").strong())
                            .min_size(egui::vec2(col_user_w, 24.0))
                            .fill(colors.bg_card)
                    );
                });

                ui.add_space(5.0);
                ui.colored_label(colors.border, "────────────────────────────────────────────────────────────────────────────────────────");
                ui.add_space(5.0);

                let row_height = 26.0;
                let total_rows = if self.tree_view { self.cached_tree_list.len() } else { self.cached_list.len() };

                // Processes Scroll Table List with Virtual Scrolling to minimize CPU usage
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .auto_shrink([false; 2])
                    .show_rows(ui, row_height, total_rows, |ui, row_range| {
                        ui.spacing_mut().item_spacing.y = 2.0;

                        if total_rows == 0 {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(egui::RichText::new("No matching processes found").color(colors.text_secondary));
                            });
                            return;
                        }

                        for idx in row_range {
                            let (p, indent, expanded, has_children) = if self.tree_view {
                                let item = &self.cached_tree_list[idx];
                                (&item.0, item.1, item.2, item.3)
                            } else {
                                (&self.cached_list[idx], 0, false, false)
                            };
                            let is_highlighted = next_selected_pid == Some(p.pid);
                            
                            let row_bg = if is_highlighted {
                                colors.accent.linear_multiply(0.2) // Subtle highlighted tint
                            } else {
                                egui::Color32::TRANSPARENT
                            };

                            let response = ui.add(
                                egui::Button::new("")
                                    .min_size(egui::vec2(ui.available_width(), 26.0))
                                    .fill(row_bg)
                                    .frame(true)
                            );

                            let p_clone = p.clone();
                            let pid = p.pid;
                            let p_name = p.name.clone();
                            let p_exe = p.exe_path.clone();
                            let tx = self.tx.clone();

                            response.context_menu(|ui| {
                                ui.set_min_width(220.0);
                                ui.style_mut().visuals.widgets.hovered.bg_fill = colors.accent.linear_multiply(0.3);
                                ui.style_mut().visuals.widgets.active.bg_fill = colors.accent;

                                ui.label(egui::RichText::new("⚙ İşlem Seçenekleri").strong().color(colors.text_secondary));
                                ui.separator();

                                // 1. Görevi Sonlandır
                                if ui.button(egui::RichText::new("⏹  Görevi Sonlandır").color(colors.text_primary)).clicked() {
                                    match ProcessManager::kill_process(pid, false) {
                                        Ok(_) => {
                                            self.kill_success = Some(format!("{} (PID {}) işlemine sonlandırma sinyali gönderildi.", p_name, pid));
                                            self.kill_error = None;
                                            self.selected_pid = None;
                                        }
                                        Err(e) => {
                                            self.kill_error = Some(format!("{} sonlandırılamadı: {}", p_name, e));
                                            self.kill_success = None;
                                        }
                                    }
                                    ui.close_menu();
                                }

                                // 2. Yönetici Olarak Sonlandır
                                if ui.button(egui::RichText::new("🛡  Yönetici Olarak Sonlandır (Zorla)").color(colors.danger)).clicked() {
                                    let name_inner = p_name.clone();
                                    let tx_inner = tx.clone();
                                    let ctx = ui.ctx().clone();
                                    std::thread::spawn(move || {
                                        let output = Command::new("pkexec")
                                            .arg("kill")
                                            .arg("-9")
                                            .arg(pid.to_string())
                                            .output();
                                        
                                        let res = match output {
                                            Ok(out) => {
                                                if out.status.success() {
                                                    Ok(format!("{} (PID {}) yönetici ayrıcalıklarıyla başarıyla zorla kapatıldı.", name_inner, pid))
                                                } else {
                                                    let err = String::from_utf8_lossy(&out.stderr).to_string();
                                                    let err = if err.trim().is_empty() {
                                                        "Yetkilendirme iptal edildi veya başarısız oldu.".to_string()
                                                    } else {
                                                        err
                                                    };
                                                    Err(format!("{} (PID {}) yönetici olarak kapatılamadı: {}", name_inner, pid, err))
                                                }
                                            }
                                            Err(e) => Err(format!("pkexec çalıştırılamadı: {}", e)),
                                        };
                                        let _ = tx_inner.send(res);
                                        ctx.request_repaint();
                                    });
                                    ui.close_menu();
                                }

                                ui.separator();

                                // 3. Dosya Konumunu Aç
                                let has_exe = !p_exe.is_empty();
                                let open_loc_btn = ui.add_enabled(
                                    has_exe,
                                    egui::Button::new(egui::RichText::new("📂  Dosya Konumunu Aç").color(colors.text_primary))
                                );
                                if open_loc_btn.clicked() {
                                    let exe_path_inner = p_exe.clone();
                                    let tx_inner = tx.clone();
                                    let ctx = ui.ctx().clone();
                                    std::thread::spawn(move || {
                                        let path = std::path::Path::new(&exe_path_inner);
                                        if let Some(parent) = path.parent() {
                                            let parent_str = parent.to_string_lossy().to_string();
                                            let output = Command::new("xdg-open")
                                                .arg(&parent_str)
                                                .output();
                                            if let Err(e) = output {
                                                let _ = tx_inner.send(Err(format!("Dosya konumu açılamadı: {}", e)));
                                                ctx.request_repaint();
                                            }
                                        }
                                    });
                                    ui.close_menu();
                                }

                                // 4. İnternette Ara
                                if ui.button(egui::RichText::new("🌐  İnternette Ara").color(colors.text_primary)).clicked() {
                                    let name_inner = p_name.clone();
                                    let tx_inner = tx.clone();
                                    let ctx = ui.ctx().clone();
                                    std::thread::spawn(move || {
                                        let query = format!("https://www.google.com/search?q={} Linux process", name_inner);
                                        let output = Command::new("xdg-open")
                                            .arg(&query)
                                            .output();
                                        if let Err(e) = output {
                                            let _ = tx_inner.send(Err(format!("Tarayıcı açılamadı: {}", e)));
                                            ctx.request_repaint();
                                        }
                                    });
                                    ui.close_menu();
                                }

                                ui.separator();

                                // 5. Özellikler
                                if ui.button(egui::RichText::new("ℹ  Özellikler").color(colors.accent_hover)).clicked() {
                                    self.show_properties = Some(p_clone);
                                    ui.close_menu();
                                }
                            });

                            if response.clicked() {
                                if is_highlighted {
                                    next_selected_pid = None; // Deselect
                                } else {
                                    next_selected_pid = Some(p.pid);
                                }
                            }


                            let rect = response.rect;
                            let y_pos = rect.min.y + 4.0;

                            let col_name_w = rect.width() * 0.35;
                            let col_pid_w = rect.width() * 0.10;
                            let col_cpu_w = rect.width() * 0.12;
                            let col_mem_w = rect.width() * 0.15;
                            let col_gpu_w = rect.width() * 0.15;

                            // Col 1: Icon and Name
                            let indent_shift = indent as f32 * 16.0;
                            let start_offset = if self.tree_view { 16.0 } else { 8.0 };
                            let text_pos = egui::pos2(rect.min.x + start_offset + 22.0 + indent_shift, y_pos);

                            if self.tree_view && has_children {
                                let arrow_str = if expanded { "▼" } else { "▶" };
                                let arrow_rect = egui::Rect::from_min_size(
                                    egui::pos2(rect.min.x + indent_shift + 4.0, y_pos - 1.0),
                                    egui::vec2(12.0, 16.0)
                                );
                                let arrow_btn = ui.put(
                                    arrow_rect,
                                    egui::Button::new(egui::RichText::new(arrow_str).size(8.0).strong().color(colors.text_secondary))
                                        .fill(egui::Color32::TRANSPARENT)
                                        .frame(false)
                                );
                                if arrow_btn.clicked() {
                                    toggle_pid = Some((p.pid, expanded));
                                }
                            }

                            if let Some(ref path) = p.icon_path {
                                let icon_rect = egui::Rect::from_min_size(
                                    egui::pos2(rect.min.x + start_offset + indent_shift, y_pos - 1.0),
                                    egui::vec2(16.0, 16.0)
                                );
                                ui.put(
                                    icon_rect,
                                    egui::Image::new(path)
                                        .max_width(16.0)
                                        .max_height(16.0)
                                        .rounding(2.0)
                                );
                            } else {
                                let fallback = get_process_icon(&p.name);
                                ui.painter().text(
                                    egui::pos2(rect.min.x + start_offset + indent_shift, y_pos),
                                    egui::Align2::LEFT_TOP,
                                    fallback,
                                    egui::FontId::proportional(12.0),
                                    colors.text_primary,
                                );
                            }

                            ui.painter().text(
                                text_pos,
                                egui::Align2::LEFT_TOP,
                                &p.name,
                                egui::FontId::proportional(12.0),
                                colors.text_primary,
                            );

                            // Col 2: PID
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_name_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                p.pid.to_string(),
                                egui::FontId::proportional(12.0),
                                colors.text_secondary,
                            );

                            // Col 3: CPU %
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_name_w + col_pid_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                format!("{:.1}%", p.cpu_usage),
                                egui::FontId::proportional(12.0),
                                if p.cpu_usage > 40.0 { colors.danger } else if p.cpu_usage > 10.0 { colors.warning } else { colors.text_primary },
                            );

                            // Col 4: Memory (MB/GB)
                            let mem_str = format_bytes(p.memory_bytes);
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_name_w + col_pid_w + col_cpu_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                mem_str,
                                egui::FontId::proportional(12.0),
                                colors.text_primary,
                            );

                            // Col 5: GPU Memory (MB/GB)
                            let gpu_mem_str = format_bytes(p.gpu_memory_bytes);
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_name_w + col_pid_w + col_cpu_w + col_mem_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                gpu_mem_str,
                                egui::FontId::proportional(12.0),
                                colors.text_primary,
                            );

                            // Col 6: User
                            ui.painter().text(
                                egui::pos2(rect.min.x + col_name_w + col_pid_w + col_cpu_w + col_mem_w + col_gpu_w + 8.0, y_pos),
                                egui::Align2::LEFT_TOP,
                                &p.username,
                                egui::FontId::proportional(12.0),
                                colors.text_secondary,
                            );
                        }
                    });

                (clicked_col, next_selected_pid)
            }).inner;

            if let Some((pid, exp)) = toggle_pid {
                if exp {
                    self.expanded_pids.remove(&pid);
                } else {
                    self.expanded_pids.insert(pid);
                }
                self.cached_tree_list = build_flat_tree(
                    raw_processes,
                    &self.expanded_pids,
                    &self.search_query,
                    self.sort_column,
                    self.sort_direction,
                );
            }

            // Apply state mutations back to self
            self.selected_pid = next_selected_pid;
            if let Some(col) = clicked_col {
                if self.sort_column == col {
                    self.sort_direction = if self.sort_direction == SortDirection::Ascending {
                        SortDirection::Descending
                    } else {
                        SortDirection::Ascending
                    };
                } else {
                    self.sort_column = col;
                    self.sort_direction = SortDirection::Descending;
                }
            }
        });

        // Draw the Process Properties Modal
        if self.show_properties.is_some() {
            let mut open = true;
            let p = self.show_properties.as_ref().unwrap().clone();
            
            let modal = egui::Window::new(format!("⚙ Özellikler - {}", p.name))
                .open(&mut open)
                .resizable(true)
                .default_width(450.0)
                .min_width(350.0)
                .collapsible(false);

            let mut close_clicked = false;

            modal.show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.add_space(5.0);
                    // Process general info header
                    ui.horizontal(|ui| {
                        if let Some(ref path) = p.icon_path {
                            ui.add(
                                egui::Image::new(path)
                                    .max_width(32.0)
                                    .max_height(32.0)
                                    .rounding(4.0)
                            );
                        } else {
                            let fallback = get_process_icon(&p.name);
                            ui.label(egui::RichText::new(fallback).font(egui::FontId::proportional(28.0)));
                        }
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&p.name).strong().size(18.0).color(colors.text_primary));
                            ui.label(egui::RichText::new(format!("PID: {}", p.pid)).color(colors.text_secondary));
                        });
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Grid of Properties
                    egui::Grid::new("properties_grid")
                        .num_columns(2)
                        .spacing([15.0, 10.0])
                        .show(ui, |ui| {
                            // Parent PID
                            ui.label(egui::RichText::new("Üst İşlem (Parent PID):").color(colors.text_secondary));
                            ui.label(egui::RichText::new(p.parent_pid.map(|id| id.to_string()).unwrap_or_else(|| "Yok (None)".to_string())).color(colors.text_primary));
                            ui.end_row();

                            // CPU Usage
                            ui.label(egui::RichText::new("CPU Kullanımı:").color(colors.text_secondary));
                            ui.label(egui::RichText::new(format!("{:.1}%", p.cpu_usage)).color(if p.cpu_usage > 40.0 { colors.danger } else { colors.text_primary }).strong());
                            ui.end_row();

                            // RAM Usage
                            ui.label(egui::RichText::new("Bellek (RAM):").color(colors.text_secondary));
                            ui.label(egui::RichText::new(format_bytes(p.memory_bytes)).color(colors.text_primary));
                            ui.end_row();

                            // GPU Memory
                            ui.label(egui::RichText::new("GPU Bellek:").color(colors.text_secondary));
                            ui.label(egui::RichText::new(format_bytes(p.gpu_memory_bytes)).color(colors.text_primary));
                            ui.end_row();

                            // Status
                            ui.label(egui::RichText::new("Durum (Status):").color(colors.text_secondary));
                            ui.label(egui::RichText::new(&p.status).color(colors.text_primary));
                            ui.end_row();

                            // User Account
                            ui.label(egui::RichText::new("Kullanıcı (User):").color(colors.text_secondary));
                            ui.label(egui::RichText::new(&p.username).color(colors.text_primary));
                            ui.end_row();
                        });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Executable path with a "Copy" button
                    ui.label(egui::RichText::new("Dosya Yolu (Executable Path):").color(colors.text_secondary));
                    ui.add_space(2.0);
                    if p.exe_path.is_empty() {
                        ui.label(egui::RichText::new("Bilinmiyor (Unknown)").italics().color(colors.text_secondary));
                    } else {
                        ui.horizontal(|ui| {
                            let path_label = ui.add(
                                egui::Label::new(egui::RichText::new(&p.exe_path).color(colors.text_primary))
                                    .truncate()
                            );
                            path_label.on_hover_text(&p.exe_path);
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("📋 Kopyala").clicked() {
                                    ui.ctx().copy_text(p.exe_path.clone());
                                }
                            });
                        });
                    }

                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Kapat").clicked() {
                                close_clicked = true;
                            }
                        });
                    });
                });
            });

            if !open || close_clicked {
                self.show_properties = None;
            }
        }
    }

}

fn format_bytes(bytes: u64) -> String {
    let kb = bytes as f64 / 1024.0;
    let mb = kb / 1024.0;
    let gb = mb / 1024.0;

    if gb > 1.0 {
        format!("{:.1} GB", gb)
    } else if mb > 1.0 {
        format!("{:.1} MB", mb)
    } else if kb > 1.0 {
        format!("{:.1} KB", kb)
    } else {
        format!("{} B", bytes)
    }
}

fn get_process_icon(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("firefox") { "🦊" }
    else if lower.contains("chrome") || lower.contains("chromium") { "🌐" }
    else if lower.contains("code") || lower.contains("cursor") || lower.contains("vscode") { "💻" }
    else if lower.contains("discord") { "💬" }
    else if lower.contains("steam") { "🎮" }
    else if lower.contains("spotify") { "🎵" }
    else if lower.contains("bash") || lower.contains("zsh") || lower.contains("fish") || lower.contains("sh") { "🐚" }
    else if lower.contains("cargo") || lower.contains("rust") { "🦀" }
    else if lower.contains("python") { "🐍" }
    else if lower.contains("docker") { "🐳" }
    else if lower.contains("git") { "🐙" }
    else if lower.contains("slack") { "💬" }
    else if lower.contains("kitty") || lower.contains("alacritty") || lower.contains("terminal") { "📟" }
    else if lower.contains("systemd") || lower.contains("kthreadd") || lower.contains("udevd") || lower.contains("dbus") { "⚙" }
    else if lower.contains("ae_taskmanager") || lower.contains("taskmanager") { "📊" }
    else { "📦" }
}

/// Recursively structures the flat process tree list based on parent/child relationships.
///
/// Builds a parent PID mapping index, sorts child siblings, filters out isolated
/// roots, and traverses nodes recursively while tracking depth indices and expansion toggles.
fn build_flat_tree(
    processes: &[ProcessInfo],
    expanded_pids: &std::collections::HashSet<u32>,
    search_query: &str,
    sort_column: SortColumn,
    sort_direction: SortDirection,
) -> Vec<(ProcessInfo, usize, bool, bool)> {
    let mut parent_to_children = std::collections::HashMap::new();
    for p in processes {
        if let Some(parent) = p.parent_pid {
            parent_to_children.entry(parent).or_insert_with(Vec::new).push(p.clone());
        }
    }

    // Sort siblings
    for children in parent_to_children.values_mut() {
        children.sort_by(|a, b| {
            let ordering = match sort_column {
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::Cpu => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Memory => a.memory_bytes.cmp(&b.memory_bytes),
                SortColumn::GpuMemory => a.gpu_memory_bytes.cmp(&b.gpu_memory_bytes),
            };
            if sort_direction == SortDirection::Descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    let all_pids: std::collections::HashSet<u32> = processes.iter().map(|p| p.pid).collect();
    let mut roots: Vec<ProcessInfo> = processes.iter()
        .filter(|p| {
            p.parent_pid.is_none() || !all_pids.contains(&p.parent_pid.unwrap())
        })
        .cloned()
        .collect();

    roots.sort_by(|a, b| {
        let ordering = match sort_column {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Pid => a.pid.cmp(&b.pid),
            SortColumn::Cpu => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Memory => a.memory_bytes.cmp(&b.memory_bytes),
            SortColumn::GpuMemory => a.gpu_memory_bytes.cmp(&b.gpu_memory_bytes),
        };
        if sort_direction == SortDirection::Descending {
            ordering.reverse()
        } else {
            ordering
        }
    });

    let mut flat_list = Vec::new();

    fn add_node(
        node: &ProcessInfo,
        indent: usize,
        parent_to_children: &std::collections::HashMap<u32, Vec<ProcessInfo>>,
        expanded_pids: &std::collections::HashSet<u32>,
        flat_list: &mut Vec<(ProcessInfo, usize, bool, bool)>,
    ) {
        let has_children = parent_to_children.contains_key(&node.pid);
        let expanded = expanded_pids.contains(&node.pid);

        flat_list.push((node.clone(), indent, expanded, has_children));

        if has_children && expanded {
            if let Some(children) = parent_to_children.get(&node.pid) {
                for child in children {
                    add_node(child, indent + 1, parent_to_children, expanded_pids, flat_list);
                }
            }
        }
    }

    for root in &roots {
        add_node(root, 0, &parent_to_children, expanded_pids, &mut flat_list);
    }

    if !search_query.is_empty() {
        let query = search_query.to_lowercase();
        flat_list.retain(|(p, _, _, _)| {
            p.name.to_lowercase().contains(&query) || p.pid.to_string().contains(&query)
        });
    }

    flat_list
}

