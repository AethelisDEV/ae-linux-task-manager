use std::env;

pub struct X11Diagnostics {
    pub is_active: bool,
    pub display_name: String,
    pub xauthority: Option<String>,
    pub window_manager: String,
}

impl X11Diagnostics {
    pub fn collect() -> Self {
        let display = env::var("DISPLAY").unwrap_or_default();
        let xauthority = env::var("XAUTHORITY").ok();
        let xdg_session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default().to_lowercase();
        
        // Sometimes DISPLAY is set even under Wayland (for XWayland), so we check XDG_SESSION_TYPE.
        let is_active = !display.is_empty() && xdg_session_type != "wayland";

        let xdg_current_desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
        let window_manager = if xdg_current_desktop.contains("gnome") {
            "Mutter (GNOME)".to_string()
        } else if xdg_current_desktop.contains("kde") {
            "KWin (KDE)".to_string()
        } else if xdg_current_desktop.contains("xfce") {
            "Xfwm4 (XFCE)".to_string()
        } else if xdg_current_desktop.contains("i3") {
            "i3wm".to_string()
        } else if xdg_current_desktop.contains("openbox") {
            "Openbox".to_string()
        } else if is_active {
            "Generic X11 Window Manager".to_string()
        } else {
            "None".to_string()
        };

        Self {
            is_active,
            display_name: if display.is_empty() { "N/A".to_string() } else { display },
            xauthority,
            window_manager,
        }
    }
}
