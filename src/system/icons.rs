use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct IconResolver {
    // Maps binary/process name (lowercase) to icon name (e.g. "firefox" -> "firefox", "nautilus" -> "org.gnome.Nautilus")
    binary_to_icon: HashMap<String, String>,
    // Maps icon name (lowercase) to resolved absolute path (e.g. "firefox" -> "/usr/share/icons/.../firefox.png")
    resolved_paths: HashMap<String, Option<String>>,
    // User's active theme
    active_theme: Option<String>,
}

impl IconResolver {
    pub fn new() -> Self {
        let active_theme = get_active_icon_theme();
        let mut resolver = Self {
            binary_to_icon: HashMap::new(),
            resolved_paths: HashMap::new(),
            active_theme,
        };

        resolver.scan_desktop_files();
        resolver
    }

    /// Scans standard Linux .desktop locations to associate binary/executable names with icon names
    fn scan_desktop_files(&mut self) {
        let mut search_dirs = Vec::new();
        search_dirs.push(PathBuf::from("/usr/share/applications"));
        search_dirs.push(PathBuf::from("/usr/local/share/applications"));
        if let Ok(home) = std::env::var("HOME") {
            search_dirs.push(PathBuf::from(&home).join(".local/share/applications"));
        }

        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "desktop") {
                        self.parse_desktop_file(&path);
                    }
                }
            }
        }
    }

    /// Parses a single .desktop file to extract Exec and Icon values
    fn parse_desktop_file(&mut self, path: &Path) {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return,
        };

        let reader = BufReader::new(file);
        let mut current_exec = None;
        let mut current_icon = None;
        let mut in_desktop_entry = false;

        for line in reader.lines().flatten() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                if trimmed == "[Desktop Entry]" {
                    in_desktop_entry = true;
                } else {
                    in_desktop_entry = false;
                }
            }

            if !in_desktop_entry {
                continue;
            }

            if trimmed.starts_with("Exec=") {
                current_exec = extract_bin_name(trimmed);
            } else if trimmed.starts_with("Icon=") {
                let icon_val = trimmed.strip_prefix("Icon=").unwrap_or(trimmed).trim();
                let cleaned_icon = icon_val.trim_matches('"').trim_matches('\'').to_string();
                if !cleaned_icon.is_empty() {
                    current_icon = Some(cleaned_icon);
                }
            }

            // Once both are found, store them and return
            if current_exec.is_some() && current_icon.is_some() {
                break;
            }
        }

        if let (Some(exec), Some(icon)) = (current_exec, current_icon) {
            self.binary_to_icon.insert(exec, icon);
        }
    }

    /// Resolves a process name (or executable path) to its matching icon on disk.
    /// Uses cache to guarantee O(1) performance on subsequent queries.
    pub fn get_icon_path(&mut self, process_name: &str, exe_path: &str) -> Option<String> {
        let name_lower = process_name.to_lowercase();
        
        // 1. Check if we already resolved this process name
        if let Some(cached) = self.resolved_paths.get(&name_lower) {
            return cached.clone();
        }

        // Extract executable filename if path is available
        let exe_lower = if !exe_path.is_empty() {
            Path::new(exe_path)
                .file_name()
                .map(|f| f.to_string_lossy().to_lowercase())
        } else {
            None
        };

        // 2. Determine the icon name to search for
        let mut search_icon_name = None;

        // Try mapping exe_name first, then process_name
        if let Some(ref exe) = exe_lower {
            if let Some(icon) = self.binary_to_icon.get(exe) {
                search_icon_name = Some(icon.clone());
            }
        }

        if search_icon_name.is_none() {
            if let Some(icon) = self.binary_to_icon.get(&name_lower) {
                search_icon_name = Some(icon.clone());
            }
        }

        // 2b. Smart substring matching fallback (e.g. spotify-helper -> spotify, firefox-bin -> firefox)
        if search_icon_name.is_none() {
            for (key, val) in &self.binary_to_icon {
                if key.len() > 2 { // Avoid extremely short keys matching everything
                    if name_lower.contains(key) || (exe_lower.is_some() && exe_lower.as_ref().unwrap().contains(key)) {
                        search_icon_name = Some(val.clone());
                        break;
                    }
                }
            }
        }

        // Fallback to process/exe name itself as the icon name
        let icon_name = search_icon_name.unwrap_or_else(|| {
            exe_lower.unwrap_or(name_lower)
        });

        // 3. Resolve icon name to a file path
        let resolved = resolve_icon_to_path(&icon_name, self.active_theme.as_deref());
        
        // 4. Cache and return the result
        let result_str = resolved.map(|p| format!("file://{}", p.to_string_lossy()));
        self.resolved_paths.insert(process_name.to_lowercase(), result_str.clone());
        result_str
    }
}

/// Extract binary name from a desktop file's Exec line
fn extract_bin_name(exec_line: &str) -> Option<String> {
    let stripped = exec_line.strip_prefix("Exec=").unwrap_or(exec_line).trim();
    if stripped.is_empty() {
        return None;
    }
    
    let first_token = if stripped.starts_with('"') {
        let parts: Vec<&str> = stripped[1..].split('"').collect();
        parts.first().cloned().unwrap_or("")
    } else {
        let parts: Vec<&str> = stripped.split_whitespace().collect();
        parts.first().cloned().unwrap_or("")
    };

    if first_token.is_empty() {
        return None;
    }

    let path = Path::new(first_token);
    let bin_name = path.file_name()?.to_string_lossy().to_string();

    Some(bin_name.to_lowercase())
}

/// Retrieve user's active GTK or GNOME icon theme
fn get_active_icon_theme() -> Option<String> {
    if let Ok(home) = std::env::var("HOME") {
        let settings_path = PathBuf::from(&home).join(".config/gtk-3.0/settings.ini");
        if settings_path.exists() {
            if let Ok(file) = File::open(settings_path) {
                let reader = BufReader::new(file);
                for line in reader.lines().flatten() {
                    if line.trim().starts_with("gtk-icon-theme-name") {
                        let parts: Vec<&str> = line.split('=').collect();
                        if parts.len() >= 2 {
                            return Some(parts[1].trim().trim_matches('"').trim_matches('\'').to_string());
                        }
                    }
                }
            }
        }
    }

    // Fallback: Check org.gnome.desktop.interface icon-theme
    let output = std::process::Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "icon-theme"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let theme = String::from_utf8_lossy(&out.stdout)
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            if !theme.is_empty() {
                return Some(theme);
            }
        }
    }

    None
}

/// Walks theme/fallback structures to locate the icon file
fn resolve_icon_to_path(icon_name: &str, active_theme: Option<&str>) -> Option<PathBuf> {
    let path = Path::new(icon_name);
    if path.is_absolute() && path.exists() {
        return Some(path.to_path_buf());
    }

    // Build themes list to check in priority order
    let mut themes = Vec::new();
    if let Some(t) = active_theme {
        themes.push(t.to_string());
    }
    for t in &["hicolor", "Papirus", "Yaru", "breeze", "Adwaita", "gnome"] {
        let ts = t.to_string();
        if !themes.contains(&ts) {
            themes.push(ts);
        }
    }

    // Search folders
    let mut base_dirs = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        base_dirs.push(PathBuf::from(&home).join(".local/share/icons"));
        base_dirs.push(PathBuf::from(&home).join(".icons"));
    }
    base_dirs.push(PathBuf::from("/usr/share/icons"));
    base_dirs.push(PathBuf::from("/usr/local/share/icons"));

    let pixmaps_dir = PathBuf::from("/usr/share/pixmaps");
    let extensions = &["png", "svg", "jpg", "jpeg", "xpm"];

    // Construct paths to check directly (ultra fast)
    for theme in &themes {
        for base in &base_dirs {
            let theme_dir = base.join(theme);
            if !theme_dir.exists() {
                continue;
            }

            let subpaths = &[
                "48x48/apps",
                "scalable/apps",
                "32x32/apps",
                "64x64/apps",
                "128x128/apps",
                "256x256/apps",
                "512x512/apps",
                "apps",
                "apps/48",
                "apps/scalable",
            ];

            for sub in subpaths {
                let folder = theme_dir.join(sub);
                if !folder.exists() {
                    continue;
                }

                for ext in extensions {
                    let file_path = folder.join(format!("{}.{}", icon_name, ext));
                    if file_path.exists() {
                        return Some(file_path);
                    }
                }
            }
        }
    }

    // Fallback to direct file in /usr/share/pixmaps/
    for ext in extensions {
        let file_path = pixmaps_dir.join(format!("{}.{}", icon_name, ext));
        if file_path.exists() {
            return Some(file_path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_resolution() {
        let mut resolver = IconResolver::new();
        println!("Active theme: {:?}", resolver.active_theme);
        println!("Mapped desktop entries: {}", resolver.binary_to_icon.len());
        
        // Print some mapped desktop entries
        for (_, (k, v)) in resolver.binary_to_icon.iter().enumerate().take(10) {
            println!("  Desktop map: {} -> {}", k, v);
        }

        let test_bins = &["firefox", "chrome", "nautilus", "code", "cargo", "bash", "ae_taskmanager", "kitty", "alacritty", "steam", "spotify", "discord"];
        for bin in test_bins {
            let path = resolver.get_icon_path(bin, "");
            println!("Process '{}' icon path: {:?}", bin, path);
        }
    }
}
