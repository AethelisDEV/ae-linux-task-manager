/// Process Socket & Network Connection Mapping Backend Module
///
/// This module implements a zero-dependency Linux network telemetry scanner.
/// It reads and parses `/proc/net/tcp`, `/proc/net/tcp6`, `/proc/net/udp`, and `/proc/net/udp6`
/// to list active sockets, and maps them to their owner processes (PIDs and Names)
/// by scanning file descriptors `/proc/<pid>/fd/` for matching socket inodes.

use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

/// Represents an active network socket connection.
#[derive(Debug, Clone, Default)]
pub struct NetworkConnection {
    /// The protocol used by the socket (`"TCP"` or `"UDP"`).
    pub protocol: String,
    /// The local IP and Port address (e.g., `127.0.0.1:8080`).
    pub local_address: String,
    /// The remote IP and Port address (e.g., `192.168.1.10:443` or `*:*` for listening).
    pub remote_address: String,
    /// The operational state of the connection (e.g., `LISTEN`, `ESTABLISHED`, `TIME_WAIT`).
    pub state: String,
    /// The Process ID (PID) of the application owning this socket.
    pub pid: Option<u32>,
    /// The name of the process owning this socket.
    pub process_name: String,
}

/// Helper function to parse a hexadecimal IPv4 and port representation from /proc/net format.
fn parse_ipv4_hex(hex_str: &str) -> Option<String> {
    let parts: Vec<&str> = hex_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let ip_val = u32::from_str_radix(parts[0], 16).ok()?;
    let port = u16::from_str_radix(parts[1], 16).ok()?;
    
    // /proc/net integers are written in native host-byte-order (little-endian on x86/ARM)
    let bytes = ip_val.to_le_bytes();
    let ip = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
    
    Some(format!("{}:{}", ip, port))
}

/// Helper function to parse a hexadecimal IPv6 and port representation from /proc/net format.
fn parse_ipv6_hex(hex_str: &str) -> Option<String> {
    let parts: Vec<&str> = hex_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let raw_ip = parts[0];
    if raw_ip.len() != 32 {
        return None;
    }

    let port = u16::from_str_radix(parts[1], 16).ok()?;
    let mut ip_bytes = [0u8; 16];

    for i in 0..4 {
        let chunk = &raw_ip[i * 8..(i + 1) * 8];
        let val = u32::from_str_radix(chunk, 16).ok()?;
        let bytes = val.to_le_bytes();
        ip_bytes[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }

    let ip = Ipv6Addr::from(ip_bytes);
    Some(format!("{}:{}", ip, port))
}

/// Helper function mapping TCP hexadecimal state values to standard names.
fn get_tcp_state(state_hex: &str) -> &'static str {
    match state_hex {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => "LISTEN",
        "0B" => "CLOSING",
        _ => "UNKNOWN",
    }
}

/// Dynamic indexer mapping open socket inodes to their owning processes (PIDs and Names).
///
/// Scans `/proc/<pid>/fd/` for every active numeric PID directory. If a file descriptor is
/// a symbolic link to a socket (`socket:[<inode>]`), records the mapping.
fn build_inode_process_map() -> HashMap<u64, (u32, String)> {
    let mut map = HashMap::new();
    
    let proc_path = Path::new("/proc");
    let entries = match fs::read_dir(proc_path) {
        Ok(e) => e,
        Err(_) => return map,
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        
        let pid: u32 = match file_name.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        
        let comm_path = path.join("comm");
        let name = fs::read_to_string(comm_path)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
            
        let fd_path = path.join("fd");
        let fd_entries = match fs::read_dir(fd_path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        for fd_entry in fd_entries.flatten() {
            if let Ok(target) = fs::read_link(fd_entry.path()) {
                let target_str = target.to_string_lossy();
                if target_str.starts_with("socket:[") && target_str.ends_with(']') {
                    let inode_str = &target_str["socket:[".len()..target_str.len() - 1];
                    if let Ok(inode) = inode_str.parse::<u64>() {
                        map.insert(inode, (pid, name.clone()));
                    }
                }
            }
        }
    }
    
    map
}

/// Helper function to parse socket lines from a specific `/proc/net/` text file.
fn parse_proc_net_file(
    file_path: &str,
    protocol: &str,
    is_ipv6: bool,
    inode_map: &HashMap<u64, (u32, String)>,
    list: &mut Vec<NetworkConnection>,
) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in content.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        let local_hex = parts[1];
        let remote_hex = parts[2];
        let state_hex = parts[3];
        let inode_str = parts[9];

        let local_addr = if is_ipv6 {
            parse_ipv6_hex(local_hex)
        } else {
            parse_ipv4_hex(local_hex)
        }.unwrap_or_else(|| "Invalid".to_string());

        let remote_addr = if is_ipv6 {
            parse_ipv6_hex(remote_hex)
        } else {
            parse_ipv4_hex(remote_hex)
        }.unwrap_or_else(|| "Invalid".to_string());

        let state = if protocol == "TCP" {
            get_tcp_state(state_hex).to_string()
        } else {
            "ACTIVE".to_string()
        };

        let inode = inode_str.parse::<u64>().unwrap_or(0);
        let mut pid = None;
        let mut process_name = "Unknown".to_string();

        if inode > 0 {
            if let Some(&(p_id, ref p_name)) = inode_map.get(&inode) {
                pid = Some(p_id);
                process_name = p_name.clone();
            }
        }

        list.push(NetworkConnection {
            protocol: protocol.to_string(),
            local_address: local_addr,
            remote_address: remote_addr,
            state,
            pid,
            process_name,
        });
    }
}

/// Dynamically scans the system to resolve all active network socket connections.
///
/// Parses `/proc/net/tcp`, `/proc/net/tcp6`, `/proc/net/udp`, and `/proc/net/udp6`.
/// Concurrently links open inodes with `/proc/<pid>/fd/` descriptors to resolve PID owners.
pub fn get_active_connections() -> Vec<NetworkConnection> {
    let inode_map = build_inode_process_map();
    let mut list = Vec::new();

    parse_proc_net_file("/proc/net/tcp", "TCP", false, &inode_map, &mut list);
    parse_proc_net_file("/proc/net/tcp6", "TCP", true, &inode_map, &mut list);
    parse_proc_net_file("/proc/net/udp", "UDP", false, &inode_map, &mut list);
    parse_proc_net_file("/proc/net/udp6", "UDP", true, &inode_map, &mut list);

    list
}
