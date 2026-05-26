pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod process;
pub mod icons;
pub mod gpu;
pub mod systemd;
pub mod startup;
pub mod network_conn;
pub mod file_lock;

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use sysinfo::{System, Networks, Disks, Users, ProcessesToUpdate};

#[derive(Clone)]
pub struct SystemState {
    pub cpu: cpu::CpuStats,
    pub memory: memory::MemoryStats,
    pub network: network::NetworkStats,
    pub disk: disk::DiskStats,
    pub processes: process::ProcessManager,
    pub gpu: gpu::GpuStats,
    pub uptime: u64,
    pub hostname: String,
    pub kernel_version: String,
    pub os_version: String,
    pub last_update: Instant,
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            cpu: cpu::CpuStats::new(),
            memory: memory::MemoryStats::new(),
            network: network::NetworkStats::new(),
            disk: disk::DiskStats::new(),
            processes: process::ProcessManager::new(),
            gpu: gpu::GpuStats::new(),
            uptime: 0,
            hostname: String::new(),
            kernel_version: String::new(),
            os_version: String::new(),
            last_update: Instant::now(),
        }
    }
}

pub struct SystemAggregator {
    state: Arc<RwLock<SystemState>>,
    _handle: thread::JoinHandle<()>,
}

impl SystemAggregator {
    pub fn new(ctx: eframe::egui::Context) -> Self {
        let state = Arc::new(RwLock::new(SystemState::new()));
        let state_clone = Arc::clone(&state);

        // Spawn telemetry polling in background thread
        let handle = thread::spawn(move || {
            let mut sys = System::new();
            let mut networks = Networks::new_with_refreshed_list();
            let mut disks = Disks::new_with_refreshed_list();
            let mut users = Users::new_with_refreshed_list();
            let mut gpu = gpu::GpuStats::new();

            // Perform initial high-cost static information query
            let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
            let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
            let os_version = System::os_version().unwrap_or_else(|| "Linux".to_string());

            let mut last_users_refresh = Instant::now();

            // Initialize background system
            sys.refresh_cpu_all();
            sys.refresh_memory();
            sys.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                sysinfo::ProcessRefreshKind::new()
                    .with_cpu()
                    .with_memory()
                    .with_user(sysinfo::UpdateKind::Always),
            );
            networks.refresh();
            disks.refresh();

            loop {
                // Refresh system telemetry data specifically to minimize CPU overhead
                sys.refresh_cpu_all();
                sys.refresh_memory();
                sys.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    true,
                    sysinfo::ProcessRefreshKind::new()
                        .with_cpu()
                        .with_memory()
                        .with_user(sysinfo::UpdateKind::Always),
                );
                
                // Only refresh user accounts once every 30 seconds to prevent heavy I/O overhead
                if last_users_refresh.elapsed() > Duration::from_secs(30) {
                    users.refresh_list();
                    last_users_refresh = Instant::now();
                }

                let uptime = System::uptime();

                // Mutate the thread-safe state
                {
                    let mut lock = state_clone.write();
                    gpu.update();
                    
                    lock.cpu.update(&sys);
                    lock.memory.update(&sys);
                    lock.network.update(&mut networks);
                    lock.disk.update(&mut disks);
                    
                    // We will pass the gpu.process_memory to the processes update so it can map GPU memory per PID
                    lock.processes.update(&sys, &users, &gpu.process_memory);
                    lock.gpu = gpu.clone();
                    
                    lock.uptime = uptime;
                    lock.hostname = hostname.clone();
                    lock.kernel_version = kernel_version.clone();
                    lock.os_version = os_version.clone();
                    lock.last_update = Instant::now();
                }

                // Force GUI thread to repaint exactly when new data is ready
                ctx.request_repaint();

                // Poll interval: 1 second
                thread::sleep(Duration::from_millis(1000));
            }
        });

        Self {
            state,
            _handle: handle,
        }
    }

    pub fn read_state(&self) -> parking_lot::RwLockReadGuard<'_, SystemState> {
        self.state.read()
    }
}
