use sysinfo::{System, CpuRefreshKind, RefreshKind};

#[derive(Clone, Default)]
pub struct CpuStats {
    pub global_usage: f32,
    pub core_usages: Vec<f32>,
    pub avg_frequency_mhz: u64,
    pub model: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub history: Vec<f32>, // Rolling history of global CPU usage
}

impl CpuStats {
    pub fn new() -> Self {
        Self {
            history: vec![0.0; 60], // Store last 60 seconds
            ..Default::default()
        }
    }

    pub fn refresh_spec() -> RefreshKind {
        RefreshKind::new().with_cpu(CpuRefreshKind::new().with_cpu_usage())
    }

    pub fn update(&mut self, sys: &System) {
        self.global_usage = sys.global_cpu_usage();
        
        let cpus = sys.cpus();
        self.logical_cores = cpus.len();
        self.physical_cores = sys.physical_core_count().unwrap_or(self.logical_cores);

        self.core_usages = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();

        // Calculate average frequency across all cores
        if !cpus.is_empty() {
            let total_freq: u64 = cpus.iter().map(|cpu| cpu.frequency()).sum();
            self.avg_frequency_mhz = total_freq / cpus.len() as u64;
        } else {
            self.avg_frequency_mhz = 0;
        }

        if self.model.is_empty() && !cpus.is_empty() {
            self.model = cpus[0].brand().to_string();
        }

        // Maintain rolling history
        self.history.remove(0);
        self.history.push(self.global_usage);
    }
}
