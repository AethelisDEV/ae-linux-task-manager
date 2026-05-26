/// XDG Autostart Startup Applications Backend Module
///
/// This module implements the official Freedesktop XDG Autostart specification.
/// It enables scanning user-specific and system-wide startup applications (`.desktop` entries)
/// and toggling their activation states via copy-on-write local configuration overrides.

use std::fs;
use std::path::Path;
use std::collections::HashSet;

/// Represents an application configured to launch automatically on desktop session start.
#[derive(Debug, Clone, Default)]
pub struct StartupApp {
    /// The user-visible name of the startup application.
    pub name: String,
    /// The shell command executed to launch the application.
    pub exec: String,
    /// The absolute file path of the underlying `.desktop` configuration file.
    pub path: String,
    /// True if the application is currently active/enabled for autostart in this session.
    pub enabled: bool,
}

/// Helper function to parse an individual `.desktop` file and extract its metadata.
fn parse_desktop_file(path: &Path) -> Option<StartupApp> {
    let content = fs::read_to_string(path).ok()?;
    
    let mut name = String::new();
    let mut exec = String::new();
    let mut enabled = true;
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Name=") {
            name = line["Name=".len()..].trim().to_string();
        } else if line.starts_with("Exec=") {
            exec = line["Exec=".len()..].trim().to_string();
        } else if line.starts_with("X-GNOME-Autostart-enabled=") {
            let val = line["X-GNOME-Autostart-enabled=".len()..].trim().to_lowercase();
            if val == "false" {
                enabled = false;
            }
        } else if line.starts_with("Hidden=") {
            let val = line["Hidden=".len()..].trim().to_lowercase();
            if val == "true" {
                enabled = false;
            }
        }
    }
    
    if name.is_empty() {
        name = path.file_stem()?.to_string_lossy().to_string();
    }
    
    Some(StartupApp {
        name,
        exec,
        path: path.to_string_lossy().to_string(),
        enabled,
    })
}

/// Scans standard Linux XDG Autostart directories to compile a list of all autostart entries.
///
/// Traverses User config autostart (`~/.config/autostart/`) first, then system config autostart
/// (`/etc/xdg/autostart/`). User settings override system-wide defaults according to XDG rules.
pub fn list_startup_apps() -> Vec<StartupApp> {
    let mut apps = Vec::new();
    let mut seen_names = HashSet::new();

    // 1. Scan User Autostart: ~/.config/autostart/
    if let Ok(home) = std::env::var("HOME") {
        let user_path = Path::new(&home).join(".config/autostart");
        if let Ok(entries) = fs::read_dir(&user_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "desktop") {
                    if let Some(app) = parse_desktop_file(&path) {
                        seen_names.insert(path.file_name().unwrap().to_os_string());
                        apps.push(app);
                    }
                }
            }
        }
    }

    // 2. Scan System-wide Autostart: /etc/xdg/autostart/
    let sys_path = Path::new("/etc/xdg/autostart");
    if let Ok(entries) = fs::read_dir(sys_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "desktop") {
                let file_name = path.file_name().unwrap().to_os_string();
                // Overridden by user autostart settings if seen in ~/.config
                if !seen_names.contains(&file_name) {
                    if let Some(app) = parse_desktop_file(&path) {
                        apps.push(app);
                    }
                }
            }
        }
    }

    apps
}

/// Configures the automatic startup activation state of an application.
///
/// Follows XDG copy-on-write behavior:
/// - If editing a system-wide application, copies the `.desktop` file locally to
///   `~/.config/autostart/` and modifies it.
/// - Sets `X-GNOME-Autostart-enabled` and `Hidden` keys inside `[Desktop Entry]` cleanly.
///
/// # Errors
/// Returns `Err(String)` if files cannot be copied, read, or written due to permissions/IO errors.
pub fn set_startup_status(app: &StartupApp, enable: bool) -> Result<(), String> {
    let app_path = Path::new(&app.path);
    let file_name = app_path.file_name().ok_or_else(|| "Invalid desktop file name".to_string())?;

    let home = std::env::var("HOME").map_err(|e| format!("HOME environment variable missing: {}", e))?;
    let user_autostart_dir = Path::new(&home).join(".config/autostart");

    // Guarantee autostart directory exists
    let _ = fs::create_dir_all(&user_autostart_dir);
    let target_path = user_autostart_dir.join(file_name);

    // Copy system file locally if overridden
    if app_path.starts_with("/etc/xdg/autostart") || app_path != target_path {
        fs::copy(app_path, &target_path)
            .map_err(|e| format!("Failed to copy autostart file locally: {}", e))?;
    }

    let content = fs::read_to_string(&target_path)
        .map_err(|e| format!("Failed to read autostart file content: {}", e))?;

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut found_enabled = false;
    let mut found_hidden = false;

    for line in lines.iter_mut() {
        if line.starts_with("X-GNOME-Autostart-enabled=") {
            *line = format!("X-GNOME-Autostart-enabled={}", enable);
            found_enabled = true;
        } else if line.starts_with("Hidden=") {
            *line = format!("Hidden={}", !enable);
            found_hidden = true;
        }
    }

    if !found_enabled {
        if let Some(pos) = lines.iter().position(|l| l.trim() == "[Desktop Entry]") {
            lines.insert(pos + 1, format!("X-GNOME-Autostart-enabled={}", enable));
        } else {
            lines.push(format!("X-GNOME-Autostart-enabled={}", enable));
        }
    }
    
    if !found_hidden && !enable {
        if let Some(pos) = lines.iter().position(|l| l.trim() == "[Desktop Entry]") {
            lines.insert(pos + 1, "Hidden=true".to_string());
        } else {
            lines.push("Hidden=true".to_string());
        }
    }

    let new_content = lines.join("\n");
    fs::write(&target_path, new_content)
        .map_err(|e| format!("Failed to write autostart configuration: {}", e))?;

    Ok(())
}
