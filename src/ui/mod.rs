pub mod sidebar;
pub mod tabs {
    pub mod about;
    pub mod performance;
    pub mod processes;
    pub mod services;
    pub mod startup;
    pub mod connections;
    pub mod file_locks;
}
pub mod theme;
pub mod widgets {
    pub mod chart;
}

use eframe::egui;
use crate::system::SystemState;
use sidebar::{render_sidebar, Tab};
use tabs::about::AboutTab;
use tabs::performance::PerformanceTab;
use tabs::processes::ProcessesTab;
use theme::ThemeColors;

pub struct TaskManagerUi {
    pub current_tab: Tab,
    pub processes_tab: ProcessesTab,
    pub performance_tab: PerformanceTab,
    pub services_tab: tabs::services::ServicesTab,
    pub startup_tab: tabs::startup::StartupTab,
    pub connections_tab: tabs::connections::ConnectionsTab,
    pub file_locks_tab: tabs::file_locks::FileLocksTab,
    pub about_tab: AboutTab,
    pub colors: ThemeColors,
}

impl TaskManagerUi {
    pub fn new() -> Self {
        Self {
            current_tab: Tab::Processes,
            processes_tab: ProcessesTab::new(),
            performance_tab: PerformanceTab::new(),
            services_tab: tabs::services::ServicesTab::new(),
            startup_tab: tabs::startup::StartupTab::new(),
            connections_tab: tabs::connections::ConnectionsTab::new(),
            file_locks_tab: tabs::file_locks::FileLocksTab::new(),
            about_tab: AboutTab::new(),
            colors: ThemeColors::dark(),
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context, state: &SystemState) {
        // Render application sidebar
        egui::SidePanel::left("navigation_sidebar")
            .resizable(false)
            .default_width(180.0)
            .frame(
                egui::Frame::none()
                    .fill(self.colors.bg_sidebar)
                    .stroke(egui::Stroke::new(1.0, self.colors.border))
            )
            .show(ctx, |ui| {
                ui.add_space(10.0);
                render_sidebar(ui, &mut self.current_tab, &self.colors);
            });

        // Renders the main dashboard body based on selected tab
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.colors.bg_dark)
                    .inner_margin(16.0)
            )
            .show(ctx, |ui| {
                match self.current_tab {
                    Tab::Processes => {
                        self.processes_tab.render(ui, &state.processes.list, state.last_update, &self.colors);
                    }
                    Tab::Performance => {
                        self.performance_tab.render(ui, state, &self.colors);
                    }
                    Tab::Services => {
                        self.services_tab.render(ui, &self.colors);
                    }
                    Tab::Startup => {
                        self.startup_tab.render(ui, &self.colors);
                    }
                    Tab::Connections => {
                        self.connections_tab.render(ui, &self.colors);
                    }
                    Tab::FileLocks => {
                        self.file_locks_tab.render(ui, &self.colors);
                    }
                    Tab::About => {
                        self.about_tab.render(ui, state, &self.colors);
                    }
                }
            });
    }
}

