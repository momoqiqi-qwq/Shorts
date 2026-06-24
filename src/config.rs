use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl ThemeMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolbarPosition {
    Top,
    Bottom,
    Left,
    Right,
    FollowCursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    Png,
    Jpeg,
    Webp,
    Bmp,
    Gif,
    Avif,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::Png => "png",
            ExportFormat::Jpeg => "jpg",
            ExportFormat::Webp => "webp",
            ExportFormat::Bmp => "bmp",
            ExportFormat::Gif => "gif",
            ExportFormat::Avif => "avif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_egui(self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(self.r, self.g, self.b, self.a)
    }

    pub fn from_egui(color: egui::Color32) -> Self {
        let [r, g, b, a] = color.to_array();
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub mask_rgba: RgbaColor,
    pub stroke_rgba: RgbaColor,
    pub stroke_width: f32,
    pub toolbar_position: ToolbarPosition,
    pub show_magnifier: bool,
    pub show_size_label: bool,
    pub snap_to_windows: bool,
    pub dim_unfocused_monitors: bool,
    pub rainbow_border: bool,
    pub rainbow_speed: f32,
    pub rainbow_width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub jpeg_quality: u8,
    pub webp_quality: u8,
    pub webp_lossless: bool,
    pub avif_quality: f32,
    pub avif_speed: u8,
    pub filename_template: String,
    pub save_dir: PathBuf,
    pub auto_copy_clipboard: bool,
    pub auto_save: bool,
    pub open_after_save: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub delay_ms: u64,
    pub freeze_screen_before_select: bool,
    pub play_sound: bool,
    pub remember_last_shape: bool,
    pub show_quick_tip: bool,
    pub min_selection_px: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub confirm: String,
    pub cancel: String,
    pub copy: String,
    pub save: String,
    pub full_screen: String,
    pub rectangle: String,
    pub ellipse: String,
    pub lasso: String,
    pub polygon: String,
    pub window: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub overlay: OverlayConfig,
    pub export: ExportConfig,
    pub behavior: BehaviorConfig,
    pub hotkeys: HotkeyConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        let pictures = dirs::picture_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        Self {
            theme: ThemeMode::System,
            overlay: OverlayConfig {
                mask_rgba: RgbaColor::new(0, 0, 0, 140),
                stroke_rgba: RgbaColor::new(0, 210, 255, 255),
                stroke_width: 2.0,
                toolbar_position: ToolbarPosition::FollowCursor,
                show_magnifier: true,
                show_size_label: true,
                snap_to_windows: true,
                dim_unfocused_monitors: true,
                rainbow_border: false,
                rainbow_speed: 0.35,
                rainbow_width: 3.0,
            },
            export: ExportConfig {
                format: ExportFormat::Png,
                jpeg_quality: 92,
                webp_quality: 88,
                webp_lossless: false,
                avif_quality: 75.0,
                avif_speed: 5,
                filename_template: "screenshot_{timestamp}.{ext}".to_owned(),
                save_dir: pictures.join("Screenshots"),
                auto_copy_clipboard: true,
                auto_save: true,
                open_after_save: false,
            },
            behavior: BehaviorConfig {
                delay_ms: 0,
                freeze_screen_before_select: true,
                play_sound: false,
                remember_last_shape: true,
                show_quick_tip: true,
                min_selection_px: 6,
            },
            hotkeys: HotkeyConfig {
                confirm: "Enter".to_owned(),
                cancel: "Esc".to_owned(),
                copy: "Ctrl+C".to_owned(),
                save: "Ctrl+S".to_owned(),
                full_screen: "F".to_owned(),
                rectangle: "R".to_owned(),
                ellipse: "E".to_owned(),
                lasso: "L".to_owned(),
                polygon: "P".to_owned(),
                window: "W".to_owned(),
            },
        }
    }
}

impl AppConfig {
    pub fn path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            return config_dir.join("shotrs").join("config.toml");
        }
        PathBuf::from(".config/shotrs/config.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        match fs::read_to_string(path) {
            Ok(raw) => toml::from_str(&raw).unwrap_or_else(|_| Self::default()),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn apply_cli_theme_override(&mut self, theme: Option<ThemeMode>) {
        if let Some(theme) = theme {
            self.theme = theme;
        }
    }
}
