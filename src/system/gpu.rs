use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone)]
pub struct GpuStats {
    pub has_gpu: bool,
    pub brand: String,                     // "NVIDIA", "AMD", "Intel", "Unknown"
    pub model: String,                     // e.g. "GeForce RTX 3070"
    pub usage_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub history: Vec<f32>,                 // Rolling 60 seconds history
    pub process_memory: HashMap<u32, u64>, // Maps PID -> GPU VRAM bytes

    is_nvidia: bool,
    is_amd: bool,
    checked_presence: bool,
}

impl GpuStats {
    pub fn new() -> Self {
        Self {
            has_gpu: false,
            brand: String::from("Unknown"),
            model: String::from("No Active GPU"),
            usage_percent: 0.0,
            memory_used_bytes: 0,
            memory_total_bytes: 0,
            history: vec![0.0; 60],
            process_memory: HashMap::new(),
            is_nvidia: false,
            is_amd: false,
            checked_presence: false,
        }
    }

    /// Performs one-time detection of the GPU type to avoid checking repeatedly
    fn detect_gpu(&mut self) {
        if self.checked_presence {
            return;
        }

        // 1. Check if nvidia-smi command is present
        let has_nvidia_smi = Command::new("which")
            .arg("nvidia-smi")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if has_nvidia_smi {
            self.is_nvidia = true;
            self.brand = String::from("NVIDIA");
            self.has_gpu = true;
        } 
        // 2. Check if AMD sysfs card0 folder exists
        else if Path::new("/sys/class/drm/card0/device/gpu_busy_percent").exists() {
            self.is_amd = true;
            self.brand = String::from("AMD");
            self.has_gpu = true;

            // Attempt to read vendor to confirm brand (0x1002 is AMD, 0x8086 is Intel)
            if let Ok(vendor) = fs::read_to_string("/sys/class/drm/card0/device/vendor") {
                let vendor_trimmed = vendor.trim();
                if vendor_trimmed.contains("0x1002") {
                    self.brand = String::from("AMD");
                    self.model = String::from("AMD Radeon Graphics");
                } else if vendor_trimmed.contains("0x8086") {
                    self.brand = String::from("Intel");
                    self.model = String::from("Intel HD Graphics");
                }
            } else {
                self.model = String::from("AMD/Intel Graphics");
            }
        }

        self.checked_presence = true;
    }

    /// Updates global GPU and per-process VRAM metrics in the background telemetry thread
    pub fn update(&mut self) {
        self.detect_gpu();

        if !self.has_gpu {
            self.history.remove(0);
            self.history.push(0.0);
            return;
        }

        if self.is_nvidia {
            self.update_nvidia();
        } else if self.is_amd {
            self.update_amd();
        }

        // Maintain rolling history
        self.history.remove(0);
        self.history.push(self.usage_percent);
    }

    /// Query global and compute process VRAM for NVIDIA cards using nvidia-smi
    fn update_nvidia(&mut self) {
        // Query global metrics: utilization (%), memory used (MB), memory total (MB), GPU name
        let output = Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=utilization.gpu,memory.used,memory.total,name",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                let parts: Vec<&str> = text.split(',').collect();
                if parts.len() >= 4 {
                    self.usage_percent = parts[0].trim().parse::<f32>().unwrap_or(0.0);
                    
                    let used_mb = parts[1].trim().parse::<u64>().unwrap_or(0);
                    self.memory_used_bytes = used_mb * 1024 * 1024;

                    let total_mb = parts[2].trim().parse::<u64>().unwrap_or(0);
                    self.memory_total_bytes = total_mb * 1024 * 1024;

                    self.model = parts[3].trim().to_string();
                }
            }
        }

        // Query active processes using GPU VRAM: pid, used_gpu_memory (MB)
        let mut proc_mem = HashMap::new();
        let app_output = Command::new("nvidia-smi")
            .args(&[
                "--query-compute-apps=pid,used_memory",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(out) = app_output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        if let (Ok(pid), Ok(mem_mb)) = (parts[0].trim().parse::<u32>(), parts[1].trim().parse::<u64>()) {
                            proc_mem.insert(pid, mem_mb * 1024 * 1024);
                        }
                    }
                }
            }
        }
        self.process_memory = proc_mem;
    }

    /// Query global metrics for AMD/Intel graphics cards using sysfs
    fn update_amd(&mut self) {
        // Read global usage percent
        if let Ok(usage) = fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent") {
            self.usage_percent = usage.trim().parse::<f32>().unwrap_or(0.0);
        }

        // Read VRAM used bytes
        if let Ok(used) = fs::read_to_string("/sys/class/drm/card0/device/mem_info_vram_used") {
            self.memory_used_bytes = used.trim().parse::<u64>().unwrap_or(0);
        }

        // Read VRAM total bytes
        if let Ok(total) = fs::read_to_string("/sys/class/drm/card0/device/mem_info_vram_total") {
            self.memory_total_bytes = total.trim().parse::<u64>().unwrap_or(0);
        }

        // AMD process-level VRAM is not natively exposed in sysfs, keep empty
        self.process_memory.clear();
    }
}
