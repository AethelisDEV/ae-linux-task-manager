pub mod wayland;
pub mod x11;

pub use wayland::WaylandDiagnostics;
pub use x11::X11Diagnostics;

use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    Wayland,
    X11,
    XWayland,
    Unknown,
}

impl DisplayServer {
    pub fn detect() -> Self {
        let xdg_session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default().to_lowercase();
        let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
        let display = env::var("DISPLAY").unwrap_or_default();

        if xdg_session_type == "wayland" || !wayland_display.is_empty() {
            // Check if X11 is also present, which would indicate XWayland availability
            if !display.is_empty() {
                DisplayServer::XWayland
            } else {
                DisplayServer::Wayland
            }
        } else if xdg_session_type == "x11" || !display.is_empty() {
            DisplayServer::X11
        } else {
            DisplayServer::Unknown
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DisplayServer::Wayland => "Wayland (Native)".to_string(),
            DisplayServer::X11 => "X11 (Native)".to_string(),
            DisplayServer::XWayland => "Wayland (with XWayland compatibility)".to_string(),
            DisplayServer::Unknown => "Unknown Display Server".to_string(),
        }
    }
}

pub struct PlatformInfo {
    pub display_server: DisplayServer,
    pub wayland: WaylandDiagnostics,
    pub x11: X11Diagnostics,
}

impl PlatformInfo {
    pub fn collect() -> Self {
        Self {
            display_server: DisplayServer::detect(),
            wayland: WaylandDiagnostics::collect(),
            x11: X11Diagnostics::collect(),
        }
    }
}
