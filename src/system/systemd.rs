/// Systemd Services Management Backend Module
///
/// This module provides safe, decoupled abstractions for querying systemd service units
/// and executing administrative lifecycle actions (start, stop, restart, enable, disable)
/// using graphical privilege escalation.

use std::process::Command;

/// Represents detailed load and activation state metrics of a Systemd service unit.
#[derive(Debug, Clone, Default)]
pub struct ServiceInfo {
    /// The unique systemd unit identifier (e.g., `bluetooth.service`).
    pub name: String,
    /// The load status indicating if the unit configuration file is loaded (e.g., `loaded`, `not-found`).
    pub load_state: String,
    /// The high-level activation state indicating operational status (e.g., `active`, `inactive`, `failed`).
    pub active_state: String,
    /// The low-level operational activation sub-state (e.g., `running`, `dead`, `exited`).
    pub sub_state: String,
    /// A short description outlining the service's system purpose.
    pub description: String,
}

/// Queries the local Systemd manager via `systemctl` to retrieve a list of all installed service units.
///
/// Executes `systemctl list-units --type=service --all --no-legend --no-pager` in a synchronous child process.
/// Parses the tabular white-space delimited output columns into a structured `ServiceInfo` list.
///
/// # Errors
/// Returns `Err(String)` if `systemctl` fails to execute, terminates with non-zero exit code,
/// or returns standard error telemetry.
pub fn list_systemd_services() -> Result<Vec<ServiceInfo>, String> {
    let output = Command::new("systemctl")
        .args(&["list-units", "--type=service", "--all", "--no-legend", "--no-pager"])
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut services = Vec::new();

    for line in stdout_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let name = parts[0].to_string();
        let load_state = parts[1].to_string();
        let active_state = parts[2].to_string();
        let sub_state = parts[3].to_string();
        
        let description = if parts.len() > 4 {
            parts[4..].join(" ")
        } else {
            String::new()
        };

        services.push(ServiceInfo {
            name,
            load_state,
            active_state,
            sub_state,
            description,
        });
    }

    Ok(services)
}

/// Performs a lifecycle command on a targeted Systemd service unit with escalated root privileges.
///
/// Graphically prompts the desktop session user for administrative authentication using Polkit (`pkexec`),
/// then executes `systemctl <action> <service>`.
///
/// Valid actions are: `"start"`, `"stop"`, `"restart"`, `"enable"`, `"disable"`.
///
/// # Errors
/// Returns `Err(String)` if:
/// - The requested action is invalid (security check).
/// - The graphical authentication dialog is cancelled or denied.
/// - The underlying `systemctl` operation returns an execution failure.
pub fn manage_service(service: &str, action: &str) -> Result<(), String> {
    let valid_actions = ["start", "stop", "restart", "enable", "disable"];
    if !valid_actions.contains(&action) {
        return Err("Invalid systemd action requested".to_string());
    }

    let output = Command::new("pkexec")
        .args(&["systemctl", action, service])
        .output()
        .map_err(|e| format!("Failed to spawn pkexec: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        let err = if err.trim().is_empty() {
            "Yetkilendirme iptal edildi veya başarısız oldu (Authorization cancelled or failed)".to_string()
        } else {
            err
        };
        Err(err)
    }
}
