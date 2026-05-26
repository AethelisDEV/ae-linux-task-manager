/// Localization Module for AE TaskManager
///
/// Implements high-fidelity system-language auto-detection
/// and supports bilingual translations (English & Turkish).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Turkish,
}

impl Language {
    /// Auto-detects system language from environment variables.
    /// Falls back to English if not Turkish.
    pub fn detect() -> Self {
        if let Ok(lang) = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_MESSAGES"))
        {
            if lang.to_lowercase().starts_with("tr") {
                return Language::Turkish;
            }
        }
        Language::English
    }
}

/// Translates a given translation key into the selected language.
pub fn tr(key: &'static str, lang: Language) -> &'static str {
    match lang {
        Language::Turkish => translate_tr(key),
        Language::English => translate_en(key),
    }
}

fn translate_en(key: &'static str) -> &'static str {
    match key {
        // Sidebar tabs
        "tab_processes" => "☰  Processes",
        "tab_performance" => "📈  Performance",
        "tab_services" => "⚙  Services",
        "tab_startup" => "⚡  Startup Apps",
        "tab_connections" => "🌐  Connections",
        "tab_file_locks" => "🔒  File Locks",
        "tab_about" => "ℹ  System Info",
        "display_backend" => "Display Backend:",
        
        // General UI buttons
        "btn_refresh" => "🔄 Refresh",
        "btn_refresh_list" => "🔄 Refresh List",
        "label_search" => "🔍 Search:",
        
        // Processes Tab
        "proc_tree_view" => "🌳 Tree View",
        "proc_hdr_pid" => "PID",
        "proc_hdr_name" => "Name",
        "proc_hdr_cpu" => "CPU%",
        "proc_hdr_ram" => "RAM%",
        "proc_hdr_read" => "Disk Read",
        "proc_hdr_write" => "Disk Write",
        "proc_hdr_user" => "User",
        "proc_hdr_path" => "Path",
        "proc_search_hint" => "Filter processes by name, PID, or user...",
        
        // Processes Context Menu & Modals
        "menu_terminate" => "⏹  Terminate Task",
        "menu_force_terminate" => "🛡  Force Terminate (Admin)",
        "menu_open_location" => "📂  Open File Location",
        "menu_search_web" => "🔍  Search Web",
        "menu_properties" => "📝  Properties",
        "modal_proc_details" => "Process Details",
        "modal_ppid" => "Parent PID:",
        "modal_status" => "Status:",
        "modal_virt" => "Virtual Memory:",
        "modal_cmdline" => "Command Line:",
        "modal_owner" => "Owner User:",
        "btn_copy" => "📋 Copy to Clipboard",
        "btn_close" => "Close",
        
        // Services Tab
        "svc_search_hint" => "Search service name or description...",
        "svc_hdr_name" => "Service Name",
        "svc_hdr_load" => "Load State",
        "svc_hdr_active" => "Active State",
        "svc_hdr_sub" => "Sub State",
        "svc_hdr_desc" => "Description",
        "svc_no_matches" => "No matching services found",
        "menu_svc_start" => "▶  Start Service",
        "menu_svc_stop" => "⏹  Stop Service",
        "menu_svc_restart" => "🔄  Restart Service",
        "menu_svc_enable" => "⚡  Enable on Startup",
        "menu_svc_disable" => "❌  Disable on Startup",
        
        // Startup Tab
        "start_title" => "Startup Applications",
        "start_subtitle" => "Manage system-wide and user-specific autostart application entries.",
        "start_no_entries" => "No startup entries found",
        "start_hdr_status" => "Status",
        "start_hdr_app" => "Application Name",
        "start_hdr_cmd" => "Launch Command",
        "start_hdr_loc" => "Location",
        "start_hdr_details" => "Details",
        "status_enabled" => "Enabled",
        "status_disabled" => "Disabled",
        "loc_user" => "User Local",
        "loc_system" => "System Wide",
        
        // Connections Tab
        "conn_title" => "Active Network Connections Map",
        "conn_subtitle" => "Real-time TCP & UDP socket mapping to local process owners.",
        "conn_hdr_proto" => "Protocol",
        "conn_hdr_local" => "Local Endpoint",
        "conn_hdr_remote" => "Remote Endpoint",
        "conn_hdr_state" => "State",
        "conn_hdr_owner" => "Process Owner",
        
        // File Locks Tab
        "lock_title" => "File Open & Lock Tracker",
        "lock_subtitle" => "Search open file descriptors to find which active processes are holding handles on specified paths.",
        "lock_search_hint" => "Enter absolute file/directory path...",
        "lock_btn_scan" => "Scan Path Handle",
        "lock_no_matches" => "No processes holding handles on this path",
        "lock_hdr_pid" => "PID",
        "lock_hdr_name" => "Process Name",
        "lock_hdr_fd" => "Descriptor (FD)",
        "lock_hdr_mode" => "Access Mode",
        "lock_hdr_path" => "File Path",
        "lock_hdr_action" => "Action",
        "mode_read" => "Read",
        "mode_write" => "Write",
        "mode_readwrite" => "ReadWrite",
        "btn_terminate" => "Terminate",

        // Default fallback
        _ => key,
    }
}

fn translate_tr(key: &'static str) -> &'static str {
    match key {
        // Sidebar tabs
        "tab_processes" => "☰  Süreçler",
        "tab_performance" => "📈  Performans",
        "tab_services" => "⚙  Servisler",
        "tab_startup" => "⚡  Başlangıç",
        "tab_connections" => "🌐  Ağ Bağlantıları",
        "tab_file_locks" => "🔒  Dosya Kilitleri",
        "tab_about" => "ℹ  Sistem Bilgisi",
        "display_backend" => "Ekran Sunucusu:",
        
        // General UI buttons
        "btn_refresh" => "🔄 Yenile",
        "btn_refresh_list" => "🔄 Listeyi Yenile",
        "label_search" => "🔍 Ara:",
        
        // Processes Tab
        "proc_tree_view" => "🌳 Ağaç Görünümü (Tree)",
        "proc_hdr_pid" => "PID",
        "proc_hdr_name" => "İsim",
        "proc_hdr_cpu" => "CPU%",
        "proc_hdr_ram" => "RAM%",
        "proc_hdr_read" => "Disk Oku",
        "proc_hdr_write" => "Disk Yaz",
        "proc_hdr_user" => "Kullanıcı",
        "proc_hdr_path" => "Yol",
        "proc_search_hint" => "Süreç adı, PID veya kullanıcıya göre filtrele...",
        
        // Processes Context Menu & Modals
        "menu_terminate" => "⏹  Görevi Sonlandır",
        "menu_force_terminate" => "🛡  Yönetici Olarak Kapat (Admin)",
        "menu_open_location" => "📂  Dosya Konumunu Aç",
        "menu_search_web" => "🔍  Web'de Ara",
        "menu_properties" => "📝  Detaylar",
        "modal_proc_details" => "Süreç Özellikleri",
        "modal_ppid" => "Üst PID (PPID):",
        "modal_status" => "Durum:",
        "modal_virt" => "Sanal Bellek:",
        "modal_cmdline" => "Komut Satırı:",
        "modal_owner" => "Sahip Kullanıcı:",
        "btn_copy" => "📋 Panoya Kopyala",
        "btn_close" => "Kapat",
        
        // Services Tab
        "svc_search_hint" => "Servis adı veya açıklama ara...",
        "svc_hdr_name" => "Servis Adı",
        "svc_hdr_load" => "Yükleme",
        "svc_hdr_active" => "Etkinlik",
        "svc_hdr_sub" => "Alt Durum",
        "svc_hdr_desc" => "Açıklama",
        "svc_no_matches" => "Eşleşen servis bulunamadı",
        "menu_svc_start" => "▶  Servisi Başlat",
        "menu_svc_stop" => "⏹  Servisi Durdur",
        "menu_svc_restart" => "🔄  Yeniden Başlat",
        "menu_svc_enable" => "⚡  Başlangıçta Etkinleştir",
        "menu_svc_disable" => "❌  Açılışta Devre Dışı Bırak",
        
        // Startup Tab
        "start_title" => "Başlangıç Uygulamaları",
        "start_subtitle" => "Sistem genelinde ve kullanıcıya özel otomatik başlatılan uygulamaları yönetin.",
        "start_no_entries" => "Başlangıç uygulaması bulunamadı",
        "start_hdr_status" => "Durum",
        "start_hdr_app" => "Uygulama Adı",
        "start_hdr_cmd" => "Başlatma Komutu",
        "start_hdr_loc" => "Konum",
        "start_hdr_details" => "Detaylar",
        "status_enabled" => "Etkin",
        "status_disabled" => "Devre Dışı",
        "loc_user" => "Kullanıcı Özel",
        "loc_system" => "Sistem Geneli",
        
        // Connections Tab
        "conn_title" => "Aktif Ağ Bağlantıları Haritası",
        "conn_subtitle" => "TCP ve UDP soketlerinin anlık süreç sahipleriyle haritalandırılması.",
        "conn_hdr_proto" => "Protokol",
        "conn_hdr_local" => "Yerel Adres",
        "conn_hdr_remote" => "Uzak Adres",
        "conn_hdr_state" => "Durum",
        "conn_hdr_owner" => "Süreç Sahibi",
        
        // File Locks Tab
        "lock_title" => "Dosya Kilidi ve Açık Dosya İzleyici",
        "lock_subtitle" => "Belirli yollarda kilit veya açık tanıtıcı (handle) tutan süreçleri anında bulun.",
        "lock_search_hint" => "Mutlak dosya veya klasör yolu girin...",
        "lock_btn_scan" => "Yolu Tara",
        "lock_no_matches" => "Bu yol üzerinde açık kilit tutan süreç bulunamadı",
        "lock_hdr_pid" => "PID",
        "lock_hdr_name" => "Süreç İsim",
        "lock_hdr_fd" => "Tanıtıcı (FD)",
        "lock_hdr_mode" => "Erişim Modu",
        "lock_hdr_path" => "Dosya Yolu",
        "lock_hdr_action" => "İşlem",
        "mode_read" => "Oku",
        "mode_write" => "Yaz",
        "mode_readwrite" => "OkuYaz",
        "btn_terminate" => "Görevi Sonlandır",

        // Default fallback
        _ => key,
    }
}
