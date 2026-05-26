use sysinfo::{System, ProcessRefreshKind, RefreshKind};
use std::collections::HashMap;
use std::process::Command;

#[derive(Clone, Debug, Default)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub status: String,
    pub username: String,
    pub exe_path: String,
    pub icon_path: Option<String>,
    pub gpu_memory_bytes: u64,
}

#[derive(Clone)]
pub struct ProcessManager {
    pub list: Vec<ProcessInfo>,
    resolver: crate::system::icons::IconResolver,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            resolver: crate::system::icons::IconResolver::new(),
        }
    }

    pub fn refresh_spec() -> RefreshKind {
        RefreshKind::new().with_processes(
            ProcessRefreshKind::new()
                .with_cpu()
                .with_memory()
                .with_user(sysinfo::UpdateKind::Always)
        )
    }

    pub fn update(&mut self, sys: &System, users: &sysinfo::Users, gpu_mem: &HashMap<u32, u64>) {
        // Build user map
        let mut user_map = HashMap::new();
        for user in users.iter() {
            user_map.insert(user.id().clone(), user.name().to_string());
        }

        let mut list = Vec::with_capacity(sys.processes().len());
        for (pid, proc) in sys.processes().iter() {
            let pid_u32 = pid.as_u32();
            let parent_pid = proc.parent().map(|p| p.as_u32());
            let name = proc.name().to_string_lossy().to_string();
            let cpu_usage = proc.cpu_usage();
            let memory_bytes = proc.memory();
            
            // Format status to human-readable string
            let status = format!("{:?}", proc.status());

            // Map user ID to string
            let username = if let Some(uid) = proc.user_id() {
                user_map.get(uid).cloned().unwrap_or_else(|| uid.to_string())
            } else {
                "root".to_string()
            };

            let exe_path = proc.exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let icon_path = self.resolver.get_icon_path(&name, &exe_path);
            let gpu_memory_bytes = gpu_mem.get(&pid_u32).cloned().unwrap_or(0);

            list.push(ProcessInfo {
                pid: pid_u32,
                parent_pid,
                name,
                cpu_usage,
                memory_bytes,
                status,
                username,
                exe_path,
                icon_path,
                gpu_memory_bytes,
            });
        }
        self.list = list;
    }

    /// Terminates a process by sending SIGTERM (kill -15).
    /// If forceful is true, sends SIGKILL (kill -9).
    pub fn kill_process(pid: u32, forceful: bool) -> Result<(), String> {
        let signal = if forceful { "-9" } else { "-15" };
        
        let output = Command::new("kill")
            .arg(signal)
            .arg(pid.to_string())
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&out.stderr).to_string();
                    Err(if err.is_empty() { "Unknown error (perhaps permission denied)".to_string() } else { err })
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Escalates privileges graphically via pkexec and kills the process with SIGKILL (kill -9).
    pub fn kill_process_as_admin(pid: u32) -> Result<(), String> {
        let output = Command::new("pkexec")
            .arg("kill")
            .arg("-9")
            .arg(pid.to_string())
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&out.stderr).to_string();
                    Err(if err.trim().is_empty() {
                        "Yetkilendirme iptal edildi veya başarısız oldu (Authentication cancelled/failed)".to_string()
                    } else {
                        err
                    })
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

