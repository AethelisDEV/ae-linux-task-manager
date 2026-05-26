use sysinfo::{System, MemoryRefreshKind, RefreshKind};

#[derive(Clone, Default)]
pub struct MemoryStats {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub history: Vec<f32>, // Rolling history of memory usage percentage
}

impl MemoryStats {
    pub fn new() -> Self {
        Self {
            history: vec![0.0; 60],
            ..Default::default()
        }
    }

    pub fn refresh_spec() -> RefreshKind {
        RefreshKind::new().with_memory(MemoryRefreshKind::new().with_ram().with_swap())
    }

    pub fn update(&mut self, sys: &System) {
        self.total_bytes = sys.total_memory();
        self.used_bytes = sys.used_memory();
        self.swap_total_bytes = sys.total_swap();
        self.swap_used_bytes = sys.used_swap();

        let usage_percent = if self.total_bytes > 0 {
            (self.used_bytes as f32 / self.total_bytes as f32) * 100.0
        } else {
            0.0
        };

        // Maintain rolling history
        self.history.remove(0);
        self.history.push(usage_percent);
    }
}
