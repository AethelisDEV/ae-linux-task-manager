# 🖥️ AE TaskManager

[![Rust](https://img.shields.io/badge/rust-%23E34F26.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![egui](https://img.shields.io/badge/egui-GUI-%230052FF.svg?style=for-the-badge)](https://github.com/emilk/egui)
[![Platform](https://img.shields.io/badge/Platform-Linux-%23FCC624.svg?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](LICENSE)

**AE TaskManager**, Rust dili ve `egui` grafik arayüz (GUI) kütüphanesi kullanılarak geliştirilmiş; Linux işletim sistemleri için yüksek performanslı, zengin özelliklere sahip, modern ve kullanıcı dostu bir **Sistem İzleyici ve Gelişmiş Tanı Aracıdır**.

Windows 11 tasarım dilinden ilham alan modern, yuvarlatılmış köşeli karanlık teması, akıcı animasyonları ve tamamen arka planda (non-blocking) çalışan thread yapısı sayesinde arayüzde donma veya takılma olmadan pürüzsüz bir deneyim sunar.

---

## ✨ Öne Çıkan Özellikler

### 1. ⚙️ Süreç Yönetimi (Processes) & Ağaç Görünümü
* **Detaylı Süreç Listesi**: Aktif tüm süreçleri PID, İsim, CPU%, RAM%, Disk Okuma/Yazma, Kullanıcı ve Dosya Yolu bilgileriyle listeleme, arama ve filtreleme.
* **Sağ Tık Bağlam Menüsü (Context Menu)**:
  * ❌ **Görevi Sonlandır**: Süreci normal yetkilerle kapatır.
  * 🛡️ **Yönetici Olarak Zorla Kapat**: Yetki yükseltme ekranı (`pkexec`) aracılığıyla süreci root yetkileriyle zorla sonlandırır.
  * 📂 **Dosya Konumunu Aç**: Sürecin çalıştığı dizini varsayılan dosya yöneticisinde (`xdg-open`) açar.
  * 🔍 **Web'de Ara**: Süreç adını varsayılan tarayıcınızda aratır.
  * 📝 **Özellikler Paneli**: Sürecin tüm detaylarını şık bir modal pencerede gösterir ve bilgileri panoya kopyalama imkanı sunar.
* **🌳 Gelişmiş Ağaç Görünümü (Process Tree)**: Süreçleri üst-alt ilişkilerine (Parent-Child PID) göre hiyerarşik, daraltılabilir/genişletilebilir bir ağaç yapısında listeler.

### 2. 📊 Gerçek Zamanlı Performans Grafikleri (Performance)
* CPU, Bellek (RAM), GPU (varsa), Disk I/O ve Ağ (Download/Upload) kullanımlarını milisaniyelik hassasiyetle takip eden ve canlı olarak güncellenen premium grafik arayüzü.

### 3. 🛠️ Systemd Servis Yöneticisi (Services)
* Sistemdeki aktif/pasif tüm `systemd` servislerini listeleme ve durumlarına göre arama.
* Arka planda güvenli çalışan servis kontrolleri:
  * ▶️ Servisi Başlat / ⏸️ Durdur / 🔄 Yeniden Başlat.
  * 🟢 Etkinleştir (Enable) / 🔴 Devre Dışı Bırak (Disable).
* Sistem genelini etkileyen işlemler için **Polkit (`pkexec`) entegrasyonu** ile grafiksel şifre sorma ekranı.

### 4. 🚀 Başlangıç Uygulamaları Yöneticisi (Startup)
* Sistem genelinde (`/etc/xdg/autostart`) ve kullanıcı özelinde (`~/.config/autostart`) tanımlı `.desktop` başlangıç girişlerini tarar.
* Freedesktop XDG standartlarına tam uyumlu olarak, başlangıç uygulamalarını tek tıkla açıp kapatma (güvenli copy-on-write mekanizmasıyla).

### 5. 🌐 Canlı Ağ Bağlantıları Haritası (Network Sockets)
* `/proc/net/` altındaki soket bilgilerini doğrudan okuyarak aktif tüm TCP/UDP (IPv4/IPv6) bağlantı noktalarını, uzak IP adreslerini ve durumları listeler.
* Bağlantıyı gerçekleştiren aktif süreci (PID ve Süreç Adı) doğrudan haritalandırır.

### 6. 🔒 Dosya Kilidi & Açık Dosya İzleyici (File Locks)
* Belirli bir dosya veya dizin üzerinde kilit/erişim hakkı tutan süreçleri `/proc/*/fd/` altındaki symlink'leri tarayarak anında tespit eder.
* Kilit tutan süreci doğrudan arayüzden sonlandırma imkanı sunar.

---

## 🛠️ Mimari ve Performans Prensipleri

* **Asenkron Çalışma (Non-blocking Thread Model)**: Ağ taraması, servis yönetimi, dosya kilidi arama gibi ağır veya bloklayıcı işlemler, ana arayüz (GUI) thread'ini dondurmamak için arka plandaki işçi iş parçacıklarında (`std::sync::mpsc` kanalları ile) yürütülür.
* **Yüksek Uyumluluk & Emojiler**: Linux üzerinde yüksek çözünürlüklü sembol ve emojilerin gösterilmesi için sistemdeki `Noto Color Emoji` ve `Noto Sans Symbols` font dosyalarını otomatik olarak arar ve arayüze entegre eder.
* **Güvenlik & Sadelik**: Projede hiçbir `unsafe` kod bloğu kullanılmamış olup, tamamen güvenli Rust (Safe Rust) standartlarına sadık kalınmıştır.

---

## 🚀 Kurulum ve Çalıştırma

### Gereksinimler

Projenin derlenmesi ve çalıştırılması için sisteminizde Rust geliştirme ortamı ve bazı kütüphanelerin bulunması gerekir.

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

### Derleme ve Çalıştırma

Projeyi klonladıktan sonra dizine gidin ve release (optimize edilmiş) modda çalıştırın:

```bash
# Depoyu yerel bilgisayarınıza klonlayın (veya dizine geçin)
cd "AE TaskManager"

# Uygulamayı optimize edilmiş modda derleyin ve çalıştırın
cargo run --release
```

---

## 📄 Lisans

Bu proje **MIT Lisansı** altında lisanslanmıştır. Detaylar için [LICENSE](LICENSE) dosyasına göz atabilirsiniz.

---

*Geliştiren: **[AethelisDEV](https://github.com/AethelisDEV)***
