use eframe::egui;

pub struct ThemeColors {
    pub bg_dark: egui::Color32,
    pub bg_card: egui::Color32,
    pub bg_sidebar: egui::Color32,
    pub border: egui::Color32,
    pub text_primary: egui::Color32,
    pub text_secondary: egui::Color32,
    pub accent: egui::Color32,
    pub accent_hover: egui::Color32,
    pub success: egui::Color32,
    pub danger: egui::Color32,
    pub warning: egui::Color32,
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            bg_dark: egui::Color32::from_rgb(18, 18, 20),      // Deep Charcoal
            bg_card: egui::Color32::from_rgb(30, 30, 34),      // Muted Slate card
            bg_sidebar: egui::Color32::from_rgb(24, 24, 26),   // Slightly darker sidebar
            border: egui::Color32::from_rgb(45, 45, 50),       // Subtle panel borders
            text_primary: egui::Color32::from_rgb(245, 245, 247), // Soft White
            text_secondary: egui::Color32::from_rgb(150, 150, 160), // Muted Grey
            accent: egui::Color32::from_rgb(99, 102, 241),      // Indigo Accent (Windows Modern style)
            accent_hover: egui::Color32::from_rgb(129, 140, 248),
            success: egui::Color32::from_rgb(52, 211, 153),    // Emerald Green
            danger: egui::Color32::from_rgb(248, 113, 113),     // Coral Red
            warning: egui::Color32::from_rgb(251, 191, 36),    // Amber Yellow
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Candidates for Noto Color Emoji font
    let emoji_paths = [
        "/usr/share/fonts/noto/NotoColorEmoji.ttf",
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
        "/usr/share/fonts/noto-emoji/NotoColorEmoji.ttf",
        "/usr/share/fonts/TTF/NotoColorEmoji.ttf",
    ];

    // Candidates for Noto Symbols font
    let symbol_paths = [
        "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
        "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
    ];

    // Load first available emoji font
    for path in emoji_paths.iter() {
        if let Ok(font_bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "noto-emoji".to_owned(),
                egui::FontData::from_owned(font_bytes),
            );
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.push("noto-emoji".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.push("noto-emoji".to_owned());
            }
            break;
        }
    }

    // Load first available symbol font
    for path in symbol_paths.iter() {
        if let Ok(font_bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "noto-symbols".to_owned(),
                egui::FontData::from_owned(font_bytes),
            );
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.push("noto-symbols".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.push("noto-symbols".to_owned());
            }
            break;
        }
    }

    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &egui::Context) {
    setup_custom_fonts(ctx);
    let mut style = (*ctx.style()).clone();
    
    // Windows 11 style rounded corners
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.open.rounding = egui::Rounding::same(8.0);
    
    style.visuals.window_rounding = egui::Rounding::same(12.0);
    style.visuals.menu_rounding = egui::Rounding::same(8.0);

    // Subtle styling tweaks
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 34);
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 45, 50));
    
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(38, 38, 44);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(0.0, egui::Color32::TRANSPARENT);
    
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(52, 52, 60);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(99, 102, 241));

    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(99, 102, 241);
    
    // Scrollbars
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_outer_margin = 2.0;

    // Apply the configured style
    ctx.set_style(style);
}

/// Helper to render a card frame
pub fn card_style(colors: &ThemeColors) -> egui::Frame {
    egui::Frame::none()
        .fill(colors.bg_card)
        .stroke(egui::Stroke::new(1.0, colors.border))
        .rounding(10.0)
        .inner_margin(12.0)
}
