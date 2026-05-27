# 🖥️ AE TaskManager

[![Rust](https://img.shields.io/badge/rust-%23E34F26.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![egui](https://img.shields.io/badge/egui-GUI-%230052FF.svg?style=for-the-badge)](https://github.com/emilk/egui)
[![Platform](https://img.shields.io/badge/Platform-Linux-%23FCC624.svg?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](LICENSE)

**AE TaskManager** is a high-performance, feature-rich, and modern **System Monitor and Advanced Diagnostics Tool** for Linux operating systems, written in Rust utilizing the `egui` immediate-mode graphical user interface library.

Inspired by modern visual aesthetics (such as Windows 11 design styling), it features a sleek dark theme with rounded corners, smooth animations, and a fully decoupled **non-blocking worker thread architecture** that guarantees a smooth user experience without interface freezing or stuttering.

---

## ✨ Features

### 1. ⚙️ Process Management & Tree View
* **Comprehensive Process List**: Monitor all active processes with details like PID, Name, CPU%, Memory (RSS)%, Disk Read/Write, Owner User, and Executable Path, with instant searching and column sorting.
* **Right-Click Context Menu**:
  * ❌ **Terminate Task**: Safely terminate a process with standard user permissions.
  * 🛡️ **Force Terminate (Admin)**: Instantly force-kills a process using root permissions via graphical Polkit authorization (`pkexec`).
  * 📂 **Open File Location**: Opens the directory of the running process in the system's default file manager (`xdg-open`).
  * 🔍 **Search Web**: Performs a quick search of the process name in your default browser.
  * 📝 **Properties**: Displays comprehensive process parameters in a custom-styled modal window with a copy-to-clipboard option.
* **🌳 Expandable Process Tree View**: Groups processes hierarchically by parent/child PID relationships with collapsible/expandable nodes.

### 2. 📊 Real-Time Performance Charts (Telemetry)
* Dynamic, high-fidelity real-time line charts tracking CPU utilization, Memory (RAM) consumption, GPU Memory usage (if available), Disk Read/Write, and Network I/O (Upload/Download) with millisecond-level responsiveness.

### 3. 🛠️ Systemd Services Manager
* Scans and lists all systemd service units, showing Load, Active, and Sub states with dynamic text filtering.
* Non-blocking control mechanisms running on background threads:
  * ▶️ Start Service / ⏹️ Stop Service / 🔄 Restart Service.
  * ⚡ Enable on Startup / ❌ Disable on Startup.
* Safe administrative escalation via standard Linux Polkit (`pkexec`) displaying standard graphical password dialogs.

### 4. 🚀 Startup Applications Manager
* Scans system-wide (`/etc/xdg/autostart`) and user-specific (`~/.config/autostart`) `.desktop` startup entry files.
* Allows user-level autostart overrides using standard copy-on-write mechanisms via interactive toggle switches.

### 5. 🌐 Active Network Connections Map (Sockets)
* Maps active network TCP/UDP (IPv4/IPv6) endpoints by directly parsing `/proc/net/` sockets on-demand.
* Resolves local/remote endpoints, connection states, and maps them to the respective owning **Process Name** and **PID** without spawning slow external commands.

### 6. 🔒 File Open & Lock Tracker
* Sweeps `/proc/*/fd/` links dynamically to trace which active processes are holding open handles or kilit descriptors on a target directory or file path.
* Offers inline **Terminate Task** triggers to instantly resolve locked files.

---

## 🛠️ Architecture and Performance Principles

* **Asynchronous Execution (Non-blocking Threads)**: Heavily blocking commands like file descriptor sweeps, D-Bus queries, and systemctl actions are dispatched to background threads utilizing Rust `std::sync::mpsc` channels, keeping the GUI extremely responsive.
* **Intelligent Localization (EN/TR)**: Features system language auto-detection (checks `LANG`, `LC_ALL`, and `LC_MESSAGES` variables). If the system language is Turkish, the interface automatically loads in **Turkish**. Otherwise, it defaults to **English**.
* **Zero Unsafe**: Built entirely following modern Rust safety standards with zero `unsafe` blocks.

---

## 🚀 Installation & Build Yönergeleri

### Prerequisites

To compile and run this application, make sure your system has Rust (Cargo) and some necessary developer packages installed.

**Ubuntu / Debian / Pop!_OS:**
```bash
sudo apt update
sudo apt install build-essential libdbus-1-dev pkg-config
```

**Fedora / RHEL:**
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install dbus-devel pkg-config
```

**Arch Linux / Manjaro:**
```bash
sudo pacman -Syu base-devel dbus
```

### Build & Run

Clone the repository, enter the project directory, and compile/run in optimized release mode:

```bash
# Enter the project directory
cd "ae-linux-task-manager"

# Compile and run in release mode
cargo run --release
```

---

## 📄 License

This project is licensed under the Apache License 2.0. For details, see the [LICENSE](LICENSE) and [NOTICE](NOTICE) files.

---

*Developed by **[AethelisDEV](https://github.com/AethelisDEV)***
