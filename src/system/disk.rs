use sysinfo::Disks;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

#[derive(Clone)]
pub struct DiskStats {
    pub read_bytes_sec: u64,
    pub write_bytes_sec: u64,
    pub total_read_bytes: u64,
    pub total_write_bytes: u64,
    pub partitions: Vec<PartitionInfo>,
    pub read_history: Vec<f32>,  // Rolling history of Read speed in KB/s
    pub write_history: Vec<f32>, // Rolling history of Write speed in KB/s
    
    last_poll_time: Instant,
}

#[derive(Clone, Default)]
pub struct PartitionInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub file_system: String,
}

impl DiskStats {
    pub fn new() -> Self {
        Self {
            read_bytes_sec: 0,
            write_bytes_sec: 0,
            total_read_bytes: 0,
            total_write_bytes: 0,
            partitions: Vec::new(),
            read_history: vec![0.0; 60],
            write_history: vec![0.0; 60],
            last_poll_time: Instant::now(),
        }
    }

    pub fn update(&mut self, disks: &mut Disks) {
        disks.refresh();

        self.partitions = disks.iter().map(|disk| {
            PartitionInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                file_system: disk.file_system().to_string_lossy().to_string(),
            }
        }).collect();

        // Calculate global read/write speed by parsing /proc/diskstats (UNIX-like)
        let now = Instant::now();
        let duration = now.duration_since(self.last_poll_time).as_secs_f32();
        self.last_poll_time = now;

        if let Ok((current_read, current_write)) = self.read_proc_diskstats() {
            if self.total_read_bytes > 0 && duration > 0.0 {
                let delta_read = current_read.saturating_sub(self.total_read_bytes);
                let delta_write = current_write.saturating_sub(self.total_write_bytes);
                
                self.read_bytes_sec = (delta_read as f32 / duration) as u64;
                self.write_bytes_sec = (delta_write as f32 / duration) as u64;
            }
            
            self.total_read_bytes = current_read;
            self.total_write_bytes = current_write;
        }

        // Maintain rolling history (convert to KB/s for graph)
        let read_kb = (self.read_bytes_sec as f32) / 1024.0;
        let write_kb = (self.write_bytes_sec as f32) / 1024.0;

        self.read_history.remove(0);
        self.read_history.push(read_kb);

        self.write_history.remove(0);
        self.write_history.push(write_kb);
    }

    /// Reads total sectors read and written from /proc/diskstats and converts to bytes.
    /// Sector size in Linux is always 512 bytes.
    fn read_proc_diskstats(&self) -> Result<(u64, u64), std::io::Error> {
        let file = File::open("/proc/diskstats")?;
        let reader = BufReader::new(file);

        let mut total_sectors_read = 0u64;
        let mut total_sectors_written = 0u64;

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 14 {
                let dev_name = parts[2];
                // Focus on physical disks (e.g. sda, sdb, nvme0n1, nvme1n1) and exclude partitions/virtual disks
                // sda1 has digits, sda doesn't. nvme0n1 has digits but it's the physical drive, whereas nvme0n1p1 is partition.
                let is_physical_disk = (dev_name.starts_with("sd") && dev_name.len() == 3)
                    || (dev_name.starts_with("nvme") && dev_name.contains('n') && !dev_name.contains('p'));

                if is_physical_disk {
                    // Field 5 (index 5): Sectors read successfully
                    // Field 9 (index 9): Sectors written successfully
                    if let (Ok(r_sect), Ok(w_sect)) = (parts[5].parse::<u64>(), parts[9].parse::<u64>()) {
                        total_sectors_read += r_sect;
                        total_sectors_written += w_sect;
                    }
                }
            }
        }

        // 1 sector = 512 bytes
        Ok((total_sectors_read * 512, total_sectors_written * 512))
    }
}
