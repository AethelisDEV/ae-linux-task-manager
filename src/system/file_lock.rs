/// File Lock & Descriptor Tracking Backend Module
///
/// Implements a high-performance, zero-unsafe diagnostic utility for Linux systems.
/// Traverses open file descriptors under `/proc/*/fd/` on-demand to match target file
/// or directory paths, identifying which active processes are locking files.

use std::fs;
use std::path::Path;
use std::os::unix::fs::MetadataExt;

/// Represents process owner details locking a targeted file path.
#[derive(Debug, Clone, Default)]
pub struct FileLockInfo {
    /// The Process ID (PID) of the application holding the open file descriptor.
    pub pid: u32,
    /// The name of the process (e.g. `libreoffice`).
    pub process_name: String,
    /// The name of the system user account executing the process.
    pub username: String,
    /// The exact target path of the opened file (helps with directory sweeps).
    pub open_path: String,
}

/// Helper function to parse `/etc/passwd` and map a system UID to a username.
fn resolve_username_from_uid(uid: u32) -> String {
    if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(u) = parts[2].parse::<u32>() {
                    if u == uid {
                        return parts[0].to_string();
                    }
                }
            }
        }
    }
    uid.to_string()
}

/// Scans the entire Linux `/proc` filesystem on-demand to discover PIDs locking a target path.
///
/// If `target_path` is a directory, matches any file opened inside that directory recursively.
/// Employs canonicalized path resolution to prevent false negatives from symlinks.
pub fn find_locking_processes(target_path: &str) -> Vec<FileLockInfo> {
    let mut locks = Vec::new();

    let path_obj = Path::new(target_path);
    // Canonicalize path to handle relative sections and resolve symlinks
    let canon_target = fs::canonicalize(path_obj).unwrap_or_else(|_| path_obj.to_path_buf());

    let proc_path = Path::new("/proc");
    let entries = match fs::read_dir(proc_path) {
        Ok(e) => e,
        Err(_) => return locks,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();

        // Target numeric PID folders only
        let pid: u32 = match file_name.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let fd_path = path.join("fd");
        let fd_entries = match fs::read_dir(fd_path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Determine owner username using folder UID metadata
        let mut username = "Unknown".to_string();
        if let Ok(meta) = fs::metadata(&path) {
            username = resolve_username_from_uid(meta.uid());
        }

        let comm_path = path.join("comm");
        let process_name = fs::read_to_string(comm_path)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        // Scan descriptors inside this PID's fd folder
        for fd_entry in fd_entries.flatten() {
            if let Ok(target) = fs::read_link(fd_entry.path()) {
                let target_canon = fs::canonicalize(&target).unwrap_or(target);
                
                // Check if target matches exactly or sits within search directory
                let matches = target_canon == canon_target 
                    || target_canon.starts_with(&canon_target);

                if matches {
                    locks.push(FileLockInfo {
                        pid,
                        process_name: process_name.clone(),
                        username: username.clone(),
                        open_path: target_canon.to_string_lossy().to_string(),
                    });
                    break; // Found at least one lock for this process, skip remaining fds
                }
            }
        }
    }

    locks
}
