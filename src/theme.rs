use crate::config::{AppConfig, RgbaColor, ThemeMode};

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub background: egui::Color32,
    pub panel: egui::Color32,
    pub text: egui::Color32,
    pub accent: egui::Color32,
    pub danger: egui::Color32,
}

impl ThemePalette {
    pub fn from_config(config: &AppConfig, system_dark: bool) -> Self {
        match config.theme {
            ThemeMode::Light => Self::light(),
            ThemeMode::Dark => Self::dark(),
            ThemeMode::System => {
                if system_dark { Self::dark() } else { Self::light() }
            }
        }
    }

    pub fn light() -> Self {
        Self {
            background: egui::Color32::from_rgb(246, 248, 251),
            panel: egui::Color32::from_rgb(255, 255, 255),
            text: egui::Color32::from_rgb(20, 25, 32),
            accent: egui::Color32::from_rgb(0, 122, 255),
            danger: egui::Color32::from_rgb(220, 53, 69),
        }
    }

    pub fn dark() -> Self {
        Self {
            background: egui::Color32::from_rgb(20, 23, 28),
            panel: egui::Color32::from_rgb(32, 36, 44),
            text: egui::Color32::from_rgb(238, 241, 246),
            accent: egui::Color32::from_rgb(0, 210, 255),
            danger: egui::Color32::from_rgb(255, 103, 103),
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, config: &AppConfig) {
    let system_dark = ctx.input(|input| input.raw.system_theme == Some(egui::Theme::Dark));
    let palette = ThemePalette::from_config(config, system_dark);

    match config.theme {
        ThemeMode::Light => ctx.set_visuals(egui::Visuals::light()),
        ThemeMode::Dark => ctx.set_visuals(egui::Visuals::dark()),
        ThemeMode::System => {
            if system_dark {
                ctx.set_visuals(egui::Visuals::dark());
            } else {
                ctx.set_visuals(egui::Visuals::light());
            }
        }
    }

    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.visuals.widgets.active.bg_fill = palette.accent.gamma_multiply(0.22);
        style.visuals.widgets.hovered.bg_fill = palette.accent.gamma_multiply(0.16);
        style.visuals.panel_fill = palette.background;
        style.visuals.window_fill = palette.panel;
        style.visuals.override_text_color = Some(palette.text);
    });
}

pub fn rainbow_color(seconds: f64, speed: f32) -> egui::Color32 {
    let t = (seconds as f32 * speed).fract();
    let h = t * 6.0;
    let x = 1.0 - ((h % 2.0) - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

pub fn color_button(ui: &mut egui::Ui, label: &str, color: &mut RgbaColor) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut egui_color = color.to_egui();
        if ui.color_edit_button_srgba(&mut egui_color).changed() {
            *color = RgbaColor::from_egui(egui_color);
        }
        ui.monospace(format!("#{:02X}{:02X}{:02X}{:02X}", color.r, color.g, color.b, color.a));
    });
}
