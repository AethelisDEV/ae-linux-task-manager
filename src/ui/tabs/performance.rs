use eframe::egui;
use crate::system::SystemState;
use crate::ui::theme::{ThemeColors, card_style};
use crate::ui::widgets::chart::PerformanceChart;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfSubTab {
    Cpu,
    Memory,
    Disk,
    Network,
    Gpu,
}

pub struct PerformanceTab {
    pub active_subtab: PerfSubTab,
}

impl PerformanceTab {
    pub fn new() -> Self {
        Self {
            active_subtab: PerfSubTab::Cpu,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &SystemState, colors: &ThemeColors) {
        ui.horizontal(|ui| {
            // Left list of sub-tabs (Miniature summary cards)
            ui.vertical(|ui| {
                ui.set_width(180.0);
                ui.spacing_mut().item_spacing.y = 8.0;

                // 1. CPU Card
                let is_cpu = self.active_subtab == PerfSubTab::Cpu;
                let cpu_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new(format!("💻  CPU\n      {:.1}%", state.cpu.global_usage))
                            .strong()
                            .color(if is_cpu { colors.text_primary } else { colors.text_secondary })
                    )
                    .fill(if is_cpu { colors.accent } else { colors.bg_card })
                    .min_size(egui::vec2(170.0, 50.0))
                );
                if cpu_btn.clicked() { self.active_subtab = PerfSubTab::Cpu; }

                // 2. Memory Card
                let is_mem = self.active_subtab == PerfSubTab::Memory;
                let used_gb = state.memory.used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let total_gb = state.memory.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let mem_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new(format!("🧠  Memory\n      {:.1} / {:.1} GB", used_gb, total_gb))
                            .strong()
                            .color(if is_mem { colors.text_primary } else { colors.text_secondary })
                    )
                    .fill(if is_mem { colors.accent } else { colors.bg_card })
                    .min_size(egui::vec2(170.0, 50.0))
                );
                if mem_btn.clicked() { self.active_subtab = PerfSubTab::Memory; }

                // 3. GPU Card
                let is_gpu = self.active_subtab == PerfSubTab::Gpu;
                let gpu_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new(format!("🎮  GPU\n      {:.1}%", state.gpu.usage_percent))
                            .strong()
                            .color(if is_gpu { colors.text_primary } else { colors.text_secondary })
                    )
                    .fill(if is_gpu { colors.accent } else { colors.bg_card })
                    .min_size(egui::vec2(170.0, 50.0))
                );
                if gpu_btn.clicked() { self.active_subtab = PerfSubTab::Gpu; }

                // 4. Disk Card
                let is_disk = self.active_subtab == PerfSubTab::Disk;
                let disk_r_str = format_speed(state.disk.read_bytes_sec);
                let disk_w_str = format_speed(state.disk.write_bytes_sec);
                let disk_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new(format!("💾  Disk IO\n      R:{} W:{}", disk_r_str, disk_w_str))
                            .strong()
                            .color(if is_disk { colors.text_primary } else { colors.text_secondary })
                    )
                    .fill(if is_disk { colors.accent } else { colors.bg_card })
                    .min_size(egui::vec2(170.0, 50.0))
                );
                if disk_btn.clicked() { self.active_subtab = PerfSubTab::Disk; }

                // 5. Network Card
                let is_net = self.active_subtab == PerfSubTab::Network;
                let net_rx_str = format_speed(state.network.rx_bytes_sec);
                let net_tx_str = format_speed(state.network.tx_bytes_sec);
                let net_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new(format!("🌐  Network\n      ▼:{} ▲:{}", net_rx_str, net_tx_str))
                            .strong()
                            .color(if is_net { colors.text_primary } else { colors.text_secondary })
                    )
                    .fill(if is_net { colors.accent } else { colors.bg_card })
                    .min_size(egui::vec2(170.0, 50.0))
                );
                if net_btn.clicked() { self.active_subtab = PerfSubTab::Network; }
            });

            ui.add_space(10.0);

            // Right side: Main detailed chart and specs
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());

                // Renders active tab chart in a beautiful card
                card_style(colors).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    
                    match self.active_subtab {
                        PerfSubTab::Cpu => {
                            let chart = PerformanceChart::new("CPU Usage History", "%", 100.0, colors.accent);
                            chart.draw(ui, &state.cpu.history);
                        }
                        PerfSubTab::Memory => {
                            let chart = PerformanceChart::new("Memory Usage History", "%", 100.0, colors.success);
                            chart.draw(ui, &state.memory.history);
                        }
                        PerfSubTab::Disk => {
                            // Find the max speed in history to scale graph dynamically
                            let max_r = state.disk.read_history.iter().fold(1.0f32, |m, &v| m.max(v));
                            let max_w = state.disk.write_history.iter().fold(1.0f32, |m, &v| m.max(v));
                            let max_val = max_r.max(max_w) * 1.2; // 20% breathing room
                            
                            let chart = PerformanceChart::new("Disk I/O Activity", "KB/s", max_val, colors.warning);
                            // Draw read stats rolling history
                            chart.draw(ui, &state.disk.read_history);
                        }
                        PerfSubTab::Network => {
                            let max_rx = state.network.rx_history.iter().fold(1.0f32, |m, &v| m.max(v));
                            let max_tx = state.network.tx_history.iter().fold(1.0f32, |m, &v| m.max(v));
                            let max_val = max_rx.max(max_tx) * 1.2; // 20% breathing room

                            let chart = PerformanceChart::new("Network Bandwidth (Rx/Tx)", "KB/s", max_val, colors.danger);
                            chart.draw(ui, &state.network.rx_history);
                        }
                        PerfSubTab::Gpu => {
                            let chart = PerformanceChart::new("GPU Load History", "%", 100.0, colors.accent);
                            chart.draw(ui, &state.gpu.history);
                        }
                    }
                });

                ui.add_space(10.0);

                // Specifications Panel in a card
                card_style(colors).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height() - 10.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        match self.active_subtab {
                            PerfSubTab::Cpu => {
                                ui.heading(egui::RichText::new("Processor Specifications").color(colors.text_primary));
                                ui.add_space(10.0);

                                grid_row(ui, "CPU Model", &state.cpu.model, colors);
                                grid_row(ui, "Average Clock Speed", &format!("{} MHz", state.cpu.avg_frequency_mhz), colors);
                                grid_row(ui, "Physical Cores", &state.cpu.physical_cores.to_string(), colors);
                                grid_row(ui, "Logical Cores", &state.cpu.logical_cores.to_string(), colors);
                                grid_row(ui, "System Uptime", &format_uptime(state.uptime), colors);
                            }
                            PerfSubTab::Memory => {
                                ui.heading(egui::RichText::new("Memory Allocations").color(colors.text_primary));
                                ui.add_space(10.0);

                                let to_gb = |b| b as f64 / (1024.0 * 1024.0 * 1024.0);
                                grid_row(ui, "Physical Total", &format!("{:.2} GB", to_gb(state.memory.total_bytes)), colors);
                                grid_row(ui, "Physical Used", &format!("{:.2} GB", to_gb(state.memory.used_bytes)), colors);
                                grid_row(ui, "Physical Free", &format!("{:.2} GB", to_gb(state.memory.total_bytes - state.memory.used_bytes)), colors);
                                grid_row(ui, "Swap Space Total", &format!("{:.2} GB", to_gb(state.memory.swap_total_bytes)), colors);
                                grid_row(ui, "Swap Space Used", &format!("{:.2} GB", to_gb(state.memory.swap_used_bytes)), colors);
                            }
                            PerfSubTab::Disk => {
                                ui.heading(egui::RichText::new("Disk & Mount Interfaces").color(colors.text_primary));
                                ui.add_space(10.0);

                                if state.disk.partitions.is_empty() {
                                    ui.label("No active mount points found.");
                                } else {
                                    for part in &state.disk.partitions {
                                        let to_gb = |b| b as f64 / (1024.0 * 1024.0 * 1024.0);
                                        let details = format!(
                                            "{} | Total: {:.1} GB | Free: {:.1} GB ({:.1}%)",
                                            part.file_system,
                                            to_gb(part.total_space),
                                            to_gb(part.available_space),
                                            if part.total_space > 0 { (part.available_space as f32 / part.total_space as f32) * 100.0 } else { 0.0 }
                                        );
                                        grid_row(ui, &format!("{} ({})", part.name, part.mount_point), &details, colors);
                                    }
                                }
                            }
                            PerfSubTab::Network => {
                                ui.heading(egui::RichText::new("Active Interfaces").color(colors.text_primary));
                                ui.add_space(10.0);

                                for iface in &state.network.interfaces {
                                    let to_mb = |b| b as f64 / (1024.0 * 1024.0);
                                    let details = format!(
                                        "Down: {} | Up: {} | Total: Rx {:.1} MB / Tx {:.1} MB",
                                        format_speed(iface.rx_bytes_sec),
                                        format_speed(iface.tx_bytes_sec),
                                        to_mb(iface.total_rx),
                                        to_mb(iface.total_tx)
                                    );
                                    grid_row(ui, &iface.name, &details, colors);
                                }
                            }
                            PerfSubTab::Gpu => {
                                ui.heading(egui::RichText::new("Graphics Processor Specifications").color(colors.text_primary));
                                ui.add_space(10.0);

                                let to_gb = |b| b as f64 / (1024.0 * 1024.0 * 1024.0);
                                grid_row(ui, "GPU Model", &state.gpu.model, colors);
                                grid_row(ui, "Brand / Vendor", &state.gpu.brand, colors);
                                grid_row(ui, "Video RAM Total", &format!("{:.2} GB", to_gb(state.gpu.memory_total_bytes)), colors);
                                grid_row(ui, "Video RAM Used", &format!("{:.2} GB", to_gb(state.gpu.memory_used_bytes)), colors);
                                grid_row(ui, "Video RAM Free", &format!("{:.2} GB", to_gb(state.gpu.memory_total_bytes.saturating_sub(state.gpu.memory_used_bytes))), colors);
                            }
                        }
                    });
                });
            });
        });
    }
}

fn grid_row(ui: &mut egui::Ui, title: &str, value: &str, colors: &ThemeColors) {
    ui.horizontal(|ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(title).color(colors.text_secondary).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).color(colors.text_primary));
        });
    });
    ui.add_space(2.0);
    ui.colored_label(colors.border, "───────────────────────────────────────────────────────────────────────");
    ui.add_space(2.0);
}

fn format_speed(bytes_sec: u64) -> String {
    let kb = bytes_sec as f64 / 1024.0;
    let mb = kb / 1024.0;

    if mb > 1.0 {
        format!("{:.1} MB/s", mb)
    } else if kb > 1.0 {
        format!("{:.1} KB/s", kb)
    } else {
        format!("{} B/s", bytes_sec)
    }
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else {
        format!("{}m {}s", minutes, seconds)
    }
}
