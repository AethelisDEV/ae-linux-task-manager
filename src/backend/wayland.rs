use std::env;
use std::path::PathBuf;

pub struct WaylandDiagnostics {
    pub is_active: bool,
    pub socket_name: String,
    pub socket_path: Option<String>,
    pub compositor: String,
}

impl WaylandDiagnostics {
    pub fn collect() -> Self {
        let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_default();
        let is_active = !wayland_display.is_empty();

        let socket_path = if is_active && !xdg_runtime_dir.is_empty() {
            let mut path = PathBuf::from(&xdg_runtime_dir);
            path.push(&wayland_display);
            if path.exists() {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Heuristics for compositor
        let xdg_current_desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
        let compositor = if xdg_current_desktop.contains("gnome") {
            "Mutter (GNOME)".to_string()
        } else if xdg_current_desktop.contains("kde") {
            "KWin (KDE)".to_string()
        } else if xdg_current_desktop.contains("sway") {
            "Sway".to_string()
        } else if xdg_current_desktop.contains("hyprland") {
            "Hyprland".to_string()
        } else if is_active {
            "Generic Wayland Compositor".to_string()
        } else {
            "None".to_string()
        };

        Self {
            is_active,
            socket_name: if wayland_display.is_empty() { "N/A".to_string() } else { wayland_display },
            socket_path,
            compositor,
        }
    }
}
